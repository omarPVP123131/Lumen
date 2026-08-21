use cranelift::prelude::settings;
use cranelift::prelude::*;
use cranelift_module::{DataId, FuncId, Linkage, Module};
use cranelift_object::{ObjectBuilder, ObjectModule, ObjectProduct};
use lumen_ir::ir::{Func as LumenFunc, Instr, Op, Program};
use std::collections::{BTreeMap, HashMap};

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
    // BUG-009: variantes sin salto de línea + emisor de '\n', para que
    // `imprimir(a, b)` produzca una sola línea también en el backend Cranelift.
    print_i64_nonl_id: FuncId,
    print_str_nonl_id: FuncId,
    // BUG-127: los booleanos se imprimían como 1/0 en vez de true/false.
    print_bool_id: FuncId,
    print_bool_nonl_id: FuncId,
    print_newline_id: FuncId,
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
        let pid_nonl = module
            .declare_function("_rt_print_i64_nonl", Linkage::Import, &psig)
            .unwrap();
        let pstr_nonl_id = module
            .declare_function("_rt_print_str_nonl", Linkage::Import, &psig)
            .unwrap();
        let pbool_id = module
            .declare_function("_rt_print_bool", Linkage::Import, &psig)
            .unwrap();
        let pbool_nonl_id = module
            .declare_function("_rt_print_bool_nonl", Linkage::Import, &psig)
            .unwrap();
        let nlsig = module.make_signature();
        let pnl_id = module
            .declare_function("_rt_print_newline", Linkage::Import, &nlsig)
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
            print_i64_nonl_id: pid_nonl,
            print_str_nonl_id: pstr_nonl_id,
            print_bool_id: pbool_id,
            print_bool_nonl_id: pbool_nonl_id,
            print_newline_id: pnl_id,
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

    pub fn compile(mut self, program: &Program) -> Result<ObjectProduct, String> {
        for (name, func) in &program.funcs {
            self.declare(name, func);
        }
        let names: Vec<String> = program.funcs.keys().cloned().collect();
        for n in &names {
            if let Some(f) = program.funcs.get(n) {
                self.compile_body(n, f)?;
            }
        }
        // BUG-124: `program.entry` es "__main__", que sólo existe si el fichero
        // tiene código de nivel superior. Un programa que sólo define
        // `funcion vacio principal()` no lo tiene, así que `entry_point` no
        // encontraba la entrada y generaba un `main` que retornaba 0 sin
        // ejecutar nada: el binario anunciaba «✓ Binario nativo», no imprimía
        // NADA y salía con código 0. El backend C ya hacía esta misma cascada
        // al emitir su `main`; Cranelift se había quedado sin ella.
        let entrada = if program.funcs.contains_key(&program.entry) {
            program.entry.clone()
        } else if program.funcs.contains_key("main") {
            "main".to_string()
        } else if program.funcs.contains_key("principal") {
            "principal".to_string()
        } else {
            program.entry.clone()
        };
        self.entry_point(&entrada);
        Ok(self.module.finish())
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

    fn compile_body(&mut self, name: &str, func: &LumenFunc) -> Result<(), String> {
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

        // BUG-075: la pila de operandos guardaba `Value` SSA crudos que se
        // consumían tras un cambio de bloque, produciendo
        // "uses value vN from non-dominating inst" en el verificador de
        // Cranelift. Se derrama la pila a variables SSA en cada frontera de
        // bloque y se recarga al entrar; `def_var`/`use_var` son sólo
        // contabilidad del constructor SSA, no emiten código.
        let spill_cap = instrs.len().min(4096);
        let mut spill_vars: Vec<Variable> = Vec::with_capacity(spill_cap);
        for _ in 0..spill_cap {
            let v = builder.declare_var(types::I64);
            builder.def_var(v, zero);
            spill_vars.push(v);
        }

        // ── Emisión lineal ──
        let mut cur = entry_block;
        let mut stack: Vec<Value> = Vec::new();
        // pila paralela: true = string (puntero), false = i64
        let mut kinds: Vec<bool> = Vec::new();
        let mut var_kinds: HashMap<String, bool> = HashMap::new();
        // BUG-127: qué valores de la pila son BOOLEANOS, para imprimirlos como
        // `true`/`false` y no como 1/0. Se lleva aparte porque `kinds` sólo
        // distingue puntero de entero y lo consultan 49 sitios.
        let mut booleanos: std::collections::HashSet<Value> = std::collections::HashSet::new();
        let mut terminated = false;
        for ins in instrs {
            if let Instr::Label(n) = ins {
                let target = label_block[n];
                if target != cur {
                    // BUG-075: derramar la pila viva ANTES del salto, para que
                    // el bloque destino lea valores que lo dominan.
                    let live = stack.len().min(spill_vars.len());
                    if !terminated {
                        for (i, val) in stack.iter().take(live).enumerate() {
                            builder.def_var(spill_vars[i], *val);
                        }
                        // el bloque actual termina saltando al label
                        builder.ins().jump(target, &[]);
                    }
                    cur = target;
                    builder.switch_to_block(cur);
                    builder.ensure_inserted_block();
                    // recargar la pila en el nuevo bloque
                    for i in 0..live {
                        stack[i] = builder.use_var(spill_vars[i]);
                    }
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
                    // BUG-126: este backend no tiene decimales, pero empujaba
                    // un 0 y seguía como si nada: `imprimir(1.5 + 2.5)` daba
                    // «0» en un binario que decía haberse generado bien. Es el
                    // patrón de BUG-050/084: o artefacto correcto, o negarse.
                    record_unsupported_any("<decimal>");
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::ConstBool(b) => {
                    // BUG-127
                    let v = builder.ins().iconst(i64, if *b { 1 } else { 0 });
                    booleanos.insert(v);
                    stack.push(v);
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
                // BUG-023: en Cranelift las variables ya tienen alcance por
                // función, así que declarar y asignar generan el mismo código.
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
                        // BUG-125: `ushr` es el desplazamiento LOGICO (mete
                        // ceros por la izquierda). El de LÚMEN es aritmético y
                        // conserva el signo, como en la VM y en el backend C:
                        // `-1 >> 1` es -1, no 9223372036854775807.
                        Op::ShiftRight => builder.ins().sshr(a, b),
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
                                let r = builder.ins().select(c, one, zero);
                                booleanos.insert(r); // BUG-127
                                r
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
                    } else if booleanos.contains(&v) {
                        self.print_bool_id // BUG-127
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
                            // BUG-009: imprime los argumentos sin salto y cierra
                            // con un único '\n', para que `imprimir(a, b)`
                            // produzca una sola línea igual que en la VM.
                            for (av, ak) in args.iter().zip(arg_kinds.iter()) {
                                let fid = if *ak {
                                    self.print_str_nonl_id
                                } else if booleanos.contains(av) {
                                    self.print_bool_nonl_id // BUG-127
                                } else {
                                    self.print_i64_nonl_id
                                };
                                let fref = self.module.declare_func_in_func(fid, builder.func);
                                builder.ins().call(fref, &[*av]);
                            }
                            let nlref = self
                                .module
                                .declare_func_in_func(self.print_newline_id, builder.func);
                            builder.ins().call(nlref, &[]);
                            stack.push(builder.ins().iconst(i64, 0));
                            kinds.push(false);
                        }
                        "leer" | "read" | "__str_len" | "__str_longitud" | "largo" | "len"
                        | "agregar" | "push" | "a_texto" | "to_texto" | "__str_from"
                        | "__map_nuevo" | "__map_poner" | "__map_obtener" | "__map_contiene" => {
                            // BUG-084: estos builtins no tienen runtime en el
                            // backend Cranelift y devolvían 0 en SILENCIO, así que
                            // `largo(l)` daba 0 y el binario producía resultados
                            // falsos sin avisar — el mismo patrón de "binario que
                            // miente" que BUG-050 ya corrigió para el backend C.
                            // Se registran para que el CLI pueda abortar.
                            record_unsupported_any(name);
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
                                // BUG-084: llamada que este backend no sabe generar.
                                record_unsupported_any(name);
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
                    record_unsupported_any("<indexado de lista>");
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::ArrayLen => {
                    record_unsupported_any("<longitud de lista>");
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::StructNew(_, _) => {
                    record_unsupported_any("<construccion de struct>");
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::StructGet => {
                    record_unsupported_any("<acceso a campo>");
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::StructSet => {
                    record_unsupported_any("<asignacion a campo>");
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    let _ = kinds.pop();
                    let _ = kinds.pop();
                }
                Instr::EnumCtor { .. } => {
                    record_unsupported_any("<constructor de enum>");
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::ResultOk | Instr::OptionSome | Instr::OptionNone | Instr::ResultErr => {
                    record_unsupported_any("<opcion/resultado>");
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::TryUnwrap | Instr::TupleAccess(_) | Instr::Read => {
                    record_unsupported_any("<try/tupla/leer>");
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::TupleNew(_) => {
                    record_unsupported_any("<tupla>");
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::FuncRef(_) => {
                    record_unsupported_any("<referencia a funcion>");
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::CallValue(_) => {
                    record_unsupported_any("<llamada indirecta>");
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                // BUG-027: descartar el valor sobrante de una sentencia-expresión.
                Instr::Drop => {
                    stack.pop();
                    kinds.pop();
                }
                Instr::Phi(_, _) | Instr::Nop => {}
                // BUG-095: el catch-all silencioso tragaba instrucciones que
                // este backend NO implementa —`intentar`/`atrapar`
                // (Push/PopHandler) y el emparejado de patrones
                // (MatchType/MatchPayload)—, así que un programa con
                // `intentar { 10/0 } atrapar { -1 }` compilaba sin una sola
                // advertencia y devolvía 10 donde la VM devuelve -1. Es el
                // mismo agujero que BUG-084 cerró para builtins y estructuras,
                // pero por la vía de las instrucciones sueltas.
                Instr::PushHandler(_) | Instr::PopHandler => {
                    record_unsupported_any("<intentar/atrapar>");
                }
                Instr::MatchType(_) => {
                    record_unsupported_any("<elegir sobre variantes>");
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                Instr::MatchPayload => {
                    record_unsupported_any("<payload de patron>");
                    let _ = stack.pop();
                    let _ = kinds.pop();
                    stack.push(builder.ins().iconst(i64, 0));
                    kinds.push(false);
                }
                otra => {
                    // Cualquier instrucción futura que nadie haya portado a
                    // este backend se registra en vez de desaparecer.
                    record_unsupported_any(&format!("<{:?}>", otra));
                }
            }
        }

        builder.seal_all_blocks();
        builder.finalize();
        if std::env::var_os("LUMEN_AOT_DEBUG").is_some() {
            eprintln!("[aot] --- clif {} ---\n{}", name, ctx.func.display());
        }
        // BUG-086: un fallo del verificador de Cranelift abortaba el proceso
        // entero con un panic (y el rastro de pila de un "crash del compilador").
        // Es un error de compilación normal: se propaga para que la CLI lo
        // presente como tal.
        if let Err(e) = self.module.define_function(info.id, &mut ctx) {
            return Err(format!(
                "el backend Cranelift no pudo generar la funcion '{}': {}",
                name, e
            ));
        }
        self.module.clear_context(&mut ctx);
        Ok(())
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
            // BUG-124: si no hay punto de entrada, el binario no puede hacer
            // nada útil. Se registra para que el CLI aborte en vez de entregar
            // un ejecutable vacío que aparenta haber funcionado.
            record_unsupported_any("<sin punto de entrada>");
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
            if let Instr::Load(n) | Instr::Store(n) = ins {
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
                Instr::Store(n) => {
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
                    // BUG-096: se emitía la llamada sin comprobar que la función
                    // existiera en el módulo, así que `largo(l)` producía
                    // `call i64 @largo(...)` sin ningún `declare`/`define`: LLVM
                    // IR INVÁLIDO que no pasa el verificador ni enlaza. La CLI
                    // lo anunciaba igualmente con «✓ Archivo LLVM IR generado».
                    if !program.funcs.contains_key(n.as_str()) {
                        record_unsupported_any(n);
                    }
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
                // BUG-096: catch-all mudo, igual que el de Cranelift (BUG-095):
                // a este backend le faltan 28 de los 42 opcodes (listas,
                // structs, `opcion`/`resultado`, `intentar`...) y todos
                // desaparecían sin dejar rastro en el IR ni un aviso.
                otra => {
                    record_unsupported_any(&format!("<{:?}>", otra));
                }
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
    let product = compiler.compile(program)?;
    let obj = &product.object;
    let bytes = obj.write().map_err(|e| format!("Write error: {}", e))?;
    std::fs::write(output, &bytes).map_err(|e| format!("IO: {}", e))?;
    Ok(())
}

/// BUG-050: builtins que el backend C no implementa y que, al compilar, se
/// convertían en un stub que devuelve `void`. El binario salía adelante y
/// mentía: `hijri` daba `void`, `regex_coincide` siempre `false`. Los
/// acumulamos durante la generación para que `build --native` pueda avisar.
static UNSUPPORTED: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

/// BUG-084: variante que registra CUALQUIER nombre. El backend Cranelift no
/// implementa builtins comunes (`largo`, `agregar`, `a_texto`...) que no llevan
/// el prefijo `__`, y `record_unsupported` los descartaba.
fn record_unsupported_any(name: &str) {
    if let Ok(mut v) = UNSUPPORTED.lock() {
        if !v.iter().any(|x| x == name) {
            v.push(name.to_string());
        }
    }
}

fn record_unsupported(name: &str) {
    if name.starts_with("__") {
        if let Ok(mut v) = UNSUPPORTED.lock() {
            if !v.iter().any(|x| x == name) {
                v.push(name.to_string());
            }
        }
    }
}

/// Builtins no soportados detectados en la última llamada a `compile_to_c`,
/// ordenados alfabéticamente. Vacía el registro.
pub fn take_unsupported_builtins() -> Vec<String> {
    match UNSUPPORTED.lock() {
        Ok(mut v) => {
            let mut out = std::mem::take(&mut *v);
            out.sort();
            out
        }
        Err(_) => Vec::new(),
    }
}

const C_RUNTIME: &str = include_str!("lumen_rt.h");

pub fn compile_to_c(program: &Program) -> String {
    if let Ok(mut v) = UNSUPPORTED.lock() {
        v.clear();
    }
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
    for (name, func) in &program.funcs {
        for p in &func.params {
            add_name(p);
        }
        for ins in &func.instrs {
            // BUG-022: `StoreLocal` faltaba en este barrido. Un nombre que
            // SÓLO se escribe con `StoreLocal` —como la variable del
            // `atrapar (e)`— no quedaba registrado, y `_fv()` devuelve 0
            // cuando no encuentra el nombre: el valor iba a parar al slot del
            // PRIMER global, machacándolo en silencio.
            if let Instr::Load(n) | Instr::Store(n) | Instr::StoreLocal(n) | Instr::FuncRef(n) = ins
            {
                add_name(n);
            }
            if let Instr::Call(n, _) = ins {
                if !program.funcs.contains_key(n) && !unknown.iter().any(|u| u == n) {
                    unknown.push(n.clone());
                }
            }
        }
        let _ = name;
    }

    // BUG-046: el llamador guarda sus variables antes de una llamada y las
    // restaura al volver (los "locales" del backend C son globales C, así que
    // sin esto la recursión se pisaría a sí misma). El problema es que ese
    // save/restore también revertía las variables GLOBALES del programa que la
    // función acabara de modificar: `funcion vacio subir() { g = g + 1; }` no
    // tenía ningún efecto visible, en silencio. Las globales son las que
    // declara `__main__`, así que se excluyen del save/restore.
    let mut program_globals: Vec<String> = Vec::new();
    if let Some(main_fn) = program
        .funcs
        .get("__main__")
        .or_else(|| program.funcs.get(&program.entry))
    {
        for ins in &main_fn.instrs {
            if let Instr::Store(n) | Instr::StoreLocal(n) = ins {
                if !program_globals.iter().any(|x| x == n) {
                    program_globals.push(n.clone());
                }
            }
        }
    }

    // Si ALGUNA función ensombrece una global —como parámetro o como
    // declaración local (`StoreLocal`, BUG-023)— ese nombre designa dos cosas
    // distintas según dónde se lea. En ese caso hay que seguir guardándolo y
    // restaurándolo: lo local no debe sobrevivir a la llamada ni pisar la
    // global. Son casos raros, y preferimos conservar el aislamiento a
    // arriesgar una fuga entre ámbitos.
    let mut shadowing_params: Vec<String> = Vec::new();
    for (fname, func) in &program.funcs {
        if fname == "__main__" {
            continue;
        }
        for p in &func.params {
            if program_globals.contains(p) && !shadowing_params.contains(p) {
                shadowing_params.push(p.clone());
            }
        }
        for ins in &func.instrs {
            if let Instr::StoreLocal(n) = ins {
                if program_globals.contains(n) && !shadowing_params.contains(n) {
                    shadowing_params.push(n.clone());
                }
            }
        }
    }

    let mut name_sets: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, func) in &program.funcs {
        let mut set: Vec<String> = func.params.clone();
        for ins in &func.instrs {
            if let Instr::Load(n) | Instr::Store(n) = ins {
                if !set.iter().any(|x| x == n) {
                    set.push(n.clone());
                }
            }
        }
        set.retain(|n| !program_globals.contains(n) || shadowing_params.contains(n));
        name_sets.insert(name.clone(), set);
    }

    out.push_str("static void _init(void) {\n");
    // BUG-051: fija la referencia de pila para detectar recursion infinita.
    out.push_str("  _stack_init();\n");
    for n in &names {
        out.push_str(&format!("  _reg(\"{}\");\n", esc(n)));
    }
    for (name, func) in &program.funcs {
        if !func.params.is_empty() {
            let plist: Vec<String> = func
                .params
                .iter()
                .map(|p| format!("\"{}\"", esc(p)))
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

    for name in program.funcs.keys() {
        out.push_str(&format!("static Val _f_{}(void);\n", mangle(name)));
    }
    for n in &unknown {
        out.push_str(&format!("static Val _f_{}(void);\n", mangle(n)));
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
        out.push_str(&emit_func(name, func, program, &name_sets, &gv_of));
    }
    for n in &unknown {
        if std::env::var("LUMEN_AOT_DEBUG_UNKNOWN").is_ok() {
            eprintln!("AOT-UNKNOWN: {}", n);
        }
        out.push_str(&format!(
            "static Val _f_{}(void) {{ return _v_void(); }}\n\n",
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
        // BUG-045: `nm` podía llegar NULL (p. ej. al invocar un valor que no es
        // una referencia a función válida) y `strcmp` segfalleaba. La VM
        // devuelve void en ese caso; aquí también, en vez de reventar.
        "static Val _call_by_name(const char* nm) {{\n  if (!nm) return _v_void();\n  for (int _i = 0; _i < {}; _i++) if (_lfn_names[_i] && !strcmp(_lfn_names[_i], nm)) return _lfn_ptrs[_i]();\n  return _v_void();\n}}\n\n",
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
    }
    out.push_str("  return 0;\n}\n");
    out
}

fn esc(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
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

/// Espejo de `is_shadowable_builtin` de la VM (BUG-012): builtins "de
/// conveniencia" que una función del usuario con el mismo nombre puede
/// ensombrecer. Los del núcleo (`imprimir`, `a_texto`, `__*`) nunca lo son.
fn is_shadowable_builtin_c(name: &str) -> bool {
    matches!(
        name,
        "abs"
            | "absoluto"
            | "minimo"
            | "min"
            | "maximo"
            | "max"
            | "raiz"
            | "sqrt"
            | "potencia"
            | "pow"
            | "piso"
            | "floor"
            | "techo"
            | "ceil"
            | "redondear"
            | "round"
            | "es_numero"
            | "is_number"
            | "a_entero"
            | "to_int"
            | "to_entero"
            | "a_decimal"
            | "to_float"
            | "a_numero"
            | "to_number"
            | "a_entero_seguro"
            | "to_int_safe"
            | "a_decimal_seguro"
            | "to_float_safe"
            // BUG-018: idem que en la VM.
            | "leer"
            | "read"
    )
}

fn emit_func(
    name: &str,
    func: &LumenFunc,
    program: &Program,
    name_sets: &BTreeMap<String, Vec<String>>,
    gv_of: &dyn Fn(&str) -> String,
) -> String {
    let mut s = String::new();
    s.push_str(&format!("static Val _f_{}(void) {{\n", mangle(name)));
    // BUG-047: `Drop` sólo miraba `SP > 0`, que es la base de la pila GLOBAL,
    // no la de este marco. Dentro de un bucle la pila del marco se vacía en la
    // primera vuelta y en las siguientes el `Drop` se comía un valor del
    // LLAMADOR (`imprimir("[", inv("Hola"), "]")` perdía el "[" y sacaba un 0).
    // Igual que la VM con `frame.stack_base`, cada función recuerda el SP de
    // entrada y nunca descarta por debajo de él.
    s.push_str("  const int _base = SP;\n");
    // BUG-051: las funciones LUMEN son funciones C recursivas; sin este control
    // una recursion infinita desbordaba la pila del proceso (SEGFAULT mudo).
    s.push_str("  _ckdepth();\n");

    // Emite la llamada a una función definida por el usuario (guarda/restaura las
    // variables del llamador y enlaza los parámetros). Se usa tanto en la rama
    // normal como cuando el usuario ensombrece un builtin suave (BUG-012).
    let emit_user_call = |callee: &LumenFunc, n: &str, argc: usize| -> String {
        let mut s = String::new();
        let plen = callee.params.len().min(argc);
        let caller_names: Vec<String> = name_sets.get(name).cloned().unwrap_or_default();
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
            s.push_str(&format!("  {} = _dcp(POP());\n", gv_of(&callee.params[i])));
        }
        for _ in plen..argc {
            s.push_str("  (void)POP();\n");
        }
        s.push_str(&format!("  {{ Val _r = _f_{}(); PUSH(_r); }}\n", mangle(n)));
        // BUG-037: fotografía de los parámetros del callee para `__frame_param`
        // (write-back de `prestado mut`), antes de restaurar las del llamador.
        s.push_str(&format!("  _fpc = {};\n", plen.min(32)));
        for i in 0..plen.min(32) {
            s.push_str(&format!("  _fp[{}] = {};\n", i, gv_of(&callee.params[i])));
        }
        s.push_str(&post);
        s
    };

    for instr in &func.instrs {
        match instr {
            // BUG-027: sacar y tirar la cima (valor de una sentencia-expresión).
            // BUG-034: `Drop` puede encontrarse la pila vacía (una
            // sentencia-expresión cuyo valor ya se consumió, p. ej.
            // `l.agregar(x);`, que termina en un `Store`). La VM lo tolera
            // porque `Vec::pop()` sobre vacío no hace nada, pero aquí `POP()`
            // es `ST[--SP]`: dejaba `SP` en -1 y el siguiente `PUSH` escribía
            // en `ST[-1]`, fuera de los límites del array. Comprobamos antes.
            Instr::Drop => s.push_str("  if (SP > _base) { (void)POP(); }\n"),
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
                s.push_str(&format!("  PUSH({});\n", gv_of(n)));
            }
            Instr::Store(n) | Instr::StoreLocal(n) => {
                s.push_str(&format!("  {} = _dcp(POP());\n", gv_of(n)));
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
                // BUG-118: cada rama de builtin extrae un número FIJO de
                // argumentos, pero el llamador empuja `argc`. Si no coinciden
                // (p.ej. `__str_codigo("abc", 0)`, que empuja 2 y la rama saca
                // 1), el sobrante se queda en la pila y desplaza todo lo que
                // viene después: el binario nativo dejaba de imprimir y salía
                // con código 0, en silencio. `sema` caza la mayoría de los
                // descuadres con E040, pero no todos.
                // En vez de tocar las 94 ramas, se equilibra aquí: se recuerda
                // SP antes de la llamada y, al terminar, se deja exactamente
                // el resultado con `argc` argumentos consumidos.
                let _marca_sp = s.len();
                if n == "imprimir" || n == "print" {
                    if *argc > 0 {
                        // BUG-009: concatena los argumentos en una sola línea,
                        // igual que la VM (un único '\n' al final).
                        s.push_str(&format!(
                            "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); for (int _k2 = 0; _k2 < {}; _k2++) printf(\"%s\", _fmt(_t[_k2])); printf(\"\\n\"); }}\n",
                            argc, argc, argc
                        ));
                    } else {
                        s.push_str("  printf(\"\\n\");\n");
                    }
                    s.push_str("  PUSH(_v_void());\n");
                } else if is_shadowable_builtin_c(n)
                    && program.funcs.contains_key(n.as_str())
                {
                    // BUG-012: una función del usuario con el mismo nombre gana sobre
                    // los builtins "de conveniencia", igual que en la VM.
                    let callee = &program.funcs[n.as_str()];
                    s.push_str(&emit_user_call(callee, n, *argc));
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
                } else if n == "a_entero" || n == "to_int" || n == "to_entero" {
                    // BUG-007: mismas conversiones que la VM en el binario nativo.
                    s.push_str("  { Val _x = POP(); PUSH(_b_a_entero(_x)); }\n");
                } else if n == "a_decimal"
                    || n == "to_float"
                    || n == "a_numero"
                    || n == "to_number"
                {
                    s.push_str("  { Val _x = POP(); PUSH(_b_a_decimal(_x)); }\n");
                } else if n == "es_numero" || n == "is_number" {
                    s.push_str("  { Val _x = POP(); PUSH(_b_es_numero(_x)); }\n");
                } else if n == "abs" || n == "absoluto" {
                    // BUG-001
                    s.push_str("  { Val _x = POP(); PUSH(_b_abs(_x)); }\n");
                } else if n == "minimo" || n == "min" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_b_minmax(_a, _b, 0)); }\n");
                } else if n == "maximo" || n == "max" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_b_minmax(_a, _b, 1)); }\n");
                } else if n == "raiz" || n == "sqrt" {
                    s.push_str("  { Val _x = POP(); PUSH(_b_math1(_x, 0)); }\n");
                } else if n == "piso" || n == "floor" {
                    s.push_str("  { Val _x = POP(); PUSH(_b_math1(_x, 1)); }\n");
                } else if n == "techo" || n == "ceil" {
                    s.push_str("  { Val _x = POP(); PUSH(_b_math1(_x, 2)); }\n");
                } else if n == "redondear" || n == "round" {
                    s.push_str("  { Val _x = POP(); PUSH(_b_math1(_x, 3)); }\n");
                } else if n == "potencia" || n == "pow" {
                    s.push_str("  { Val _e = POP(); Val _b = POP(); PUSH(_b_potencia(_b, _e)); }\n");
                } else if n == "agregar" || n == "push" {
                    s.push_str("  { Val _x = POP(); Val _a = POP(); PUSH(_arr_push(_a, _x)); }\n");
                } else if n == "largo" || n == "len" || n == "__str_len" || n == "__str_longitud" {
                    s.push_str("  { Val _x = POP(); if (_x.t == T_ARR || _x.t == T_TUP || _x.t == T_MAP) PUSH(_v_int(_x.argc)); else if (_x.t == T_STR) PUSH(_v_int(_utf8_len(_x.s))); else PUSH(_v_int(0)); }\n");
                } else if n == "a_entero_seguro" || n == "to_int_safe" {
                    s.push_str("  { Val _x = POP(); PUSH(_b_a_entero_seguro(_x)); }\n");
                } else if n == "a_decimal_seguro" || n == "to_float_safe" {
                    s.push_str("  { Val _x = POP(); PUSH(_b_a_decimal_seguro(_x)); }\n");
                } else if n == "__frame_param" {
                    s.push_str("  { Val _i = POP(); PUSH(_frame_param((int64_t)_asf(_i))); }\n");
                } else if n == "__enum_variante" || n == "__enum_variant" {
                    s.push_str("  { Val _x = POP(); PUSH(_v_str(_enum_variante(_x))); }\n");
                } else if n == "__enum_campo" || n == "__enum_field" {
                    s.push_str("  { Val _i = POP(); Val _x = POP(); PUSH(_enum_campo(_x, (int64_t)_asf(_i))); }\n");
                } else if n == "__enum_aridad" || n == "__enum_arity" {
                    s.push_str("  { Val _x = POP(); PUSH(_v_int(_enum_aridad(_x))); }\n");
                } else if n == "__enum_nombre" || n == "__enum_name" {
                    s.push_str("  { Val _x = POP(); PUSH(_v_str(_x.t == T_ENM && _x.en ? _x.en : \"\")); }\n");
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
                    s.push_str("  { Val _s = POP(); Val _p = POP(); PUSH(_v_bool(_regex_m(_p.s, _s.s))); }\n");
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
                } else if n == "__str_mayusculas" {
                    s.push_str("  { Val _s = POP(); PUSH(_v_str(_case_str(_s.s, 1))); }\n");
                } else if n == "__str_minusculas" {
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
                    s.push_str(&emit_user_call(callee, n, *argc));
                } else {
                    // BUG-050: aquí acaba toda llamada que el backend C no sabe
                    // generar. Se compilaba a un stub `return _v_void()`, así
                    // que el binario se producía igual y devolvía valores falsos
                    // en silencio (fechas a `void`, regex siempre `false`...).
                    // Lo anotamos para que el CLI pueda avisar en vez de mentir.
                    record_unsupported(n);
                    for _ in 0..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    s.push_str(&format!("  {{ Val _r = _f_{}(); PUSH(_r); }}\n", mangle(n)));
                }
                // Sólo se equilibra la pila de los builtins: las llamadas a
                // funciones del usuario ya consumen sus argumentos con
                // exactitud (`emit_user_call`).
                if !program.funcs.contains_key(n.as_str()) {
                    let cuerpo = s.split_off(_marca_sp);
                    s.push_str("  { int _sp0 = SP;\n");
                    s.push_str(&cuerpo);
                    s.push_str(&format!(
                        "  {{ Val _rv = (SP > 0) ? POP() : _v_void(); SP = _sp0 - {}; if (SP < 0) SP = 0; PUSH(_rv); }} }}\n",
                        argc
                    ));
                }
            }
            Instr::FuncRef(n) => {
                // BUG-032: si la lambda captura variables del entorno, se
                // copian AHORA en un entorno propio de esta closure. Antes el
                // binario nativo leía las globales en el momento de la llamada,
                // así que dos closures de la misma factoría compartían estado y
                // devolvían valores erróneos en silencio (`mk(5)` daba 101).
                let caps: Vec<String> = program
                    .funcs
                    .get(n)
                    .map(|f| f.captures.clone())
                    .unwrap_or_default();
                if caps.is_empty() {
                    s.push_str(&format!(
                        "  PUSH(_vfref(\"{}\", &_f_{}));\n",
                        esc(n),
                        mangle(n)
                    ));
                } else {
                    let list = caps
                        .iter()
                        .map(|c| format!("\"{}\"", esc(c)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    s.push_str(&format!(
                        "  {{ static const char* _cn[] = {{ {} }};\n    PUSH(_vfclos(\"{}\", &_f_{}, _cn, {}));\n  }}\n",
                        list,
                        esc(n),
                        mangle(n),
                        caps.len()
                    ));
                }
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
                    // BUG-009: una sola línea concatenando los argumentos.
                    for i in 0..*argc {
                        s.push_str(&format!("      printf(\"%s\", _fmt(_t[{}]));\n", i));
                    }
                    s.push_str("      printf(\"\\n\");\n");
                    s.push_str("      PUSH(_v_void());\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"leer\") || !strcmp(_cf.s, \"read\")) {\n      PUSH(_read_ln());\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"a_texto\") || !strcmp(_cf.s, \"to_texto\") || !strcmp(_cf.s, \"__str_from\")) {\n      PUSH(_v_str(_fmt(_t[0])));\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"agregar\") || !strcmp(_cf.s, \"push\")) {\n      PUSH(_arr_push(_t[0], _t[1]));\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"largo\") || !strcmp(_cf.s, \"len\") || !strcmp(_cf.s, \"__str_len\") || !strcmp(_cf.s, \"__str_longitud\")) {\n      PUSH(_v_int(_t[0].t == T_STR ? _utf8_len(_t[0].s) : _t[0].argc));\n");
                    s.push_str("    } else {\n");
                    // BUG-061: la llamada INDIRECTA (una lambda guardada en una
                    // variable) no guardaba las variables del llamador, al
                    // contrario que la llamada directa. Como los parámetros
                    // viven en slots globales, una lambda recursiva se pisaba a
                    // sí misma: `fib(n-1)` machacaba `n` antes de que se
                    // evaluara `fib(n-2)`, y el resultado salía mal EN SILENCIO
                    // (`fib(10)` daba -80 en vez de 55). Se guarda y restaura
                    // igual que en `emit_user_call`.
                    let caller_names: Vec<String> =
                        name_sets.get(name).cloned().unwrap_or_default();
                    if !caller_names.is_empty() {
                        s.push_str(&format!("      {{ Val _svi[{}];\n", caller_names.len()));
                        for (i, cn) in caller_names.iter().enumerate() {
                            s.push_str(&format!("        _svi[{}] = {};\n", i, gv_of(cn)));
                        }
                    }
                    for i in 0..*argc {
                        s.push_str(&format!(
                            "      gv[_fv(_par(_cf.s, {}))] = _dcp(_t[{}]);\n",
                            i, i
                        ));
                    }
                    s.push_str("      { Val _r = _fref_call(_cf); PUSH(_r); }\n");
                    if !caller_names.is_empty() {
                        s.push_str("        ");
                        for (i, cn) in caller_names.iter().enumerate() {
                            // BUG-149: si la closure capturo esta variable, su
                            // entorno manda sobre el valor guardado.
                            s.push_str(&format!(
                                "{} = _env_or(_cf, \"{}\", _svi[{}]); ",
                                gv_of(cn),
                                esc(cn),
                                i
                            ));
                        }
                        s.push_str("}\n");
                    }
                    s.push_str("    }\n  }\n");
                }
            }
            // BUG-044: una función `vacio` retorna sin dejar valor en la pila,
            // así que `POP()` (que es `ST[--SP]`, sin comprobación) leía fuera
            // del array y dejaba `SP` en -1 — SEGFAULT en el binario nativo.
            // La VM devuelve `void` en ese caso; hacemos lo mismo.
            Instr::Return => {
                // BUG-051: libera el nivel de profundidad en cada salida.
                s.push_str("  { _depth--; return (SP > _base) ? POP() : _v_void(); }\n")
            }
            Instr::Print => s.push_str("  printf(\"%s\\n\", _fmt(POP()));\n"),
            Instr::Read => s.push_str("  PUSH(_read_ln());\n"),
            Instr::ArrayNew(n) => {
                s.push_str(&format!(
                    "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); PUSH(_arrn(_t, {})); }}\n",
                    n, n, n
                ));
            }
            Instr::ArrayPush => s.push_str("  { Val _x = POP(); Val _a = POP(); PUSH(_arr_push(_a, _x)); }\n"),
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
            // BUG-022: `setjmp` devuelve 0 al instalar el manejador y 1 cuando
            // el runtime salta aqui desde un error. En ese caso se restaura la
            // pila de valores y se deja el mensaje para el `atrapar`.
            Instr::PushHandler(t) => s.push_str(&format!(
                "  if (_hnd_n < MAX_HND) {{ _hnd[_hnd_n].sp = SP; _hnd[_hnd_n].depth = _depth; \
                 if (setjmp(_hnd[_hnd_n++].env)) {{ SP = _hnd[_hnd_n - 1].sp; \
                 _depth = _hnd[_hnd_n - 1].depth; _hnd_n--; PUSH(_v_str(_hnd_msg)); goto L_{}; }} }}\n",
                t
            )),
            Instr::PopHandler => s.push_str("  if (_hnd_n > 0) { _hnd_n--; }\n"),
            Instr::Phi(..) | Instr::Nop | Instr::Halt => {
                if let Instr::Halt = instr {
                    s.push_str("  { _depth--; return _v_void(); }\n");
                }
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
mod tests_portabilidad {
    /// BUG-165: `lumen_rt.h` incluia <sys/resource.h> FUERA del bloque POSIX,
    /// asi que ningun binario nativo compilaba en Windows: 19 tests caian con
    /// "fatal error: sys/resource.h: No such file or directory". El fichero ya
    /// tenia un `#if !defined(_WIN32)` bien puesto; la cabecera simplemente
    /// estaba arriba del todo. Este test vigila que no vuelva a colarse.
    #[test]
    fn cabeceras_posix_solo_dentro_de_guardas() {
        let src = include_str!("lumen_rt.h");
        let posix = [
            "sys/resource.h",
            "unistd.h",
            "pthread.h",
            "regex.h",
            "dirent.h",
            "sys/mman.h",
            "sys/wait.h",
        ];
        // Profundidad de anidamiento dentro de un `#if ... _WIN32 ...`.
        let mut pila: Vec<bool> = Vec::new();
        let mut fuera: Vec<(usize, String)> = Vec::new();
        for (i, linea) in src.lines().enumerate() {
            let t = linea.trim();
            if t.starts_with("#if") {
                pila.push(t.contains("_WIN32"));
                continue;
            }
            if t.starts_with("#endif") {
                pila.pop();
                continue;
            }
            if t.starts_with("#include") {
                let protegido = pila.iter().any(|p| *p);
                if !protegido {
                    if let Some(h) = posix.iter().find(|h| t.contains(**h)) {
                        fuera.push((i + 1, format!("{} — {}", h, t)));
                    }
                }
            }
        }
        assert!(
            fuera.is_empty(),
            "cabeceras POSIX sin guarda de _WIN32 (rompen la compilacion nativa en Windows): {:?}",
            fuera
        );
    }

    /// BUG-166: el bloque `#else` de la guarda de regex (Windows y macOS)
    /// contenia stubs que devolvian SIEMPRE 0, asi que `__regex_coincide`
    /// respondia "false" a cualquier patron en esas dos plataformas mientras
    /// la VM respondia "true". Ahora hay un motor propio por backtracking.
    ///
    /// El CI de Linux nunca compila esa rama, que es justo como el fallo
    /// sobrevivio a BUG-080. Este test la extrae del header, la compila con el
    /// `cc` local y la ejecuta contra un banco de casos, de modo que la rama
    /// no-POSIX queda ejercitada en cualquier plataforma.
    #[test]
    fn regex_de_la_rama_no_posix_coincide_con_la_vm() {
        let src = include_str!("lumen_rt.h");
        let ini = src
            .find("   BUG-166:")
            .expect("no se encuentra el motor de regex no-POSIX");
        let fin = src[ini..]
            .find("\n#endif")
            .map(|k| ini + k)
            .expect("no se encuentra el cierre de la rama no-POSIX");
        let motor = &src[ini..fin];
        assert!(
            motor.contains("_rx_alt") && motor.contains("_regex_rep"),
            "la rama no-POSIX no contiene un motor de regex: ¿han vuelto los stubs?"
        );
        assert!(
            !motor.contains("return 0; /* stub"),
            "la rama no-POSIX volvio a ser un stub"
        );

        // Casos verificados contra la salida real de la VM (`__regex_coincide`).
        let casos: &[(&str, &str, i32)] = &[
            (r"\d+", "abc123", 1),
            (r"\d+", "sin numeros", 0),
            (r"\w+", "hola", 1),
            ("^abc", "abcdef", 1),
            ("^abc", "xabcdef", 0),
            ("abc$", "xxabc", 1),
            ("abc$", "abcx", 0),
            ("a.c", "abc", 1),
            ("a.c", "ac", 0),
            ("[0-9]+", "x42", 1),
            ("[^0-9]+", "4242", 0),
            ("a*b", "b", 1),
            ("a+b", "b", 0),
            ("colou?r", "colour", 1),
            ("gato|perro", "un perro", 1),
            ("gato|perro", "un pez", 0),
            ("(ab)+c", "ababc", 1),
            ("(ab)+c", "c", 0),
            (r"\s", "ab", 0),
            (r"\S+", "  x", 1),
            ("[a-z]+@[a-z]+", "mail@web", 1),
            (r"\.", "a.b", 1),
            (r"\.", "ab", 0),
            (r"[A-Z]\w+", "Hola mundo", 1),
            (r"^\d{1,}", "42x", 1),
            (r"^\d{3}$", "123", 1),
            (r"^\d{3}$", "12", 0),
            ("a{2,3}b", "ab", 0),
            ("a{2,3}b", "aab", 1),
        ];

        let mut tabla = String::new();
        for (p, s, esp) in casos {
            tabla.push_str(&format!(
                "  {{\"{}\",\"{}\",{}}},\n",
                p.replace('\\', "\\\\"),
                s.replace('\\', "\\\\"),
                esp
            ));
        }
        let programa = format!(
            "#include <stdio.h>\n#include <stdlib.h>\n#include <string.h>\n/*{motor}\n\
             struct C {{ const char* p; const char* s; int esp; }};\n\
             static struct C CASOS[] = {{\n{tabla}}};\n\
             int main(void) {{\n\
             \x20 int n = (int)(sizeof(CASOS)/sizeof(CASOS[0])), mal = 0;\n\
             \x20 for (int i = 0; i < n; i++) {{\n\
             \x20   int r = _regex_m(CASOS[i].p, CASOS[i].s);\n\
             \x20   if (r != CASOS[i].esp) {{ printf(\"MAL /%s/ vs '%s' => %d esperado %d\\n\", CASOS[i].p, CASOS[i].s, r, CASOS[i].esp); mal++; }}\n\
             \x20 }}\n\
             \x20 struct R {{ const char* p; const char* s; const char* r; const char* e; }};\n\
             \x20 struct R REPS[] = {{\n\
             \x20   {{\"\\\\d+\", \"a1b22c\", \"#\", \"a#b#c\"}}, {{\"o\", \"foo\", \"0\", \"f00\"}},\n\
             \x20   {{\"\\\\d{{2}}\", \"a12b3\", \"N\", \"aNb3\"}},\n\
             \x20   {{\"[a-z]?|a\", \"x_y\", \"#\", \"#_#\"}}, {{\"a?\", \"bab\", \"#\", \"#b#b#\"}},\n\
             \x20   {{\"a*\", \"bab\", \"#\", \"#b#b#\"}}, {{\"\", \"abc\", \"#\", \"#a#b#c#\"}},\n\
             \x20   {{\"x?\", \"\", \"#\", \"#\"}}, {{\"\\\\d*\", \"a1b\", \"#\", \"#a#b#\"}},\n\
             \x20   {{\"^\", \"abc\", \"#\", \"#abc\"}}, {{\"$\", \"abc\", \"#\", \"abc#\"}},\n\
             \x20 }};\n\
             \x20 for (int i = 0; i < (int)(sizeof(REPS)/sizeof(REPS[0])); i++) {{\n\
             \x20   char* g = _regex_rep(REPS[i].p, REPS[i].s, REPS[i].r);\n\
             \x20   if (strcmp(g, REPS[i].e)) {{ printf(\"MAL rep /%s/ sobre '%s' => '%s' esperado '%s'\\n\", REPS[i].p, REPS[i].s, g, REPS[i].e); mal++; }}\n\
             \x20 }}\n\
             \x20 printf(\"%d\\n\", mal);\n\
             \x20 return mal != 0;\n}}\n"
        );

        let dir = std::env::temp_dir().join(format!("lumen_rx_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let cfile = dir.join("rx.c");
        let bin = dir.join("rx.bin");
        std::fs::write(&cfile, programa).expect("no se pudo escribir el banco de regex");

        let cc = std::env::var("CC").unwrap_or_else(|_| "cc".to_string());
        let comp = std::process::Command::new(&cc)
            .arg("-O1")
            .arg("-o")
            .arg(&bin)
            .arg(&cfile)
            .output();
        let comp = match comp {
            Ok(c) => c,
            Err(_) => {
                eprintln!("aviso: sin compilador C ({cc}); se omite el banco de regex");
                let _ = std::fs::remove_dir_all(&dir);
                return;
            }
        };
        assert!(
            comp.status.success(),
            "la rama no-POSIX del regex no compila: {}",
            String::from_utf8_lossy(&comp.stderr)
        );
        let run = std::process::Command::new(&bin)
            .output()
            .expect("no se pudo ejecutar el banco de regex");
        let salida = String::from_utf8_lossy(&run.stdout).to_string();
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            run.status.success(),
            "el motor de regex no-POSIX diverge de la VM:\n{salida}"
        );
    }
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
                captures: vec![],
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
                captures: vec![],
                instrs: vec![Instr::ConstInt(1), Instr::Return],
            },
        );
        program.funcs.insert(
            "dead".into(),
            Func {
                name: "dead".into(),
                params: vec![],
                entry: 0,
                captures: vec![],
                instrs: vec![Instr::ConstInt(0), Instr::Return],
            },
        );
        // AOT only compiles entry-reachable functions
        let compiler = AotCompiler::new();
        let product = compiler.compile(&program).expect("compile debe funcionar");
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
                entry: 0,
                captures: vec![],
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
        assert!(c.contains("static Val _f_main(void)"));
        assert!(c.contains("int main(void)"));
        assert!(c.contains("PUSH(_v_int(40))"));
        assert!(c.contains("_bin(1, _a, _b)"));
        assert!(c.contains("printf(\"%s\\n\", _fmt(POP()))"));
    }

    #[test]
    fn test_c_backend_gcc_runtime() {
        // Skip on Windows: el runtime C usa POSIX (opendir, regex) no disponible nativamente
        if cfg!(windows) {
            return;
        }
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        if std::env::var_os("MSYSTEM").is_some() {
            return;
        }
        let program = sample_program("main");
        let c = compile_to_c(&program);
        let dir = std::env::temp_dir().join(format!("lumen_aot_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let c_path = dir.join("test.c");
        let exe_path = dir.join("test.exe");
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
        let out = std::process::Command::new(&exe_path).output().unwrap();
        let test_out = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
        assert_eq!(test_out, "42\n");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_jit_engine_execution() {
        let mut jit = JitEngine::new().expect("Failed to initialize JIT engine");
        let func = Func {
            name: "jit_add".to_string(),
            params: vec!["a".to_string(), "b".to_string()],
            entry: 0,
            captures: vec![],
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
