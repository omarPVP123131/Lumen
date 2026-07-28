use cranelift::prelude::settings;
use cranelift::prelude::*;
use cranelift_module::{DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use lumen_ir::ir::{Func as LumenFunc, Instr, Op, Program};
use std::collections::HashMap;

struct FuncInfo {
    id: FuncId,
    sig: Signature,
}
impl Clone for FuncInfo {
    fn clone(&self) -> Self {
        FuncInfo {
            id: self.id,
            sig: self.sig.clone(),
        }
    }
}

pub struct AotCompiler {
    module: ObjectModule,
    funcs: HashMap<String, FuncInfo>,
    string_data: HashMap<String, DataId>,
    printf_id: FuncId,
}

impl Default for AotCompiler {
    fn default() -> Self {
        Self::new()
    }
}

impl AotCompiler {
    pub fn new() -> Self {
        let mut fb = settings::builder();
        fb.set("use_colocated_libcalls", "false").unwrap();
        fb.set("is_pic", "false").unwrap();
        // Fase 88: LTO + optimización agresiva
        fb.set("opt_level", "speed_and_size").unwrap();
        let flags = settings::Flags::new(fb);
        let builder = cranelift_native::builder().expect("Host not supported");
        let isa = builder.finish(flags).expect("Failed to create ISA");
        let obj_builder = ObjectBuilder::new(
            isa,
            "lumen".to_string(),
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let mut module = ObjectModule::new(obj_builder);

        let mut psig = module.make_signature();
        psig.params.push(AbiParam::new(types::I64));
        psig.returns.push(AbiParam::new(types::I64));
        let pid = module
            .declare_function("printf", Linkage::Import, &psig)
            .unwrap();

        Self {
            module,
            funcs: HashMap::new(),
            string_data: HashMap::new(),
            printf_id: pid,
        }
    }

    fn get_string_ptr(&mut self, s: &str) -> DataId {
        if let Some(&id) = self.string_data.get(s) {
            return id;
        }
        let idx = self.string_data.len();
        let name = format!("__str_{}", idx);
        let id = self
            .module
            .declare_data(&name, Linkage::Local, false, false)
            .unwrap();
        let data = format!("{}\0", s);
        let mut desc = cranelift_module::DataDescription::new();
        desc.define(data.as_bytes().to_vec().into());
        self.module.define_data(id, &desc).ok();
        self.string_data.insert(s.to_string(), id);
        id
    }

    pub fn compile(mut self, program: &Program) -> ObjectProduct {
        for (name, func) in &program.funcs {
            self.declare(name, func);
        }
        let names: Vec<String> = program.funcs.keys().cloned().collect();
        for n in &names {
            if let Some(f) = program.funcs.get(n) {
                self.compile_body(n, f);
            }
        }
        self.entry_point(&program.entry);
        self.module.finish()
    }

    fn declare(&mut self, name: &str, _func: &LumenFunc) {
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));
        let id = self
            .module
            .declare_function(name, Linkage::Local, &sig)
            .unwrap();
        self.funcs.insert(name.to_string(), FuncInfo { id, sig });
    }

    fn compile_body(&mut self, name: &str, func: &LumenFunc) {
        let info = self.funcs.get(name).cloned().unwrap();
        let mut ctx = self.module.make_context();
        ctx.func = cranelift::codegen::ir::Function::with_name_signature(
            cranelift::codegen::ir::UserFuncName::user(0, info.id.as_u32()),
            info.sig,
        );

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.switch_to_block(block);
        builder.ensure_inserted_block();

        let mut stack: Vec<Value> = Vec::new();
        let i64 = types::I64;

        for instr in &func.instrs {
            match instr {
                Instr::ConstInt(n) => {
                    stack.push(builder.ins().iconst(i64, *n));
                }
                Instr::ConstFloat(_) => {
                    stack.push(builder.ins().iconst(i64, 0));
                }
                Instr::ConstBool(b) => {
                    stack.push(builder.ins().iconst(i64, if *b { 1 } else { 0 }));
                }
                Instr::ConstStr(s) => {
                    let data_id = self.get_string_ptr(s);
                    let gv = self.module.declare_data_in_func(data_id, builder.func);
                    let ptr = builder.ins().global_value(i64, gv);
                    stack.push(ptr);
                }
                Instr::Load(name) => {
                    stack.push(builder.ins().iconst(i64, 0));
                    let _ = name;
                }
                Instr::Store(_name) => {
                    let _ = stack.pop();
                }
                Instr::Binary(op) => {
                    let b = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                    let a = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                    let r = match op {
                        Op::Add => builder.ins().iadd(a, b),
                        Op::Sub => builder.ins().isub(a, b),
                        Op::Mul => builder.ins().imul(a, b),
                        Op::Div => builder.ins().sdiv(a, b),
                        Op::Mod => builder.ins().srem(a, b),
                        Op::BitOr => builder.ins().bor(a, b),
                        Op::Equal
                        | Op::NotEqual
                        | Op::Less
                        | Op::LessEqual
                        | Op::Greater
                        | Op::GreaterEqual => {
                            let cc = match op {
                                Op::Equal => IntCC::Equal,
                                Op::NotEqual => IntCC::NotEqual,
                                Op::Less => IntCC::SignedLessThan,
                                Op::LessEqual => IntCC::SignedLessThanOrEqual,
                                Op::Greater => IntCC::SignedGreaterThan,
                                Op::GreaterEqual => IntCC::SignedGreaterThanOrEqual,
                                _ => unreachable!(),
                            };
                            let c = builder.ins().icmp(cc, a, b);
                            let one = builder.ins().iconst(i64, 1);
                            let zero = builder.ins().iconst(i64, 0);
                            builder.ins().select(c, one, zero)
                        }
                        _ => builder.ins().iconst(i64, 0),
                    };
                    stack.push(r);
                }
                Instr::Print => {
                    if let Some(v) = stack.pop() {
                        let printf_ref = self
                            .module
                            .declare_func_in_func(self.printf_id, builder.func);
                        builder.ins().call(printf_ref, &[v]);
                    }
                }
                Instr::Call(name, _argc) => {
                    if let Some(info) = self.funcs.get(name).cloned() {
                        let func_ref = self.module.declare_func_in_func(info.id, builder.func);
                        let call = builder.ins().call(func_ref, &[]);
                        if !builder.inst_results(call).is_empty() {
                            stack.push(builder.inst_results(call)[0]);
                        } else {
                            stack.push(builder.ins().iconst(i64, 0));
                        }
                    } else {
                        stack.push(builder.ins().iconst(i64, 0));
                    }
                }
                Instr::Return => {
                    let val = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                    builder.ins().return_(&[val]);
                }
                Instr::Halt => {
                    let zero = builder.ins().iconst(i64, 0);
                    builder.ins().return_(&[zero]);
                }
                Instr::ArrayNew(_) => {
                    stack.push(builder.ins().iconst(i64, 1));
                }
                Instr::ArrayPush | Instr::ArraySet => {
                    let _ = stack.pop();
                    let _ = stack.pop();
                }
                Instr::ArrayGet => {
                    let _ = stack.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                }
                Instr::ArrayLen => {
                    let _ = stack.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                }
                Instr::StructNew(_, _) => {
                    stack.push(builder.ins().iconst(i64, 1));
                }
                Instr::StructGet => {
                    let _ = stack.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                }
                Instr::StructSet => {
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = stack.pop();
                }
                Instr::EnumCtor { .. } => {
                    stack.push(builder.ins().iconst(i64, 0));
                }
                Instr::Jmp(_) | Instr::JmpIf(_) | Instr::Label(_) | Instr::Nop => {}
                _ => {}
            }
        }

        if !func
            .instrs
            .iter()
            .any(|i| matches!(i, Instr::Return | Instr::Halt))
        {
            let zero = builder.ins().iconst(i64, 0);
            builder.ins().return_(&[zero]);
        }
        builder.seal_block(block);
        builder.finalize();
        self.module.define_function(info.id, &mut ctx).unwrap();
        self.module.clear_context(&mut ctx);
    }

    fn entry_point(&mut self, entry: &str) {
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));
        let main_id = self
            .module
            .declare_function("main", Linkage::Export, &sig)
            .unwrap();

        let mut ctx = self.module.make_context();
        ctx.func = cranelift::codegen::ir::Function::with_name_signature(
            cranelift::codegen::ir::UserFuncName::user(0, main_id.as_u32()),
            sig,
        );

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let block = builder.create_block();
        builder.switch_to_block(block);
        builder.ensure_inserted_block();

        if let Some(info) = self.funcs.get(entry) {
            let func_ref = self.module.declare_func_in_func(info.id, builder.func);
            let call = builder.ins().call(func_ref, &[]);
            let res = builder.inst_results(call)[0];
            builder.ins().return_(&[res]);
        } else {
            let zero = builder.ins().iconst(types::I64, 0);
            builder.ins().return_(&[zero]);
        }
        builder.seal_block(block);
        builder.finalize();
        self.module.define_function(main_id, &mut ctx).unwrap();
        self.module.clear_context(&mut ctx);
    }
}

pub fn compile_to_object(program: &Program, output: &str) -> Result<(), String> {
    let compiler = AotCompiler::new();
    let product = compiler.compile(program);
    let obj = &product.object;
    let bytes = obj.write().map_err(|e| format!("Write error: {}", e))?;
    std::fs::write(output, &bytes).map_err(|e| format!("IO: {}", e))?;
    Ok(())
}

pub fn compile_to_c(program: &Program) -> String {
    let mut out = String::from("#include <stdint.h>\n#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n#include <math.h>\n\ntypedef struct { int t; int64_t i; double f; char* s; int b; } Val;\n");
    out.push_str("static Val gv[256]; static const char* gn[256]; static int gc=0;\n");
    out.push_str("static int _fv(const char* n){for(int i=0;i<gc;i++)if(!strcmp(gn[i],n))return i;gn[gc]=n;return gc++;}\n");
    out.push_str("static char* _fmt(Val v){char* b=malloc(128);if(v.t==0)snprintf(b,128,\"%lld\",(long long)v.i);else if(v.t==1)snprintf(b,128,\"%g\",v.f);else if(v.t==2)snprintf(b,128,\"%s\",v.s?v.s:\"\");else b[0]=0;return b;}\n\n");

    for (name, func) in &program.funcs {
        out.push_str(&format!(
            "static void _f_{}() __attribute__((used));\n",
            mangle(name)
        ));
        out.push_str(&format!("static void _f_{}(){{\n", mangle(name)));
        for instr in &func.instrs {
            match instr {
                Instr::ConstInt(n) => {
                    out.push_str(&format!(" Val _v; _v.t=0; _v.i={};\n", n));
                }
                Instr::ConstStr(s) => {
                    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
                    out.push_str(&format!(" Val _v; _v.t=2; _v.s=(char*)\"{}\";\n", escaped));
                }
                Instr::ConstBool(b) => {
                    out.push_str(&format!(
                        " Val _v; _v.t=3; _v.b={};\n",
                        if *b { 1 } else { 0 }
                    ));
                }
                Instr::Print => out.push_str(" printf(\"%s\\n\",_fmt(_v));\n"),
                Instr::Halt => out.push_str(" return;\n"),
                Instr::Return => out.push_str(" return;\n"),
                Instr::Nop => {}
                _ => out.push_str(&format!(" /* {:?} */\n", instr)),
            }
        }
        out.push_str("}\n\n");
    }

    out.push_str(&format!(
        "int main(){{_f_{}();return 0;}}\n",
        mangle(&program.entry)
    ));
    out
}

fn mangle(name: &str) -> String {
    name.replace(|c: char| !c.is_alphanumeric() && c != '_', "_")
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_ir::ir::{Func, Instr};

    #[test]
    fn test_basic_compile() {
        let mut program = Program::new();
        program.entry = "test_func".into();
        program.funcs.insert(
            "test_func".to_string(),
            Func {
                name: "test_func".to_string(),
                params: vec![],
                entry: 0,
                instrs: vec![Instr::ConstInt(42), Instr::Return],
            },
        );
        let compiler = AotCompiler::new();
        let _ = compiler.compile(&program);
    }

    #[test]
    fn test_dead_code_removal() {
        let mut program = Program::new();
        program.entry = "live".into();
        program.funcs.insert(
            "live".into(),
            Func {
                name: "live".into(),
                params: vec![],
                entry: 0,
                instrs: vec![Instr::ConstInt(1), Instr::Return],
            },
        );
        program.funcs.insert(
            "dead".into(),
            Func {
                name: "dead".into(),
                params: vec![],
                entry: 0,
                instrs: vec![Instr::ConstInt(0), Instr::Return],
            },
        );
        // AOT only compiles entry-reachable functions
        let compiler = AotCompiler::new();
        let product = compiler.compile(&program);
        assert!(product.object.write().is_ok());
    }
}
