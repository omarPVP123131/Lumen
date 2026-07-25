// LUMEN AOT — Cranelift Native Backend
// Compila LUMEN IR → código máquina nativo vía Cranelift JIT
// Sin dependencias externas (no requiere gcc/clang)

use cranelift::prelude::*;
use cranelift_module::{FuncId, Linkage, Module};
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
}

impl AotCompiler {
    pub fn new() -> Self {
        let mut fb = settings::builder();
        fb.set("use_colocated_libcalls", "false").unwrap();
        fb.set("is_pic", "false").unwrap();
        fb.set("opt_level", "none").unwrap();
        let flags = settings::Flags::new(fb);

        let builder = cranelift_native::builder().expect("Host machine not supported by Cranelift");
        let isa = builder
            .finish(flags)
            .expect("Failed to create Cranelift ISA");

        let obj_builder = ObjectBuilder::new(
            isa,
            "lumen".to_string(),
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let module = ObjectModule::new(obj_builder);

        Self {
            module,
            funcs: HashMap::new(),
        }
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
        ctx.func.name = cranelift::codegen::ir::UserFuncName::user(0, info.id.as_u32());
        ctx.func.signature = info.sig.clone();

        {
            let mut func_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let block = builder.create_block();
            builder.append_block_params_for_function_params(block);
            builder.switch_to_block(block);
            builder.seal_block(block);

            let mut stack: Vec<Value> = Vec::new();
            let i64 = types::I64;

            for instr in &func.instrs {
                match instr {
                    Instr::ConstInt(n) => {
                        stack.push(builder.ins().iconst(i64, *n));
                    }
                    Instr::ConstBool(b) => {
                        stack.push(builder.ins().iconst(i64, if *b { 1 } else { 0 }));
                    }
                    Instr::ConstFloat(n) => {
                        stack.push(builder.ins().iconst(i64, *n as i64));
                    }
                    Instr::Binary(op) => {
                        let b = stack.pop().unwrap();
                        let a = stack.pop().unwrap();
                        let r = match op {
                            Op::Add => builder.ins().iadd(a, b),
                            Op::Sub => builder.ins().isub(a, b),
                            Op::Mul => builder.ins().imul(a, b),
                            Op::Div => builder.ins().sdiv(a, b),
                            Op::Mod => builder.ins().srem(a, b),
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
                    Instr::Return => {
                        let val = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                        builder.ins().return_(&[val]);
                    }
                    Instr::Halt | Instr::Nop => {}
                    _ => {}
                }
            }

            // Ensure function has a return
            builder.seal_all_blocks();
            if func.instrs.is_empty()
                || !func
                    .instrs
                    .iter()
                    .any(|i| matches!(i, Instr::Return | Instr::Halt))
            {
                let zero = builder.ins().iconst(i64, 0);
                builder.ins().return_(&[zero]);
            }

            builder.finalize();
        }

        self.module.define_function(info.id, &mut ctx).unwrap();
        self.module.clear_context(&mut ctx);
    }

    fn entry_point(&mut self, entry: &str) {
        let mut sig = self.module.make_signature();
        sig.returns.push(AbiParam::new(types::I64));
        let main_id = self
            .module
            .declare_function("lumen_entry", Linkage::Export, &sig)
            .unwrap();

        let mut ctx = self.module.make_context();
        ctx.func.name = cranelift::codegen::ir::UserFuncName::user(0, main_id.as_u32());
        ctx.func.signature = sig;

        {
            let mut func_ctx2 = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx2);
            let block = builder.create_block();
            builder.switch_to_block(block);
            builder.seal_block(block);

            if let Some(info) = self.funcs.get(entry) {
                let func_ref = self.module.declare_func_in_func(info.id, &mut builder.func);
                let call = builder.ins().call(func_ref, &[]);
                let res = builder.inst_results(call)[0];
                builder.ins().return_(&[res]);
            } else {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
            }

            builder.finalize();
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_ir::ir::Func;
    use std::collections::BTreeMap;

    #[test]
    fn test_basic_compile() {
        let mut funcs = BTreeMap::new();
        funcs.insert(
            "main".to_string(),
            Func {
                name: "main".to_string(),
                params: vec![],
                entry: 0,
                instrs: vec![Instr::ConstInt(42), Instr::Return],
            },
        );
        let program = Program {
            funcs,
            entry: "main".to_string(),
        };
        let compiler = AotCompiler::new();
        let _ = compiler.compile(&program);
    }
}
