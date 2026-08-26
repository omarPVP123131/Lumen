use cranelift::prelude::settings;
use cranelift::prelude::*;
use cranelift_module::{DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use lumen_ir::ir::{Func as LumenFunc, Instr, Op, Program};
use std::collections::{BTreeMap, HashMap};
use std::sync::Mutex;

static UNSUPPORTED_BUILTINS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Registra un builtin no soportado por el backend AOT
pub fn record_unsupported_builtin(name: &str) {
    if let Ok(mut list) = UNSUPPORTED_BUILTINS.lock() {
        if !list.iter().any(|x| x == name) {
            list.push(name.to_string());
        }
    }
}

/// Extrae y limpia la lista de builtins no soportados detectados durante la compilación AOT
pub fn take_unsupported_builtins() -> Vec<String> {
    if let Ok(mut list) = UNSUPPORTED_BUILTINS.lock() {
        std::mem::take(&mut *list)
    } else {
        Vec::new()
    }
}

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
    print_i64_id: FuncId,
    print_str_id: FuncId,
    concat_ss_id: FuncId,
    concat_si_id: FuncId,
    concat_is_id: FuncId,
    str_eq_id: FuncId,
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
        let pid = module
            .declare_function("_rt_print_i64", Linkage::Import, &psig)
            .unwrap();
        let pstr_id = module
            .declare_function("_rt_print_str", Linkage::Import, &psig)
            .unwrap();
        // firma con retorno i64 para concat/eq
        let mut rsig = module.make_signature();
        rsig.params.push(AbiParam::new(types::I64));
        rsig.params.push(AbiParam::new(types::I64));
        rsig.returns.push(AbiParam::new(types::I64));
        let css_id = module
            .declare_function("_rt_concat_ss", Linkage::Import, &rsig)
            .unwrap();
        let csi_id = module
            .declare_function("_rt_concat_si", Linkage::Import, &rsig)
            .unwrap();
        let cis_id = module
            .declare_function("_rt_concat_is", Linkage::Import, &rsig)
            .unwrap();
        let seq_id = module
            .declare_function("_rt_str_eq", Linkage::Import, &rsig)
            .unwrap();

        Self {
            module,
            funcs: HashMap::new(),
            string_data: HashMap::new(),
            print_i64_id: pid,
            print_str_id: pstr_id,
            concat_ss_id: css_id,
            concat_si_id: csi_id,
            concat_is_id: cis_id,
            str_eq_id: seq_id,
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

    fn declare(&mut self, name: &str, func: &LumenFunc) {
        let mut sig = self.module.make_signature();
        for _ in &func.params {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));
        let id = self
            .module
            .declare_function(name, Linkage::Local, &sig)
            .unwrap();
        self.funcs.insert(name.to_string(), FuncInfo { id, sig });
    }

    fn compile_body(&mut self, name: &str, func: &LumenFunc) {
        if std::env::var_os("LUMEN_AOT_DEBUG").is_some() {
            eprintln!("[aot] compile_body({}) instrs={}", name, func.instrs.len());
            for ins in &func.instrs {
                eprintln!("[aot]   {:?}", ins);
            }
        }
        let info = self.funcs.get(name).cloned().unwrap();
        let mut ctx = self.module.make_context();
        ctx.func = cranelift::codegen::ir::Function::with_name_signature(
            cranelift::codegen::ir::UserFuncName::user(0, info.id.as_u32()),
            info.sig,
        );

        let mut func_ctx = FunctionBuilderContext::new();
        let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
        let i64 = types::I64;

        // ── Variables: SSA real del frontend (registros, no memoria) ──
        // def_var en el entry con 0 garantiza dominancia de toda variable
        // (los VarDecl sin inicializador emiten ConstInt 0 + Store).
        use cranelift::frontend::Variable;
        let mut vars: HashMap<String, Variable> = HashMap::new();
        fn var_of(
            builder: &mut FunctionBuilder,
            vars: &mut HashMap<String, Variable>,
            n: &str,
        ) -> Variable {
            if let Some(&v) = vars.get(n) {
                return v;
            }
            let v = builder.declare_var(types::I64);
            vars.insert(n.to_string(), v);
            v
        }

        // ── Bloques: uno por Label (branches con referencia adelantada) ──
        let instrs = &func.instrs;

        // ── Entrada: bloque 0 con params de la firma → variables ──
        let entry_block = builder.create_block();

        let mut label_block: HashMap<usize, Block> = HashMap::new();
        for ins in instrs {
            if let Instr::Label(n) = ins {
                let b = builder.create_block();
                label_block.insert(*n, b);
            }
        }
        // block_of(i): bloque del label más reciente <= i (fallthrough)
        let mut block_at: Vec<Block> = Vec::with_capacity(instrs.len());
        let mut cur_fall = entry_block;
        for (i, ins) in instrs.iter().enumerate() {
            if let Instr::Label(n) = ins {
                if let Some(&b) = label_block.get(n) {
                    cur_fall = b;
                }
            }
            block_at.push(cur_fall);
            let _ = i;
        }

        builder.switch_to_block(entry_block);
        for _ in &func.params {
            builder.append_block_param(entry_block, i64);
        }
        builder.ensure_inserted_block();
        let entry_params: Vec<Value> = builder.block_params(entry_block).to_vec();
        // En registro: todas las variables usadas en la función se definen con 0.
        // Los params de la firma se sobre-escriben con su valor real.
        let mut used_names: Vec<String> = Vec::new();
        for ins in instrs {
            if let Instr::Load(n) | Instr::Store(n) | Instr::StoreLocal(n) = ins {
                if !used_names.iter().any(|x| x == n) {
                    used_names.push(n.clone());
                }
            }
        }
        let zero = builder.ins().iconst(i64, 0);
        for n in &used_names {
            let v = var_of(&mut builder, &mut vars, n);
            builder.def_var(v, zero);
        }
        for (pi, pname) in func.params.iter().enumerate() {
            let val = entry_params.get(pi).copied().unwrap_or(zero);
            let v = var_of(&mut builder, &mut vars, pname);
            builder.def_var(v, val);
        }

        // ── Emisión lineal ──
        let mut cur = entry_block;
        let mut stack: Vec<Value> = Vec::new();
        // pila paralela: true = string (puntero), false = i64
        let mut kinds: Vec<bool> = Vec::new();
        let mut var_kinds: HashMap<String, bool> = HashMap::new();
        let mut terminated = false;
        for ins in instrs {
            if let Instr::Label(n) = ins {
                let target = label_block[n];
                if target != cur {
                    if !terminated {
                        // el bloque actual termina saltando al label
                        builder.ins().jump(target, &[]);
                    }
                    cur = target;
                    builder.switch_to_block(cur);
                    builder.ensure_inserted_block();
                }
                terminated = false;
                continue;
            }
            if terminated {
                continue;
            }
            match ins {
                Instr::ConstInt(n) => {
                    stack.push(builder.ins().iconst(i64, *n));
                    kinds.push(false);
                }
                Instr::ConstFloat(_) => {
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::ConstBool(b) => {
                    stack.push(builder.ins().iconst(i64, if *b { 1 } else { 0 }));
                    kinds.push(false);
                }
                Instr::ConstStr(s) => {
                    let data_id = self.get_string_ptr(s);
                    let gv = self.module.declare_data_in_func(data_id, builder.func);
                    let ptr = builder.ins().global_value(i64, gv);
                    stack.push(ptr);
                    kinds.push(true);
                }
                Instr::Load(n) => {
                    let v = var_of(&mut builder, &mut vars, n);
                    stack.push(builder.use_var(v));
                    kinds.push(*var_kinds.get(n).unwrap_or(&false));
                }
                Instr::Store(n) | Instr::StoreLocal(n) => {
                    if let Some(v) = stack.pop() {
                        let k = kinds.pop().unwrap_or(false);
                        var_kinds.insert(n.to_string(), k);
                        let vv = var_of(&mut builder, &mut vars, n);
                        builder.def_var(vv, v);
                    }
                }
                Instr::Binary(op) => {
                    let b = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                    let a = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                    let kb = kinds.pop().unwrap_or(false);
                    let ka = kinds.pop().unwrap_or(false);
                    let r = match op {
                        Op::Add | Op::Concat if ka || kb => {
                            let (fid, aa, bb) = match (ka, kb) {
                                (true, true) => (self.concat_ss_id, a, b),
                                (true, false) => (self.concat_si_id, a, b),
                                _ => (self.concat_is_id, a, b),
                            };
                            let fref = self.module.declare_func_in_func(fid, builder.func);
                            let call = builder.ins().call(fref, &[aa, bb]);
                            builder.inst_results(call)[0]
                        }
                        Op::Add => builder.ins().iadd(a, b),
                        Op::Sub => builder.ins().isub(a, b),
                        Op::Mul => builder.ins().imul(a, b),
                        Op::Div => {
                            let z = builder.ins().iconst(i64, 0);
                            let nonzero = builder.ins().icmp(IntCC::NotEqual, b, z);
                            let one = builder.ins().iconst(i64, 1);
                            let safe = builder.ins().select(nonzero, b, one);
                            builder.ins().sdiv(a, safe)
                        }
                        Op::Mod => {
                            let z = builder.ins().iconst(i64, 0);
                            let nonzero = builder.ins().icmp(IntCC::NotEqual, b, z);
                            let one = builder.ins().iconst(i64, 1);
                            let safe = builder.ins().select(nonzero, b, one);
                            builder.ins().srem(a, safe)
                        }
                        Op::BitOr => builder.ins().bor(a, b),
                        Op::BitAnd => builder.ins().band(a, b),
                        Op::BitXor => builder.ins().bxor(a, b),
                        Op::ShiftLeft => builder.ins().ishl(a, b),
                        Op::ShiftRight => builder.ins().ushr(a, b),
                        Op::Equal
                        | Op::NotEqual
                        | Op::Less
                        | Op::LessEqual
                        | Op::Greater
                        | Op::GreaterEqual => {
                            if matches!(op, Op::Equal | Op::NotEqual) && ka && kb {
                                // comparación de strings: strcmp vía shim
                                let fref = self
                                    .module
                                    .declare_func_in_func(self.str_eq_id, builder.func);
                                let call = builder.ins().call(fref, &[a, b]);
                                let eq = builder.inst_results(call)[0];
                                let z = builder.ins().iconst(i64, 0);
                                let one = builder.ins().iconst(i64, 1);
                                if matches!(op, Op::Equal) {
                                    eq
                                } else {
                                    let is_zero = builder.ins().icmp(IntCC::Equal, eq, z);
                                    builder.ins().select(is_zero, one, z)
                                }
                            } else {
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
                        }
                        Op::Concat => a,
                        Op::And | Op::Or => {
                            let z = builder.ins().iconst(i64, 0);
                            let za = builder.ins().icmp(IntCC::NotEqual, a, z);
                            let zb = builder.ins().icmp(IntCC::NotEqual, b, z);
                            let r = if *op == Op::And {
                                builder.ins().band(za, zb)
                            } else {
                                builder.ins().bor(za, zb)
                            };
                            let one = builder.ins().iconst(i64, 1);
                            builder.ins().select(r, one, z)
                        }
                        Op::Negate | Op::Not | Op::BitNot => a,
                    };
                    stack.push(r);
                    kinds.push(matches!(op, Op::Add | Op::Concat) && (ka || kb));
                }
                Instr::Unary(Op::Not) => {
                    if let Some(v) = stack.pop() {
                        let _ = kinds.pop();
                        let z = builder.ins().iconst(i64, 0);
                        let is_zero = builder.ins().icmp(IntCC::Equal, v, z);
                        let one = builder.ins().iconst(i64, 1);
                        stack.push(builder.ins().select(is_zero, one, z));
                        kinds.push(false);
                    }
                }
                Instr::Unary(Op::Negate) => {
                    if let Some(v) = stack.pop() {
                        let _ = kinds.pop();
                        stack.push(builder.ins().ineg(v));
                        kinds.push(false);
                    }
                }
                Instr::Unary(Op::BitNot) => {
                    if let Some(v) = stack.pop() {
                        let _ = kinds.pop();
                        stack.push(builder.ins().bnot(v));
                        kinds.push(false);
                    }
                }
                Instr::Print => {
                    let v = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                    let k = kinds.pop().unwrap_or(false);
                    let fid = if k {
                        self.print_str_id
                    } else {
                        self.print_i64_id
                    };
                    let fref = self.module.declare_func_in_func(fid, builder.func);
                    builder.ins().call(fref, &[v]);
                }
                Instr::Call(name, argc) => {
                    let mut args: Vec<Value> = Vec::with_capacity(*argc);
                    let mut arg_kinds: Vec<bool> = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        args.push(stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0)));
                        arg_kinds.push(kinds.pop().unwrap_or(false));
                    }
                    args.reverse();
                    arg_kinds.reverse();
                    match name.as_str() {
                        "imprimir" | "print" => {
                            for (av, ak) in args.iter().zip(arg_kinds.iter()) {
                                let fid = if *ak {
                                    self.print_str_id
                                } else {
                                    self.print_i64_id
                                };
                                let fref = self.module.declare_func_in_func(fid, builder.func);
                                builder.ins().call(fref, &[*av]);
                            }
                            stack.push(builder.ins().iconst(i64, 0));
                            kinds.push(false);
                        }
                        "leer" | "read" | "__str_len" | "__str_longitud" | "largo" | "len"
                        | "agregar" | "push" | "a_texto" | "to_texto" | "__str_from"
                        | "__map_nuevo" | "__map_poner" | "__map_obtener" | "__map_contiene" => {
                            // builtins de colecciones/strings sin runtime nativo en
                            // el backend Cranelift: resultado placeholder
                            stack.push(builder.ins().iconst(i64, 0));
                            kinds.push(false);
                        }
                        _ => {
                            if let Some(info) = self.funcs.get(name).cloned() {
                                let func_ref =
                                    self.module.declare_func_in_func(info.id, builder.func);
                                let call = builder.ins().call(func_ref, &args);
                                let res = builder.inst_results(call)[0];
                                stack.push(res);
                                kinds.push(false);
                            } else {
                                stack.push(builder.ins().iconst(i64, 0));
                                kinds.push(false);
                            }
                        }
                    }
                }
                Instr::Return => {
                    let val = stack.pop().unwrap_or_else(|| builder.ins().iconst(i64, 0));
                    let _ = kinds.pop();
                    builder.ins().return_(&[val]);
                    terminated = true;
                    stack.clear();
                }
                Instr::Halt => {
                    let zero = builder.ins().iconst(i64, 0);
                    builder.ins().return_(&[zero]);
                    terminated = true;
                    stack.clear();
                }
                Instr::Jmp(target) => {
                    if let Some(&b) = label_block.get(target) {
                        builder.ins().jump(b, &[]);
                    }
                    terminated = true;
                }
                Instr::JmpIf(target) => {
                    if let Some(v) = stack.pop() {
                        if let Some(&b) = label_block.get(target) {
                            let z = builder.ins().iconst(i64, 0);
                            let is_zero = builder.ins().icmp(IntCC::Equal, v, z);
                            let next_block = builder.create_block();
                            builder.ins().brif(is_zero, b, &[], next_block, &[]);
                            builder.switch_to_block(next_block);
                            builder.ensure_inserted_block();
                            cur = next_block;
                            terminated = false;
                        }
                    }
                }
                Instr::ArrayNew(_) => {
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::ArrayPush | Instr::ArraySet => {
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    let _ = kinds.pop();
                }
                Instr::ArrayGet => {
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::ArrayLen => {
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::StructNew(_, _) => {
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::StructGet => {
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::StructSet => {
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    let _ = kinds.pop();
                    let _ = kinds.pop();
                }
                Instr::EnumCtor { .. } => {
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::ResultOk | Instr::OptionSome | Instr::OptionNone | Instr::ResultErr => {
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::TryUnwrap | Instr::TupleAccess(_) | Instr::Read => {
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::TupleNew(_) => {
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::FuncRef(_) => {
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::CallValue(_) => {
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::Phi(_, _) | Instr::Nop => {}
                _ => {}
            }
        }

        builder.seal_all_blocks();
        builder.finalize();
        if std::env::var_os("LUMEN_AOT_DEBUG").is_some() {
            eprintln!("[aot] --- clif {} ---\n{}", name, ctx.func.display());
        }
        if let Err(e) = self.module.define_function(info.id, &mut ctx) {
            eprintln!("[aot] define_function({}) fallo: {:?}", name, e);
            panic!("aot define_function fallo");
        }
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

use cranelift_jit::{JITBuilder, JITModule};

pub struct JitEngine {
    builder_context: FunctionBuilderContext,
    ctx: cranelift::codegen::Context,
    module: JITModule,
}

impl JitEngine {
    pub fn new() -> Result<Self, String> {
        let mut flag_builder = settings::builder();
        flag_builder
            .set("use_colocated_libcalls", "false")
            .map_err(|e| e.to_string())?;
        flag_builder
            .set("is_pic", "false")
            .map_err(|e| e.to_string())?;
        flag_builder
            .set("opt_level", "speed")
            .map_err(|e| e.to_string())?;
        let isa_builder = cranelift_native::builder().map_err(|e| e.to_string())?;
        let isa = isa_builder
            .finish(settings::Flags::new(flag_builder))
            .map_err(|e| e.to_string())?;
        let builder = JITBuilder::with_isa(isa, cranelift_module::default_libcall_names());
        let module = JITModule::new(builder);
        let ctx = module.make_context();
        Ok(Self {
            builder_context: FunctionBuilderContext::new(),
            ctx,
            module,
        })
    }

    pub fn compile_function(&mut self, name: &str, func: &LumenFunc) -> Result<*const u8, String> {
        let mut sig = self.module.make_signature();
        for _ in &func.params {
            sig.params.push(AbiParam::new(types::I64));
        }
        sig.returns.push(AbiParam::new(types::I64));

        let func_id = self
            .module
            .declare_function(name, Linkage::Export, &sig)
            .map_err(|e| e.to_string())?;

        self.ctx.func.signature = sig;
        self.ctx.func.name = cranelift::codegen::ir::UserFuncName::user(0, func_id.as_u32());

        {
            let mut builder = FunctionBuilder::new(&mut self.ctx.func, &mut self.builder_context);
            let entry_block = builder.create_block();
            builder.switch_to_block(entry_block);
            for _ in &func.params {
                builder.append_block_param(entry_block, types::I64);
            }
            builder.ensure_inserted_block();

            let mut stack: Vec<cranelift::prelude::Value> = Vec::new();
            let mut has_returned = false;
            for ins in &func.instrs {
                match ins {
                    Instr::ConstInt(n) => stack.push(builder.ins().iconst(types::I64, *n)),
                    Instr::ConstFloat(_) => stack.push(builder.ins().iconst(types::I64, 0)),
                    Instr::ConstBool(b) => {
                        stack.push(builder.ins().iconst(types::I64, if *b { 1 } else { 0 }))
                    }
                    Instr::Binary(op) => {
                        let b = stack
                            .pop()
                            .unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
                        let a = stack
                            .pop()
                            .unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
                        let r = match op {
                            Op::Add => builder.ins().iadd(a, b),
                            Op::Sub => builder.ins().isub(a, b),
                            Op::Mul => builder.ins().imul(a, b),
                            Op::BitAnd => builder.ins().band(a, b),
                            Op::BitOr => builder.ins().bor(a, b),
                            Op::BitXor => builder.ins().bxor(a, b),
                            _ => builder.ins().iadd(a, b),
                        };
                        stack.push(r);
                    }
                    Instr::Return => {
                        let val = stack
                            .pop()
                            .unwrap_or_else(|| builder.ins().iconst(types::I64, 0));
                        builder.ins().return_(&[val]);
                        has_returned = true;
                    }
                    _ => {}
                }
            }
            if !has_returned {
                let zero = builder.ins().iconst(types::I64, 0);
                builder.ins().return_(&[zero]);
            }
            builder.seal_all_blocks();
            builder.finalize();
        }

        self.module
            .define_function(func_id, &mut self.ctx)
            .map_err(|e| e.to_string())?;
        self.module.clear_context(&mut self.ctx);
        self.module
            .finalize_definitions()
            .map_err(|e| e.to_string())?;

        let code = self.module.get_finalized_function(func_id);
        Ok(code)
    }
}

/// Capacidad del backend LLVM textual: subconjunto verificado correcto.
/// Todo lo demás debe rechazarse ANTES de generar artefactos rotos.
pub fn llvm_supported(program: &Program) -> Vec<String> {
    let mut bad: Vec<String> = Vec::new();
    let mut note = |f: &str| {
        if !bad.iter().any(|x| x == f) {
            bad.push(f.to_string());
        }
    };
    for func in program.funcs.values() {
        for ins in &func.instrs {
            match ins {
                Instr::ConstInt(_) | Instr::ConstBool(_) | Instr::Load(_)
                | Instr::Store(_) | Instr::StoreLocal(_) | Instr::Return | Instr::Jmp(_)
                | Instr::JmpIf(_) | Instr::Label(_) | Instr::Phi(..) | Instr::Nop
                | Instr::Halt => {}
                Instr::ConstStr(_) => note("textos"),
                Instr::ConstFloat(_) => note("decimales"),
                Instr::Unary(_) => note("operadores unarios"),
                Instr::Print | Instr::Read => note("imprimir/leer"),
                Instr::Binary(op) => match op {
                    Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod | Op::Equal
                    | Op::NotEqual | Op::Less | Op::LessEqual | Op::Greater
                    | Op::GreaterEqual | Op::BitOr | Op::BitAnd | Op::BitXor
                    | Op::ShiftLeft | Op::ShiftRight => {}
                    _ => note("operadores lógicos/concatenación"),
                },
                Instr::Call(n, _) => {
                    // Solo llamadas a funciones propias del programa
                    if !program.funcs.contains_key(n) {
                        note(format!("builtins ({})", n).as_str());
                    }
                }
                Instr::ArrayNew(_) | Instr::ArrayGet | Instr::ArraySet | Instr::ArrayLen
                | Instr::ArrayPush | Instr::ArrayPushVar(_) => note("listas"),
                Instr::StructNew(..) | Instr::StructGet | Instr::StructSet => note("estructuras"),
                Instr::EnumCtor { .. } | Instr::MatchVariant(_) => note("enumeraciones"),
                Instr::ResultOk | Instr::ResultErr | Instr::TryUnwrap | Instr::OptionSome
                | Instr::OptionNone => note("resultado/opción"),
                Instr::MatchType(_) | Instr::MatchPayload => note("elegir con tipos"),
                Instr::TupleNew(_) | Instr::TupleAccess(_) => note("tuplas"),
                Instr::FuncRef(_) | Instr::CallValue(_) => note("funciones como valores"),
                Instr::MakeRef(_) => note("prestado mut"),
                Instr::PushHandler(_) | Instr::PopHandler => note("intentar/atrapar"),
                Instr::ScopePush | Instr::ScopePop => {}
            }
        }
    }
    bad.sort();
    bad.dedup();
    bad
}

/// Capacidad del backend Cranelift (objeto nativo): subconjunto verificado.
pub fn cranelift_supported(program: &Program) -> Vec<String> {
    let mut bad: Vec<String> = Vec::new();
    let mut note = |f: &str| {
        if !bad.iter().any(|x| x == f) {
            bad.push(f.to_string());
        }
    };
    for func in program.funcs.values() {
        for ins in &func.instrs {
            match ins {
                Instr::ConstInt(_) | Instr::ConstBool(_) | Instr::ConstStr(_)
                | Instr::Load(_) | Instr::Store(_) | Instr::StoreLocal(_) | Instr::Unary(_)
                | Instr::Return | Instr::Jmp(_) | Instr::JmpIf(_) | Instr::Label(_)
                | Instr::Print | Instr::Nop | Instr::Halt | Instr::Call(_, _) => {}
                Instr::ConstFloat(_) => note("decimales"),
                Instr::Binary(op) => match op {
                    Op::Add | Op::Sub | Op::Mul | Op::BitAnd | Op::BitOr | Op::BitXor => {}
                    _ => note("división/módulo/comparaciones/cambios de bit"),
                },
                _ => note("agregados/cierres/excepciones/referencias"),
            }
        }
    }
    bad.sort();
    bad.dedup();
    bad
}

pub fn compile_to_llvm_ir(program: &Program) -> String {
    let mut out = String::new();
    out.push_str("; ModuleID = 'lumen'\n");
    out.push_str("source_filename = \"lumen.nv\"\n");
    out.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128\"\n\n");

    out.push_str("declare void @_rt_print_i64(i64)\n");
    out.push_str("declare void @_rt_print_str(i8*)\n");
    out.push_str("declare i8* @_rt_concat_ss(i8*, i8*)\n");
    out.push_str("declare i8* @_rt_concat_si(i8*, i64)\n");
    out.push_str("declare i8* @_rt_concat_is(i64, i8*)\n");
    out.push_str("declare i64 @_rt_str_eq(i8*, i8*)\n");
    out.push_str("declare i32 @printf(i8*, ...)\n");
    out.push_str("declare i8* @malloc(i64)\n");
    out.push_str("declare void @free(i8*)\n\n");

    for (name, func) in &program.funcs {
        let mangled = mangle(name);
        let param_types: Vec<String> = (0..func.params.len())
            .map(|i| format!("i64 %p{}", i))
            .collect();
        out.push_str(&format!(
            "define i64 @{}({}) {{\n",
            mangled,
            param_types.join(", ")
        ));
        out.push_str("entry:\n");

        let mut reg_counter = 0usize;
        let mut next_reg = || {
            let r = format!("%r{}", reg_counter);
            reg_counter += 1;
            r
        };

        let mut var_ptrs: HashMap<String, String> = HashMap::new();
        for (i, p) in func.params.iter().enumerate() {
            let ptr = next_reg();
            out.push_str(&format!("  {} = alloca i64\n", ptr));
            out.push_str(&format!("  store i64 %p{}, i64* {}\n", i, ptr));
            var_ptrs.insert(p.clone(), ptr);
        }
        for ins in &func.instrs {
            if let Instr::Load(n) | Instr::Store(n) | Instr::StoreLocal(n) = ins {
                if !var_ptrs.contains_key(n) {
                    let ptr = next_reg();
                    out.push_str(&format!("  {} = alloca i64\n", ptr));
                    out.push_str(&format!("  store i64 0, i64* {}\n", ptr));
                    var_ptrs.insert(n.clone(), ptr);
                }
            }
        }

        let mut stack: Vec<String> = Vec::new();
        let mut terminated = false;

        for ins in &func.instrs {
            if let Instr::Label(t) = ins {
                if !terminated {
                    out.push_str(&format!("  br label %L_{}\n", t));
                }
                out.push_str(&format!("L_{}:\n", t));
                terminated = false;
                continue;
            }
            if terminated {
                continue;
            }
            match ins {
                Instr::ConstInt(n) => {
                    stack.push(format!("{}", n));
                }
                Instr::ConstFloat(f) => {
                    stack.push(format!("{}", *f as i64));
                }
                Instr::ConstBool(b) => {
                    stack.push(format!("{}", if *b { 1 } else { 0 }));
                }
                Instr::Load(n) => {
                    if let Some(ptr) = var_ptrs.get(n) {
                        let r = next_reg();
                        out.push_str(&format!("  {} = load i64, i64* {}\n", r, ptr));
                        stack.push(r);
                    } else {
                        stack.push("0".to_string());
                    }
                }
                Instr::Store(n) | Instr::StoreLocal(n) => {
                    let val = stack.pop().unwrap_or_else(|| "0".to_string());
                    if let Some(ptr) = var_ptrs.get(n) {
                        out.push_str(&format!("  store i64 {}, i64* {}\n", val, ptr));
                    }
                }
                Instr::Binary(op) => {
                    let b = stack.pop().unwrap_or_else(|| "0".to_string());
                    let a = stack.pop().unwrap_or_else(|| "0".to_string());
                    let r = next_reg();
                    match op {
                        Op::Add => out.push_str(&format!("  {} = add i64 {}, {}\n", r, a, b)),
                        Op::Sub => out.push_str(&format!("  {} = sub i64 {}, {}\n", r, a, b)),
                        Op::Mul => out.push_str(&format!("  {} = mul i64 {}, {}\n", r, a, b)),
                        Op::Div => out.push_str(&format!("  {} = sdiv i64 {}, {}\n", r, a, b)),
                        Op::Mod => out.push_str(&format!("  {} = srem i64 {}, {}\n", r, a, b)),
                        Op::BitAnd => out.push_str(&format!("  {} = and i64 {}, {}\n", r, a, b)),
                        Op::BitOr => out.push_str(&format!("  {} = or i64 {}, {}\n", r, a, b)),
                        Op::BitXor => out.push_str(&format!("  {} = xor i64 {}, {}\n", r, a, b)),
                        Op::ShiftLeft => out.push_str(&format!("  {} = shl i64 {}, {}\n", r, a, b)),
                        Op::ShiftRight => {
                            out.push_str(&format!("  {} = ashr i64 {}, {}\n", r, a, b))
                        }
                        Op::Equal
                        | Op::NotEqual
                        | Op::Less
                        | Op::LessEqual
                        | Op::Greater
                        | Op::GreaterEqual => {
                            let pred = match op {
                                Op::Equal => "eq",
                                Op::NotEqual => "ne",
                                Op::Less => "slt",
                                Op::LessEqual => "sle",
                                Op::Greater => "sgt",
                                Op::GreaterEqual => "sge",
                                _ => unreachable!(),
                            };
                            let cmp_reg = next_reg();
                            out.push_str(&format!(
                                "  {} = icmp {} i64 {}, {}\n",
                                cmp_reg, pred, a, b
                            ));
                            out.push_str(&format!("  {} = zext i1 {} to i64\n", r, cmp_reg));
                        }
                        _ => out.push_str(&format!("  {} = add i64 {}, {}\n", r, a, b)),
                    }
                    stack.push(r);
                }
                Instr::Return => {
                    let val = stack.pop().unwrap_or_else(|| "0".to_string());
                    out.push_str(&format!("  ret i64 {}\n", val));
                    terminated = true;
                    stack.clear();
                }
                Instr::Halt => {
                    out.push_str("  ret i64 0\n");
                    terminated = true;
                    stack.clear();
                }
                Instr::Jmp(t) => {
                    out.push_str(&format!("  br label %L_{}\n", t));
                    terminated = true;
                }
                Instr::JmpIf(t) => {
                    let cond = stack.pop().unwrap_or_else(|| "0".to_string());
                    let cmp_r = next_reg();
                    let fall_num = next_reg();
                    let next_l = format!("L_fall_{}", fall_num.trim_start_matches('%'));
                    out.push_str(&format!("  {} = icmp eq i64 {}, 0\n", cmp_r, cond));
                    out.push_str(&format!(
                        "  br i1 {}, label %L_{}, label %{}\n",
                        cmp_r, t, next_l
                    ));
                    out.push_str(&format!("{}:\n", next_l));
                    terminated = false;
                }
                Instr::Call(n, argc) => {
                    let mut args = Vec::new();
                    for _ in 0..*argc {
                        args.push(stack.pop().unwrap_or_else(|| "0".to_string()));
                    }
                    args.reverse();
                    let r = next_reg();
                    let formatted_args: Vec<String> =
                        args.iter().map(|a| format!("i64 {}", a)).collect();
                    out.push_str(&format!(
                        "  {} = call i64 @{}({})\n",
                        r,
                        mangle(n),
                        formatted_args.join(", ")
                    ));
                    stack.push(r);
                }
                _ => {}
            }
        }
        if !terminated {
            out.push_str("  ret i64 0\n");
        }
        out.push_str("}\n\n");
    }

    out.push_str("define i32 @main() {\nentry:\n");
    let entry_name = if program.funcs.contains_key(&program.entry) {
        &program.entry
    } else if program.funcs.contains_key("main") {
        "main"
    } else if program.funcs.contains_key("principal") {
        "principal"
    } else {
        ""
    };
    if !entry_name.is_empty() {
        out.push_str(&format!("  %res = call i64 @{}()\n", mangle(entry_name)));
    }
    out.push_str("  ret i32 0\n}\n");

    out
}

pub fn compile_to_object(program: &Program, output: &str) -> Result<(), String> {
    let compiler = AotCompiler::new();
    let product = compiler.compile(program);
    let obj = &product.object;
    let bytes = obj.write().map_err(|e| format!("Write error: {}", e))?;
    std::fs::write(output, &bytes).map_err(|e| format!("IO: {}", e))?;
    Ok(())
}

const C_RUNTIME: &str = include_str!("lumen_rt.h");

pub fn compile_to_c(program: &Program) -> String {
    let mut out = String::new();
    out.push_str(C_RUNTIME);
    out.push('\n');

    let mut names: Vec<String> = Vec::new();
    let mut add_name = |n: &str| {
        if !names.iter().any(|x| x == n) {
            names.push(n.to_string());
        }
    };
    let mut unknown: Vec<String> = Vec::new();

    // Renombrado de params por función: `{fn}::{param}`. El namespace gv[] es
    // plano y compartido entre funciones; sin renombrado, un param del callee
    // con el mismo nombre que una variable del llamador se pisan mutuamente
    // (y con referencias prestado mut el slot se autorreferencia).
    let mut renames: HashMap<String, HashMap<String, String>> = HashMap::new();
    for (fname, func) in &program.funcs {
        let mut m: HashMap<String, String> = HashMap::new();
        for p in &func.params {
            m.insert(p.clone(), format!("{}::{}", fname, p));
        }
        renames.insert(fname.clone(), m);
    }
    // Traduce un nombre de variable al slot real dentro de la función `fname`
    let resolve_var =
        |fname: &str, n: &str| -> String { renames.get(fname).and_then(|m| m.get(n)).cloned().unwrap_or_else(|| n.to_string()) };

    // Plan de slots por función (params renombrados + sombreado de bloques)
    let mut var_plans: HashMap<String, HashMap<usize, String>> = HashMap::new();
    for (fname, func) in &program.funcs {
        let pr = renames.get(fname).cloned().unwrap_or_default();
        var_plans.insert(fname.clone(), plan_var_keys(func, &pr));
    }

    for (name, func) in &program.funcs {
        let plan = &var_plans[name];
        for p in &func.params {
            add_name(&resolve_var(name, p));
        }
        for (i, ins) in func.instrs.iter().enumerate() {
            match ins {
                Instr::Load(_) | Instr::Store(_) | Instr::StoreLocal(_)
                | Instr::ArrayPushVar(_) | Instr::MakeRef(_) => {
                    if let Some(k) = plan.get(&i) {
                        add_name(k);
                    }
                }
                Instr::FuncRef(n) => {
                    add_name(n);
                }
                Instr::Call(cn, _) => {
                    if !program.funcs.contains_key(cn) && !unknown.iter().any(|u| u == cn) {
                        unknown.push(cn.clone());
                        record_unsupported_builtin(cn);
                    }
                }
                _ => {}
            }
        }
        let _ = name;
    }

    let mut name_sets: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, func) in &program.funcs {
        // Slots reales de la función: params renombrados + keys planificadas
        let mut set: Vec<String> = func
            .params
            .iter()
            .map(|p| resolve_var(name, p))
            .collect();
        let plan = &var_plans[name];
        for k in plan.values() {
            if !set.iter().any(|x| x == k) {
                set.push(k.clone());
            }
        }
        name_sets.insert(name.clone(), set);
    }

    out.push_str("static void _init(void) {\n");
    for n in &names {
        out.push_str(&format!("  _reg(\"{}\");\n", esc(n)));
    }
    for (name, func) in &program.funcs {
        if !func.params.is_empty() {
            // Registry para CallValue dinámico: claves RENOMBRADAS de params
            let plist: Vec<String> = func
                .params
                .iter()
                .map(|p| format!("\"{}\"", esc(&resolve_var(name, p))))
                .collect();
            out.push_str(&format!(
                "  _regpars(\"{}\", (const char*[]){{ {} }}, {});\n",
                esc(name),
                plist.join(", "),
                func.params.len()
            ));
        }
    }
    out.push_str("}\n\n");

    out.push_str("/* noinline: evita que el optimizador mueva locales del llamador\n   a través de los puntos setjmp/longjmp de intentar/atrapar */\n");
    out.push_str("#if defined(__GNUC__) || defined(__clang__)\n#define LUMEN_NOINLINE __attribute__((noinline,noclone))\n#else\n#define LUMEN_NOINLINE\n#endif\n\n");
    for name in program.funcs.keys() {
        out.push_str(&format!(
            "static LUMEN_NOINLINE Val _f_{}(void);\n",
            mangle(name)
        ));
    }
    for n in &unknown {
        out.push_str(&format!(
            "static LUMEN_NOINLINE Val _f_{}(void);\n",
            mangle(n)
        ));
    }
    out.push_str("static Val _call_by_name(const char* nm);\n");
    out.push('\n');

    // Índice constante de cada registro (mismo orden que _init/_reg): evita
    // strcmp lineal en cada Load/Store del camino caliente.
    let mut name_idx: HashMap<String, usize> = HashMap::new();
    for (i, n) in names.iter().enumerate() {
        name_idx.insert(n.clone(), i);
    }
    let gv_of = |n: &str| -> String {
        match name_idx.get(n) {
            Some(i) => format!("gv[{}]", i),
            None => format!("gv[_fv(\"{}\")]", esc(n)),
        }
    };

    for (name, func) in &program.funcs {
        let plan = var_plans[name].clone();
        out.push_str(&emit_func(name, func, program, &name_sets, &gv_of, &renames, &plan));
    }
    for n in &unknown {
        out.push_str(&format!(
            "static LUMEN_NOINLINE Val _f_{}(void) {{ return _v_void(); }}\n\n",
            mangle(n)
        ));
    }

    let mut fnames: Vec<&String> = program.funcs.keys().collect();
    fnames.sort();
    out.push_str("static Val (*_lfn_ptrs[])(void) = {\n");
    for n in &fnames {
        out.push_str(&format!("  &_f_{},\n", mangle(n)));
    }
    out.push_str("};\n");
    out.push_str("static const char* _lfn_names[] = {\n");
    for n in &fnames {
        out.push_str(&format!("  \"{}\",\n", esc(n)));
    }
    out.push_str("};\n");
    out.push_str(&format!(
        "static Val _call_by_name(const char* nm) {{\n  for (int _i = 0; _i < {}; _i++) if (!strcmp(_lfn_names[_i], nm)) return _lfn_ptrs[_i]();\n  return _v_void();\n}}\n\n",
        fnames.len()
    ));

    let entry = if program.funcs.contains_key(&program.entry) {
        program.entry.clone()
    } else if program.funcs.contains_key("main") {
        "main".to_string()
    } else if program.funcs.contains_key("principal") {
        "principal".to_string()
    } else {
        String::new()
    };

    out.push_str("int main(void) {\n  _init();\n");
    if !entry.is_empty() {
        out.push_str(&format!("  (void)_f_{}();\n", mangle(&entry)));
        out.push_str("  if (_err) {\n    fprintf(stderr, \"%s\\n\", _last_err_msg ? _last_err_msg : \"Error\");\n    return 3;\n  }\n");
    }
    out.push_str("  return 0;\n}\n");
    out
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Plan de slots por instrucción: resuelve cada Load/Store/StoreLocal/
/// ArrayPushVar/MakeRef al key real del namespace plano gv[], considerando
/// params renombrados y SOMBRADO POR BLOQUES (ScopePush/ScopePop).
/// El planeo es estático sobre el flujo lineal: cada sitio textual de
/// `sea x` recibe su propio slot (`x#N`), así los bloques hermanos con el
/// mismo nombre no se pisan y las iteraciones de un bucle reusan su slot.
/// Fuente única de verdad para la colección de nombres, name_sets y el emisor.
fn plan_var_keys(
    func: &LumenFunc,
    param_renames: &HashMap<String, String>,
) -> HashMap<usize, String> {
    let mut plan: HashMap<usize, String> = HashMap::new();
    let mut scopes: Vec<HashMap<String, String>> = vec![HashMap::new()];
    for (raw, key) in param_renames {
        scopes[0].insert(raw.clone(), key.clone());
    }
    let mut counter = 0usize;
    fn resolve(scopes: &[HashMap<String, String>], n: &str) -> Option<String> {
        for s in scopes.iter().rev() {
            if let Some(k) = s.get(n) {
                return Some(k.clone());
            }
        }
        None
    }
    for (i, ins) in func.instrs.iter().enumerate() {
        match ins {
            Instr::ScopePush => scopes.push(HashMap::new()),
            Instr::ScopePop => {
                if scopes.len() > 1 {
                    scopes.pop();
                }
            }
            Instr::StoreLocal(n) => {
                // Declaración: siempre en el scope ACTUAL. Reusar key solo si
                // ya fue declarada en este mismo nivel; un nombre de un nivel
                // exterior se SOMBREA con key nueva.
                let top = scopes.last_mut().expect("scope base siempre presente");
                let key = match top.get(n) {
                    Some(k) => k.clone(),
                    None => {
                        counter += 1;
                        let k = format!("{}#{}", n, counter);
                        top.insert(n.clone(), k.clone());
                        k
                    }
                };
                plan.insert(i, key);
            }
            Instr::Load(n)
            | Instr::Store(n)
            | Instr::ArrayPushVar(n)
            | Instr::MakeRef(n) => {
                let key = resolve(&scopes, n).unwrap_or_else(|| n.to_string());
                plan.insert(i, key);
            }
            _ => {}
        }
    }
    plan
}

fn op_code(op: &Op) -> i64 {
    match op {
        Op::Add => 1,
        Op::Concat => 2,
        Op::Sub => 3,
        Op::Mul => 4,
        Op::Div => 5,
        Op::Mod => 6,
        Op::Equal => 7,
        Op::NotEqual => 8,
        Op::Less => 9,
        Op::LessEqual => 10,
        Op::Greater => 11,
        Op::GreaterEqual => 12,
        Op::And => 13,
        Op::Or => 14,
        Op::BitOr => 15,
        Op::BitAnd => 16,
        Op::ShiftLeft => 17,
        Op::ShiftRight => 18,
        Op::BitXor => 19,
        Op::Negate => 20,
        Op::Not => 21,
        Op::BitNot => 22,
    }
}

/// Simula el stack del código lineal para saber qué variables fueron pasadas
/// por referencia (MakeRef) a cada llamada. El backend C guarda/restaura los
/// globals del llamador (_sv) alrededor de cada llamada; los slots que son
/// objetivo de un prestado mut deben EXCLUIRSE del restore o la mutación se
/// desharía. Devuelve índice de instrucción Call -> nombres referenciados.
fn collect_ref_args(
    func: &LumenFunc,
    plan: &HashMap<usize, String>,
) -> HashMap<usize, Vec<String>> {
    let mut out: HashMap<usize, Vec<String>> = HashMap::new();
    let mut st: Vec<Option<String>> = Vec::new();
    let popn = |st: &mut Vec<Option<String>>, k: usize| {
        if st.len() < k {
            return false;
        }
        st.truncate(st.len() - k);
        true
    };
    for (idx, instr) in func.instrs.iter().enumerate() {
        match instr {
            Instr::ConstInt(_) | Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_)
            | Instr::Load(_) | Instr::Read | Instr::FuncRef(_) => st.push(None),
            Instr::MakeRef(n) => {
                // Usar el key planificado (con sombreado) para que la
                // exclusión del save/restore coincida con name_sets
                let key = plan.get(&idx).cloned().unwrap_or_else(|| n.clone());
                st.push(Some(key))
            }
            Instr::Binary(_) => {
                if !popn(&mut st, 2) {
                    break;
                }
                st.push(None);
            }
            Instr::Unary(_) | Instr::ArrayLen | Instr::TryUnwrap | Instr::MatchType(_)
            | Instr::MatchPayload | Instr::TupleAccess(_) | Instr::MatchVariant(_)
            | Instr::ResultOk | Instr::ResultErr | Instr::OptionSome | Instr::OptionNone => {
                if !popn(&mut st, 1) {
                    break;
                }
                st.push(None);
            }
            Instr::StructGet | Instr::ArrayGet => {
                if !popn(&mut st, 2) {
                    break;
                }
                st.push(None);
            }
            Instr::StructSet | Instr::ArraySet | Instr::ArrayPush => {
                if !popn(&mut st, 3) {
                    break;
                }
                st.push(None);
            }
            Instr::ArrayNew(n) | Instr::StructNew(_, n) | Instr::TupleNew(n) => {
                if !popn(&mut st, *n) {
                    break;
                }
                st.push(None);
            }
            Instr::EnumCtor { argc, .. } => {
                if !popn(&mut st, *argc) {
                    break;
                }
                st.push(None);
            }
            Instr::ArrayPushVar(_) => {
                if !popn(&mut st, 2) {
                    break;
                }
            }
            Instr::Store(_) | Instr::StoreLocal(_) | Instr::Print => {
                if !popn(&mut st, 1) {
                    break;
                }
            }
            Instr::Call(_, argc) => {
                if st.len() < *argc {
                    break;
                }
                let refs: Vec<String> = st[st.len() - argc..]
                    .iter()
                    .filter_map(|o| o.clone())
                    .collect();
                out.insert(idx, refs);
                st.truncate(st.len() - argc);
                st.push(None);
            }
            Instr::CallValue(argc) => {
                if !popn(&mut st, argc + 1) {
                    break;
                }
                st.push(None);
            }
            Instr::JmpIf(_) => {
                if !popn(&mut st, 1) {
                    break;
                }
            }
            Instr::Return | Instr::Halt | Instr::Jmp(_) | Instr::Label(_)
            | Instr::ScopePush | Instr::ScopePop | Instr::PushHandler(_)
            | Instr::PopHandler | Instr::Nop | Instr::Phi(_, _) => {}
        }
    }
    out
}

fn emit_func(
    name: &str,
    func: &LumenFunc,
    program: &Program,
    name_sets: &BTreeMap<String, Vec<String>>,
    gv_of: &dyn Fn(&str) -> String,
    renames: &HashMap<String, HashMap<String, String>>,
    plan: &HashMap<usize, String>,
) -> String {
    // Slot de un param de una función llamada (callee)
    let callee_slot_of = |callee: &str, pn: &str| -> String {
        renames
            .get(callee)
            .and_then(|m| m.get(pn))
            .cloned()
            .unwrap_or_else(|| pn.to_string())
    };
    let mut s = String::new();
    s.push_str(&format!(
        "static LUMEN_NOINLINE Val _f_{}(void) {{\n",
        mangle(name)
    ));

    let ref_args = collect_ref_args(func, plan);
    // Resolvedor de slot por instrucción (params renombrados + sombreado)
    let var_at = |i: usize, n: &str| -> String {
        plan.get(&i).cloned().unwrap_or_else(|| n.to_string())
    };
    let mut handler_labels: Vec<usize> = Vec::new();
    for (i, instr) in func.instrs.iter().enumerate() {
        // ¿Instrucción cuyas llamadas al runtime pueden lanzar error?
        let risky = matches!(
            instr,
            Instr::Binary(_)
                | Instr::Unary(_)
                | Instr::ArrayGet
                | Instr::ArraySet
                | Instr::ArrayPush
                | Instr::StructGet
                | Instr::StructSet
                | Instr::TryUnwrap
                | Instr::MatchPayload
            | Instr::Call(_, _)
            | Instr::CallValue(_)
    );
    match instr {
            Instr::ConstInt(n) => s.push_str(&format!("  PUSH(_v_int({}));\n", n)),
            Instr::ConstFloat(f) => {
                if f.is_finite() {
                    s.push_str(&format!("  PUSH(_v_flt({:?}));\n", f));
                } else if f.is_nan() {
                    s.push_str("  PUSH(_v_flt(0.0/0.0));\n");
                } else if *f > 0.0 {
                    s.push_str("  PUSH(_v_flt(INFINITY));\n");
                } else {
                    s.push_str("  PUSH(_v_flt(-INFINITY));\n");
                }
            }
            Instr::ConstBool(b) => {
                s.push_str(&format!("  PUSH(_v_bool({}));\n", if *b { 1 } else { 0 }));
            }
            Instr::ConstStr(x) => {
                s.push_str(&format!(
                    "  PUSH(_v_str(\"{}\"));\n",
                    x.replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t")
                ));
            }
            Instr::Load(n) => {
                s.push_str(&format!("  PUSH(_deref({}));\n", gv_of(&var_at(i, n))));
            }
            Instr::Store(n) | Instr::StoreLocal(n) => {
                // Si el slot contiene una referencia (prestado mut), escribir
                // a través del puntero; si no, asignación normal.
                let g = gv_of(&var_at(i, n));
                s.push_str(&format!(
                    "  {{ Val _sv_ = POP(); if ({g}.t == T_PTR && {g}.p) *{g}.p = _dcp(_sv_); else {g} = _dcp(_sv_); }}\n",
                    g = g
                ));
            }
            Instr::Binary(op) => {
                let code = op_code(op);
                s.push_str(&format!(
                    "  {{ Val _b = POP(); Val _a = POP(); PUSH(_bin({}, _a, _b)); }}\n",
                    code
                ));
            }
            Instr::Unary(op) => {
                match op {
                    Op::Negate => s.push_str("  PUSH(_neg(POP()));\n"),
                    Op::Not => s.push_str("  PUSH(_not(POP()));\n"),
                    Op::BitNot => s.push_str("  PUSH(_bnot(POP()));\n"),
                    _ => s.push_str("  PUSH(_neg(POP()));\n"),
                }
            }
            Instr::Call(n, argc) => {
                if n == "imprimir" || n == "print" {
                    if *argc > 0 {
                        // Fuzzing 3.3.6: concatenar argumentos en UNA línea
                        // (paridad con VM: imprimir("a:", x) => "a:<x>")
                        s.push_str(&format!(
                            "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); char* _cmb = (char*)malloc(65536); _cmb[0] = 0; for (int _k2 = 0; _k2 < {}; _k2++) {{ char* _p = _fmt(_t[_k2]); strcat(_cmb, _p); free(_p); }} printf(\"%s\\n\", _cmb); free(_cmb); }}\n",
                            argc, argc, argc
                        ));
                    } else {
                        // Sin argumentos: línea vacía (paridad con VM)
                        s.push_str("  printf(\"\\n\");\n");
                    }
                    s.push_str("  PUSH(_v_void());\n");
                } else if n == "leer" || n == "read" {
                    for _ in 0..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    s.push_str("  PUSH(_read_ln());\n");
                } else if n == "a_texto" || n == "to_texto" || n == "__str_from" {
                    if *argc > 0 {
                        s.push_str(&format!(
                            "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); PUSH(_v_str(_fmt(_t[0]))); }}\n",
                            argc, argc
                        ));
                    } else {
                        s.push_str("  PUSH(_v_str(\"\"));\n");
                    }
                } else if n == "agregar" || n == "push" {
                    s.push_str("  { Val _x = POP(); Val _a = POP(); PUSH(_arr_push(_a, _x)); }\n");
                } else if n == "largo" || n == "len" || n == "__str_len" || n == "__str_longitud" {
                    s.push_str("  { Val _x = POP(); if (_x.t == T_ARR || _x.t == T_TUP || _x.t == T_MAP) PUSH(_v_int(_x.argc)); else if (_x.t == T_STR) PUSH(_v_int((int64_t)strlen(_x.s))); else PUSH(_v_int(0)); }\n");
                } else if n == "__tipo_de" || n == "__typeof" {
                    s.push_str("  { Val _x = POP(); PUSH(_v_str(_tipo_de_b(_x))); }\n");
                } else if n == "__ffi_asm" {
                    s.push_str("  { Val _code = POP(); __asm__ volatile(\"nop\"); PUSH(_v_int(0)); }\n");
                } else if n == "__ffi_c_eval" || n == "__ffi_rust_eval" {
                    s.push_str("  { Val _code = POP(); PUSH(_v_int(0)); }\n");
                } else if n == "__map_nuevo" {
                    s.push_str("  PUSH(_map_new());\n");
                } else if n == "__map_poner" {
                    s.push_str("  { Val _x = POP(); Val _k = POP(); Val _m = POP(); PUSH(_map_set(_m, _k, _x)); }\n");
                } else if n == "__map_obtener" {
                    s.push_str("  { Val _k = POP(); Val _m = POP(); PUSH(_map_get(_m, _k)); }\n");
                } else if n == "__map_contiene" {
                    s.push_str("  { Val _k = POP(); Val _m = POP(); PUSH(_map_has(_m, _k)); }\n");
                } else if n == "__map_longitud" {
                    s.push_str("  { Val _m = POP(); PUSH(_map_len(_m)); }\n");
                } else if n == "__map_claves" || n == "__map_keys" {
                    s.push_str("  { Val _m = POP(); PUSH(_map_keys(_m)); }\n");
                } else if n == "__lista_invertir" || n == "__list_reverse" {
                    s.push_str("  { Val _a = POP(); PUSH(_arr_rev(_a)); }\n");
                } else if n == "__lista_ordenar" || n == "__list_sort" {
                    s.push_str("  { Val _a = POP(); PUSH(_arr_sort(_a)); }\n");
                } else if n == "__conjunto_nuevo" || n == "__set_new" {
                    s.push_str("  PUSH(_set_new());\n");
                } else if n == "__conjunto_agregar" || n == "__set_add" {
                    s.push_str("  { Val _x = POP(); Val _s = POP(); PUSH(_set_add(_s, _x)); }\n");
                } else if n == "__conjunto_tiene" || n == "__set_has" {
                    s.push_str("  { Val _x = POP(); Val _s = POP(); PUSH(_set_has(_s, _x)); }\n");
                } else if n == "__conjunto_unir" || n == "__set_union" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_set_union(_a, _b)); }\n");
                } else if n == "__conjunto_interseccion" || n == "__set_inter" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_set_inter(_a, _b)); }\n");
                } else if n == "__conjunto_diferencia" || n == "__set_diff" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_set_diff(_a, _b)); }\n");
                } else if n == "__deque_nuevo" {
                    s.push_str("  PUSH(_set_new());\n");
                } else if n == "__deque_agregar_final" || n == "__deque_agregar_frente" {
                    s.push_str(&format!(
                        "  {{ Val _x = POP(); Val _d = POP(); PUSH({}(_d, _x)); }}\n",
                        if n.ends_with("frente") { "_push_front" } else { "_arr_push" }
                    ));
                } else if n == "__deque_quitar_frente" {
                    s.push_str("  { Val _d = POP(); PUSH(_pop_front(_d)); }\n");
                } else if n == "__deque_quitar_final" {
                    s.push_str("  { Val _d = POP(); PUSH(_pop_back(_d)); }\n");
                } else if n == "__deque_longitud" {
                    s.push_str("  { Val _d = POP(); PUSH(_v_int(_d.argc)); }\n");
                } else if n == "__monticulo_nuevo" {
                    s.push_str("  PUSH(_set_new());\n");
                } else if n == "__monticulo_agregar" {
                    s.push_str("  { Val _x = POP(); Val _h = POP(); PUSH(_heap_agregar(_h, _x)); }\n");
                } else if n == "__monticulo_ver" {
                    s.push_str("  { Val _h = POP(); PUSH(_h.argc > 0 ? _h.items[0] : _v_void()); }\n");
                } else if n == "__monticulo_quitar" {
                    s.push_str("  { Val _h = POP(); PUSH(_pop_front(_h)); }\n");
                } else if n == "__monticulo_longitud" {
                    s.push_str("  { Val _h = POP(); PUSH(_v_int(_h.argc)); }\n");
                } else if n == "__enlazada_nuevo" {
                    s.push_str("  PUSH(_set_new());\n");
                } else if n == "__enlazada_agregar_final" || n == "__enlazada_agregar_frente" {
                    s.push_str(&format!(
                        "  {{ Val _x = POP(); Val _l = POP(); PUSH({}(_l, _x)); }}\n",
                        if n.ends_with("frente") { "_push_front" } else { "_arr_push" }
                    ));
                } else if n == "__enlazada_quitar_frente" {
                    s.push_str("  { Val _l = POP(); PUSH(_pop_front(_l)); }\n");
                } else if n == "__enlazada_quitar_final" {
                    s.push_str("  { Val _l = POP(); PUSH(_pop_back(_l)); }\n");
                } else if n == "__enlazada_longitud" {
                    s.push_str("  { Val _l = POP(); PUSH(_v_int(_l.argc)); }\n");
                } else if n == "__regex_coincide" {
                    s.push_str("  { Val _s = POP(); Val _p = POP(); PUSH(_regex_m_val(_p.s, _s.s)); }\n");
                } else if n == "__sistema_pid" || n == "__process_pid" {
                    s.push_str("  PUSH(_v_int(_rt_pid()));\n");
                } else if n == "__regex_capturar" || n == "__regex_captures" {
                    s.push_str("  { Val _s = POP(); Val _p = POP(); PUSH(_regex_caps(_p.s, _s.s)); }\n");
                } else if n == "__regex_reemplazar" {
                    s.push_str("  { Val _r = POP(); Val _t = POP(); Val _p = POP(); PUSH(_v_str(_regex_rep(_p.s, _t.s, _r.s))); }\n");
                } else if n == "__unicode_normalizar" {
                    if *argc > 1 {
                        s.push_str("  { Val _f = POP(); Val _s = POP(); PUSH(_v_str(_norm(_s.s, _f.s))); }\n");
                    } else {
                        s.push_str("  { Val _s = POP(); PUSH(_v_str(_norm(_s.s, \"NFC\"))); }\n");
                    }
                } else if n == "__str_padding_inicio" {
                    s.push_str("  { Val _f = POP(); Val _w = POP(); Val _s = POP(); PUSH(_v_str(_pad_str(_s.s, _w.i, _f.s, 1))); }\n");
                } else if n == "__str_padding_fin" {
                    s.push_str("  { Val _f = POP(); Val _w = POP(); Val _s = POP(); PUSH(_v_str(_pad_str(_s.s, _w.i, _f.s, 0))); }\n");
                } else if n == "__codificacion_utf8" {
                    s.push_str("  { Val _s = POP(); PUSH(_utf8_bytes(_s.s)); }\n");
                } else if n == "__escritor_buffer" {
                    s.push_str("  { Val _c = POP(); Val _p = POP(); PUSH(_buf_write(_p.s, _c.s)); }\n");
                } else if n == "__fs_listar" || n == "__fs_listdir" {
                    s.push_str("  { Val _p = POP(); PUSH(_fs_list(_p.s)); }\n");
                } else if n == "__env_listar" || n == "__env_list" {
                    s.push_str("  PUSH(_env_list());\n");
                } else if n == "__tiempo_ahora" {
                    s.push_str("  PUSH(_time_now());\n");
                } else if n == "__tiempo_formatear" || n == "__time_format" {
                    if *argc > 1 {
                        s.push_str("  { Val _f = POP(); Val _t = POP(); PUSH(_v_str(_time_fmt(_t.i, _f.s))); }\n");
                    } else {
                        s.push_str("  { Val _t = POP(); PUSH(_v_str(_time_fmt(_t.i, \"\"))); }\n");
                    }
                } else if n == "__tiempo_diferencia" || n == "__time_diff" {
                    s.push_str("  { Val _t2 = POP(); Val _t1 = POP(); int64_t _da = _t1.i - _t2.i; if (_da < 0) _da = -_da; PUSH(_v_int(_da)); }\n");
                } else if n == "__tarea_lanzar" {
                    s.push_str("  { Val _nm = POP(); PUSH(_v_str(_nm.s)); }\n");
                } else if n == "__tarea_esperar" {
                    s.push_str("  { Val _id = POP(); PUSH(_call_by_name(_id.s)); }\n");
                } else if n == "__coro_crear" || n == "__coro_create" {
                    let na = if *argc > 0 { *argc - 1 } else { 0 };
                    s.push_str(&format!(
                        "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); Val _nm = POP();\n",
                        na, na
                    ));
                    s.push_str(&format!(
                        "    int _pc = _parcnt(_nm.s); if (_pc > {}) _pc = {};\n",
                        na, na
                    ));
                    s.push_str("    for (int _k2 = 0; _k2 < _pc; _k2++) gv[_fv(_par(_nm.s, _k2))] = _dcp(_t[_k2]);\n");
                    s.push_str("    PUSH(_coro_create(_nm.s)); }\n");
                } else if n == "__coro_ceder" || n == "__coro_yield" {
                    s.push_str("  PUSH(_coro_cede());\n");
                } else if n == "__coro_reanudar" || n == "__coro_resume" {
                    s.push_str("  { Val _id = POP(); PUSH(_coro_resume(_id.s)); }\n");
                } else if n == "__hash_sha256" {
                    s.push_str("  { Val _s = POP(); PUSH(_v_str(_hash_hex(_s.s, 256))); }\n");
                } else if n == "__hash_sha512" {
                    s.push_str("  { Val _s = POP(); PUSH(_v_str(_hash_hex(_s.s, 512))); }\n");
                } else if n == "__json_parsear" {
                    s.push_str("  { Val _s = POP(); PUSH(_json_parse(_s.s)); }\n");
                } else if n == "__json_texto" {
                    s.push_str("  { Val _x = POP(); PUSH(_v_str(_json_text(_x))); }\n");
                } else if n == "__str_mayusculas" || n == "__str_upper" {
                    s.push_str("  { Val _s = POP(); PUSH(_v_str(_case_str(_s.s, 1))); }\n");
                } else if n == "__str_minusculas" || n == "__str_lower" {
                    s.push_str("  { Val _s = POP(); PUSH(_v_str(_case_str(_s.s, 0))); }\n");
                } else if n == "__str_contiene" {
                    s.push_str("  { Val _n = POP(); Val _h = POP(); PUSH(_v_bool(strstr(_h.s, _n.s) != NULL)); }\n");
                } else if n == "__str_split" || n == "__str_dividir" {
                    s.push_str("  { Val _d = POP(); Val _t = POP(); PUSH(_str_split(_t.s, _d.s)); }\n");
                } else if n == "__str_recortar" || n == "__str_trim" {
                    s.push_str("  { Val _s = POP(); PUSH(_v_str(_trim(_s.s))); }\n");
                } else if n == "__str_subcadena" || n == "__str_slice" {
                    s.push_str("  { Val _e = POP(); Val _st = POP(); Val _s = POP(); PUSH(_v_str(_sub(_s.s, _st.i, _e.i))); }\n");
                } else if n == "__str_to_chars" || n == "__str_a_caracteres" {
                    s.push_str("  { Val _s = POP(); PUSH(_to_chars(_s.s)); }\n");
                } else if n == "__str_empieza_con" || n == "__str_starts_with" {
                    s.push_str("  { Val _p = POP(); Val _s = POP(); PUSH(_v_bool(strncmp(_s.s, _p.s, strlen(_p.s)) == 0)); }\n");
                } else if n == "__str_codigo" || n == "__str_ord" {
                    s.push_str("  { Val _s = POP(); PUSH(_str_codes(_s.s)); }\n");
                } else if n == "__str_reemplazar" || n == "__str_replace" {
                    s.push_str("  { Val _t = POP(); Val _f = POP(); Val _s = POP(); PUSH(_v_str(_replace(_s.s, _f.s, _t.s))); }\n");
                } else if n == "__str_concat_list" || n == "__str_concatenar_lista" {
                    s.push_str("  { Val _l = POP(); PUSH(_concat_list(_l)); }\n");
                } else if n == "__tiempo_parsear" || n == "__tiempo_parse" {
                    s.push_str("  { Val _f = POP(); Val _d = POP(); (void)_f; PUSH(_v_int(_time_parse(_d.s))); }\n");
                } else if n == "__ffi_cargar" {
                    s.push_str("  { (void)POP(); PUSH(_v_int(1)); }\n");
                } else if n == "__ffi_asignar" {
                    s.push_str("  { Val _n = POP(); PUSH(_v_int(_ffi_ptr_alloc((size_t)_n.i))); }\n");
                } else if n == "__ffi_peek" {
                    s.push_str("  { Val _o = POP(); Val _p = POP(); intptr_t _pp = (intptr_t)_p.i + _o.i; unsigned _v0 = *(unsigned char*)_pp | ((unsigned)*(unsigned char*)(_pp+1) << 8) | ((unsigned)*(unsigned char*)(_pp+2) << 16) | ((unsigned)*(unsigned char*)(_pp+3) << 24); PUSH(_v_int((int64_t)_v0)); }\n");
                } else if n == "__ffi_poke" {
                    s.push_str("  { Val _w = POP(); Val _o = POP(); Val _p = POP(); intptr_t _pp = (intptr_t)_p.i + _o.i; *(unsigned char*)_pp = (unsigned char)_w.i; *(unsigned char*)(_pp+1) = (unsigned char)(_w.i >> 8); *(unsigned char*)(_pp+2) = (unsigned char)(_w.i >> 16); *(unsigned char*)(_pp+3) = (unsigned char)(_w.i >> 24); PUSH(_v_void()); }\n");
                } else if n == "__ffi_liberar" {
                    s.push_str("  { Val _n = POP(); Val _p = POP(); (void)_n; free((void*)(intptr_t)_p.i); PUSH(_v_void()); }\n");
                } else if n == "__ffi_leer" {
                    s.push_str("  { Val _l = POP(); Val _o = POP(); Val _p = POP(); char* _m = (char*)malloc((size_t)_l.i + 1); memcpy(_m, (void*)(intptr_t)_p.i + _o.i, (size_t)_l.i); _m[_l.i] = 0; PUSH(_v_str(_m)); }\n");
                } else if n == "__ffi_llamar" || n == "__ffi_call" {
                    s.push_str("  { Val _rt = POP(); Val _ar = POP(); Val _tp = POP(); Val _nm = POP(); Val _hc = POP(); (void)_tp; PUSH(_ffi_call(_hc, _nm.s, _ar, _rt.s)); }\n");
                } else if let Some(callee) = program.funcs.get(n) {
                    let plen = callee.params.len().min(*argc);
                    // Excluir del save/restore _sv los slots que esta llamada
                    // recibe por referencia: la mutación del callee debe persistir.
                    let excluded: Vec<String> = ref_args
                        .get(&i)
                        .cloned()
                        .unwrap_or_default()
                        .iter()
                        .map(|rn| rn.clone())
                        .collect();
                    let caller_names: Vec<String> =
                        name_sets.get(name).cloned().unwrap_or_default();
                    let caller_names: Vec<String> = caller_names
                        .into_iter()
                        .filter(|cn| !excluded.contains(cn))
                        .collect();
                    let mut pre = String::new();
                    let mut post = String::new();
                    if !caller_names.is_empty() {
                        pre.push_str("  { Val _sv[");
                        pre.push_str(&caller_names.len().to_string());
                        pre.push_str("];\n");
                        for (i, cn) in caller_names.iter().enumerate() {
                            pre.push_str(&format!("    _sv[{}] = {};\n", i, gv_of(cn)));
                        }
                        post.push_str("    ");
                        for (i, cn) in caller_names.iter().enumerate() {
                            post.push_str(&format!("{} = _sv[{}]; ", gv_of(cn), i));
                        }
                        post.push_str("}\n");
                    }
                    s.push_str(&pre);
                    for i in (0..plen).rev() {
                        s.push_str(&format!(
                            "  {} = _dcp(POP());\n",
                            gv_of(&callee_slot_of(n, &callee.params[i]))
                        ));
                    }
                    for _ in plen..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    s.push_str(&format!("  {{ Val _r = _f_{}(); PUSH(_r); }}\n", mangle(n)));
                    s.push_str(&post);
                } else {
                    for _ in 0..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    s.push_str(&format!("  {{ Val _r = _f_{}(); PUSH(_r); }}\n", mangle(n)));
                }
            }
            Instr::FuncRef(n) => {
                s.push_str(&format!(
                    "  PUSH(_vfref(\"{}\", &_f_{}));\n",
                    esc(n),
                    mangle(n)
                ));
            }
            Instr::CallValue(argc) => {
                if *argc == 0 {
                    s.push_str(
                        "  { Val _cf = POP(); if (!strcmp(_cf.s, \"largo\") || !strcmp(_cf.s, \"len\") || !strcmp(_cf.s, \"__str_longitud\")) { PUSH(_v_int(0)); } else { Val _r = _fref_call(_cf); PUSH(_r); } }\n",
                    );
                } else {
                    s.push_str(&format!(
                        "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); Val _cf = POP();\n",
                        argc, argc
                    ));
                    s.push_str("    if (!strcmp(_cf.s, \"imprimir\") || !strcmp(_cf.s, \"print\")) {\n");
                    for i in 0..*argc {
                        s.push_str(&format!("      printf(\"%s\\n\", _fmt(_t[{}]));\n", i));
                    }
                    s.push_str("      PUSH(_v_void());\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"leer\") || !strcmp(_cf.s, \"read\")) {\n      PUSH(_read_ln());\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"a_texto\") || !strcmp(_cf.s, \"to_texto\") || !strcmp(_cf.s, \"__str_from\")) {\n      PUSH(_v_str(_fmt(_t[0])));\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"agregar\") || !strcmp(_cf.s, \"push\")) {\n      PUSH(_arr_push(_t[0], _t[1]));\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"largo\") || !strcmp(_cf.s, \"len\") || !strcmp(_cf.s, \"__str_len\") || !strcmp(_cf.s, \"__str_longitud\")) {\n      PUSH(_v_int(_t[0].t == T_STR ? (int64_t)strlen(_t[0].s) : _t[0].argc));\n");
                    s.push_str("    } else {\n");
                    for i in 0..*argc {
                        s.push_str(&format!(
                            "      gv[_fv(_par(_cf.s, {}))] = _dcp(_t[{}]);\n",
                            i, i
                        ));
                    }
                    s.push_str("      { Val _r = _fref_call(_cf); PUSH(_r); }\n");
                    s.push_str("    }\n  }\n");
                }
            }
            Instr::Return => s.push_str("  return POP();\n"),
            Instr::Print => s.push_str("  printf(\"%s\\n\", _fmt(POP()));\n"),
            Instr::Read => s.push_str("  PUSH(_read_ln());\n"),
            Instr::ArrayNew(n) => {
                s.push_str(&format!(
                    "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); PUSH(_arrn(_t, {})); }}\n",
                    n, n, n
                ));
            }
            Instr::ArrayPush => s.push_str("  { Val _x = POP(); Val _a = POP(); PUSH(_arr_push(_a, _x)); }\n"),
            Instr::MakeRef(vname) => {
                // prestado mut (bug #6): apilar puntero al slot gv[] de la variable.
                // El slot gv es estático → la dirección es estable durante todo el run.
                s.push_str(&format!("  PUSH(_v_ptr(&{}));\n", gv_of(&var_at(i, vname))));
            }
            Instr::PushHandler(catch_label) => {
                // intentar/atrapar sin unwinding: guardar SP; los chequeos
                // _ERRCHK posteriores usan la etiqueta estática del catch.
                handler_labels.push(*catch_label);
                s.push_str("  { _h_sp[_hn] = SP; _hn++; }\n");
            }
            Instr::PopHandler => {
                s.push_str("  _try_end();\n");
            }
            Instr::ScopePush | Instr::ScopePop => {
                // Limitación documentada: el backend C usa un namespace plano de
                // variables; el sombreado por bloques sigue la semántica del VM
                // solo si los nombres no colisionan entre bloques hermanos.
            }
            Instr::MatchVariant(vname) => {
                // Destructuring de enums en elegir: comparar solo el variant.
                s.push_str(&format!(
                    "  {{ Val _mv = _deref(POP()); PUSH(_v_bool(_mv.t == T_ENM && _mv.vr && !strcmp(_mv.vr, \"{}\"))); }}\n",
                    esc(vname)
                ));
            }
            Instr::ArrayPushVar(vname) => {
                // AOT: push + store al slot (con guard de refs prestado mut)
                let vn = gv_of(&var_at(i, vname));
                s.push_str("  { Val _x = POP(); Val _a = POP(); PUSH(_arr_push(_a, _x)); }\n");
                s.push_str(&format!(
                    "  {{ Val _sv_ = POP(); if ({vn}.t == T_PTR && {vn}.p) *{vn}.p = _dcp(_sv_); else {vn} = _dcp(_sv_); }}\n",
                    vn = vn
                ));
            }
            Instr::ArrayGet => s.push_str("  { Val _i = POP(); Val _a = POP(); PUSH(_arr_get(_a, _i.i)); }\n"),
            Instr::ArraySet => {
                s.push_str("  { Val _x = POP(); Val _i = POP(); Val _a = POP(); PUSH(_arr_set(_a, _i.i, _x)); }\n");
            }
            Instr::ArrayLen => s.push_str("  PUSH(_arr_len(POP()));\n"),
            Instr::StructNew(sn, n) => {
                s.push_str(&format!(
                    "  {{ const char* _ns[{}]; for (int _k = {} - 1; _k >= 0; _k--) {{ Val _nv = POP(); _ns[_k] = _nv.s; }}\n",
                    n, n
                ));
                s.push_str(&format!(
                    "    Val _vs[{}]; for (int _k = {} - 1; _k >= 0; _k--) _vs[_k] = POP();\n",
                    n, n
                ));
                s.push_str(&format!(
                    "    PUSH(_st_new(\"{}\", {}, _vs, _ns)); }}\n",
                    esc(sn),
                    n
                ));
            }
            Instr::StructGet => {
                s.push_str("  { Val _f = POP(); Val _s = POP(); PUSH(_st_get(_s, _f.s)); }\n");
            }
            Instr::StructSet => {
                s.push_str("  { Val _x = POP(); Val _f = POP(); Val _s = POP(); PUSH(_st_set(_s, _f.s, _x)); }\n");
            }
            Instr::ResultOk => s.push_str("  PUSH(_res(POP(), 1));\n"),
            Instr::ResultErr => s.push_str("  PUSH(_res(POP(), 0));\n"),
            Instr::TryUnwrap => s.push_str(
                "  { Val _u = POP(); if (_u.t == T_OK) { PUSH(_u.items[0]); } else { return _u; } }\n",
            ),
            Instr::OptionSome => s.push_str("  PUSH(_some(POP()));\n"),
            Instr::OptionNone => s.push_str("  PUSH(_none());\n"),
            Instr::MatchType(k) => {
                let test = match *k {
                    0 => "_u.t == T_SOM",
                    1 => "_u.t == T_OK",
                    2 => "_u.t == T_ERR",
                    _ => "0",
                };
                s.push_str(&format!(
                    "  {{ Val _u = POP(); PUSH(_v_bool({})); }}\n",
                    test
                ));
            }
            Instr::MatchPayload => {
                s.push_str("  { Val _u = POP(); if (_u.t == T_SOM || _u.t == T_OK || _u.t == T_ERR) { PUSH(_u.items[0]); } else { PUSH(_u); } }\n");
            }
            Instr::TupleNew(n) => {
                s.push_str(&format!(
                    "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); PUSH(_tupn(_t, {})); }}\n",
                    n, n, n
                ));
            }
            Instr::TupleAccess(i) => {
                s.push_str(&format!("  PUSH(_arr_get(POP(), {}));\n", i));
            }
            Instr::EnumCtor {
                enum_name,
                variant,
                argc,
            } => {
                s.push_str(&format!(
                    "  {{ Val _a[{}]; for (int _k = {} - 1; _k >= 0; _k--) _a[_k] = POP(); PUSH(_enm(\"{}\", \"{}\", {}, _a)); }}\n",
                    argc, argc, esc(enum_name), esc(variant), argc
                ));
            }
            Instr::Jmp(t) => s.push_str(&format!("  goto L_{};\n", t)),
            Instr::JmpIf(t) => s.push_str(&format!("  if (!_truthy(POP())) goto L_{};\n", t)),
            Instr::Label(t) => s.push_str(&format!("  L_{}:;\n", t)),
            Instr::Phi(..) | Instr::Nop | Instr::Halt => {
                if let Instr::Halt = instr {
                    s.push_str("  return _v_void();\n");
                }
            }
        }
        // Chequeo de error tras operaciones riesgosas (intentar/atrapar sin
        // unwinding): salta al catch abierto más cercano o propaga al llamador.
        if risky {
            match handler_labels.last() {
                Some(l) => s.push_str(&format!(
                    "  if (__builtin_expect(_err,0)) {{ _hn--; SP = _h_sp[_hn]; PUSH(_v_str(_last_err_msg)); _err = 0; goto L_{}; }}\n",
                    l
                )),
                None => s.push_str("  if (__builtin_expect(_err,0)) return _v_void();\n"),
            }
        }
    }
    s.push_str("}\n\n");
    s
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
                defaults: vec![],
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
                defaults: vec![],
                entry: 0,
                instrs: vec![Instr::ConstInt(1), Instr::Return],
            },
        );
        program.funcs.insert(
            "dead".into(),
            Func {
                name: "dead".into(),
                params: vec![],
                defaults: vec![],
                entry: 0,
                instrs: vec![Instr::ConstInt(0), Instr::Return],
            },
        );
        // AOT only compiles entry-reachable functions
        let compiler = AotCompiler::new();
        let product = compiler.compile(&program);
        assert!(product.object.write().is_ok());
    }

    fn sample_program(name: &str) -> Program {
        let mut program = Program::new();
        program.entry = name.to_string();
        program.funcs.insert(
            name.to_string(),
            Func {
                name: name.to_string(),
                params: vec![],
                defaults: vec![],
                entry: 0,
                instrs: vec![
                    Instr::ConstInt(40),
                    Instr::ConstInt(2),
                    Instr::Binary(Op::Add),
                    Instr::Print,
                    Instr::Halt,
                ],
            },
        );
        program
    }

    #[test]
    fn test_c_backend_structural() {
        let program = sample_program("main");
        let c = compile_to_c(&program);
        assert!(c.contains("static LUMEN_NOINLINE Val _f_main(void)"));
        assert!(c.contains("int main(void)"));
        assert!(c.contains("PUSH(_v_int(40))"));
        assert!(c.contains("_bin(1, _a, _b)"));
        assert!(c.contains("printf(\"%s\\n\", _fmt(POP()))"));
    }

    #[test]
    fn test_c_backend_gcc_runtime() {
        // Requiere gcc disponible (Linux CI o MSYS2 en Windows)
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        // Requiere un gcc FUNCIONAL: bajo los hooks de git el PATH puede
        // resolver a un gcc roto; se prueba con un programa trivial.
        let probe_dir = std::env::temp_dir().join("lumen_gcc_probe");
        std::fs::create_dir_all(&probe_dir).unwrap();
        let probe_c = probe_dir.join("p.c");
        let probe_exe = probe_dir.join("p.exe");
        std::fs::write(&probe_c, "int main(void){return 0;}\n").unwrap();
        let _ = std::fs::remove_file(&probe_exe);
        match std::process::Command::new("gcc")
            .args([
                probe_c.to_str().unwrap(),
                "-o",
                probe_exe.to_str().unwrap(),
            ])
            .output()
        {
            Ok(o) if o.status.success() && probe_exe.exists() => {}
            _ => return, // gcc ausente o roto en este entorno: omitir test
        }
        let source = r#"
            estructura Punto { x: entero, y: entero }
            funcion vacio subir(prestado mut Punto p, entero d) {
                p.y = p.y + d;
            }
            funcion entero fib(n: entero) {
                si (n < 2) { retornar n; }
                retornar fib(n - 1) + fib(n - 2);
            }
            funcion texto color(texto c) {
                elegir (c) {
                    caso "rojo": { retornar "R"; }
                    defecto: { retornar "?"; }
                }
            }
            funcion vacio main() {
                sea p = Punto { x: 1, y: 2 };
                subir(p, 40);
                imprimir(p.y);
                intentar {
                    sea xs = [1];
                    imprimir(xs[5]);
                } atrapar (e) {
                    imprimir("atrapado");
                }
                sea f = comptime { fib(10) };
                imprimir(f);
                imprimir(color("rojo"));
                sea a = 20;
                sea b = 22;
                imprimir(a + b);
                // Fuzzing 3.3.6: indexado y largo de textos en el nativo
                sea s = "abc";
                imprimir(s[1]);
                imprimir(s.largo());
                imprimir("x:", s[2]);
            }
        "#;
        let tokens = lumen_lexer::Lexer::new(source).tokenize();
        let (mut program, _) = lumen_parser::Parser::new(tokens.0).parse();
        let sem_errors = lumen_sema::SemanticAnalyzer::new().analyze(&mut program);
        assert!(sem_errors.is_empty(), "sema fallo: {:?}", sem_errors);
        lumen_ir::comptime::ComptimeEvaluator::new(&program)
            .rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        let c = compile_to_c(&ir);
        // Dir único por corrida: el pid se reusa entre ejecuciones del binario
        // de tests y el antivirus puede bloquear un .exe recién escrito.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "lumen_aot_test_{}_{}",
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let c_path = dir.join("test_full.c");
        let exe_path = dir.join("test_full.exe");
        std::fs::write(&c_path, c).unwrap();
        let status = std::process::Command::new("gcc")
            .arg(c_path.to_str().unwrap())
            .args(["-O2", "-o", exe_path.to_str().unwrap(), "-lm"])
            .output();
        let status = status.unwrap_or_else(|e| panic!("gcc fallo al invocar: {:?}", e));
        if !status.status.success() {
            panic!(
                "gcc fallo al compilar:\n{}",
                String::from_utf8_lossy(&status.stderr)
            );
        }
        let out = loop {
            match std::process::Command::new(&exe_path).output() {
                Ok(o) => break o,
                Err(_) => {
                    // Retry: el AV de Windows puede bloquear el .exe recién creado
                    std::thread::sleep(std::time::Duration::from_millis(300));
                }
            }
        };
        let test_out = String::from_utf8_lossy(&out.stdout)
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            test_out,
            vec!["42", "atrapado", "55", "R", "42", "b", "3", "x:c"],
            "salida completa: {:?}",
            test_out
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_jit_engine_execution() {
        let mut jit = JitEngine::new().expect("Failed to initialize JIT engine");
        let func = Func {
            name: "jit_add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            defaults: vec![None, None],
            entry: 0,
            instrs: vec![
                Instr::ConstInt(30),
                Instr::ConstInt(12),
                Instr::Binary(Op::Add),
                Instr::Return,
            ],
        };
        let code_ptr = jit
            .compile_function("jit_add", &func)
            .expect("JIT compile failed");
        assert!(!code_ptr.is_null());
        let callable: fn(i64, i64) -> i64 = unsafe { std::mem::transmute(code_ptr) };
        let res = callable(0, 0);
        assert_eq!(res, 42);
    }

    #[test]
    fn test_llvm_ir_backend_structural() {
        let program = sample_program("main");
        let llvm = compile_to_llvm_ir(&program);
        assert!(llvm.contains("define i64 @main"));
        assert!(llvm.contains("define i32 @main()"));
        assert!(llvm.contains("add i64"));
    }
}
