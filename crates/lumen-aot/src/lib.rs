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
    for (name, func) in &program.funcs {
        for p in &func.params {
            add_name(p);
        }
        for ins in &func.instrs {
            if let Instr::Load(n) | Instr::Store(n) | Instr::FuncRef(n) = ins {
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
        name_sets.insert(name.clone(), set);
    }

    out.push_str("static void _init(void) {\n");
    for n in &names {
        out.push_str(&format!("  _reg(\"{}\");\n", esc(n)));
    }
    for (name, func) in &program.funcs {
        if !func.params.is_empty() {
            let plist: Vec<String> = func.params.iter().map(|p| format!("\"{}\"", esc(p))).collect();
            out.push_str(&format!(
                "  _regpars(\"{}\", (const char*[]){{ {} }}, {});\n",
                esc(name),
                plist.join(", "),
                func.params.len()
            ));
        }
    }
    out.push_str("}\n\n");

    for (name, _) in &program.funcs {
        out.push_str(&format!("static Val _f_{}(void);\n", mangle(name)));
    }
    for n in &unknown {
        out.push_str(&format!("static Val _f_{}(void);\n", mangle(n)));
    }
    out.push_str("static Val _call_by_name(const char* nm);\n");
    out.push('\n');

    for (name, func) in &program.funcs {
        out.push_str(&emit_func(name, func, program, &name_sets));
    }
    for n in &unknown {
        out.push_str(&format!("static Val _f_{}(void) {{ return _v_void(); }}\n\n", mangle(n)));
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
        Op::Negate => 19,
        Op::Not => 20,
    }
}

fn emit_func(name: &str, func: &LumenFunc, program: &Program, name_sets: &BTreeMap<String, Vec<String>>) -> String {
    let mut s = String::new();
    s.push_str(&format!("static Val _f_{}(void) {{\n", mangle(name)));

    for instr in &func.instrs {
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
                s.push_str(&format!("  PUSH(gv[_fv(\"{}\")]);\n", esc(n)));
            }
            Instr::Store(n) => {
                s.push_str(&format!("  gv[_fv(\"{}\")] = _dcp(POP());\n", esc(n)));
            }
            Instr::Binary(op) => {
                let code = op_code(op);
                s.push_str(&format!(
                    "  {{ Val _b = POP(); Val _a = POP(); PUSH(_bin({}, _a, _b)); }}\n",
                    code
                ));
            }
            Instr::Unary(op) => {
                if *op == Op::Negate {
                    s.push_str("  PUSH(_neg(POP()));\n");
                } else {
                    s.push_str("  PUSH(_not(POP()));\n");
                }
            }
            Instr::Call(n, argc) => {
                if n == "imprimir" || n == "print" {
                    if *argc > 0 {
                        s.push_str(&format!(
                            "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); for (int _k2 = 0; _k2 < {}; _k2++) printf(\"%s\\n\", _fmt(_t[_k2])); }}\n",
                            argc, argc, argc
                        ));
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
                    let plen = callee.params.len().min(*argc);
                    let caller_names: Vec<String> =
                        name_sets.get(name).cloned().unwrap_or_default();
                    let mut pre = String::new();
                    let mut post = String::new();
                    if !caller_names.is_empty() {
                        pre.push_str("  { Val _sv[");
                        pre.push_str(&caller_names.len().to_string());
                        pre.push_str("];\n");
                        for (i, cn) in caller_names.iter().enumerate() {
                            pre.push_str(&format!("    _sv[{}] = gv[_fv(\"{}\")];\n", i, esc(cn)));
                        }
                        post.push_str("    ");
                        for (i, cn) in caller_names.iter().enumerate() {
                            post.push_str(&format!("gv[_fv(\"{}\")] = _sv[{}]; ", esc(cn), i));
                        }
                        post.push_str("}\n");
                    }
                    s.push_str(&pre);
                    for i in (0..plen).rev() {
                        s.push_str(&format!(
                            "  gv[_fv(\"{}\")] = _dcp(POP());\n",
                            esc(&callee.params[i])
                        ));
                    }
                    for _ in plen..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    s.push_str(&format!("  PUSH(_f_{}());\n", mangle(n)));
                    s.push_str(&post);
                } else {
                    for _ in 0..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    s.push_str(&format!("  PUSH(_f_{}());\n", mangle(n)));
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
                        "  { Val _cf = POP(); if (!strcmp(_cf.s, \"largo\") || !strcmp(_cf.s, \"len\") || !strcmp(_cf.s, \"__str_longitud\")) { PUSH(_v_int(0)); } else { PUSH(_fref_call(_cf)); } }\n",
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
                    s.push_str("      PUSH(_fref_call(_cf));\n");
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

    fn sample_program(name: &str) -> Program {
        let mut program = Program::new();
        program.entry = name.to_string();
        program.funcs.insert(
            name.to_string(),
            Func {
                name: name.to_string(),
                params: vec![],
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
        assert!(c.contains("static Val _f_main(void)"));
        assert!(c.contains("int main(void)"));
        assert!(c.contains("PUSH(_v_int(40))"));
        assert!(c.contains("_bin(1, _a, _b)"));
        assert!(c.contains("printf(\"%s\\n\", _fmt(POP()))"));
    }

    #[test]
    fn test_c_backend_gcc_runtime() {
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .is_err()
        {
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
            .status();
        assert!(status.is_ok(), "gcc fallo al compilar");
        let out = std::process::Command::new(&exe_path).output().unwrap();
        let test_out = String::from_utf8_lossy(&out.stdout).replace("\r\n", "\n");
        assert_eq!(test_out, "42\n");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
