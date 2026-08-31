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
/// v3.5.40: canonicaliza `ArraySetVar(n)` → `ArraySet + Store(n)` para los
/// backends AOT (Cranelift/LLVM/C), que ya manejan ese par con semántica de
/// valores: cada celda es dueña exclusiva de su buffer (los Stores
/// deep-copian) → el set sobre la celda es O(1) y la paridad observable con
/// la VM se mantiene (en la VM, ArraySetVar es el mismo par con
/// Arc::make_mut sobre refcount 1). El IR canónico NO debe exponerse a la
/// VM/JIT (ahí ArraySetVar es el fast-path real).
pub fn lower_arraysetvar(program: &Program) -> Program {
    let mut p = program.clone();
    for func in p.funcs.values_mut() {
        let mut out: Vec<Instr> = Vec::with_capacity(func.instrs.len() + 8);
        for ins in func.instrs.drain(..) {
            if let Instr::ArraySetVar(n) = ins {
                out.push(Instr::ArraySet);
                out.push(Instr::Store(n));
            } else {
                out.push(ins);
            }
        }
        func.instrs = out;
    }
    p
}

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

// ── Índices de helpers _lw_* en AotCompiler::lw (v3.5.6, handles opacos) ──
const LW_INT: usize = 0;
const LW_FLT: usize = 1;
const LW_BOOL: usize = 2;
const LW_STR: usize = 3;
const LW_VOID: usize = 4;
const LW_NONE: usize = 5;
const LW_PRINT: usize = 6;
const LW_PRINT_BLANK: usize = 7;
const LW_JOIN: usize = 8;
const LW_BIN: usize = 9;
const LW_UN: usize = 10;
const LW_TRUTHY_I: usize = 11;
const LW_ARR_NEW: usize = 12;
const LW_ARR_PUSH: usize = 13;
const LW_ARR_GET: usize = 14;
const LW_ARR_SET: usize = 15;
const LW_ARR_LEN: usize = 16;
const LW_ARR_REV: usize = 17;
const LW_ARR_SORT: usize = 18;
const LW_ST_NEW: usize = 19;
const LW_ST_ADD: usize = 20;
const LW_ST_GET: usize = 21;
const LW_ST_SET: usize = 22;
const LW_TUP_NEW: usize = 23;
const LW_TUP_PUSH: usize = 24;
const LW_TUP_GET: usize = 25;
const LW_READ: usize = 26;
const LW_TYPEOF: usize = 27;
const LW_TO_TEXT: usize = 28;
const LW_SUB: usize = 29;
const LW_CONCAT_LIST: usize = 30;
const LW_SOME: usize = 31;
const LW_OK: usize = 32;
const LW_ERR: usize = 33;
const LW_MAP_NEW: usize = 34;
const LW_MAP_SET: usize = 35;
const LW_MAP_GET: usize = 36;
const LW_MAP_HAS: usize = 37;
const LW_MAP_LEN: usize = 38;
const LW_MAP_KEYS: usize = 39;
// ── Incremento B (v3.5.7) ──
const LW_TRY_BEGIN: usize = 40;
const LW_TRY_END: usize = 41;
const LW_ERR_ACTIVE: usize = 42;
const LW_ERR_TAKE: usize = 43;
const LW_KIND: usize = 44;
const LW_PAYLOAD: usize = 45;
const LW_ENM_NEW: usize = 46;
const LW_ENM_VARIANT_IS: usize = 47;
const LW_FREF: usize = 48;
const LW_FREF_ADDR: usize = 49;
const LW_MKREF: usize = 50;
const LW_LOAD_SLOT: usize = 51;
const LW_STORE_SLOT: usize = 52;
const LW_STORE_SLOT_DIRECT: usize = 53;
const LW_DCP: usize = 54;
const LW_ARR_PUSH_IP: usize = 55;
const LW_ABS: usize = 56;
const LW_SQRT: usize = 57;
const LW_POW: usize = 58;
const LW_FLOOR: usize = 59;
const LW_CEIL: usize = 60;
const LW_ROUND: usize = 61;
// ── v3.5.17: hilos reales en Cranelift/LLVM ──
const LW_CSTR: usize = 62;
const LW_THR_SPAWN: usize = 63;
const LW_THR_JOIN: usize = 64;
const LW_THR_ARG_HANDLE: usize = 65;
const LW_CHAN_NEW: usize = 66;
const LW_CHAN_SEND: usize = 67;
const LW_CHAN_RECV: usize = 68;
const LW_MUTEX_NEW: usize = 69;
const LW_MUTEX_LOCK_CALL: usize = 70;
const LW_CAL_HIJRI: usize = 71;
const LW_CAL_PERSA: usize = 72;
const LW_TIME_NOW: usize = 73;
const LW_TIME_FMT: usize = 74;
const LW_TIME_DIFF: usize = 75;
const LW_TIME_PARSE: usize = 76;
const LW_STR_CHARS: usize = 77;
const LW_STR_UPPER: usize = 78;
const LW_STR_LOWER: usize = 79;
const LW_STR_PAD_START: usize = 80;
const LW_STR_PAD_END: usize = 81;
// v3.5.25: extrae el entero de un handle (slots i64 de Cranelift).
const LW_H2I: usize = 82;
const LW_THROW_DIV: usize = 83;
const LW_IARR_PUSH: usize = 84;
const LW_IARR_GET: usize = 85;
// v3.5.30: fast-paths de strings (Cranelift): largo crudo, a_texto de
// entero crudo y concatenación triple literal+valor+literal.
const LW_ARR_LEN_I: usize = 86;
const LW_TO_TEXT_I: usize = 87;
const LW_CONCAT3: usize = 88;
const LW_CONCAT3_I: usize = 89;
const LW_CONCAT3_LEN_I: usize = 90;
const LW_COUNT: usize = 91;

/// sizeof(Val) en lumen_rt.h — tamaño de las celdas para variables
/// referenciadas (prestado mut) y params en los backends nativos.
/// (v3.5.7: el campo cap rellenó el padding de argc; sigue siendo 80 —
/// ver _lw_val_size_check en el shim)
const LW_VAL_SIZE: u32 = 80;

/// Builtins de `Call` soportados por el backend Cranelift vía `_lw_*`
/// (nombre del builtin → índice de helper + aridad esperada).
fn lw_builtin(name: &str) -> Option<(usize, usize)> {
    match name {
        "leer" | "read" => Some((LW_READ, 0)),
        "largo" | "len" | "length" | "__str_len" | "__str_longitud" => Some((LW_ARR_LEN, 1)),
        "agregar" | "push" => Some((LW_ARR_PUSH, 2)),
        "a_texto" | "to_texto" | "__str_from" => Some((LW_TO_TEXT, 1)),
        "__tipo_de" | "__typeof" => Some((LW_TYPEOF, 1)),
        "__str_subcadena" => Some((LW_SUB, 3)),
        "__str_concat_list" => Some((LW_CONCAT_LIST, 1)),
        "__lista_invertir" | "__list_reverse" => Some((LW_ARR_REV, 1)),
        "__lista_ordenar" | "__list_sort" => Some((LW_ARR_SORT, 1)),
        "__map_nuevo" => Some((LW_MAP_NEW, 0)),
        "__map_poner" => Some((LW_MAP_SET, 3)),
        "__map_obtener" => Some((LW_MAP_GET, 2)),
        "__map_contiene" => Some((LW_MAP_HAS, 2)),
        "__map_longitud" => Some((LW_MAP_LEN, 1)),
        "__map_claves" | "__map_keys" => Some((LW_MAP_KEYS, 1)),
        // Matematicas (paridad VM: los builtins tienen prioridad sobre funcs usuario)
        "abs" | "absoluto" => Some((LW_ABS, 1)),
        "raiz" | "sqrt" => Some((LW_SQRT, 1)),
        "potencia" | "pow" => Some((LW_POW, 2)),
        "piso" | "floor" => Some((LW_FLOOR, 1)),
        "techo" | "ceil" => Some((LW_CEIL, 1)),
        "redondear" | "round" => Some((LW_ROUND, 1)),
        _ => None,
    }
}

pub struct AotCompiler {
    module: ObjectModule,
    funcs: HashMap<String, FuncInfo>,
    string_data: HashMap<String, DataId>,
    /// Helpers `_lw_*` indexados por las constantes LW_* (handles opacos).
    lw: Vec<FuncId>,
    /// Celdas globales (variables top-level usadas desde varias funciones).
    globals: HashMap<String, DataId>,
}

impl Default for AotCompiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Delta de profundidad del stack de operandos por instrucción (para el
/// pre-pass de labels que reciben valores, p. ej. ternarios).
fn instr_depth_delta(ins: &Instr) -> i32 {
    match ins {
        Instr::ConstInt(_) | Instr::ConstFloat(_) | Instr::ConstBool(_) | Instr::ConstStr(_) => 1,
        Instr::Load(_) => 1,
        Instr::Store(_) | Instr::StoreLocal(_) => -1,
        Instr::Binary(_) => -1,
        Instr::Unary(_) => 0,
        Instr::Print => -1,
        Instr::Read => 1,
        Instr::Call(_, argc) => 1 - *argc as i32,
        Instr::MakeRef(_) => 1,
        Instr::ArrayNew(n) => 1 - *n as i32,
        Instr::ArrayPush => -1,
        Instr::ArrayPushVar(_) => 0,
        Instr::ArrayGet => -1,
        Instr::ArraySet => -2,
        // v3.5.40: ArraySetVar equivale al par canónico ArraySet + Store.
        Instr::ArraySetVar(_) => -3,
        Instr::ArrayLen => 0,
        Instr::StructNew(_, n) => 1 - 2 * (*n as i32),
        Instr::StructGet => -1,
        Instr::StructSet => -2,
        Instr::TupleNew(n) => 1 - *n as i32,
        Instr::TupleAccess(_) => 0,
        Instr::OptionSome | Instr::ResultOk | Instr::ResultErr => 0,
        Instr::OptionNone => 1,
        Instr::PushHandler(_) | Instr::PopHandler => 0,
        Instr::TryUnwrap => 0,
        Instr::MatchType(_) | Instr::MatchPayload | Instr::MatchVariant(_) => 0,
        Instr::EnumCtor { argc, .. } => 1 - *argc as i32,
        Instr::FuncRef(_) => 1,
        Instr::CallValue(argc) => -(*argc as i32),
        Instr::Return | Instr::Halt => -1,
        Instr::Jmp(_) => 0,
        Instr::JmpIf(_) => -1,
        Instr::Label(_) | Instr::Phi(..) | Instr::Nop | Instr::ScopePush | Instr::ScopePop => 0,
    }
}

/// Nombre de la función de entrada del programa (paridad entry_point).
fn program_entry_name(program: &Program) -> Option<String> {
    if program.funcs.contains_key(&program.entry) {
        Some(program.entry.clone())
    } else if program.funcs.contains_key("main") {
        Some("main".to_string())
    } else if program.funcs.contains_key("principal") {
        Some("principal".to_string())
    } else {
        None
    }
}

/// Variables GLOBALES: declaradas (StoreLocal) en la función de entrada y
/// usadas (Load/Store/StoreLocal/ArrayPushVar/MakeRef) desde OTRAS funciones.
/// Reciben celda de datos compartida entre funciones (paridad gv[] del C).
/// v3.5.15: CAPTURAS (Cranelift). Calcula, para cada función anidada, las
/// variables que captura de sus ancestros y las promueve a celdas globales con
/// nombre mangado `{ancestro}::{var}`. Devuelve:
///  - captures[func] = map var_capturada -> celda_global_mangada
///  - set de todos los nombres de celdas globales de captura (para global_names)
fn compute_captures(
    program: &Program,
) -> (
    HashMap<String, HashMap<String, String>>,
    std::collections::HashSet<String>,
) {
    let mut captures: HashMap<String, HashMap<String, String>> = HashMap::new();
    let mut cells: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (fname, func) in &program.funcs {
        // Params y locales propios de la función
        let mut own: std::collections::HashSet<String> = func.params.iter().cloned().collect();
        for ins in &func.instrs {
            if let Instr::StoreLocal(n) = ins {
                own.insert(n.clone());
            }
        }
        // Variables referenciadas que NO son propias → candidatas a captura
        let mut m: HashMap<String, String> = HashMap::new();
        for ins in &func.instrs {
            let refs: Vec<&String> = match ins {
                Instr::Load(n) | Instr::Store(n) | Instr::ArrayPushVar(n) | Instr::MakeRef(n) => {
                    vec![n]
                }
                _ => vec![],
            };
            for n in refs {
                if own.contains(n) || m.contains_key(n) {
                    continue;
                }
                // Buscar el ancestro más cercano que lo defina (param o local)
                let mut cur = program.parents.get(fname).cloned();
                while let Some(anc) = cur {
                    let defines = program.funcs.get(&anc).is_some_and(|af| {
                        let mut has = af.params.iter().any(|p| p == n);
                        if !has {
                            for ins2 in &af.instrs {
                                if let Instr::StoreLocal(x) = ins2 {
                                    if x == n {
                                        has = true;
                                        break;
                                    }
                                }
                            }
                        }
                        has
                    });
                    if defines {
                        let cell = format!("{}::{}", anc, n);
                        m.insert(n.clone(), cell.clone());
                        cells.insert(cell);
                        break;
                    }
                    cur = program.parents.get(&anc).cloned();
                }
            }
        }
        if !m.is_empty() {
            captures.insert(fname.clone(), m);
        }
    }
    (captures, cells)
}

fn program_global_names(program: &Program) -> std::collections::HashSet<String> {
    let entry_fn: Option<&str> = if program.funcs.contains_key(&program.entry) {
        Some(program.entry.as_str())
    } else if program.funcs.contains_key("main") {
        Some("main")
    } else if program.funcs.contains_key("principal") {
        Some("principal")
    } else {
        None
    };
    let mut declared_in_entry: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Some(ef) = entry_fn.and_then(|n| program.funcs.get(n)) {
        for ins in &ef.instrs {
            if let Instr::StoreLocal(n) = ins {
                declared_in_entry.insert(n.clone());
            }
        }
    }
    let mut used_elsewhere: std::collections::HashSet<String> = std::collections::HashSet::new();
    for (fname, func) in &program.funcs {
        if Some(fname.as_str()) == entry_fn {
            continue;
        }
        for ins in &func.instrs {
            match ins {
                Instr::Load(n)
                | Instr::Store(n)
                | Instr::StoreLocal(n)
                | Instr::ArrayPushVar(n)
                | Instr::MakeRef(n) => {
                    used_elsewhere.insert(n.clone());
                }
                _ => {}
            }
        }
    }
    declared_in_entry
        .intersection(&used_elsewhere)
        .cloned()
        .collect()
}

/// Simula profundidades y devuelve la profundidad del stack con la que se
/// llega a cada label (los catch labels reciben 1 valor: el mensaje).
fn simulate_label_depths(
    instrs: &[Instr],
    catch_labels: &std::collections::HashSet<usize>,
) -> HashMap<usize, usize> {
    let mut label_depth: HashMap<usize, usize> = HashMap::new();
    let mut depth: i32 = 0;
    for ins in instrs {
        match ins {
            Instr::Label(l) => {
                if let Some(&d) = label_depth.get(l) {
                    depth = d as i32;
                }
            }
            Instr::Jmp(t) => {
                label_depth.insert(*t, depth.max(0) as usize);
            }
            Instr::JmpIf(t) => {
                let d = (depth - 1).max(0) as usize;
                label_depth.insert(*t, d);
                depth = d as i32;
            }
            _ => {
                depth += instr_depth_delta(ins);
                if depth < 0 {
                    depth = 0;
                }
            }
        }
    }
    for l in catch_labels {
        label_depth.insert(*l, 1);
    }
    label_depth
}

/// v3.5.30: valor i64 actual de una variable entera — SSA si el cache lo
/// tiene (promoción de bucles), si no, carga del slot y lo memoriza.
fn cr_int_val(
    builder: &mut FunctionBuilder,
    cache: &mut HashMap<String, cranelift::codegen::ir::Value>,
    int_slots: &HashMap<String, cranelift::codegen::ir::StackSlot>,
    n: &str,
) -> cranelift::codegen::ir::Value {
    if let Some(&v) = cache.get(n) {
        return v;
    }
    let v = builder.ins().stack_load(types::I64, int_slots[n], 0);
    cache.insert(n.to_string(), v);
    v
}

/// v3.5.30: materializa los valores SSA cacheados en sus slots i64 (los
/// bordes de control que no pasan block-params usan esto antes de saltar).
/// No limpia el cache: la emisión es por-bloque y el otro camino del brif
/// sigue necesitando los valores SSA.
fn cr_flush_ints(
    builder: &mut FunctionBuilder,
    cache: &HashMap<String, cranelift::codegen::ir::Value>,
    int_slots: &HashMap<String, cranelift::codegen::ir::StackSlot>,
) {
    for (n, &v) in cache.iter() {
        if let Some(&ss) = int_slots.get(n) {
            builder.ins().stack_store(v, ss, 0);
        }
    }
}

impl AotCompiler {
    pub fn new() -> Self {
        let mut fb = settings::builder();
        fb.set("use_colocated_libcalls", "false").unwrap();
        fb.set("is_pic", "true").unwrap();
        // Fase 88: LTO + optimización agresiva
        fb.set("opt_level", "speed_and_size").unwrap();
        let flags = settings::Flags::new(fb);
        // v3.5.30: en macOS el triple de host es `*-apple-darwin`; con él,
        // cranelift-object escribe LC_BUILD_VERSION con platform=0
        // (PLATFORM_UNKNOWN) y el ld64 nuevo rechaza el objeto con
        // "ld: unknown platform in 'x.o'". Cambiando el OS a MacOSX se
        // escribe PLATFORM_MACOS y el enlace funciona. En el resto de
        // plataformas el triple queda idéntico al de cranelift_native.
        let mut triple = target_lexicon::Triple::host();
        if matches!(
            triple.operating_system,
            target_lexicon::OperatingSystem::Darwin(_)
        ) {
            triple.operating_system = target_lexicon::OperatingSystem::MacOSX(None);
        }
        let mut builder = cranelift::codegen::isa::lookup(triple)
            .map_err(|_| "Host not supported")
            .expect("Host not supported");
        cranelift_native::infer_native_flags(&mut builder).expect("Failed to infer native flags");
        let isa = builder.finish(flags).expect("Failed to create ISA");
        let obj_builder = ObjectBuilder::new(
            isa,
            "lumen".to_string(),
            cranelift_module::default_libcall_names(),
        )
        .unwrap();
        let mut module = ObjectModule::new(obj_builder);

        // Declaración de los helpers _lw_* (definidos en el shim de link:
        // lw_shim_source() = lumen_rt.h + LW_RUNTIME). Modelo de handles
        // opacos: todo valor LÚMEN es un i64 (puntero a Val).
        let decl = |module: &mut ObjectModule, name: &str, params: &[Type], ret: Option<Type>| {
            let mut sig = module.make_signature();
            for p in params {
                sig.params.push(AbiParam::new(*p));
            }
            if let Some(r) = ret {
                sig.returns.push(AbiParam::new(r));
            }
            module
                .declare_function(name, Linkage::Import, &sig)
                .unwrap()
        };
        let i = types::I64;
        let f = types::F64;
        let mut lw = Vec::with_capacity(LW_COUNT);
        // Orden EXACTO según las constantes LW_*
        lw.push(decl(&mut module, "_lw_int", &[i], Some(i))); // LW_INT
        lw.push(decl(&mut module, "_lw_flt", &[f], Some(i))); // LW_FLT
        lw.push(decl(&mut module, "_lw_bool", &[i], Some(i))); // LW_BOOL
        lw.push(decl(&mut module, "_lw_str", &[i], Some(i))); // LW_STR
        lw.push(decl(&mut module, "_lw_void", &[], Some(i))); // LW_VOID
        lw.push(decl(&mut module, "_lw_none", &[], Some(i))); // LW_NONE
        lw.push(decl(&mut module, "_lw_print", &[i], None)); // LW_PRINT
        lw.push(decl(&mut module, "_lw_print_blank", &[], None)); // LW_PRINT_BLANK
        lw.push(decl(&mut module, "_lw_join", &[i, i], Some(i))); // LW_JOIN
        lw.push(decl(&mut module, "_lw_bin", &[i, i, i], Some(i))); // LW_BIN
        lw.push(decl(&mut module, "_lw_un", &[i, i], Some(i))); // LW_UN
        lw.push(decl(&mut module, "_lw_truthy_i", &[i], Some(i))); // LW_TRUTHY_I
        lw.push(decl(&mut module, "_lw_arr_new", &[], Some(i))); // LW_ARR_NEW
        lw.push(decl(&mut module, "_lw_arr_push", &[i, i], Some(i))); // LW_ARR_PUSH
        lw.push(decl(&mut module, "_lw_arr_get", &[i, i], Some(i))); // LW_ARR_GET
        lw.push(decl(&mut module, "_lw_arr_set", &[i, i, i], Some(i))); // LW_ARR_SET
        lw.push(decl(&mut module, "_lw_arr_len", &[i], Some(i))); // LW_ARR_LEN
        lw.push(decl(&mut module, "_lw_arr_rev", &[i], Some(i))); // LW_ARR_REV
        lw.push(decl(&mut module, "_lw_arr_sort", &[i], Some(i))); // LW_ARR_SORT
        lw.push(decl(&mut module, "_lw_st_new", &[], Some(i))); // LW_ST_NEW
        lw.push(decl(&mut module, "_lw_st_add", &[i, i, i], Some(i))); // LW_ST_ADD
        lw.push(decl(&mut module, "_lw_st_get", &[i, i], Some(i))); // LW_ST_GET
        lw.push(decl(&mut module, "_lw_st_set", &[i, i, i], Some(i))); // LW_ST_SET
        lw.push(decl(&mut module, "_lw_tup_new", &[], Some(i))); // LW_TUP_NEW
        lw.push(decl(&mut module, "_lw_tup_push", &[i, i], Some(i))); // LW_TUP_PUSH
        lw.push(decl(&mut module, "_lw_tup_get", &[i, i], Some(i))); // LW_TUP_GET
        lw.push(decl(&mut module, "_lw_read", &[], Some(i))); // LW_READ
        lw.push(decl(&mut module, "_lw_typeof", &[i], Some(i))); // LW_TYPEOF
        lw.push(decl(&mut module, "_lw_to_text", &[i], Some(i))); // LW_TO_TEXT
        lw.push(decl(&mut module, "_lw_sub", &[i, i, i], Some(i))); // LW_SUB
        lw.push(decl(&mut module, "_lw_concat_list", &[i], Some(i))); // LW_CONCAT_LIST
        lw.push(decl(&mut module, "_lw_some", &[i], Some(i))); // LW_SOME
        lw.push(decl(&mut module, "_lw_ok", &[i], Some(i))); // LW_OK
        lw.push(decl(&mut module, "_lw_err", &[i], Some(i))); // LW_ERR
        lw.push(decl(&mut module, "_lw_map_new", &[], Some(i))); // LW_MAP_NEW
        lw.push(decl(&mut module, "_lw_map_set", &[i, i, i], Some(i))); // LW_MAP_SET
        lw.push(decl(&mut module, "_lw_map_get", &[i, i], Some(i))); // LW_MAP_GET
        lw.push(decl(&mut module, "_lw_map_has", &[i, i], Some(i))); // LW_MAP_HAS
        lw.push(decl(&mut module, "_lw_map_len", &[i], Some(i))); // LW_MAP_LEN
        lw.push(decl(&mut module, "_lw_map_keys", &[i], Some(i))); // LW_MAP_KEYS
                                                                   // ── Incremento B (v3.5.7) ──
        lw.push(decl(&mut module, "_lw_try_begin", &[], None)); // LW_TRY_BEGIN
        lw.push(decl(&mut module, "_lw_try_end", &[], None)); // LW_TRY_END
        lw.push(decl(&mut module, "_lw_err_active", &[], Some(i))); // LW_ERR_ACTIVE
        lw.push(decl(&mut module, "_lw_err_take", &[], Some(i))); // LW_ERR_TAKE
        lw.push(decl(&mut module, "_lw_kind", &[i], Some(i))); // LW_KIND
        lw.push(decl(&mut module, "_lw_payload", &[i], Some(i))); // LW_PAYLOAD
        lw.push(decl(&mut module, "_lw_enm_new", &[i, i, i], Some(i))); // LW_ENM_NEW
        lw.push(decl(&mut module, "_lw_enm_variant_is", &[i, i], Some(i))); // LW_ENM_VARIANT_IS
        lw.push(decl(&mut module, "_lw_fref", &[i, i], Some(i))); // LW_FREF
        lw.push(decl(&mut module, "_lw_fref_addr", &[i], Some(i))); // LW_FREF_ADDR
        lw.push(decl(&mut module, "_lw_mkref", &[i], Some(i))); // LW_MKREF
        lw.push(decl(&mut module, "_lw_load_slot", &[i], Some(i))); // LW_LOAD_SLOT
        lw.push(decl(&mut module, "_lw_store_slot", &[i, i], None)); // LW_STORE_SLOT
        lw.push(decl(&mut module, "_lw_store_slot_direct", &[i, i], None)); // LW_STORE_SLOT_DIRECT
        lw.push(decl(&mut module, "_lw_dcp", &[i], Some(i))); // LW_DCP
        lw.push(decl(&mut module, "_lw_arr_push_ip", &[i, i], Some(i))); // LW_ARR_PUSH_IP
        lw.push(decl(&mut module, "_lw_abs", &[i], Some(i))); // LW_ABS
        lw.push(decl(&mut module, "_lw_sqrt", &[i], Some(i))); // LW_SQRT
        lw.push(decl(&mut module, "_lw_pow", &[i, i], Some(i))); // LW_POW
        lw.push(decl(&mut module, "_lw_floor", &[i], Some(i))); // LW_FLOOR
        lw.push(decl(&mut module, "_lw_ceil", &[i], Some(i))); // LW_CEIL
        lw.push(decl(&mut module, "_lw_round", &[i], Some(i))); // LW_ROUND
                                                                // ── v3.5.17: hilos reales (Cranelift) ──
        lw.push(decl(&mut module, "_lw_cstr", &[i], Some(i))); // LW_CSTR
        lw.push(decl(&mut module, "_lw_thr_spawn_h", &[i, i, i], Some(i))); // LW_THR_SPAWN
        lw.push(decl(&mut module, "_lw_thr_join_h", &[i], Some(i))); // LW_THR_JOIN
        lw.push(decl(&mut module, "_lw_thr_arg_handle", &[i], Some(i))); // LW_THR_ARG_HANDLE
        lw.push(decl(&mut module, "_lw_chan_new_h", &[], Some(i))); // LW_CHAN_NEW
        lw.push(decl(&mut module, "_lw_chan_send_h", &[i, i], Some(i))); // LW_CHAN_SEND
        lw.push(decl(&mut module, "_lw_chan_recv_h", &[i], Some(i))); // LW_CHAN_RECV
        lw.push(decl(&mut module, "_lw_mutex_new_h", &[], Some(i))); // LW_MUTEX_NEW
        lw.push(decl(
            &mut module,
            "_lw_mutex_lock_call_h",
            &[i, i, i],
            Some(i),
        )); // LW_MUTEX_LOCK_CALL
        lw.push(decl(&mut module, "_lw_cal_hijri_h", &[i], Some(i))); // LW_CAL_HIJRI
        lw.push(decl(&mut module, "_lw_cal_persa_h", &[i], Some(i))); // LW_CAL_PERSA
        lw.push(decl(&mut module, "_lw_time_now_h", &[], Some(i))); // LW_TIME_NOW
        lw.push(decl(&mut module, "_lw_time_fmt_h", &[i, i], Some(i))); // LW_TIME_FMT
        lw.push(decl(&mut module, "_lw_time_diff_h", &[i, i], Some(i))); // LW_TIME_DIFF
        lw.push(decl(&mut module, "_lw_time_parse_h", &[i], Some(i))); // LW_TIME_PARSE
        lw.push(decl(&mut module, "_lw_str_chars_h", &[i], Some(i))); // LW_STR_CHARS
        lw.push(decl(&mut module, "_lw_str_upper_h", &[i], Some(i))); // LW_STR_UPPER
        lw.push(decl(&mut module, "_lw_str_lower_h", &[i], Some(i))); // LW_STR_LOWER
        lw.push(decl(&mut module, "_lw_str_pad_h", &[i, i, i, i], Some(i))); // LW_STR_PAD_START
        lw.push(decl(&mut module, "_lw_str_pad_h", &[i, i, i, i], Some(i))); // LW_STR_PAD_END
        lw.push(decl(&mut module, "_lw_h2i", &[i], Some(i))); // LW_H2I
        lw.push(decl(&mut module, "_lw_throw_div", &[], None)); // LW_THROW_DIV
        lw.push(decl(&mut module, "_lw_iarr_push", &[i, i, i, i], None)); // LW_IARR_PUSH
        lw.push(decl(&mut module, "_lw_iarr_get", &[i, i, i], Some(i))); // LW_IARR_GET
        lw.push(decl(&mut module, "_lw_arr_len_i", &[i], Some(i))); // LW_ARR_LEN_I
        lw.push(decl(&mut module, "_lw_to_text_i", &[i], Some(i))); // LW_TO_TEXT_I
        lw.push(decl(&mut module, "_lw_concat3", &[i, i, i], Some(i))); // LW_CONCAT3
        lw.push(decl(&mut module, "_lw_concat3_i", &[i, i, i], Some(i))); // LW_CONCAT3_I
        lw.push(decl(&mut module, "_lw_concat3_len_i", &[i, i, i], Some(i))); // LW_CONCAT3_LEN_I
        debug_assert_eq!(lw.len(), LW_COUNT);

        Self {
            module,
            funcs: HashMap::new(),
            string_data: HashMap::new(),
            lw,
            globals: HashMap::new(),
        }
    }

    /// Dirección (i64) de la celda global `name` (data zeroinit de 80B).
    fn global_addr(&mut self, builder: &mut FunctionBuilder, name: &str) -> Value {
        let id = match self.globals.get(name) {
            Some(&id) => id,
            None => {
                let gname = format!("lw_glob_{}", mangle(name));
                let id = self
                    .module
                    .declare_data(&gname, Linkage::Local, false, false)
                    .unwrap();
                let mut desc = cranelift_module::DataDescription::new();
                desc.set_align(8);
                desc.define_zeroinit(LW_VAL_SIZE as usize);
                self.module.define_data(id, &desc).ok();
                self.globals.insert(name.to_string(), id);
                id
            }
        };
        let gv = self.module.declare_data_in_func(id, builder.func);
        builder.ins().global_value(types::I64, gv)
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

    /// Llama a un helper `_lw_*` (índice LW_*) y devuelve el handle i64.
    fn lw_call(&mut self, builder: &mut FunctionBuilder, idx: usize, args: &[Value]) -> Value {
        let fid = self.lw[idx];
        let fref = self.module.declare_func_in_func(fid, builder.func);
        let call = builder.ins().call(fref, args);
        builder.inst_results(call)[0]
    }

    /// Llama a un helper `_lw_*` sin retorno (p. ej. _lw_print).
    fn lw_call_void(&mut self, builder: &mut FunctionBuilder, idx: usize, args: &[Value]) {
        let fid = self.lw[idx];
        let fref = self.module.declare_func_in_func(fid, builder.func);
        builder.ins().call(fref, args);
    }

    /// Handle `void` para fallbacks (underflow de pila / no soportado).
    fn void_handle(&mut self, builder: &mut FunctionBuilder) -> Value {
        self.lw_call(builder, LW_VOID, &[])
    }

    /// v3.5.28: stack con kinds del backend Cranelift. Boxea un valor crudo
    /// a handle según su kind: 0=handle (pasa tal cual), 1=i64 entero crudo,
    /// 2=b8 bool crudo (icmp).
    fn cr_box(&mut self, builder: &mut FunctionBuilder, v: Value, k: u8) -> Value {
        match k {
            1 => self.lw_call(builder, LW_INT, &[v]),
            2 => {
                // b8 (icmp) → i64 0/1 vía select (cranelift 0.132 no trae bint)
                let one = builder.ins().iconst(types::I64, 1);
                let zero = builder.ins().iconst(types::I64, 0);
                let iv = builder.ins().select(v, one, zero);
                self.lw_call(builder, LW_BOOL, &[iv])
            }
            _ => v,
        }
    }

    /// v3.5.28: pop con fallback a void (paridad del unwrap_or_else previo).
    fn cr_pop(
        &mut self,
        builder: &mut FunctionBuilder,
        stack: &mut Vec<(Value, u8)>,
    ) -> (Value, u8) {
        stack
            .pop()
            .unwrap_or_else(|| (self.void_handle(builder), 0))
    }

    /// v3.5.30: cuerpo común de StoreLocal (declaración). La fusión
    /// single-use lo evita; el resto de caminos caen aquí (paridad exacta
    /// con el código inline previo).
    #[allow(clippy::too_many_arguments)]
    fn cr_store_local<F: Fn(&str) -> Option<String>>(
        &mut self,
        builder: &mut FunctionBuilder,
        n: &str,
        v: Value,
        k: u8,
        scopes: &mut [HashMap<String, cranelift::codegen::ir::StackSlot>],
        slots: &HashMap<String, cranelift::codegen::ir::StackSlot>,
        global_names: &std::collections::HashSet<String>,
        int_cache: &mut HashMap<String, cranelift::codegen::ir::Value>,
        int_slots: &HashMap<String, cranelift::codegen::ir::StackSlot>,
        promoted_vars: &std::collections::HashSet<String>,
        is_entry: bool,
        cap_cell_for: &F,
    ) {
        if let Some(_ss) = int_slots.get(n).copied() {
            // v3.5.28: slot i64 — sin box si ya viene crudo
            let iv = if k == 1 {
                v
            } else {
                let h = self.cr_box(builder, v, k);
                self.lw_call(builder, LW_H2I, &[h])
            };
            // v3.5.30: var promovida → solo cache SSA (el slot se
            // materializa en los bordes).
            int_cache.insert(n.to_string(), iv);
            if !promoted_vars.contains(n) {
                builder.ins().stack_store(iv, int_slots[n], 0);
            }
        } else {
            let h = self.cr_box(builder, v, k);
            if let Some(cell) = cap_cell_for(n) {
                let addr = self.global_addr(builder, &cell);
                self.lw_call_void(builder, LW_STORE_SLOT, &[addr, h]);
            } else if let Some(&exist) = scopes.last().unwrap().get(n) {
                let addr = builder.ins().stack_addr(types::I64, exist, 0);
                self.lw_call_void(builder, LW_STORE_SLOT, &[addr, h]);
            } else if scopes.len() == 1 && slots.contains_key(n) {
                let addr = builder.ins().stack_addr(types::I64, slots[n], 0);
                self.lw_call_void(builder, LW_STORE_SLOT, &[addr, h]);
            } else if scopes.len() == 1 && is_entry && global_names.contains(n) {
                // declaración top-level → celda global compartida
                let addr = self.global_addr(builder, n);
                self.lw_call_void(builder, LW_STORE_SLOT, &[addr, h]);
            } else {
                let ss = builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData {
                    kind: cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                    size: LW_VAL_SIZE,
                    align_shift: 3,
                    key: None,
                });
                let addr = builder.ins().stack_addr(types::I64, ss, 0);
                self.lw_call_void(builder, LW_STORE_SLOT, &[addr, h]);
                scopes.last_mut().unwrap().insert(n.to_string(), ss);
            }
        }
    }

    /// v3.5.28: Div/Mod entero NATIVO con zero-check inline (paridad exacta
    /// con `_lw_bin`: lanza "Error: Division por cero" vía _err; INT64_MIN/-1
    /// → MIN en división y 0 en módulo). El caso común (b ≠ 0, sin MIN/-1)
    /// ejecuta 1 icmp + 1 sdiv/srem sin ninguna llamada al runtime.
    fn cr_emit_divmod(
        &mut self,
        builder: &mut FunctionBuilder,
        a: Value,
        b: Value,
        is_div: bool,
    ) -> Value {
        use cranelift::codegen::ir::condcodes::IntCC;
        let zero = builder.ins().iconst(types::I64, 0);
        let is_zero = builder.ins().icmp(IntCC::Equal, b, zero);
        let fast = builder.create_block();
        let slow = builder.create_block();
        let cont = builder.create_block();
        builder.append_block_param(cont, types::I64);
        builder.ins().brif(is_zero, slow, &[], fast, &[]);
        // slow: throw (pone _err=1 con handler abierto; si no, exit(3))
        builder.switch_to_block(slow);
        builder.ensure_inserted_block();
        self.lw_call_void(builder, LW_THROW_DIV, &[]);
        builder
            .ins()
            .jump(cont, &[cranelift::codegen::ir::BlockArg::Value(zero)]);
        // fast: guarda INT64_MIN / -1 y hace la operación nativa
        builder.switch_to_block(fast);
        builder.ensure_inserted_block();
        let minv = builder.ins().iconst(types::I64, i64::MIN);
        let m1 = builder.ins().iconst(types::I64, -1);
        let is_min = builder.ins().icmp(IntCC::Equal, a, minv);
        let is_m1 = builder.ins().icmp(IntCC::Equal, b, m1);
        let both = builder.ins().band(is_min, is_m1);
        let edge = builder.create_block();
        let do_it = builder.create_block();
        builder.ins().brif(both, edge, &[], do_it, &[]);
        builder.switch_to_block(edge);
        builder.ensure_inserted_block();
        let edge_res = if is_div { minv } else { zero };
        builder
            .ins()
            .jump(cont, &[cranelift::codegen::ir::BlockArg::Value(edge_res)]);
        builder.switch_to_block(do_it);
        builder.ensure_inserted_block();
        let r = if is_div {
            builder.ins().sdiv(a, b)
        } else {
            builder.ins().srem(a, b)
        };
        builder
            .ins()
            .jump(cont, &[cranelift::codegen::ir::BlockArg::Value(r)]);
        builder.switch_to_block(cont);
        builder.ensure_inserted_block();
        builder.block_params(cont)[0]
    }

    /// v3.5.28: pop que SIEMPRE devuelve un handle (boxea crudos). Lo usan
    /// los consumidores genéricos (builtins, arrays, structs, print...).
    fn pop_handle(&mut self, builder: &mut FunctionBuilder, stack: &mut Vec<(Value, u8)>) -> Value {
        let (v, k) = self.cr_pop(builder, stack);
        self.cr_box(builder, v, k)
    }

    pub fn compile(mut self, program: &Program) -> ObjectProduct {
        let program = lower_arraysetvar(program);
        for (name, func) in &program.funcs {
            self.declare(name, func);
        }
        let mut global_names = program_global_names(&program);
        // v3.5.15: capturas — promover variables capturadas a celdas globales.
        let (captures, cap_cells) = compute_captures(&program);
        for c in &cap_cells {
            global_names.insert(c.clone());
        }
        // v3.5.28: análisis de params enteros y retornos enteros (Cranelift)
        // — interprocedurales, réplica del backend C. Habilitan el ABI de
        // enteros crudos en llamadas directas (sin boxing por operación).
        let (_cr_direct, cr_dyn) = cr_call_graph(&program);
        let cr_params_int =
            cr_params_int_analysis(&program, &cr_dyn, &global_names, &captures, &cap_cells);
        let cr_returns_int = cr_returns_int_analysis(
            &program,
            &cr_dyn,
            &cr_params_int,
            &global_names,
            &captures,
            &cap_cells,
        );
        // v3.5.28: funciones que nunca lanzan → sin ERRCHK tras sus llamadas
        // (mismo análisis que el backend C; fib pasa a ser llamada limpia).
        let cr_no_throw = no_throw_analysis(&program);
        let names: Vec<String> = program.funcs.keys().cloned().collect();
        for n in &names {
            if let Some(f) = program.funcs.get(n) {
                let is_entry = program_entry_name(&program).as_deref() == Some(n.as_str());
                self.compile_body(
                    n,
                    f,
                    &global_names,
                    &captures,
                    &cap_cells,
                    is_entry,
                    &cr_params_int,
                    &cr_returns_int,
                    &cr_no_throw,
                );
            }
        }
        self.entry_point(&program.entry);
        // v3.5.17: hilos reales — trampolines __lumen_ft_<fn> (Export) que
        // toma los args estagiados en lw_thr_args (TLS, vía helper C) y llama
        // a la función nativa. El shim C arma la tabla _lft de dispatch.
        self.thread_trampolines(&program);
        self.module.finish()
    }

    /// v3.5.17: un trampolín `__lumen_ft_<fn>` por función del programa.
    /// Firma `int64_t(void)`: lee cada argumento con `_lw_thr_arg_handle(k)`
    /// (deep-copy del Val estagiado en el TLS del hilo; void si falta), llama
    /// a la función compilada y devuelve el handle del resultado.
    fn thread_trampolines(&mut self, program: &Program) {
        let fnames: Vec<String> = program.funcs.keys().cloned().collect();
        for name in fnames {
            let Some(func) = program.funcs.get(&name) else {
                continue;
            };
            let Some(info) = self.funcs.get(&name).cloned() else {
                continue;
            };
            let tname = format!("__lumen_ft_{}", mangle(&name));
            let mut sig = self.module.make_signature();
            sig.returns.push(AbiParam::new(types::I64));
            let tid = self
                .module
                .declare_function(&tname, Linkage::Export, &sig)
                .unwrap();

            let mut ctx = self.module.make_context();
            ctx.func = cranelift::codegen::ir::Function::with_name_signature(
                cranelift::codegen::ir::UserFuncName::user(0, tid.as_u32()),
                sig,
            );
            let mut func_ctx = FunctionBuilderContext::new();
            let mut builder = FunctionBuilder::new(&mut ctx.func, &mut func_ctx);
            let block = builder.create_block();
            builder.switch_to_block(block);
            builder.ensure_inserted_block();

            let mut hargs: Vec<Value> = Vec::with_capacity(func.params.len());
            for k in 0..func.params.len() {
                let hv = if k < 8 {
                    let kv = builder.ins().iconst(types::I64, k as i64);
                    self.lw_call(&mut builder, LW_THR_ARG_HANDLE, &[kv])
                } else {
                    self.lw_call(&mut builder, LW_VOID, &[])
                };
                hargs.push(hv);
            }
            let fref = self.module.declare_func_in_func(info.id, builder.func);
            let call = builder.ins().call(fref, &hargs);
            let res = builder.inst_results(call)[0];
            builder.ins().return_(&[res]);
            builder.seal_block(block);
            builder.finalize();
            self.module.define_function(tid, &mut ctx).unwrap();
            self.module.clear_context(&mut ctx);
        }
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

    /// v3.5.25: fusión de patrones enteros sobre slots i64 (Cranelift).
    /// Load/Const; Load/Const; BinArith/Cmp; Store/JmpIf → código nativo
    /// sin boxing. Devuelve nº de instrucciones consumidas.
    #[allow(clippy::too_many_arguments)]
    fn cr_fused_int(
        &mut self,
        builder: &mut FunctionBuilder,
        instrs: &[Instr],
        ii: usize,
        int_slots: &HashMap<String, cranelift::codegen::ir::StackSlot>,
        int_cache: &mut HashMap<String, cranelift::codegen::ir::Value>,
        head_vars: &HashMap<usize, Vec<String>>,
        promoted_vars: &std::collections::HashSet<String>,
        label_block: &HashMap<usize, cranelift::codegen::ir::Block>,
        label_depth: &HashMap<usize, usize>,
        stack: &mut Vec<(cranelift::codegen::ir::Value, u8)>,
        cur: &mut cranelift::codegen::ir::Block,
    ) -> Option<usize> {
        use cranelift::codegen::ir::condcodes::IntCC;
        let i0 = &instrs[ii];
        let i1 = &instrs[ii + 1];
        let i2 = &instrs[ii + 2];
        let i3 = &instrs[ii + 3];
        // operandos A y B
        let (_na, ka) = match (i0, i1) {
            (Instr::Load(a), Instr::Load(b)) => (Some(a.clone()), Some(b.clone())),
            (Instr::Load(a), Instr::ConstInt(_k)) => (Some(a.clone()), None),
            (Instr::ConstInt(_k), Instr::Load(b)) => (None, Some(b.clone())),
            _ => return None,
        };
        let _ = ka;
        let op = match i2 {
            Instr::Binary(op) => op.clone(),
            _ => return None,
        };
        let is_arith = matches!(
            op,
            Op::Add
                | Op::Sub
                | Op::Mul
                | Op::BitAnd
                | Op::BitOr
                | Op::BitXor
                | Op::ShiftLeft
                | Op::ShiftRight
        );
        let cc = match op {
            Op::Less => Some(IntCC::SignedLessThan),
            Op::LessEqual => Some(IntCC::SignedLessThanOrEqual),
            Op::Greater => Some(IntCC::SignedGreaterThan),
            Op::GreaterEqual => Some(IntCC::SignedGreaterThanOrEqual),
            Op::Equal => Some(IntCC::Equal),
            Op::NotEqual => Some(IntCC::NotEqual),
            _ => None,
        };
        if !is_arith && cc.is_none() {
            return None;
        }
        // cargar A y B según combinación
        let _load_operand = |n: Option<String>, ci: Option<i64>| -> Option<Value> {
            if let Some(nm) = n {
                let ss = *int_slots.get(&nm)?;
                Some(builder.ins().stack_load(types::I64, ss, 0))
            } else {
                Some(builder.ins().iconst(types::I64, ci.unwrap()))
            }
        };
        match (i0, i1) {
            (Instr::Load(a), Instr::Load(b)) => {
                let _ = int_slots.get(a)?;
                let _ = int_slots.get(b)?;
                // v3.5.30: operandos vía cache SSA (promoción de bucles).
                let av = cr_int_val(builder, int_cache, int_slots, a);
                let bv = cr_int_val(builder, int_cache, int_slots, b);
                self.cr_emit_fused_tail(
                    builder,
                    instrs,
                    ii,
                    av,
                    bv,
                    is_arith,
                    cc,
                    i3,
                    int_slots,
                    int_cache,
                    head_vars,
                    promoted_vars,
                    label_block,
                    label_depth,
                    stack,
                    cur,
                )
            }
            (Instr::Load(a), Instr::ConstInt(k)) => {
                let _ = int_slots.get(a)?;
                let av = cr_int_val(builder, int_cache, int_slots, a);
                let bv = builder.ins().iconst(types::I64, *k);
                self.cr_emit_fused_tail(
                    builder,
                    instrs,
                    ii,
                    av,
                    bv,
                    is_arith,
                    cc,
                    i3,
                    int_slots,
                    int_cache,
                    head_vars,
                    promoted_vars,
                    label_block,
                    label_depth,
                    stack,
                    cur,
                )
            }
            (Instr::ConstInt(k), Instr::Load(b)) => {
                let _ = int_slots.get(b)?;
                let bv = cr_int_val(builder, int_cache, int_slots, b);
                let av = builder.ins().iconst(types::I64, *k);
                self.cr_emit_fused_tail(
                    builder,
                    instrs,
                    ii,
                    av,
                    bv,
                    is_arith,
                    cc,
                    i3,
                    int_slots,
                    int_cache,
                    head_vars,
                    promoted_vars,
                    label_block,
                    label_depth,
                    stack,
                    cur,
                )
            }
            _ => None,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn cr_emit_fused_tail(
        &mut self,
        builder: &mut FunctionBuilder,
        _instrs: &[Instr],
        _ii: usize,
        av: Value,
        bv: Value,
        is_arith: bool,
        cc: Option<cranelift::codegen::ir::condcodes::IntCC>,
        i3: &Instr,
        int_slots: &HashMap<String, cranelift::codegen::ir::StackSlot>,
        int_cache: &mut HashMap<String, cranelift::codegen::ir::Value>,
        head_vars: &HashMap<usize, Vec<String>>,
        promoted_vars: &std::collections::HashSet<String>,
        label_block: &HashMap<usize, cranelift::codegen::ir::Block>,
        label_depth: &HashMap<usize, usize>,
        stack: &mut Vec<(cranelift::codegen::ir::Value, u8)>,
        cur: &mut cranelift::codegen::ir::Block,
    ) -> Option<usize> {
        if is_arith {
            let opv = match &_instrs[_ii + 2] {
                Instr::Binary(op) => op.clone(),
                _ => return None,
            };
            let r = match opv {
                Op::Add => builder.ins().iadd(av, bv),
                Op::Sub => builder.ins().isub(av, bv),
                Op::Mul => builder.ins().imul(av, bv),
                Op::BitAnd => builder.ins().band(av, bv),
                Op::BitOr => builder.ins().bor(av, bv),
                Op::BitXor => builder.ins().bxor(av, bv),
                Op::ShiftLeft => {
                    let m = builder.ins().iconst(types::I64, 63);
                    let s = builder.ins().band(bv, m);
                    builder.ins().ishl(av, s)
                }
                Op::ShiftRight => {
                    let m = builder.ins().iconst(types::I64, 63);
                    let s = builder.ins().band(bv, m);
                    builder.ins().ushr(av, s)
                }
                _ => return None,
            };
            // Store a slot entero promovido → nativo completo (4 instrs).
            if let Instr::Store(d) | Instr::StoreLocal(d) = i3 {
                if let Some(&sd) = int_slots.get(d) {
                    if promoted_vars.contains(d) {
                        // v3.5.30: variable promovida en bucles — solo cache
                        // SSA; el slot se materializa en los bordes (flush).
                        int_cache.insert(d.clone(), r);
                        return Some(4);
                    }
                    int_cache.insert(d.clone(), r);
                    builder.ins().stack_store(r, sd, 0);
                    return Some(4);
                }
            }
            // v3.5.28: otro consumidor (Call, concat, retorno...): el
            // resultado CRUDO viaja por el stack con kind; cada consumidor
            // boxea solo si lo necesita (antes: box aquí + unbox en el
            // call-arm de params int → 2 llamadas por argumento).
            stack.push((r, 1));
            return Some(3);
        }
        // comparación + JmpIf — v3.5.28: REACTIVADA (era el coste restante de
        // los bucles: box de ambos operandos + _lw_bin + truthy por iteración).
        // Guardas: stack vacío (sin valores pendientes que pasar por block-params),
        // label destino sin depth (sin block-params ni catch) — semántica JmpIf:
        // salta si FALSO → brif(cond → sigue, !cond → destino).
        let t = match i3 {
            Instr::JmpIf(t) => *t,
            _ => return None,
        };
        if !stack.is_empty() {
            return None;
        }
        if label_depth.get(&t).copied().unwrap_or(0) != 0 {
            return None;
        }
        let tb = *label_block.get(&t)?;
        let cond = builder.ins().icmp(cc?, av, bv);
        let nb = builder.create_block();
        // v3.5.30: el borde del salto — si el destino es un head promovido,
        // los valores viajan por block-params; si no, se materializan los
        // slots (flush) en un bloque intermedio ANTES del salto.
        if let Some(vars) = head_vars.get(&t) {
            let mut args: Vec<cranelift::codegen::ir::BlockArg> = Vec::new();
            for v in vars {
                let vv = cr_int_val(builder, int_cache, int_slots, v);
                args.push(cranelift::codegen::ir::BlockArg::Value(vv));
            }
            builder.ins().brif(cond, nb, &[], tb, &args);
        } else {
            let pre = builder.create_block();
            builder.ins().brif(cond, nb, &[], pre, &[]);
            builder.switch_to_block(pre);
            builder.ensure_inserted_block();
            cr_flush_ints(builder, int_cache, int_slots);
            builder.ins().jump(tb, &[]);
        }
        builder.switch_to_block(nb);
        builder.ensure_inserted_block();
        *cur = nb;
        Some(4)
    }

    #[allow(clippy::too_many_arguments)]
    fn compile_body(
        &mut self,
        name: &str,
        func: &LumenFunc,
        global_names: &std::collections::HashSet<String>,
        captures: &HashMap<String, HashMap<String, String>>,
        cap_cells: &std::collections::HashSet<String>,
        is_entry: bool,
        params_int: &HashMap<String, Vec<bool>>,
        returns_int: &HashMap<String, bool>,
        no_throw: &std::collections::HashSet<String>,
    ) {
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

        let instrs = &func.instrs;

        // v3.5.15: resolución de capturas. Para una variable `n` en la función
        // `name`, devuelve la celda global mangada si está capturada:
        //   - si `name` captura `n` de un ancestro → captures[name][n]
        //   - si `name` define `n` y un anidado lo captura → "{name}::{n}"
        let cap_cell_for = |n: &str| -> Option<String> {
            if let Some(cm) = captures.get(name) {
                if let Some(c) = cm.get(n) {
                    return Some(c.clone());
                }
            }
            let own_cell = format!("{}::{}", name, n);
            if cap_cells.contains(&own_cell) {
                return Some(own_cell);
            }
            None
        };

        // ── Pre-pass (v3.5.7): catch labels (reciben msg por block-param),
        //    objetivos de MakeRef (celdas estables) y nombres usados ──
        let mut catch_labels: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut ref_targets: Vec<String> = Vec::new();
        let mut used_names: Vec<String> = Vec::new();
        for ins in instrs {
            match ins {
                Instr::PushHandler(l) => {
                    catch_labels.insert(*l);
                }
                Instr::MakeRef(n) => {
                    if !ref_targets.iter().any(|x| x == n) {
                        ref_targets.push(n.clone());
                    }
                }
                Instr::Load(n)
                | Instr::Store(n)
                | Instr::StoreLocal(n)
                | Instr::ArrayPushVar(n)
                    if !used_names.iter().any(|x| x == n) =>
                {
                    used_names.push(n.clone());
                }
                _ => {}
            }
        }

        // ── Bloques: uno por Label. Los labels por los que fluyen valores
        //    (ternarios, elegir como expresión) reciben block-params; los
        //    catch reciben el mensaje de error. ──
        let label_depth = simulate_label_depths(instrs, &catch_labels);
        let entry_block = builder.create_block();
        let mut label_block: HashMap<usize, Block> = HashMap::new();
        for ins in instrs {
            if let Instr::Label(n) = ins {
                let b = builder.create_block();
                let d = label_depth.get(n).copied().unwrap_or(0);
                for _ in 0..d {
                    builder.append_block_param(b, i64);
                }
                label_block.insert(*n, b);
            }
        }

        builder.switch_to_block(entry_block);
        for _ in &func.params {
            builder.append_block_param(entry_block, i64);
        }
        builder.ensure_inserted_block();
        let entry_params: Vec<Value> = builder.block_params(entry_block).to_vec();

        // v3.5.20: el GC conservador necesita el tope del stack (frame de
        // main) → llamar `_lw_gc_init()` al entrar al programa.
        if is_entry {
            let sig_gc = self.module.make_signature();
            let gc_id = self
                .module
                .declare_function("_lw_gc_init", Linkage::Import, &sig_gc)
                .unwrap();
            let gc_ref = self.module.declare_func_in_func(gc_id, builder.func);
            builder.ins().call(gc_ref, &[]);
        }

        // v3.5.28: nombres de params con ABI de entero CRUDO — se calculan
        // antes de las celdas Val: un param entero NO lleva slot Val de 80B
        // (vive en el slot i64 de la especialización). Antes se creaba e
        // inicializaba la celda Val igualmente: 2 llamadas al runtime POR
        // LLAMADA (fib: 130ns/frame → ~20ns).
        let mut cr_int_excl_pre: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for rt in &ref_targets {
            cr_int_excl_pre.insert(rt.clone());
        }
        for g in global_names {
            cr_int_excl_pre.insert(g.clone());
        }
        if let Some(cm) = captures.get(name) {
            for v in cm.keys() {
                cr_int_excl_pre.insert(v.clone());
            }
        }
        for c in cap_cells {
            if let Some(rest) = c.strip_prefix(&format!("{}::", name)) {
                cr_int_excl_pre.insert(rest.to_string());
            }
        }
        let mut cr_stored_pre: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ins in instrs {
            if let Instr::Store(sn) | Instr::StoreLocal(sn) = ins {
                cr_stored_pre.insert(sn.clone());
            }
        }
        let mut int_param_names: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for (pi, pname) in func.params.iter().enumerate() {
            if cr_int_excl_pre.contains(pname) || cr_stored_pre.contains(pname) {
                continue;
            }
            if params_int
                .get(name)
                .and_then(|v| v.get(pi))
                .copied()
                .unwrap_or(false)
            {
                int_param_names.insert(pname.clone());
            }
        }

        // ── Celdas: stack slots (Val, 72B) para params y objetivos de
        //    MakeRef. El dispatch T_PTR es en runtime (paridad backend C:
        //    Load=_deref, Store con write-through si el slot es una ref). ──
        let mut slots: HashMap<String, cranelift::codegen::ir::StackSlot> = HashMap::new();
        {
            let mut need: Vec<String> = func.params.clone();
            for n in &ref_targets {
                if !need.iter().any(|x| x == n) {
                    need.push(n.clone());
                }
            }
            for n in need {
                // v3.5.28: params enteros → sin celda Val (slot i64 aparte)
                if int_param_names.contains(&n) {
                    continue;
                }
                let ss = builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData {
                    kind: cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                    size: LW_VAL_SIZE, // sizeof(Val) en lumen_rt.h
                    align_shift: 3,    // log2(8)
                    key: None,
                });
                slots.insert(n, ss);
            }
        }

        // ── Variables: TODAS por celda (v3.5.7). Los Stores deep-copian
        //    (semántica de valores) → cada celda es dueña exclusiva de su
        //    buffer de array → ArrayPushVar puede mutar in-place O(n).
        //    StoreLocal declara (scope actual); Store asigna (binding más
        //    cercano); ScopePush/Pop manejan el sombreado por bloques. ──
        // v3.5.25: ESPECIALIZACIÓN DE ENTEROS — locales que son siempre
        // enteros viven en slots i64 (8B): las fusiones del bucle operan
        // nativamente sin boxing. Excluidas: capturadas y params.
        let mut int_exclude: std::collections::HashSet<String> = std::collections::HashSet::new();
        // v3.5.25: objetivos de MakeRef (prestado mut) viven en slots Val.
        for rt in &ref_targets {
            int_exclude.insert(rt.clone());
        }
        // Globales: otras funciones los leen/escriben por la celda compartida.
        for g in global_names {
            int_exclude.insert(g.clone());
        }
        if let Some(cm) = captures.get(name) {
            for v in cm.keys() {
                int_exclude.insert(v.clone());
            }
        }
        for c in cap_cells {
            if let Some(rest) = c.strip_prefix(&format!("{}::", name)) {
                int_exclude.insert(rest.to_string());
            }
        }
        // v3.5.29: arrays de enteros promovibles (análisis del backend C) —
        // se calculan ANTES de int_vars para que `xs[j]` propague Int.
        let cr_arr_vars = arr_vars_by_name(func, &int_exclude);
        let int_vars = int_vars_by_name(func, &int_exclude, &cr_arr_vars);
        let mut int_slots: HashMap<String, cranelift::codegen::ir::StackSlot> = HashMap::new();
        for iv in &int_vars {
            let ss = builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData {
                kind: cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                size: 8,
                align_shift: 3,
                key: None,
            });
            int_slots.insert(iv.clone(), ss);
        }
        // ── v3.5.30: PROMOCIÓN SSA DE ENTEROS EN BUCLES ──────────────────
        // Heads = labels objetivo de un salto hacia atrás (backedge). Las
        // variables enteras con algún uso posterior al head se promueven:
        // el head recibe block-params extra y el cuerpo del bucle opera en
        // SSA puro (registros) — el slot solo se toca al entrar/salir del
        // bucle. Antes: load/store del slot i64 en cada iteración (frontera
        // de `sum` vs C).
        let mut label_pos: HashMap<usize, usize> = HashMap::new();
        for (idx, ins) in instrs.iter().enumerate() {
            if let Instr::Label(n) = ins {
                label_pos.insert(*n, idx);
            }
        }
        let mut head_labels: std::collections::HashSet<usize> = std::collections::HashSet::new();
        for (idx, ins) in instrs.iter().enumerate() {
            let t = match ins {
                Instr::Jmp(t) | Instr::JmpIf(t) => *t,
                _ => continue,
            };
            if label_pos.get(&t).map(|&p| idx > p).unwrap_or(false) {
                head_labels.insert(t);
            }
        }
        // vars promovidas por head (orden estable para los block-params):
        // heads "limpios" (depth 0, sin catch — los params extra van tras los
        // del stack de valores) con vars enteras usadas después del head.
        let mut head_vars: HashMap<usize, Vec<String>> = HashMap::new();
        for &h in &head_labels {
            if label_depth.get(&h).copied().unwrap_or(0) != 0 || catch_labels.contains(&h) {
                continue;
            }
            let pos = label_pos[&h];
            let mut vars: Vec<String> = int_vars
                .iter()
                .filter(|v| {
                    instrs.iter().enumerate().any(|(idx, ins)| {
                        idx > pos
                            && matches!(
                                ins,
                                Instr::Load(n) | Instr::Store(n) | Instr::StoreLocal(n)
                                    if n == *v
                            )
                    })
                })
                .cloned()
                .collect();
            vars.sort();
            if !vars.is_empty() {
                // params extra en el bloque del head (tras los de la pila).
                for _ in &vars {
                    builder.append_block_param(label_block[&h], i64);
                }
                head_vars.insert(h, vars);
            }
        }
        // conjunto de vars promovidas en ALGÚN head: sus stores solo
        // actualizan el cache SSA (el slot se materializa en los flush).
        let mut promoted_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
        for vars in head_vars.values() {
            for v in vars {
                promoted_vars.insert(v.clone());
            }
        }
        // v3.5.30: variables con UNA sola lectura en toda la función —
        // candidatas a quedarse en la pila (StoreLocal no toca el slot y el
        // único Load re-emite el valor). Patrón: `sea s = ...; usar(s);`.
        let mut load_count: HashMap<String, usize> = HashMap::new();
        let mut load_pos: HashMap<String, usize> = HashMap::new();
        for (idx, ins) in instrs.iter().enumerate() {
            if let Instr::Load(n) = ins {
                *load_count.entry(n.clone()).or_insert(0) += 1;
                load_pos.insert(n.clone(), idx);
            }
        }
        let mut single_use: HashMap<String, usize> = HashMap::new();
        for (n, &c) in &load_count {
            if c == 1 {
                if let Some(&p) = load_pos.get(n) {
                    single_use.insert(n.clone(), p);
                }
            }
        }
        // v3.5.29: ARRAYS DE ENTEROS SIN BOXEAR — cada array promovido vive
        // en tres slots i64 (ptr, len, cap) inicializados vacíos en el entry.
        // El análisis (arr_vars_by_name) garantiza que no escapan (ni calls,
        // ni alias, ni MakeRef, ni return), así que nunca hay que
        // materializarlos como handle.
        let mut arr_ptr_slot: HashMap<String, cranelift::codegen::ir::StackSlot> = HashMap::new();
        let mut arr_len_slot: HashMap<String, cranelift::codegen::ir::StackSlot> = HashMap::new();
        let mut arr_cap_slot: HashMap<String, cranelift::codegen::ir::StackSlot> = HashMap::new();
        for av in &cr_arr_vars {
            let mk = |b: &mut FunctionBuilder| {
                b.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData {
                    kind: cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                    size: 8,
                    align_shift: 3,
                    key: None,
                })
            };
            let (sp, sl, sc) = (mk(&mut builder), mk(&mut builder), mk(&mut builder));
            let z = builder.ins().iconst(i64, 0);
            builder.ins().stack_store(z, sp, 0);
            builder.ins().stack_store(z, sl, 0);
            builder.ins().stack_store(z, sc, 0);
            arr_ptr_slot.insert(av.clone(), sp);
            arr_len_slot.insert(av.clone(), sl);
            arr_cap_slot.insert(av.clone(), sc);
        }
        // dummy i64 para mantener el modelo de profundidades de la pila en
        // los patrones de arrays promovidos (Load xs; ...; ArrayPushVar).
        let arr_dummy = if !cr_arr_vars.is_empty() {
            Some(builder.ins().iconst(i64, 0))
        } else {
            None
        };
        // v3.5.30: cache SSA de variables enteras (nombre → valor i64
        // actual). Almacenado en el entry para los params int y actualizado
        // por Load/Store; los bordes de control lo materializan o lo pasan
        // por block-params (ver head_vars).
        let mut int_cache: HashMap<String, cranelift::codegen::ir::Value> = HashMap::new();
        // v3.5.30: literales PEREZOSOS (kind 4 en la pila): token → DataId.
        // Si el literal forma parte del patrón `"lit" + X + "lit"`, no se
        // emite _lw_str — el Add fusionado recibe el puntero .data directo.
        let mut lit_lazy: HashMap<cranelift::codegen::ir::Value, DataId> = HashMap::new();
        // v3.5.30: valores dejados en la pila por la fusión single-use de
        // StoreLocal (nombre → (valor, posición del Load que lo re-emite)).
        let mut dup_pending: HashMap<String, (cranelift::codegen::ir::Value, usize)> =
            HashMap::new();
        let mut stored_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ins in instrs {
            if let Instr::Store(sn) | Instr::StoreLocal(sn) = ins {
                stored_names.insert(sn.clone());
            }
        }
        // v3.5.28: PARAMS enteros — punto fijo interprocedural (paridad con
        // `int_promotion_analysis` del backend C): un parámetro vive en slot
        // i64 solo si NINGÚN llamador estático le pasa algo no-entero y no se
        // reasigna en el cuerpo; las funciones dinámicas (FuncRef/hilos/mutex)
        // quedan excluidas. Un solo unbox en la entrada; los usos que
        // necesitan handle re-boxean en el Load — semántica idéntica. Antes
        // (promoción sin análisis) se rompía `color(texto c)` al coercionar
        // el texto a entero.
        for (pi, pname) in func.params.iter().enumerate() {
            if int_exclude.contains(pname) || stored_names.contains(pname) {
                continue;
            }
            let is_int_param = params_int
                .get(name)
                .and_then(|v| v.get(pi))
                .copied()
                .unwrap_or(false);
            if !is_int_param {
                continue;
            }
            let ss = builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData {
                kind: cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                size: 8,
                align_shift: 3,
                key: None,
            });
            int_slots.insert(pname.clone(), ss);
            // v3.5.28: el llamador directo pasa el i64 CRUDO (ABI de enteros);
            // se almacena tal cual. Las funciones dinámicas nunca llegan aquí
            // (params_int ya las excluye).
            if let Some(&hv) = entry_params.get(pi) {
                builder.ins().stack_store(hv, ss, 0);
                // v3.5.30: el cache SSA arranca con el param (sin re-carga).
                int_cache.insert(pname.clone(), hv);
            }
        }

        // void handle inicial — v3.5.28: perezoso. Solo se emite si algún
        // binding de entrada lo necesita (funciones solo-enteras como fib ya
        // no pagan la llamada _lw_void por frame).
        let needs_void0 = func.params.iter().any(|pn| {
            let own_cell = format!("{}::{}", name, pn);
            cap_cells.contains(&own_cell) || slots.contains_key(pn)
        }) || ref_targets
            .iter()
            .any(|n| !func.params.iter().any(|x| x == n) && slots.contains_key(n))
            || used_names.iter().any(|n| {
                !slots.contains_key(n) && !global_names.contains(n) && !int_slots.contains_key(n)
            });
        let void0 = if needs_void0 {
            self.lw_call(&mut builder, LW_VOID, &[])
        } else {
            // placeholder sin uso (cranelift lo elimina por DCE)
            builder.ins().iconst(i64, 0)
        };
        // Binding de entrada con store DIRECTO (sin write-through): la celda
        // puede traer un T_PTR de la llamada anterior (bug v3.5.7).
        for pname in func.params.iter() {
            let own_cell = format!("{}::{}", name, pname);
            if cap_cells.contains(&own_cell) {
                let addr = self.global_addr(&mut builder, &own_cell);
                self.lw_call_void(&mut builder, LW_STORE_SLOT_DIRECT, &[addr, void0]);
            } else if let Some(&ss) = slots.get(pname) {
                let addr = builder.ins().stack_addr(i64, ss, 0);
                self.lw_call_void(&mut builder, LW_STORE_SLOT_DIRECT, &[addr, void0]);
            }
        }
        for (pi, pname) in func.params.iter().enumerate() {
            let val = entry_params.get(pi).copied().unwrap_or(void0);
            // v3.5.15: parámetro capturado por un anidado → celda global mangada.
            let own_cell = format!("{}::{}", name, pname);
            // v3.5.28: parámetro entero (ABI crudo) → su slot i64 se liga en
            // el bloque de especialización; la celda Val queda en void.
            let is_int_param = params_int
                .get(name)
                .and_then(|v| v.get(pi))
                .copied()
                .unwrap_or(false)
                && !int_exclude.contains(pname);
            if cap_cells.contains(&own_cell) {
                let addr = self.global_addr(&mut builder, &own_cell);
                self.lw_call_void(&mut builder, LW_STORE_SLOT_DIRECT, &[addr, val]);
            } else if is_int_param {
                // nada: el slot i64 se escribe en la especialización
            } else if let Some(&ss) = slots.get(pname) {
                let addr = builder.ins().stack_addr(i64, ss, 0);
                self.lw_call_void(&mut builder, LW_STORE_SLOT_DIRECT, &[addr, val]);
            }
        }
        for n in ref_targets.iter() {
            if !func.params.iter().any(|x| x == n) {
                if let Some(&ss) = slots.get(n) {
                    let addr = builder.ins().stack_addr(i64, ss, 0);
                    self.lw_call_void(&mut builder, LW_STORE_SLOT_DIRECT, &[addr, void0]);
                }
            }
        }

        let mut scopes: Vec<HashMap<String, cranelift::codegen::ir::StackSlot>> =
            vec![HashMap::new()];
        for n in &used_names {
            // Las globales viven en data compartida (global_addr), no en
            // slot; los enteros promovidos viven en su slot i64.
            if slots.contains_key(n) || global_names.contains(n) || int_slots.contains_key(n) {
                continue;
            }
            let ss = builder.create_sized_stack_slot(cranelift::codegen::ir::StackSlotData {
                kind: cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                size: LW_VAL_SIZE,
                align_shift: 3,
                key: None,
            });
            let addr = builder.ins().stack_addr(i64, ss, 0);
            self.lw_call_void(&mut builder, LW_STORE_SLOT_DIRECT, &[addr, void0]);
            scopes[0].insert(n.clone(), ss);
        }
        fn find_slot_cl(
            scopes: &[HashMap<String, cranelift::codegen::ir::StackSlot>],
            n: &str,
        ) -> Option<cranelift::codegen::ir::StackSlot> {
            for sc in scopes.iter().rev() {
                if let Some(&v) = sc.get(n) {
                    return Some(v);
                }
            }
            None
        }

        // ── Emisión lineal (modelo de handles opacos _lw_*) ──
        let mut cur = entry_block;
        // v3.5.28: stack con kinds: (valor, 0=handle|1=i64 crudo|2=b8 crudo).
        let mut stack: Vec<(Value, u8)> = Vec::new();
        let mut terminated = false;
        let mut handlers: Vec<usize> = Vec::new();

        let mut ii = 0usize;
        while ii < instrs.len() {
            let ins = &instrs[ii];
            if let Instr::Label(n) = ins {
                let target = label_block[n];
                if target != cur {
                    let d = label_depth.get(n).copied().unwrap_or(0);
                    if !terminated {
                        // pasar por el borde los valores que el label recibe
                        // (si el flujo normal cae en un catch, void como msg);
                        // los valores crudos se boxean: los block-params son
                        // handles.
                        let mut args: Vec<cranelift::codegen::ir::BlockArg> = Vec::new();
                        if catch_labels.contains(n) {
                            args.push(cranelift::codegen::ir::BlockArg::Value(void0));
                        } else if stack.len() >= d {
                            for (v, k) in &stack[stack.len() - d..] {
                                let bv = self.cr_box(&mut builder, *v, *k);
                                args.push(cranelift::codegen::ir::BlockArg::Value(bv));
                            }
                        } else {
                            for _ in 0..d {
                                args.push(cranelift::codegen::ir::BlockArg::Value(void0));
                            }
                        }
                        // v3.5.30: borde de control — head promovido: las
                        // vars viajan por block-params; cualquier otro
                        // destino: flush de los slots i64.
                        if let Some(vars) = head_vars.get(n) {
                            for v in vars {
                                let vv = cr_int_val(&mut builder, &mut int_cache, &int_slots, v);
                                args.push(cranelift::codegen::ir::BlockArg::Value(vv));
                            }
                        } else {
                            cr_flush_ints(&mut builder, &int_cache, &int_slots);
                        }
                        builder.ins().jump(target, &args);
                    }
                    cur = target;
                    builder.switch_to_block(cur);
                    builder.ensure_inserted_block();
                    // el stack del label llega por block-params (handles);
                    // las vars promovidas llegan tras ellos y alimentan el
                    // cache SSA. Los demás labels arrancan con cache vacío
                    // (los slots quedaron materializados en los bordes).
                    let params = builder.block_params(cur).to_vec();
                    stack.clear();
                    for &pv in params.iter().take(params.len().min(d)) {
                        stack.push((pv, 0));
                    }
                    int_cache.clear();
                    if let Some(vars) = head_vars.get(n) {
                        for (k, v) in vars.iter().enumerate() {
                            int_cache.insert(v.clone(), params[d + k]);
                        }
                    }
                }
                terminated = false;
                ii += 1;
                continue;
            }
            if terminated {
                ii += 1;
                continue;
            }
            // v3.5.25: fusión de patrones enteros (slots i64 nativos).
            if ii + 4 <= instrs.len() {
                if let Some(consumed) = self.cr_fused_int(
                    &mut builder,
                    instrs,
                    ii,
                    &int_slots,
                    &mut int_cache,
                    &head_vars,
                    &promoted_vars,
                    &label_block,
                    &label_depth,
                    &mut stack,
                    &mut cur,
                ) {
                    ii += consumed;
                    continue;
                }
            }
            let mut risky = false;
            match ins {
                Instr::ConstInt(n) => {
                    // v3.5.28: entero CRUDO en el stack (sin box); los
                    // consumidores que necesitan handle boxean en el límite.
                    let c = builder.ins().iconst(i64, *n);
                    stack.push((c, 1));
                }
                Instr::ConstFloat(f) => {
                    let c = builder.ins().f64const(*f);
                    stack.push((self.lw_call(&mut builder, LW_FLT, &[c]), 0));
                }
                Instr::ConstBool(b) => {
                    let c = builder.ins().iconst(i64, if *b { 1 } else { 0 });
                    stack.push((self.lw_call(&mut builder, LW_BOOL, &[c]), 0));
                }
                Instr::ConstStr(s) => {
                    let data_id = self.get_string_ptr(s);
                    // v3.5.30: ¿patrón "lit" + X + "lit"? → token perezoso.
                    // Walk desde ii+1: la pila mantiene profundidad ≥ 1
                    // (el literal nunca se consume) hasta un Binary(Add) a
                    // profundidad 1, seguido de ConstStr + Binary(Add).
                    let mut a: Option<usize> = None;
                    {
                        // d = valores por encima del estado previo al
                        // literal. El Add fusionado es el primero que
                        // encuentra el token como operando izquierdo:
                        // d == 2 (el token + el valor intermedio X).
                        let mut d = 1i32;
                        let mut k = ii + 1;
                        while k < instrs.len() {
                            let ins_k = &instrs[k];
                            if matches!(ins_k, Instr::Binary(Op::Add)) {
                                if d == 2 {
                                    a = Some(k);
                                }
                                break;
                            }
                            let dd = instr_depth_delta(ins_k);
                            if matches!(
                                ins_k,
                                Instr::Jmp(_)
                                    | Instr::JmpIf(_)
                                    | Instr::Label(_)
                                    | Instr::Store(_)
                                    | Instr::StoreLocal(_)
                                    | Instr::MakeRef(_)
                                    | Instr::ConstStr(_)
                                    | Instr::Return
                                    | Instr::Halt
                            ) {
                                break;
                            }
                            if d + dd < 1 {
                                break;
                            }
                            d += dd;
                            k += 1;
                        }
                    }
                    if let Some(a) = a {
                        if a + 2 < instrs.len()
                            && matches!(&instrs[a + 1], Instr::ConstStr(_))
                            && matches!(&instrs[a + 2], Instr::Binary(Op::Add))
                        {
                            let ph = builder.ins().iconst(i64, 0);
                            lit_lazy.insert(ph, data_id);
                            stack.push((ph, 4));
                            // sin _lw_str: el puntero se pasa directo a
                            // _lw_concat3 en el Add fusionado.
                            if risky && !terminated {
                                self.emit_err_check(
                                    &mut builder,
                                    &handlers,
                                    &label_block,
                                    &int_cache,
                                    &int_slots,
                                );
                            }
                            ii += 1;
                            continue;
                        }
                    }
                    let gv = self.module.declare_data_in_func(data_id, builder.func);
                    let ptr = builder.ins().global_value(i64, gv);
                    stack.push((self.lw_call(&mut builder, LW_STR, &[ptr]), 0));
                }
                Instr::Load(n) => {
                    // v3.5.30: re-emisión del valor dejado en la pila por
                    // la fusión single-use de StoreLocal (sin tocar el slot).
                    if let Some(&(v, j)) = dup_pending.get(n) {
                        if j == ii {
                            dup_pending.remove(n);
                            stack.push((v, 0));
                            // el err_check de la cola NO se ejecuta al
                            // saltar → replicarlo aquí.
                            if risky && !terminated {
                                self.emit_err_check(
                                    &mut builder,
                                    &handlers,
                                    &label_block,
                                    &int_cache,
                                    &int_slots,
                                );
                            }
                            ii += 1;
                            continue;
                        }
                    }
                    // v3.5.29: ventanas de fusión de arrays de enteros
                    // promovidos: Load xs; Load j|ConstInt; ArrayGet → get
                    // nativo con bounds; Load xs; ArrayLen → len nativo.
                    if cr_arr_vars.contains(n) {
                        let mut fused = false;
                        if ii + 2 < instrs.len() {
                            if let (Instr::Load(j), Instr::ArrayGet) =
                                (&instrs[ii + 1], &instrs[ii + 2])
                            {
                                let ix = if int_slots.contains_key(j) {
                                    // v3.5.30: vía cache SSA (bucle promovido).
                                    cr_int_val(&mut builder, &mut int_cache, &int_slots, j)
                                } else if let Some(ssj) =
                                    find_slot_cl(&scopes, j).or_else(|| slots.get(j).copied())
                                {
                                    let addr = builder.ins().stack_addr(i64, ssj, 0);
                                    let h = self.lw_call(&mut builder, LW_LOAD_SLOT, &[addr]);
                                    self.lw_call(&mut builder, LW_H2I, &[h])
                                } else {
                                    builder.ins().iconst(i64, 0)
                                };
                                let ptr = builder.ins().stack_load(types::I64, arr_ptr_slot[n], 0);
                                let len = builder.ins().stack_load(types::I64, arr_len_slot[n], 0);
                                let r = self.lw_call(&mut builder, LW_IARR_GET, &[ptr, len, ix]);
                                stack.push((r, 1));
                                risky = true;
                                ii += 3;
                                fused = true;
                            } else if let (Instr::ConstInt(k), Instr::ArrayGet) =
                                (&instrs[ii + 1], &instrs[ii + 2])
                            {
                                let ix = builder.ins().iconst(i64, *k);
                                let ptr = builder.ins().stack_load(types::I64, arr_ptr_slot[n], 0);
                                let len = builder.ins().stack_load(types::I64, arr_len_slot[n], 0);
                                let r = self.lw_call(&mut builder, LW_IARR_GET, &[ptr, len, ix]);
                                stack.push((r, 1));
                                risky = true;
                                ii += 3;
                                fused = true;
                            }
                        }
                        if !fused
                            && ii + 1 < instrs.len()
                            && matches!(&instrs[ii + 1], Instr::ArrayLen)
                        {
                            let len = builder.ins().stack_load(types::I64, arr_len_slot[n], 0);
                            stack.push((len, 1));
                            ii += 2;
                            fused = true;
                        }
                        if fused {
                            if risky && !terminated {
                                self.emit_err_check(
                                    &mut builder,
                                    &handlers,
                                    &label_block,
                                    &int_cache,
                                    &int_slots,
                                );
                            }
                            continue;
                        }
                        // sin fusión (p. ej. Load xs antes de agregar): dummy
                        // kind=3 para conservar el modelo de profundidades y
                        // que los consumidores genéricos lo detecten.
                        stack.push((arr_dummy.unwrap(), 3));
                    } else if let Some(_ss) = int_slots.get(n).copied() {
                        // v3.5.28: local/param entero → slot i64, valor CRUDO.
                        // v3.5.30: vía cache SSA (promoción de bucles).
                        let v = cr_int_val(&mut builder, &mut int_cache, &int_slots, n);
                        stack.push((v, 1));
                    } else if let Some(cell) = cap_cell_for(n) {
                        let addr = self.global_addr(&mut builder, &cell);
                        stack.push((self.lw_call(&mut builder, LW_LOAD_SLOT, &[addr]), 0));
                    } else {
                        let ss = find_slot_cl(&scopes, n).or_else(|| slots.get(n).copied());
                        if let Some(ss) = ss {
                            let addr = builder.ins().stack_addr(i64, ss, 0);
                            stack.push((self.lw_call(&mut builder, LW_LOAD_SLOT, &[addr]), 0));
                        } else if global_names.contains(n) {
                            let addr = self.global_addr(&mut builder, n);
                            stack.push((self.lw_call(&mut builder, LW_LOAD_SLOT, &[addr]), 0));
                        } else {
                            stack.push((void0, 0));
                        }
                    }
                }
                Instr::StoreLocal(n) => {
                    // DECLARACIÓN: liga en el scope actual (sombreado real)
                    if cr_arr_vars.contains(n) {
                        // v3.5.29: array promovido — slots nativos ya
                        // inicializados en el entry; se descarta el dummy.
                        let _ = self.cr_pop(&mut builder, &mut stack);
                    } else if let Some(&j) = single_use.get(n) {
                        // v3.5.30: fusión single-use — si entre este
                        // StoreLocal y su ÚNICO Load(n) solo hay Loads de
                        // otras vars / constantes, el handle se queda en la
                        // pila (sin slot ni _lw_store_slot) y el Load lo
                        // re-emite. El err_check de la cola sigue activo.
                        let between_ok = j > ii
                            && j - ii <= 4
                            && (ii + 1..j).all(|k| {
                                if let Instr::Load(x) = &instrs[k] {
                                    x != n
                                } else {
                                    matches!(
                                        &instrs[k],
                                        Instr::ConstInt(_)
                                            | Instr::ConstFloat(_)
                                            | Instr::ConstStr(_)
                                            | Instr::ConstBool(_)
                                    )
                                }
                            });
                        if between_ok
                            && !global_names.contains(n)
                            && !ref_targets.iter().any(|x| x == n)
                            && cap_cell_for(n).is_none()
                            && stack.last().map(|(_, k)| *k == 0).unwrap_or(false)
                        {
                            // se SACA de la pila (modelo de profundidades
                            // intacto) pero el valor SSA se guarda para que
                            // el Load lo re-emita sin tocar el slot.
                            let (v, _) = stack.pop().unwrap();
                            dup_pending.insert(n.clone(), (v, j));
                        } else if let Some((v, k)) = stack.pop() {
                            self.cr_store_local(
                                &mut builder,
                                n,
                                v,
                                k,
                                &mut scopes,
                                &slots,
                                global_names,
                                &mut int_cache,
                                &int_slots,
                                &promoted_vars,
                                is_entry,
                                &cap_cell_for,
                            );
                        }
                    } else if let Some((v, k)) = stack.pop() {
                        self.cr_store_local(
                            &mut builder,
                            n,
                            v,
                            k,
                            &mut scopes,
                            &slots,
                            global_names,
                            &mut int_cache,
                            &int_slots,
                            &promoted_vars,
                            is_entry,
                            &cap_cell_for,
                        );
                    }
                }
                Instr::Store(n) => {
                    // ASIGNACIÓN: binding más cercano (deep-copy en el store)
                    if cr_arr_vars.contains(n) {
                        let _ = self.cr_pop(&mut builder, &mut stack);
                    } else if let Some((v, k)) = stack.pop() {
                        if let Some(_ss) = int_slots.get(n).copied() {
                            // v3.5.28: slot i64 — sin box si ya viene crudo
                            let iv = if k == 1 {
                                v
                            } else {
                                let h = self.cr_box(&mut builder, v, k);
                                self.lw_call(&mut builder, LW_H2I, &[h])
                            };
                            // v3.5.30: var promovida → solo cache SSA (el
                            // slot se materializa en los bordes).
                            int_cache.insert(n.clone(), iv);
                            if !promoted_vars.contains(n) {
                                builder.ins().stack_store(iv, int_slots[n], 0);
                            }
                        } else {
                            let h = self.cr_box(&mut builder, v, k);
                            if let Some(cell) = cap_cell_for(n) {
                                let addr = self.global_addr(&mut builder, &cell);
                                self.lw_call_void(&mut builder, LW_STORE_SLOT, &[addr, h]);
                            } else {
                                let ss = find_slot_cl(&scopes, n).or_else(|| slots.get(n).copied());
                                if let Some(ss) = ss {
                                    let addr = builder.ins().stack_addr(i64, ss, 0);
                                    self.lw_call_void(&mut builder, LW_STORE_SLOT, &[addr, h]);
                                } else if global_names.contains(n) {
                                    let addr = self.global_addr(&mut builder, n);
                                    self.lw_call_void(&mut builder, LW_STORE_SLOT, &[addr, h]);
                                } else {
                                    let ns = builder.create_sized_stack_slot(
                                        cranelift::codegen::ir::StackSlotData {
                                            kind:
                                                cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                                            size: LW_VAL_SIZE,
                                            align_shift: 3,
                                            key: None,
                                        },
                                    );
                                    let addr = builder.ins().stack_addr(i64, ns, 0);
                                    self.lw_call_void(&mut builder, LW_STORE_SLOT, &[addr, h]);
                                    scopes.last_mut().unwrap().insert(n.clone(), ns);
                                }
                            }
                        }
                    }
                }
                Instr::Binary(op) => {
                    // v3.5.30: fusión de interpolación `"lit" + X + "lit"`
                    // → una sola llamada _lw_concat3 (antes: 2 _lw_bin con
                    // strlen+arena+box cada uno). El literal izquierdo llega
                    // como token perezoso (kind 4, sin _lw_str).
                    if *op == Op::Add
                        && ii + 2 < instrs.len()
                        && matches!(&instrs[ii + 1], Instr::ConstStr(_))
                        && matches!(&instrs[ii + 2], Instr::Binary(Op::Add))
                    {
                        if let (Some(&(rhs, krhs)), Some(&(lhs, klhs))) =
                            (stack.last(), stack.get(stack.len().saturating_sub(2)))
                        {
                            // v3.5.30: medio = a_texto(entero) → itoa
                            // directo al buffer final (sin box intermedio).
                            // En ese caso el Call deja el CRUDO en la pila
                            // (kind 1) y aquí se consume directamente.
                            let mid_is_to_text = ii >= 2
                                && matches!(
                                    &instrs[ii - 1],
                                    Instr::Call(c, 1)
                                        if matches!(
                                            c.as_str(),
                                            "a_texto" | "to_texto" | "__str_from"
                                        )
                                )
                                && matches!(
                                    &instrs[ii - 2],
                                    Instr::Load(n2) if int_slots.contains_key(n2)
                                );
                            if (krhs == 0 || (krhs == 1 && mid_is_to_text))
                                && klhs == 4
                                && lit_lazy.contains_key(&lhs)
                            {
                                let d1 = lit_lazy[&lhs];
                                let s2 = match &instrs[ii + 1] {
                                    Instr::ConstStr(s) => s.clone(),
                                    _ => unreachable!(),
                                };
                                let d2 = self.get_string_ptr(&s2);
                                let gv1 = self.module.declare_data_in_func(d1, builder.func);
                                let p1 = builder.ins().global_value(i64, gv1);
                                let gv2 = self.module.declare_data_in_func(d2, builder.func);
                                let p2 = builder.ins().global_value(i64, gv2);
                                let raw = if mid_is_to_text {
                                    let n2 = match &instrs[ii - 2] {
                                        Instr::Load(n2) => n2.clone(),
                                        _ => unreachable!(),
                                    };
                                    Some(cr_int_val(&mut builder, &mut int_cache, &int_slots, &n2))
                                } else {
                                    None
                                };
                                // v3.5.30: patrón `sea s = "lit" + a_texto(i)
                                // + "lit"; largo(s)` con s de uso ÚNICO → la
                                // longitud se calcula SIN construir el string
                                // (bucle strings: 1 llamada por iteración).
                                // Entre StoreLocal(s) y Load(s) puede haber
                                // Loads de OTRAS vars enteras (p. ej.
                                // `total = total + largo(s)`): sus efectos se
                                // replican al omitir instrucciones.
                                let mut len_fused: Option<usize> = None;
                                if mid_is_to_text && ii + 3 < instrs.len() {
                                    if let Instr::StoreLocal(s) = &instrs[ii + 3] {
                                        if let Some(&j) = single_use.get(s) {
                                            let between = j > ii + 3
                                                && j - (ii + 3) <= 4
                                                && (ii + 4..j).all(|k| {
                                                    matches!(
                                                        &instrs[k],
                                                        Instr::Load(x)
                                                            if x != s
                                                                && int_slots.contains_key(x)
                                                    )
                                                })
                                                && j + 1 < instrs.len()
                                                && matches!(
                                                    &instrs[j + 1],
                                                    Instr::Call(c, 1)
                                                        if matches!(
                                                            c.as_str(),
                                                            "largo"
                                                                | "len"
                                                                | "length"
                                                                | "__str_len"
                                                                | "__str_longitud"
                                                        )
                                                );
                                            if between {
                                                len_fused = Some(j);
                                            }
                                        }
                                    }
                                }
                                stack.pop();
                                stack.pop();
                                if let Some(j) = len_fused {
                                    // efectos de los Loads omitidos
                                    for ins_k in instrs.iter().take(j).skip(ii + 3) {
                                        if let Instr::Load(x) = ins_k {
                                            let vx = cr_int_val(
                                                &mut builder,
                                                &mut int_cache,
                                                &int_slots,
                                                x,
                                            );
                                            stack.push((vx, 1));
                                        }
                                    }
                                    let r = self.lw_call(
                                        &mut builder,
                                        LW_CONCAT3_LEN_I,
                                        &[p1, raw.unwrap(), p2],
                                    );
                                    stack.push((r, 1));
                                    if risky && !terminated {
                                        self.emit_err_check(
                                            &mut builder,
                                            &handlers,
                                            &label_block,
                                            &int_cache,
                                            &int_slots,
                                        );
                                    }
                                    ii = j + 2;
                                    continue;
                                }
                                let r = if mid_is_to_text {
                                    self.lw_call(
                                        &mut builder,
                                        LW_CONCAT3_I,
                                        &[p1, raw.unwrap(), p2],
                                    )
                                } else {
                                    self.lw_call(&mut builder, LW_CONCAT3, &[p1, rhs, p2])
                                };
                                stack.push((r, 0));
                                // el err_check de la cola NO se ejecuta al
                                // saltar instrucciones → replicarlo aquí.
                                if risky && !terminated {
                                    self.emit_err_check(
                                        &mut builder,
                                        &handlers,
                                        &label_block,
                                        &int_cache,
                                        &int_slots,
                                    );
                                }
                                ii += 3;
                                continue;
                            }
                        }
                    }
                    use cranelift::codegen::ir::condcodes::IntCC;
                    let (b, kb) = self.cr_pop(&mut builder, &mut stack);
                    let (a, ka) = self.cr_pop(&mut builder, &mut stack);
                    // v3.5.30: defensivo — un token perezoso que llegó aquí
                    // (el análisis lo impide) se materializa al momento.
                    let (a, ka) = if ka == 4 {
                        let d1 = lit_lazy
                            .get(&a)
                            .copied()
                            .unwrap_or_else(|| self.get_string_ptr(""));
                        let gv = self.module.declare_data_in_func(d1, builder.func);
                        let ptr = builder.ins().global_value(i64, gv);
                        (self.lw_call(&mut builder, LW_STR, &[ptr]), 0u8)
                    } else {
                        (a, ka)
                    };
                    if ka == 1 && kb == 1 {
                        // v3.5.28: aritmética/comparación entera NATIVA (sin
                        // boxing ni llamada al runtime). Div/Mod pueden lanzar
                        // → se quedan en el camino del runtime.
                        match op {
                            Op::Add => stack.push((builder.ins().iadd(a, b), 1)),
                            Op::Sub => stack.push((builder.ins().isub(a, b), 1)),
                            Op::Mul => stack.push((builder.ins().imul(a, b), 1)),
                            Op::BitAnd => stack.push((builder.ins().band(a, b), 1)),
                            Op::BitOr => stack.push((builder.ins().bor(a, b), 1)),
                            Op::BitXor => stack.push((builder.ins().bxor(a, b), 1)),
                            Op::ShiftLeft => {
                                let m = builder.ins().iconst(i64, 63);
                                let sh = builder.ins().band(b, m);
                                stack.push((builder.ins().ishl(a, sh), 1));
                            }
                            Op::ShiftRight => {
                                let m = builder.ins().iconst(i64, 63);
                                let sh = builder.ins().band(b, m);
                                stack.push((builder.ins().ushr(a, sh), 1));
                            }
                            Op::Less => {
                                stack.push((builder.ins().icmp(IntCC::SignedLessThan, a, b), 2))
                            }
                            Op::LessEqual => stack
                                .push((builder.ins().icmp(IntCC::SignedLessThanOrEqual, a, b), 2)),
                            Op::Greater => {
                                stack.push((builder.ins().icmp(IntCC::SignedGreaterThan, a, b), 2))
                            }
                            Op::GreaterEqual => stack.push((
                                builder.ins().icmp(IntCC::SignedGreaterThanOrEqual, a, b),
                                2,
                            )),
                            Op::Equal => stack.push((builder.ins().icmp(IntCC::Equal, a, b), 2)),
                            Op::NotEqual => {
                                stack.push((builder.ins().icmp(IntCC::NotEqual, a, b), 2))
                            }
                            Op::Div => {
                                // v3.5.28: división entera nativa con zero-check
                                let r = self.cr_emit_divmod(&mut builder, a, b, true);
                                stack.push((r, 1));
                                risky = true;
                            }
                            Op::Mod => {
                                // v3.5.28: módulo entero nativo con zero-check
                                let r = self.cr_emit_divmod(&mut builder, a, b, false);
                                stack.push((r, 1));
                                risky = true;
                            }
                            _ => {
                                // resto de ops int+int: boxear — _lw_bin espera handles
                                let ah = self.lw_call(&mut builder, LW_INT, &[a]);
                                let bh = self.lw_call(&mut builder, LW_INT, &[b]);
                                let code = builder.ins().iconst(i64, op_code(op));
                                stack
                                    .push((self.lw_call(&mut builder, LW_BIN, &[code, ah, bh]), 0));
                                risky = true;
                            }
                        }
                    } else {
                        let ah = self.cr_box(&mut builder, a, ka);
                        let bh = self.cr_box(&mut builder, b, kb);
                        let code = builder.ins().iconst(i64, op_code(op));
                        stack.push((self.lw_call(&mut builder, LW_BIN, &[code, ah, bh]), 0));
                        // solo Div/Mod lanzan en _lw_bin (paridad backend C)
                        risky = matches!(op, Op::Div | Op::Mod);
                    }
                }
                Instr::Unary(op) => {
                    let (v, k) = self.cr_pop(&mut builder, &mut stack);
                    let h = self.cr_box(&mut builder, v, k);
                    let code = builder.ins().iconst(
                        i64,
                        match op {
                            Op::Not => 1,
                            Op::BitNot => 2,
                            _ => 0,
                        },
                    );
                    stack.push((self.lw_call(&mut builder, LW_UN, &[code, h]), 0));
                }
                Instr::Print => {
                    let v = self.pop_handle(&mut builder, &mut stack);
                    self.lw_call_void(&mut builder, LW_PRINT, &[v]);
                }
                Instr::Read => {
                    stack.push((self.lw_call(&mut builder, LW_READ, &[]), 0));
                }
                Instr::Call(cn, argc) => {
                    // v3.5.28: args con kind; la llamada directa a funciones
                    // compiladas usa el ABI de enteros crudos y se resuelve
                    // ANTES del dispatch de builtins.
                    let mut args_raw: Vec<(Value, u8)> = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        args_raw.push(self.cr_pop(&mut builder, &mut stack));
                    }
                    args_raw.reverse();
                    let special = matches!(
                        cn.as_str(),
                        "imprimir"
                            | "print"
                            | "__tarea_lanzar"
                            | "__task_spawn"
                            | "__hilo_lanzar"
                            | "__thread_spawn"
                            | "__tarea_esperar"
                            | "__task_await"
                            | "__hilo_esperar"
                            | "__thread_join"
                            | "__canal_nuevo"
                            | "__channel_new"
                            | "__canal_enviar"
                            | "__channel_send"
                            | "__canal_recibir"
                            | "__channel_recv"
                            | "__mutex_nuevo"
                            | "__mutex_new"
                            | "__mutex_bloquear"
                            | "__mutex_lock"
                            | "__calendario_hijri"
                            | "__calendar_hijri"
                            | "__calendario_persa"
                            | "__calendar_persian"
                            | "__tiempo_ahora"
                            | "__time_now"
                            | "__tiempo_formatear"
                            | "__time_format"
                            | "__tiempo_diferencia"
                            | "__time_diff"
                            | "__tiempo_parsear"
                            | "__tiempo_parse"
                            | "__time_parse"
                            | "__str_a_caracteres"
                            | "__str_to_chars"
                            | "__str_mayusculas"
                            | "__str_upper"
                            | "__str_minusculas"
                            | "__str_lower"
                            | "__str_padding_inicio"
                            | "__str_padding_fin"
                    );
                    let mut cr_done = false;
                    if !special && lw_builtin(cn).is_none() {
                        if let Some(finfo) = self.funcs.get(cn).cloned() {
                            let func_ref = self.module.declare_func_in_func(finfo.id, builder.func);
                            // v3.5.28: ABI de enteros — parámetros int reciben
                            // el i64 CRUDO (sin box ni dcp); el resto recibe
                            // handle (deep-copy solo de handles: los crudos
                            // recién calculados no tienen alias).
                            let pints = params_int.get(cn);
                            let mut dargs: Vec<Value> = Vec::with_capacity(args_raw.len());
                            for (j, (av, ak)) in args_raw.iter().enumerate() {
                                let is_int_param =
                                    pints.and_then(|v| v.get(j)).copied().unwrap_or(false);
                                if is_int_param {
                                    let iv = if *ak == 1 {
                                        *av
                                    } else {
                                        let h = self.cr_box(&mut builder, *av, *ak);
                                        self.lw_call(&mut builder, LW_H2I, &[h])
                                    };
                                    dargs.push(iv);
                                } else if *ak == 0 {
                                    dargs.push(self.lw_call(&mut builder, LW_DCP, &[*av]));
                                } else {
                                    dargs.push(self.cr_box(&mut builder, *av, *ak));
                                }
                            }
                            let call = builder.ins().call(func_ref, &dargs);
                            let res = builder.inst_results(call)[0];
                            let rk: u8 = if returns_int.get(cn).copied().unwrap_or(false) {
                                1
                            } else {
                                0
                            };
                            stack.push((res, rk));
                            // v3.5.28: ERRCHK solo si el callee puede lanzar
                            risky = !no_throw.contains(cn);
                            cr_done = true;
                        } else {
                            record_unsupported_builtin(cn);
                            stack.push((self.void_handle(&mut builder), 0));
                            cr_done = true;
                        }
                    }
                    if !cr_done {
                        // v3.5.30: fast-paths crudos ANTES del boxing
                        // genérico — a_texto(entero) y largo() no boxean
                        // argumentos que no lo necesitan.
                        let mut fast = false;
                        if let Some((idx, arity)) = lw_builtin(cn) {
                            if idx == LW_TO_TEXT
                                && arity == 1
                                && args_raw.first().map(|(_, ak)| *ak == 1).unwrap_or(false)
                            {
                                let (av, _) = args_raw[0];
                                // v3.5.30: si sigue el patrón de
                                // interpolación fusionada (Binary(Add);
                                // ConstStr; Binary(Add)), NO se emite la
                                // conversión: el Add fusionado toma el
                                // CRUDO directo (evita la llamada muerta).
                                let fused_next = ii + 3 < instrs.len()
                                    && matches!(&instrs[ii + 1], Instr::Binary(Op::Add))
                                    && matches!(&instrs[ii + 2], Instr::ConstStr(_))
                                    && matches!(&instrs[ii + 3], Instr::Binary(Op::Add));
                                if fused_next {
                                    stack.push((av, 1));
                                } else {
                                    let r = self.lw_call(&mut builder, LW_TO_TEXT_I, &[av]);
                                    stack.push((r, 0));
                                }
                                fast = true;
                            } else if idx == LW_ARR_LEN && arity == 1 {
                                let (av, ak) = args_raw
                                    .first()
                                    .copied()
                                    .unwrap_or_else(|| (self.void_handle(&mut builder), 0));
                                let h = if ak == 0 {
                                    av
                                } else {
                                    self.cr_box(&mut builder, av, ak)
                                };
                                let r = self.lw_call(&mut builder, LW_ARR_LEN_I, &[h]);
                                stack.push((r, 1));
                                fast = true;
                            }
                        }
                        if fast {
                            // nada más: el err_check de la cola corre igual
                        } else {
                            let mut args: Vec<Value> = Vec::with_capacity(args_raw.len());
                            for (av, ak) in &args_raw {
                                args.push(self.cr_box(&mut builder, *av, *ak));
                            }
                            match cn.as_str() {
                                "imprimir" | "print" => {
                                    if args.is_empty() {
                                        self.lw_call_void(&mut builder, LW_PRINT_BLANK, &[]);
                                    } else {
                                        let mut acc = args[0];
                                        for av in args.iter().skip(1) {
                                            acc = self.lw_call(&mut builder, LW_JOIN, &[acc, *av]);
                                        }
                                        self.lw_call_void(&mut builder, LW_PRINT, &[acc]);
                                    }
                                    stack.push((self.void_handle(&mut builder), 0));
                                }
                                // v3.5.17: hilos REALES en Cranelift. args[0] = nombre
                                // de la función (texto), args[1..] = argumentos. Se
                                // estagan los handles en un slot (int64[8]) y el shim
                                // C (_lw_thr_spawn_h) los desboxea a Val[] y crea el
                                // hilo pthread/Win32. El hilo entra por el trampolín
                                // __lumen_ft_<fn> (ver thread_trampolines).
                                "__tarea_lanzar" | "__task_spawn" | "__hilo_lanzar"
                                | "__thread_spawn" => {
                                    let na = args.len().saturating_sub(1).min(8);
                                    let ss = builder.create_sized_stack_slot(
                                        cranelift::codegen::ir::StackSlotData {
                                            kind:
                                                cranelift::codegen::ir::StackSlotKind::ExplicitSlot,
                                            size: 64, // 8 handles × 8 bytes
                                            align_shift: 3,
                                            key: None,
                                        },
                                    );
                                    for k in 0..na {
                                        builder.ins().stack_store(args[1 + k], ss, (k * 8) as i32);
                                    }
                                    let ptr = builder.ins().stack_addr(i64, ss, 0);
                                    let name_h = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let cstr = self.lw_call(&mut builder, LW_CSTR, &[name_h]);
                                    let na_v = builder.ins().iconst(i64, na as i64);
                                    let id = self.lw_call(
                                        &mut builder,
                                        LW_THR_SPAWN,
                                        &[cstr, ptr, na_v],
                                    );
                                    stack.push((self.lw_call(&mut builder, LW_INT, &[id]), 0));
                                }
                                "__tarea_esperar" | "__task_await" | "__hilo_esperar"
                                | "__thread_join" => {
                                    let id_h = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((
                                        self.lw_call(&mut builder, LW_THR_JOIN, &[id_h]),
                                        0,
                                    ));
                                }
                                // v3.5.17: canales y mutexes nativos (paridad VM).
                                "__canal_nuevo" | "__channel_new" => {
                                    stack.push((self.lw_call(&mut builder, LW_CHAN_NEW, &[]), 0));
                                }
                                "__canal_enviar" | "__channel_send" => {
                                    let cid = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let v = args
                                        .get(1)
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((
                                        self.lw_call(&mut builder, LW_CHAN_SEND, &[cid, v]),
                                        0,
                                    ));
                                }
                                "__canal_recibir" | "__channel_recv" => {
                                    let cid = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((
                                        self.lw_call(&mut builder, LW_CHAN_RECV, &[cid]),
                                        0,
                                    ));
                                }
                                "__mutex_nuevo" | "__mutex_new" => {
                                    stack.push((self.lw_call(&mut builder, LW_MUTEX_NEW, &[]), 0));
                                }
                                "__mutex_bloquear" | "__mutex_lock" => {
                                    let mid = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let fhn = args
                                        .get(1)
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let arg = args
                                        .get(2)
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((
                                        self.lw_call(
                                            &mut builder,
                                            LW_MUTEX_LOCK_CALL,
                                            &[mid, fhn, arg],
                                        ),
                                        0,
                                    ));
                                }
                                "__calendario_hijri" | "__calendar_hijri" => {
                                    let t = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((self.lw_call(&mut builder, LW_CAL_HIJRI, &[t]), 0));
                                }
                                "__calendario_persa" | "__calendar_persian" => {
                                    let t = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((self.lw_call(&mut builder, LW_CAL_PERSA, &[t]), 0));
                                }
                                "__tiempo_ahora" | "__time_now" => {
                                    stack.push((self.lw_call(&mut builder, LW_TIME_NOW, &[]), 0));
                                }
                                "__tiempo_formatear" | "__time_format" => {
                                    let t = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let f = args
                                        .get(1)
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((
                                        self.lw_call(&mut builder, LW_TIME_FMT, &[t, f]),
                                        0,
                                    ));
                                }
                                "__tiempo_diferencia" | "__time_diff" => {
                                    let t1 = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let t2 = args
                                        .get(1)
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((
                                        self.lw_call(&mut builder, LW_TIME_DIFF, &[t1, t2]),
                                        0,
                                    ));
                                }
                                "__tiempo_parsear" | "__tiempo_parse" | "__time_parse" => {
                                    let s = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack
                                        .push((self.lw_call(&mut builder, LW_TIME_PARSE, &[s]), 0));
                                }
                                // v3.5.18: strings unicode (stress_03 en Cranelift).
                                "__str_a_caracteres" | "__str_to_chars" => {
                                    let s = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((self.lw_call(&mut builder, LW_STR_CHARS, &[s]), 0));
                                }
                                "__str_mayusculas" | "__str_upper" => {
                                    let s = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((self.lw_call(&mut builder, LW_STR_UPPER, &[s]), 0));
                                }
                                "__str_minusculas" | "__str_lower" => {
                                    let s = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    stack.push((self.lw_call(&mut builder, LW_STR_LOWER, &[s]), 0));
                                }
                                "__str_padding_inicio" | "__str_padding_fin" => {
                                    let es_inicio = cn == "__str_padding_inicio";
                                    let pad_idx = if es_inicio {
                                        LW_STR_PAD_START
                                    } else {
                                        LW_STR_PAD_END
                                    };
                                    let s = args
                                        .first()
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let w = args
                                        .get(1)
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let f = args
                                        .get(2)
                                        .copied()
                                        .unwrap_or_else(|| self.void_handle(&mut builder));
                                    let start =
                                        builder.ins().iconst(i64, if es_inicio { 1 } else { 0 });
                                    stack.push((
                                        self.lw_call(&mut builder, pad_idx, &[s, w, f, start]),
                                        0,
                                    ));
                                }
                                _ => {
                                    if let Some((idx, arity)) = lw_builtin(cn) {
                                        // v3.5.30: fast-path de strings — largo
                                        // devuelve i64 CRUDO (sin box del
                                        // resultado).
                                        if idx == LW_ARR_LEN && arity == 1 {
                                            let h = args
                                                .first()
                                                .copied()
                                                .unwrap_or_else(|| self.void_handle(&mut builder));
                                            let r = self.lw_call(&mut builder, LW_ARR_LEN_I, &[h]);
                                            stack.push((r, 1));
                                        } else {
                                            let mut cargs: Vec<Value> = Vec::with_capacity(arity);
                                            for k in 0..arity {
                                                cargs.push(args.get(k).copied().unwrap_or_else(
                                                    || self.void_handle(&mut builder),
                                                ));
                                            }
                                            stack
                                                .push((self.lw_call(&mut builder, idx, &cargs), 0));
                                            risky = matches!(
                                                idx,
                                                LW_ARR_GET | LW_ARR_SET | LW_TUP_GET | LW_SUB
                                            );
                                        }
                                    } else {
                                        // imposible: user-calls y no-soportados se
                                        // resolvieron antes del match.
                                        record_unsupported_builtin(cn);
                                        stack.push((self.void_handle(&mut builder), 0));
                                    }
                                }
                            }
                        } // else del fast-path
                    } // if !cr_done
                }
                Instr::MakeRef(n) => {
                    // prestado mut: referencia a la celda estable de la variable
                    if let Some(&ss) = slots.get(n) {
                        let addr = builder.ins().stack_addr(i64, ss, 0);
                        stack.push((self.lw_call(&mut builder, LW_MKREF, &[addr]), 0));
                    } else {
                        // no debería pasar: el pre-pass crea celdas para todos
                        // los objetivos de MakeRef
                        stack.push((self.void_handle(&mut builder), 0));
                    }
                }
                Instr::ArrayPushVar(n) => {
                    // v3.5.29: push nativo sobre array de enteros promovido
                    // (slot ptr/len/cap, crecimiento amortizado, sin boxing).
                    // El dummy del Load xs queda debajo (modelo de
                    // profundidades); se empuja otro dummy como resultado.
                    if cr_arr_vars.contains(n) {
                        let (v, k) = self.cr_pop(&mut builder, &mut stack);
                        let iv = if k == 1 {
                            v
                        } else {
                            let h = self.cr_box(&mut builder, v, k);
                            self.lw_call(&mut builder, LW_H2I, &[h])
                        };
                        let pa = builder.ins().stack_addr(i64, arr_ptr_slot[n], 0);
                        let la = builder.ins().stack_addr(i64, arr_len_slot[n], 0);
                        let ca = builder.ins().stack_addr(i64, arr_cap_slot[n], 0);
                        self.lw_call_void(&mut builder, LW_IARR_PUSH, &[pa, la, ca, iv]);
                        stack.push((arr_dummy.unwrap(), 3));
                    } else {
                        let x = self.pop_handle(&mut builder, &mut stack);
                        let ss = find_slot_cl(&scopes, n).or_else(|| slots.get(n).copied());
                        let r = if let Some(ss) = ss {
                            // la celda es dueña exclusiva del buffer (stores y
                            // args se deep-copian) → push in-place amortizado O(1)
                            // v3.5.42 (bug fuzz gen_ref): write-back con
                            // LW_STORE_SLOT (write-through si el slot es T_PTR
                            // por prestado mut); _lw_store_slot_direct
                            // sobreescribía la referencia y la mutación del
                            // llamador se perdía.
                            let addr = builder.ins().stack_addr(i64, ss, 0);
                            let a = self.lw_call(&mut builder, LW_LOAD_SLOT, &[addr]);
                            let r = self.lw_call(&mut builder, LW_ARR_PUSH_IP, &[a, x]);
                            self.lw_call_void(&mut builder, LW_STORE_SLOT, &[addr, r]);
                            r
                        } else if global_names.contains(n) {
                            let addr = self.global_addr(&mut builder, n);
                            let a = self.lw_call(&mut builder, LW_LOAD_SLOT, &[addr]);
                            let r = self.lw_call(&mut builder, LW_ARR_PUSH_IP, &[a, x]);
                            self.lw_call_void(&mut builder, LW_STORE_SLOT, &[addr, r]);
                            r
                        } else {
                            self.void_handle(&mut builder)
                        };
                        stack.push((r, 0));
                    }
                }
                Instr::ArrayNew(n) => {
                    // v3.5.29: `sea xs = [...]` sobre array promovido → los
                    // elementos van directos a los slots nativos (sin
                    // LW_ARR_NEW ni pushes de handle).
                    let promoted_target: Option<String> =
                        if let Some(Instr::StoreLocal(xn)) = instrs.get(ii + 1) {
                            if cr_arr_vars.contains(xn) {
                                Some(xn.clone())
                            } else {
                                None
                            }
                        } else {
                            None
                        };
                    if let Some(xn) = promoted_target {
                        let mut items: Vec<(Value, u8)> = Vec::with_capacity(*n);
                        for _ in 0..*n {
                            items.push(self.cr_pop(&mut builder, &mut stack));
                        }
                        items.reverse();
                        for (v, k) in items {
                            let iv = if k == 1 {
                                v
                            } else {
                                let h = self.cr_box(&mut builder, v, k);
                                self.lw_call(&mut builder, LW_H2I, &[h])
                            };
                            let pa = builder.ins().stack_addr(i64, arr_ptr_slot[&xn], 0);
                            let la = builder.ins().stack_addr(i64, arr_len_slot[&xn], 0);
                            let ca = builder.ins().stack_addr(i64, arr_cap_slot[&xn], 0);
                            self.lw_call_void(&mut builder, LW_IARR_PUSH, &[pa, la, ca, iv]);
                        }
                        stack.push((arr_dummy.unwrap(), 3));
                    } else {
                        let mut items: Vec<Value> = Vec::with_capacity(*n);
                        for _ in 0..*n {
                            items.push(self.pop_handle(&mut builder, &mut stack));
                        }
                        items.reverse();
                        let mut h = self.lw_call(&mut builder, LW_ARR_NEW, &[]);
                        for it in items {
                            h = self.lw_call(&mut builder, LW_ARR_PUSH, &[h, it]);
                        }
                        stack.push((h, 0));
                    }
                }
                Instr::ArrayPush => {
                    let x = self.pop_handle(&mut builder, &mut stack);
                    let a = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_ARR_PUSH, &[a, x]), 0));
                }
                Instr::ArrayGet => {
                    let (ix, kx) = self.cr_pop(&mut builder, &mut stack);
                    let (a, ka) = self.cr_pop(&mut builder, &mut stack);
                    if ka == 3 {
                        // array promovido sin fusión (índice complejo):
                        // limitación existente compartida con el backend C —
                        // devuelve 0 en vez de crashear.
                        let _ = kx;
                        stack.push((builder.ins().iconst(i64, 0), 1));
                    } else {
                        let ixh = self.cr_box(&mut builder, ix, kx);
                        let ah = self.cr_box(&mut builder, a, ka);
                        stack.push((self.lw_call(&mut builder, LW_ARR_GET, &[ah, ixh]), 0));
                        risky = true; // fuera de rango lanza
                    }
                }
                Instr::ArraySet => {
                    let x = self.pop_handle(&mut builder, &mut stack);
                    let ix = self.pop_handle(&mut builder, &mut stack);
                    let a = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_ARR_SET, &[a, ix, x]), 0));
                    risky = true;
                }
                Instr::ArrayLen => {
                    let (a, ka) = self.cr_pop(&mut builder, &mut stack);
                    if ka == 3 {
                        stack.push((builder.ins().iconst(i64, 0), 1));
                    } else {
                        let ah = self.cr_box(&mut builder, a, ka);
                        stack.push((self.lw_call(&mut builder, LW_ARR_LEN, &[ah]), 0));
                    }
                }
                Instr::StructNew(_, n) => {
                    // Orden IR: primero los n valores, luego los n nombres.
                    let mut names: Vec<Value> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        names.push(self.pop_handle(&mut builder, &mut stack));
                    }
                    names.reverse();
                    let mut vals: Vec<Value> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        vals.push(self.pop_handle(&mut builder, &mut stack));
                    }
                    vals.reverse();
                    let mut h = self.lw_call(&mut builder, LW_ST_NEW, &[]);
                    for i in 0..*n {
                        h = self.lw_call(&mut builder, LW_ST_ADD, &[h, names[i], vals[i]]);
                    }
                    stack.push((h, 0));
                }
                Instr::StructGet => {
                    let name_h = self.pop_handle(&mut builder, &mut stack);
                    let obj = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_ST_GET, &[obj, name_h]), 0));
                }
                Instr::StructSet => {
                    let x = self.pop_handle(&mut builder, &mut stack);
                    let name_h = self.pop_handle(&mut builder, &mut stack);
                    let obj = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_ST_SET, &[obj, name_h, x]), 0));
                }
                Instr::TupleNew(n) => {
                    let mut vals: Vec<Value> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        vals.push(self.pop_handle(&mut builder, &mut stack));
                    }
                    vals.reverse();
                    let mut h = self.lw_call(&mut builder, LW_TUP_NEW, &[]);
                    for v in vals {
                        h = self.lw_call(&mut builder, LW_TUP_PUSH, &[h, v]);
                    }
                    stack.push((h, 0));
                }
                Instr::TupleAccess(i) => {
                    let t = self.pop_handle(&mut builder, &mut stack);
                    let ix = builder.ins().iconst(i64, *i as i64);
                    stack.push((self.lw_call(&mut builder, LW_TUP_GET, &[t, ix]), 0));
                    risky = true; // fuera de rango lanza
                }
                Instr::OptionSome => {
                    let v = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_SOME, &[v]), 0));
                }
                Instr::OptionNone => {
                    stack.push((self.lw_call(&mut builder, LW_NONE, &[]), 0));
                }
                Instr::ResultOk => {
                    let v = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_OK, &[v]), 0));
                }
                Instr::ResultErr => {
                    let v = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_ERR, &[v]), 0));
                }
                // ── Incremento B (v3.5.7) ──
                Instr::PushHandler(catch_l) => {
                    self.lw_call_void(&mut builder, LW_TRY_BEGIN, &[]);
                    handlers.push(*catch_l);
                }
                Instr::PopHandler => {
                    self.lw_call_void(&mut builder, LW_TRY_END, &[]);
                    handlers.pop();
                }
                Instr::TryUnwrap => {
                    // `?` sobre exito/error: T_OK → payload; si no, RETURN del
                    // valor (paridad backend C: `if (_u.t == T_OK) ... else
                    // return _u`)
                    let h = self.pop_handle(&mut builder, &mut stack);
                    let t = self.lw_call(&mut builder, LW_KIND, &[h]);
                    let err_tag = builder.ins().iconst(i64, 8); // T_ERR
                    let is_err = builder.ins().icmp(IntCC::Equal, t, err_tag);
                    let err_block = builder.create_block();
                    let cont_block = builder.create_block();
                    builder.ins().brif(is_err, err_block, &[], cont_block, &[]);
                    builder.switch_to_block(err_block);
                    builder.ensure_inserted_block();
                    // v3.5.28: si la función retorna entero crudo, el return
                    // temprano debe entregar i64 crudo (defensivo).
                    let early = if returns_int.get(name).copied().unwrap_or(false) {
                        self.lw_call(&mut builder, LW_H2I, &[h])
                    } else {
                        h
                    };
                    builder.ins().return_(&[early]);
                    builder.switch_to_block(cont_block);
                    builder.ensure_inserted_block();
                    cur = cont_block;
                    stack.push((self.lw_call(&mut builder, LW_PAYLOAD, &[h]), 0));
                }
                Instr::MatchType(k) => {
                    // elegir con tipos: 0=algun(T_SOM) 1=exito(T_OK) 2=error(T_ERR)
                    let h = self.pop_handle(&mut builder, &mut stack);
                    let t = self.lw_call(&mut builder, LW_KIND, &[h]);
                    let tag = match *k {
                        0 => 9, // T_SOM
                        1 => 7, // T_OK
                        2 => 8, // T_ERR
                        _ => -1,
                    };
                    let want = builder.ins().iconst(i64, tag);
                    let eq = builder.ins().icmp(IntCC::Equal, t, want);
                    let one = builder.ins().iconst(i64, 1);
                    let zero = builder.ins().iconst(i64, 0);
                    let sel = builder.ins().select(eq, one, zero);
                    stack.push((self.lw_call(&mut builder, LW_BOOL, &[sel]), 0));
                }
                Instr::MatchPayload => {
                    let h = self.pop_handle(&mut builder, &mut stack);
                    stack.push((self.lw_call(&mut builder, LW_PAYLOAD, &[h]), 0));
                }
                Instr::MatchVariant(vname) => {
                    let h = self.pop_handle(&mut builder, &mut stack);
                    let data_id = self.get_string_ptr(vname);
                    let gv = self.module.declare_data_in_func(data_id, builder.func);
                    let ptr = builder.ins().global_value(i64, gv);
                    stack.push((self.lw_call(&mut builder, LW_ENM_VARIANT_IS, &[h, ptr]), 0));
                }
                Instr::EnumCtor {
                    enum_name,
                    variant,
                    argc,
                } => {
                    let mut vals: Vec<Value> = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        vals.push(self.pop_handle(&mut builder, &mut stack));
                    }
                    vals.reverse();
                    let mut arr = self.lw_call(&mut builder, LW_ARR_NEW, &[]);
                    for v in vals {
                        arr = self.lw_call(&mut builder, LW_ARR_PUSH, &[arr, v]);
                    }
                    let en_id = self.get_string_ptr(enum_name);
                    let en_gv = self.module.declare_data_in_func(en_id, builder.func);
                    let en_ptr = builder.ins().global_value(i64, en_gv);
                    let vr_id = self.get_string_ptr(variant);
                    let vr_gv = self.module.declare_data_in_func(vr_id, builder.func);
                    let vr_ptr = builder.ins().global_value(i64, vr_gv);
                    stack.push((
                        self.lw_call(&mut builder, LW_ENM_NEW, &[arr, en_ptr, vr_ptr]),
                        0,
                    ));
                }
                Instr::FuncRef(fn_name) => {
                    // función como valor: dirección nativa + nombre
                    if let Some(finfo) = self.funcs.get(fn_name).cloned() {
                        let fref = self.module.declare_func_in_func(finfo.id, builder.func);
                        let addr = builder.ins().func_addr(i64, fref);
                        let nm_id = self.get_string_ptr(fn_name);
                        let nm_gv = self.module.declare_data_in_func(nm_id, builder.func);
                        let nm_ptr = builder.ins().global_value(i64, nm_gv);
                        stack.push((self.lw_call(&mut builder, LW_FREF, &[addr, nm_ptr]), 0));
                    } else {
                        record_unsupported_builtin(fn_name);
                        stack.push((self.void_handle(&mut builder), 0));
                    }
                }
                Instr::CallValue(argc) => {
                    // stack: fref, arg0..arg{n-1} → llamada indirecta
                    let mut args: Vec<Value> = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        args.push(self.pop_handle(&mut builder, &mut stack));
                    }
                    args.reverse();
                    let fref_h = self.pop_handle(&mut builder, &mut stack);
                    let addr = self.lw_call(&mut builder, LW_FREF_ADDR, &[fref_h]);
                    let mut sig = self.module.make_signature();
                    for _ in 0..*argc {
                        sig.params.push(AbiParam::new(i64));
                    }
                    sig.returns.push(AbiParam::new(i64));
                    let sig_ref = builder.import_signature(sig);
                    let mut dargs: Vec<Value> = Vec::with_capacity(args.len());
                    for av in &args {
                        dargs.push(self.lw_call(&mut builder, LW_DCP, &[*av]));
                    }
                    let call = builder.ins().call_indirect(sig_ref, addr, &dargs);
                    let res = builder.inst_results(call)[0];
                    // llamada indirecta: el resultado siempre es handle
                    stack.push((res, 0));
                    risky = true;
                }
                Instr::ScopePush => {
                    scopes.push(HashMap::new());
                }
                Instr::ScopePop => {
                    if scopes.len() > 1 {
                        scopes.pop();
                    }
                }
                Instr::Return => {
                    let (val, k) = self.cr_pop(&mut builder, &mut stack);
                    // v3.5.28: retorno crudo para funciones returns_int
                    // (llamadas directas con ABI de enteros).
                    let out = if returns_int.get(name).copied().unwrap_or(false) {
                        if k == 1 {
                            val
                        } else {
                            let h = self.cr_box(&mut builder, val, k);
                            self.lw_call(&mut builder, LW_H2I, &[h])
                        }
                    } else {
                        self.cr_box(&mut builder, val, k)
                    };
                    builder.ins().return_(&[out]);
                    terminated = true;
                    stack.clear();
                }
                Instr::Halt => {
                    let (val, k) = self.cr_pop(&mut builder, &mut stack);
                    let out = self.cr_box(&mut builder, val, k);
                    builder.ins().return_(&[out]);
                    terminated = true;
                    stack.clear();
                }
                Instr::Jmp(target) => {
                    if let Some(&b) = label_block.get(target) {
                        let d = label_depth.get(target).copied().unwrap_or(0);
                        let mut args: Vec<cranelift::codegen::ir::BlockArg> = Vec::new();
                        if catch_labels.contains(target) {
                            args.push(cranelift::codegen::ir::BlockArg::Value(void0));
                        } else if stack.len() >= d {
                            for (v, k) in &stack[stack.len() - d..] {
                                let bv = self.cr_box(&mut builder, *v, *k);
                                args.push(cranelift::codegen::ir::BlockArg::Value(bv));
                            }
                        } else {
                            for _ in 0..d {
                                args.push(cranelift::codegen::ir::BlockArg::Value(void0));
                            }
                        }
                        // v3.5.30: backedge a head promovido → params;
                        // cualquier otro salto → flush de slots i64.
                        if let Some(vars) = head_vars.get(target) {
                            for v in vars {
                                let vv = cr_int_val(&mut builder, &mut int_cache, &int_slots, v);
                                args.push(cranelift::codegen::ir::BlockArg::Value(vv));
                            }
                        } else {
                            cr_flush_ints(&mut builder, &int_cache, &int_slots);
                        }
                        builder.ins().jump(b, &args);
                    }
                    terminated = true;
                }
                Instr::JmpIf(target) => {
                    let (v, k) = self.cr_pop(&mut builder, &mut stack);
                    if let Some(&b) = label_block.get(target) {
                        let next_block = builder.create_block();
                        let d = label_depth.get(target).copied().unwrap_or(0);
                        let mut args: Vec<cranelift::codegen::ir::BlockArg> = Vec::new();
                        if catch_labels.contains(target) {
                            args.push(cranelift::codegen::ir::BlockArg::Value(void0));
                        } else if stack.len() >= d {
                            for (vv, kk) in &stack[stack.len() - d..] {
                                let bv = self.cr_box(&mut builder, *vv, *kk);
                                args.push(cranelift::codegen::ir::BlockArg::Value(bv));
                            }
                        } else {
                            for _ in 0..d {
                                args.push(cranelift::codegen::ir::BlockArg::Value(void0));
                            }
                        }
                        // v3.5.30: borde del salto — head promovido → las
                        // vars viajan por params; si no, bloque intermedio
                        // (pre) con flush de slots antes del destino. El pre
                        // se rellena TRAS el brif (que termina el bloque
                        // actual).
                        let is_head = head_vars.contains_key(target);
                        if is_head {
                            if let Some(vars) = head_vars.get(target) {
                                for v in vars {
                                    let vv =
                                        cr_int_val(&mut builder, &mut int_cache, &int_slots, v);
                                    args.push(cranelift::codegen::ir::BlockArg::Value(vv));
                                }
                            }
                        }
                        let pre = if is_head {
                            None
                        } else {
                            Some(builder.create_block())
                        };
                        let dest_block = pre.unwrap_or(b);
                        let dest_args = if is_head { args.clone() } else { Vec::new() };
                        // v3.5.28: condición cruda → brif nativo (sin
                        // _lw_truthy_i). JmpIf salta cuando es FALSO.
                        match k {
                            2 => {
                                builder.ins().brif(
                                    v,
                                    next_block,
                                    &[] as &[cranelift::codegen::ir::BlockArg],
                                    dest_block,
                                    &dest_args,
                                );
                            }
                            1 => {
                                let z = builder.ins().iconst(i64, 0);
                                let nz = builder.ins().icmp(IntCC::NotEqual, v, z);
                                builder.ins().brif(
                                    nz,
                                    next_block,
                                    &[] as &[cranelift::codegen::ir::BlockArg],
                                    dest_block,
                                    &dest_args,
                                );
                            }
                            _ => {
                                let t = self.lw_call(&mut builder, LW_TRUTHY_I, &[v]);
                                let z = builder.ins().iconst(i64, 0);
                                let is_zero = builder.ins().icmp(IntCC::Equal, t, z);
                                builder.ins().brif(
                                    is_zero,
                                    dest_block,
                                    &dest_args,
                                    next_block,
                                    &[] as &[cranelift::codegen::ir::BlockArg],
                                );
                            }
                        }
                        if let Some(pre) = pre {
                            // rellenar el bloque intermedio: flush + salto real
                            builder.switch_to_block(pre);
                            builder.ensure_inserted_block();
                            cr_flush_ints(&mut builder, &int_cache, &int_slots);
                            builder.ins().jump(b, &args);
                        }
                        builder.switch_to_block(next_block);
                        builder.ensure_inserted_block();
                        cur = next_block;
                        terminated = false;
                        // el fallthrough continúa con el stack intacto
                    }
                }
                Instr::Phi(_, _) | Instr::Nop => {}
                _ => {}
            }
            // Chequeo de error tras operación riesgosa (paridad _ERRCHK/_ERRPROP
            // del backend C): con handler abierto salta al catch con el mensaje;
            // sin handler retorna void y el llamador hace su propio chequeo.
            if risky && !terminated {
                self.emit_err_check(
                    &mut builder,
                    &handlers,
                    &label_block,
                    &int_cache,
                    &int_slots,
                );
            }
            ii += 1;
        }

        // v3.5.28: labels que quedan VACÍOS al final de la función (p. ej. el
        // join de un `elegir` cuando todos los casos terminan en Return) son
        // inalcanzables pero el verificador de Cranelift exige terminador:
        // se rellenan con trap (nunca se ejecuta). Bug latente preexistente.
        // v3.5.31: si el bloque actual quedó con instrucciones pero SIN
        // terminador (p. ej. un Label final con block-params), terminarlo
        // con return void ANTES de cambiar a otro bloque.
        if let Some(cur_block) = builder.current_block() {
            let needs_term = {
                let mut insts = builder.func.layout.block_insts(cur_block);
                match insts.next() {
                    Some(_) => match insts.next_back() {
                        Some(last) => !builder.func.dfg.insts[last].opcode().is_terminator(),
                        None => false,
                    },
                    None => false,
                }
            };
            if needs_term {
                let v = self.lw_call(&mut builder, LW_VOID, &[]);
                builder.ins().return_(&[v]);
            }
        }
        for &b in label_block.values() {
            if builder.func.layout.block_insts(b).next().is_none() {
                builder.switch_to_block(b);
                builder.ensure_inserted_block();
                // Equivalente al "fin de función implícito" de la VM:
                // retornar void (el bloque puede ser inalcanzable — return
                // es inofensivo — o alcanzable vía un caso que no retorna).
                let v = self.lw_call(&mut builder, LW_VOID, &[]);
                builder.ins().return_(&[v]);
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

    /// Chequeo de error tras operación riesgosa: si `_err` está activo, salta
    /// al catch abierto más cercano con el mensaje (block-param) o retorna
    /// void para propagar al llamador (paridad backend C).
    #[allow(clippy::too_many_arguments)]
    fn emit_err_check(
        &mut self,
        builder: &mut FunctionBuilder,
        handlers: &[usize],
        label_block: &HashMap<usize, Block>,
        int_cache: &HashMap<String, cranelift::codegen::ir::Value>,
        int_slots: &HashMap<String, cranelift::codegen::ir::StackSlot>,
    ) {
        let flag = self.lw_call(builder, LW_ERR_ACTIVE, &[]);
        let z = builder.ins().iconst(types::I64, 0);
        let is_err = builder.ins().icmp(IntCC::NotEqual, flag, z);
        let ok_block = builder.create_block();
        let dispatch = builder.create_block();
        builder.ins().brif(
            is_err,
            dispatch,
            &[] as &[cranelift::codegen::ir::BlockArg],
            ok_block,
            &[] as &[cranelift::codegen::ir::BlockArg],
        );
        builder.switch_to_block(dispatch);
        builder.ensure_inserted_block();
        // v3.5.30: el catch lee variables por slot → materializar el cache
        // SSA antes del salto (solo en la rama de error; el ok sigue vivo).
        cr_flush_ints(builder, int_cache, int_slots);
        if let Some(&catch_l) = handlers.last() {
            let msg = self.lw_call(builder, LW_ERR_TAKE, &[]);
            if let Some(&cb) = label_block.get(&catch_l) {
                builder
                    .ins()
                    .jump(cb, &[cranelift::codegen::ir::BlockArg::Value(msg)]);
            } else {
                let v = self.lw_call(builder, LW_VOID, &[]);
                builder.ins().return_(&[v]);
            }
        } else {
            let v = self.lw_call(builder, LW_VOID, &[]);
            builder.ins().return_(&[v]);
        }
        builder.switch_to_block(ok_block);
        builder.ensure_inserted_block();
    }

    fn entry_point(&mut self, entry: &str) {
        // v3.5.6: resuelve el objetivo real de entrada. Si el programa ya
        // define una función llamada "main", ESA es el símbolo de entrada del
        // binario (se exporta directamente) — declarar un wrapper "main"
        // adicional colisionaría (DuplicateDefinition).
        let target = if self.funcs.contains_key(entry) {
            entry.to_string()
        } else if self.funcs.contains_key("main") {
            "main".to_string()
        } else if self.funcs.contains_key("principal") {
            "principal".to_string()
        } else {
            String::new()
        };

        if target == "main" {
            // La función main del usuario ES el entry point: re-declararla
            // como Export (declare() la registró Local). Firma idéntica, la
            // declaración previa se fusiona sin duplicar definición.
            if let Some(info) = self.funcs.get("main") {
                let _ = self
                    .module
                    .declare_function("main", Linkage::Export, &info.sig);
            }
            return;
        }

        // Wrapper "main" que llama al objetivo (no existe función del usuario
        // llamada "main", así que no hay colisión de símbolo).
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

        if let Some(info) = self.funcs.get(&target) {
            let func_ref = self.module.declare_func_in_func(info.id, builder.func);
            builder.ins().call(func_ref, &[]);
        }
        // v3.5.20: exit code 0 (antes se devolvía el handle del resultado y
        // el proceso salía con un código basura = byte bajo del puntero).
        let zero = builder.ins().iconst(types::I64, 0);
        builder.ins().return_(&[zero]);
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

/// Capacidad del backend LLVM textual: v3.5.7 — mismo modelo de handles
/// opacos (_lw_*) que el backend Cranelift; cobertura completa del IR.
/// v3.5.17: nombres de builtins de concurrencia nativa (hilos, canales,
/// mutexes) soportados por Cranelift vía el runtime de lumen_rt.h.
fn thread_builtin(name: &str) -> bool {
    matches!(
        name,
        "__tarea_lanzar"
            | "__task_spawn"
            | "__hilo_lanzar"
            | "__thread_spawn"
            | "__tarea_esperar"
            | "__task_await"
            | "__hilo_esperar"
            | "__thread_join"
            | "__canal_nuevo"
            | "__channel_new"
            | "__canal_enviar"
            | "__channel_send"
            | "__canal_recibir"
            | "__channel_recv"
            | "__mutex_nuevo"
            | "__mutex_new"
            | "__mutex_bloquear"
            | "__mutex_lock"
            | "__calendario_hijri"
            | "__calendar_hijri"
            | "__calendario_persa"
            | "__calendar_persian"
            | "__tiempo_ahora"
            | "__time_now"
            | "__tiempo_formatear"
            | "__time_format"
            | "__tiempo_diferencia"
            | "__time_diff"
            | "__tiempo_parsear"
            | "__tiempo_parse"
            | "__time_parse"
    )
}

/// v3.5.18: builtins de string unicode soportados en Cranelift.
fn string_builtin(name: &str) -> bool {
    matches!(
        name,
        "__str_a_caracteres"
            | "__str_to_chars"
            | "__str_mayusculas"
            | "__str_upper"
            | "__str_minusculas"
            | "__str_lower"
            | "__str_padding_inicio"
            | "__str_padding_fin"
    )
}

pub fn llvm_supported(program: &Program) -> Vec<String> {
    let program = lower_arraysetvar(program);
    let mut bad = cranelift_supported(&program);
    // v3.5.17: el emisor LLVM aún no mapea los builtins de hilos (solo
    // Cranelift los tiene); rechazo explícito para no compilar hilos mudos.
    let mut note = |f: &str| {
        if !bad.iter().any(|x| x == f) {
            bad.push(f.to_string());
        }
    };
    for func in program.funcs.values() {
        for ins in &func.instrs {
            if let Instr::Call(n, _) = ins {
                if thread_builtin(n) || string_builtin(n) {
                    note(&format!("concurrencia/strings en LLVM ({})", n));
                }
            }
        }
    }
    bad.sort();
    bad.dedup();
    bad
}

/// Capacidad del backend Cranelift (objeto nativo): v3.5.7 — runtime `_lw_*`
/// de handles opacos cubre TODO el IR (incremento B: enums, closures,
/// prestado mut, intentar/atrapar, sombreado por bloques). Solo se rechazan
/// builtins sin helper asignado.
pub fn cranelift_supported(program: &Program) -> Vec<String> {
    let program = lower_arraysetvar(program);
    let mut bad: Vec<String> = Vec::new();
    let mut note = |f: &str| {
        if !bad.iter().any(|x| x == f) {
            bad.push(f.to_string());
        }
    };
    for func in program.funcs.values() {
        for ins in &func.instrs {
            if let Instr::Call(n, _) = ins {
                let is_builtin = matches!(n.as_str(), "imprimir" | "print")
                    || thread_builtin(n)
                    || string_builtin(n)
                    || lw_builtin(n).is_some();
                if !is_builtin && !program.funcs.contains_key(n) {
                    note(format!("builtins ({})", n).as_str());
                }
            }
        }
    }
    bad.sort();
    bad.dedup();
    bad
}

pub fn compile_to_llvm_ir(program: &Program) -> String {
    let program = lower_arraysetvar(program);
    // v3.5.7: backend LLVM IR textual con el MISMO modelo de handles opacos
    // que el backend Cranelift (runtime _lw_*). Paridad de cobertura.
    let mut out = String::new();
    out.push_str("; ModuleID = 'lumen'\n");
    out.push_str("source_filename = \"lumen.nv\"\n");
    out.push_str("target datalayout = \"e-m:e-p270:32:32-p271:32:32-p272:64:64-i64:64-i128:128-f80:128-n8:16:32:64-S128\"\n\n");

    // ── Declaraciones del runtime _lw_* (definidos en el shim de link:
    //    lw_shim_source() = lumen_rt.h + LW_RUNTIME) ──
    let decls: &[(&str, &str)] = &[
        ("_lw_int", "declare i64 @_lw_int(i64)"),
        ("_lw_flt", "declare i64 @_lw_flt(double)"),
        ("_lw_bool", "declare i64 @_lw_bool(i64)"),
        ("_lw_str", "declare i64 @_lw_str(i64)"),
        ("_lw_void", "declare i64 @_lw_void()"),
        ("_lw_none", "declare i64 @_lw_none()"),
        ("_lw_print", "declare void @_lw_print(i64)"),
        ("_lw_print_blank", "declare void @_lw_print_blank()"),
        ("_lw_join", "declare i64 @_lw_join(i64, i64)"),
        ("_lw_bin", "declare i64 @_lw_bin(i64, i64, i64)"),
        ("_lw_un", "declare i64 @_lw_un(i64, i64)"),
        ("_lw_truthy_i", "declare i64 @_lw_truthy_i(i64)"),
        ("_lw_arr_new", "declare i64 @_lw_arr_new()"),
        ("_lw_arr_push", "declare i64 @_lw_arr_push(i64, i64)"),
        ("_lw_arr_get", "declare i64 @_lw_arr_get(i64, i64)"),
        ("_lw_arr_set", "declare i64 @_lw_arr_set(i64, i64, i64)"),
        ("_lw_arr_len", "declare i64 @_lw_arr_len(i64)"),
        ("_lw_arr_rev", "declare i64 @_lw_arr_rev(i64)"),
        ("_lw_arr_sort", "declare i64 @_lw_arr_sort(i64)"),
        ("_lw_st_new", "declare i64 @_lw_st_new()"),
        ("_lw_st_add", "declare i64 @_lw_st_add(i64, i64, i64)"),
        ("_lw_st_get", "declare i64 @_lw_st_get(i64, i64)"),
        ("_lw_st_set", "declare i64 @_lw_st_set(i64, i64, i64)"),
        ("_lw_tup_new", "declare i64 @_lw_tup_new()"),
        ("_lw_tup_push", "declare i64 @_lw_tup_push(i64, i64)"),
        ("_lw_tup_get", "declare i64 @_lw_tup_get(i64, i64)"),
        ("_lw_read", "declare i64 @_lw_read()"),
        ("_lw_typeof", "declare i64 @_lw_typeof(i64)"),
        ("_lw_to_text", "declare i64 @_lw_to_text(i64)"),
        ("_lw_sub", "declare i64 @_lw_sub(i64, i64, i64)"),
        ("_lw_concat_list", "declare i64 @_lw_concat_list(i64)"),
        ("_lw_some", "declare i64 @_lw_some(i64)"),
        ("_lw_ok", "declare i64 @_lw_ok(i64)"),
        ("_lw_err", "declare i64 @_lw_err(i64)"),
        ("_lw_map_new", "declare i64 @_lw_map_new()"),
        ("_lw_map_set", "declare i64 @_lw_map_set(i64, i64, i64)"),
        ("_lw_map_get", "declare i64 @_lw_map_get(i64, i64)"),
        ("_lw_map_has", "declare i64 @_lw_map_has(i64, i64)"),
        ("_lw_map_len", "declare i64 @_lw_map_len(i64)"),
        ("_lw_map_keys", "declare i64 @_lw_map_keys(i64)"),
        ("_lw_try_begin", "declare void @_lw_try_begin()"),
        ("_lw_try_end", "declare void @_lw_try_end()"),
        ("_lw_err_active", "declare i64 @_lw_err_active()"),
        ("_lw_err_take", "declare i64 @_lw_err_take()"),
        ("_lw_kind", "declare i64 @_lw_kind(i64)"),
        ("_lw_payload", "declare i64 @_lw_payload(i64)"),
        ("_lw_enm_new", "declare i64 @_lw_enm_new(i64, i64, i64)"),
        (
            "_lw_enm_variant_is",
            "declare i64 @_lw_enm_variant_is(i64, i64)",
        ),
        ("_lw_fref", "declare i64 @_lw_fref(i64, i64)"),
        ("_lw_fref_addr", "declare i64 @_lw_fref_addr(i64)"),
        ("_lw_mkref", "declare i64 @_lw_mkref(i64)"),
        ("_lw_load_slot", "declare i64 @_lw_load_slot(i64)"),
        ("_lw_store_slot", "declare void @_lw_store_slot(i64, i64)"),
        (
            "_lw_store_slot_direct",
            "declare void @_lw_store_slot_direct(i64, i64)",
        ),
        ("_lw_dcp", "declare i64 @_lw_dcp(i64)"),
        ("_lw_arr_push_ip", "declare i64 @_lw_arr_push_ip(i64, i64)"),
        ("_lw_abs", "declare i64 @_lw_abs(i64)"),
        ("_lw_sqrt", "declare i64 @_lw_sqrt(i64)"),
        ("_lw_pow", "declare i64 @_lw_pow(i64, i64)"),
        ("_lw_floor", "declare i64 @_lw_floor(i64)"),
        ("_lw_ceil", "declare i64 @_lw_ceil(i64)"),
        ("_lw_round", "declare i64 @_lw_round(i64)"),
        ("_lw_arr_len_i", "declare i64 @_lw_arr_len_i(i64)"),
        ("_lw_to_text_i", "declare i64 @_lw_to_text_i(i64)"),
        ("_lw_concat3", "declare i64 @_lw_concat3(i64, i64, i64)"),
        ("_lw_concat3_i", "declare i64 @_lw_concat3_i(i64, i64, i64)"),
        (
            "_lw_concat3_len_i",
            "declare i64 @_lw_concat3_len_i(i64, i64, i64)",
        ),
    ];
    for (_, d) in decls {
        out.push_str(d);
        out.push('\n');
    }
    out.push('\n');

    // Datos de texto (ConstStr, nombres de enum/variantes, FuncRef)
    let mut strings: BTreeMap<String, String> = BTreeMap::new();
    fn collect_str(s: &str, strings: &mut BTreeMap<String, String>) {
        if strings.contains_key(s) {
            return;
        }
        let idx = strings.len();
        let name = format!("lw_str_{}", idx);
        strings.insert(s.to_string(), name);
    }
    for func in program.funcs.values() {
        for ins in &func.instrs {
            match ins {
                Instr::ConstStr(s) => collect_str(s, &mut strings),
                Instr::MatchVariant(v) => collect_str(v, &mut strings),
                Instr::FuncRef(n) => collect_str(n, &mut strings),
                Instr::EnumCtor {
                    enum_name, variant, ..
                } => {
                    collect_str(enum_name, &mut strings);
                    collect_str(variant, &mut strings);
                }
                _ => {}
            }
        }
    }
    for (s, name) in &strings {
        let mut esc = String::new();
        for b in s.bytes() {
            if b == b'\\' || b == b'"' || !(0x21..=0x7e).contains(&b) {
                esc.push_str(&format!("\\{:02X}", b));
            } else {
                esc.push(b as char);
            }
        }
        out.push_str(&format!(
            "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\"\n",
            name,
            s.len() + 1,
            esc
        ));
    }
    if !strings.is_empty() {
        out.push('\n');
    }

    let str_ptr =
        |out: &mut String, reg: &mut usize, strings: &BTreeMap<String, String>, s: &str| {
            let gname = strings[s].clone();
            let r1 = format!("%r{}", reg);
            *reg += 1;
            out.push_str(&format!(
                "  {} = getelementptr [{} x i8], [{} x i8]* @{}, i64 0, i64 0\n",
                r1,
                s.len() + 1,
                s.len() + 1,
                gname
            ));
            let r2 = format!("%r{}", reg);
            *reg += 1;
            out.push_str(&format!("  {} = ptrtoint i8* {} to i64\n", r2, r1));
            r2
        };

    // ── Celdas globales (top-level usadas desde varias funciones) ──
    let llvm_global_names = program_global_names(&program);
    for g in &llvm_global_names {
        out.push_str(&format!(
            "@lw_glob_{} = global [{} x i8] zeroinitializer, align 8\n",
            mangle(g),
            LW_VAL_SIZE
        ));
    }
    if !llvm_global_names.is_empty() {
        out.push('\n');
    }

    let llvm_entry_name = program_entry_name(&program);
    for (fname, func) in &program.funcs {
        let is_entry_fn = llvm_entry_name.as_deref() == Some(fname.as_str());
        let mangled = format!("lum_{}", mangle(fname));
        let params: Vec<String> = (0..func.params.len())
            .map(|i| format!("i64 %p{}", i))
            .collect();
        out.push_str(&format!(
            "define i64 @{}({}) {{\n",
            mangled,
            params.join(", ")
        ));
        out.push_str("entry:\n");

        let mut reg: usize = 0;
        let mut stack: Vec<String> = Vec::new();
        // Pre-pass: catch labels y objetivos de MakeRef
        let mut catch_labels: std::collections::HashSet<usize> = std::collections::HashSet::new();
        let mut ref_targets: Vec<String> = Vec::new();
        let mut used_names: Vec<String> = Vec::new();
        for ins in &func.instrs {
            match ins {
                Instr::PushHandler(l) => {
                    catch_labels.insert(*l);
                }
                Instr::MakeRef(n) => {
                    if !ref_targets.iter().any(|x| x == n) {
                        ref_targets.push(n.clone());
                    }
                }
                Instr::Load(n)
                | Instr::Store(n)
                | Instr::StoreLocal(n)
                | Instr::ArrayPushVar(n)
                    if !used_names.iter().any(|x| x == n) =>
                {
                    used_names.push(n.clone());
                }
                _ => {}
            }
        }

        // Celdas [72 x i8] para params y objetivos de MakeRef
        let mut cells: HashMap<String, String> = HashMap::new();
        {
            let mut need: Vec<String> = func.params.clone();
            for n in &ref_targets {
                if !need.iter().any(|x| x == n) {
                    need.push(n.clone());
                }
            }
            for n in need {
                let cell = format!("%cell_{}", mangle(&n));
                out.push_str(&format!("  {} = alloca [{} x i8]\n", cell, LW_VAL_SIZE));
                cells.insert(n, cell);
            }
        }
        // void inicial + binding de params
        let void0 = format!("%r{}", reg);
        reg += 1;
        out.push_str(&format!("  {} = call i64 @_lw_void()\n", void0));
        // Binding de entrada con store DIRECTO (sin write-through): la celda
        // puede traer un T_PTR de la llamada anterior (bug v3.5.7).
        for pname in &func.params {
            let cell = &cells[pname];
            let a = format!("%r{}", reg);
            reg += 1;
            out.push_str(&format!(
                "  {} = ptrtoint [{} x i8]* {} to i64\n",
                a, LW_VAL_SIZE, cell
            ));
            out.push_str(&format!(
                "  call void @_lw_store_slot_direct(i64 {}, i64 {})\n",
                a, void0
            ));
        }
        for (pi, pname) in func.params.iter().enumerate() {
            let cell = &cells[pname];
            let a = format!("%r{}", reg);
            reg += 1;
            out.push_str(&format!(
                "  {} = ptrtoint [{} x i8]* {} to i64\n",
                a, LW_VAL_SIZE, cell
            ));
            out.push_str(&format!(
                "  call void @_lw_store_slot_direct(i64 {}, i64 %p{})\n",
                a, pi
            ));
        }
        for n in &ref_targets {
            if !func.params.iter().any(|x| x == n) {
                let cell = &cells[n];
                let a = format!("%r{}", reg);
                reg += 1;
                out.push_str(&format!(
                    "  {} = ptrtoint [{} x i8]* {} to i64\n",
                    a, LW_VAL_SIZE, cell
                ));
                out.push_str(&format!(
                    "  call void @_lw_store_slot_direct(i64 {}, i64 {})\n",
                    a, void0
                ));
            }
        }

        // Variables SSA no-celda: alloca i64 en scope 0 con void.
        // Las globales viven en data compartida (@lw_glob_*), no en alloca.
        let mut scopes: Vec<HashMap<String, String>> = vec![HashMap::new()];
        for n in &used_names {
            if cells.contains_key(n) || llvm_global_names.contains(n) {
                continue;
            }
            let slot = format!("%var_{}", mangle(n));
            out.push_str(&format!("  {} = alloca i64\n", slot));
            out.push_str(&format!("  store i64 {}, i64* {}\n", void0, slot));
            scopes[0].insert(n.clone(), slot);
        }

        let find_slot = |scopes: &[HashMap<String, String>], n: &str| -> Option<String> {
            for sc in scopes.iter().rev() {
                if let Some(v) = sc.get(n) {
                    return Some(v.clone());
                }
            }
            None
        };

        // Merge-slots: labels que reciben valores por los bordes (ternarios,
        // elegir como expresión); los catch reciben el mensaje de error.
        let label_depth = simulate_label_depths(&func.instrs, &catch_labels);
        for ins in &func.instrs {
            if let Instr::Label(n) = ins {
                let d = label_depth.get(n).copied().unwrap_or(0);
                for k in 0..d {
                    out.push_str(&format!("  %mg_{}_{} = alloca i64\n", n, k));
                }
            }
        }
        // store de los valores del stack hacia los merge-slots de un label
        // (devuelve las lineas emitidas); usa void si faltan valores.
        let store_merge = |stack: &[String], void0: &str, n: usize| {
            let d = label_depth.get(&n).copied().unwrap_or(0);
            let mut lines = String::new();
            for k in 0..d {
                let v = if stack.len() >= d {
                    stack[stack.len() - d + k].clone()
                } else {
                    void0.to_string()
                };
                lines.push_str(&format!("  store i64 {}, i64* %mg_{}_{}\n", v, n, k));
            }
            lines
        };

        // Bloques por label (terminator pendiente de gestionar)
        let mut terminated = false;
        let mut handlers: Vec<usize> = Vec::new();
        let mut open_block = true;

        for ins in &func.instrs {
            if let Instr::Label(n) = ins {
                if open_block && !terminated {
                    let _mg = store_merge(&stack, &void0, *n);
                    out.push_str(&_mg);
                    out.push_str(&format!("  br label %L_{}\n", n));
                }
                out.push_str(&format!("L_{}:\n", n));
                // el stack del label llega por los merge-slots
                stack.clear();
                let d = label_depth.get(n).copied().unwrap_or(0);
                for k in 0..d {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = load i64, i64* %mg_{}_{}\n", r, n, k));
                    stack.push(r);
                }
                terminated = false;
                open_block = true;
                continue;
            }
            if terminated {
                continue;
            }
            let mut risky = false;
            match ins {
                Instr::ConstInt(v) => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_int(i64 {})\n", r, v));
                    stack.push(r);
                }
                Instr::ConstFloat(f) => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    // hex-doble LLVM (válido también para inf/nan)
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_flt(double 0x{:016X})\n",
                        r,
                        f.to_bits()
                    ));
                    stack.push(r);
                }
                Instr::ConstBool(b) => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_bool(i64 {})\n",
                        r,
                        if *b { 1 } else { 0 }
                    ));
                    stack.push(r);
                }
                Instr::ConstStr(s) => {
                    let r = str_ptr(&mut out, &mut reg, &strings, s);
                    let h = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_str(i64 {})\n", h, r));
                    stack.push(h);
                }
                Instr::Load(n) => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    if let Some(slot) = find_slot(&scopes, n) {
                        let t = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = load i64, i64* {}\n", t, slot));
                        stack.push(t);
                        let _ = r;
                    } else if let Some(cell) = cells.get(n) {
                        let a = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = ptrtoint [{} x i8]* {} to i64\n",
                            a, LW_VAL_SIZE, cell
                        ));
                        let h = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = call i64 @_lw_load_slot(i64 {})\n", h, a));
                        stack.push(h);
                    } else if llvm_global_names.contains(n) {
                        let a = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = ptrtoint [{} x i8]* @lw_glob_{} to i64\n",
                            a,
                            LW_VAL_SIZE,
                            mangle(n)
                        ));
                        let h = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = call i64 @_lw_load_slot(i64 {})\n", h, a));
                        stack.push(h);
                    } else {
                        stack.push(void0.clone());
                    }
                }
                Instr::StoreLocal(n) => {
                    if let Some(v) = stack.pop() {
                        if let Some(slot) = scopes.last().unwrap().get(n) {
                            let d = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!("  {} = call i64 @_lw_dcp(i64 {})\n", d, v));
                            out.push_str(&format!("  store i64 {}, i64* {}\n", d, slot));
                        } else if scopes.len() == 1 && is_entry_fn && llvm_global_names.contains(n)
                        {
                            let a = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!(
                                "  {} = ptrtoint [{} x i8]* @lw_glob_{} to i64\n",
                                a,
                                LW_VAL_SIZE,
                                mangle(n)
                            ));
                            out.push_str(&format!(
                                "  call void @_lw_store_slot(i64 {}, i64 {})\n",
                                a, v
                            ));
                        } else if scopes.len() == 1 && cells.contains_key(n) {
                            let cell = &cells[n];
                            let a = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!(
                                "  {} = ptrtoint [{} x i8]* {} to i64\n",
                                a, LW_VAL_SIZE, cell
                            ));
                            out.push_str(&format!(
                                "  call void @_lw_store_slot(i64 {}, i64 {})\n",
                                a, v
                            ));
                        } else {
                            let slot = format!("%var_{}_{}", mangle(n), reg);
                            reg += 1;
                            out.push_str(&format!("  {} = alloca i64\n", slot));
                            let d = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!("  {} = call i64 @_lw_dcp(i64 {})\n", d, v));
                            out.push_str(&format!("  store i64 {}, i64* {}\n", d, slot));
                            scopes.last_mut().unwrap().insert(n.clone(), slot);
                        }
                    }
                }
                Instr::Store(n) => {
                    if let Some(v) = stack.pop() {
                        if let Some(slot) = find_slot(&scopes, n) {
                            let d = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!("  {} = call i64 @_lw_dcp(i64 {})\n", d, v));
                            out.push_str(&format!("  store i64 {}, i64* {}\n", d, slot));
                        } else if let Some(cell) = cells.get(n) {
                            let a = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!(
                                "  {} = ptrtoint [{} x i8]* {} to i64\n",
                                a, LW_VAL_SIZE, cell
                            ));
                            out.push_str(&format!(
                                "  call void @_lw_store_slot(i64 {}, i64 {})\n",
                                a, v
                            ));
                        } else if llvm_global_names.contains(n) {
                            let a = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!(
                                "  {} = ptrtoint [{} x i8]* @lw_glob_{} to i64\n",
                                a,
                                LW_VAL_SIZE,
                                mangle(n)
                            ));
                            out.push_str(&format!(
                                "  call void @_lw_store_slot(i64 {}, i64 {})\n",
                                a, v
                            ));
                        } else {
                            let slot = format!("%var_{}_{}", mangle(n), reg);
                            reg += 1;
                            out.push_str(&format!("  {} = alloca i64\n", slot));
                            let d = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!("  {} = call i64 @_lw_dcp(i64 {})\n", d, v));
                            out.push_str(&format!("  store i64 {}, i64* {}\n", d, slot));
                            scopes.last_mut().unwrap().insert(n.clone(), slot);
                        }
                    }
                }
                Instr::Binary(op) => {
                    let b = stack.pop().unwrap_or_else(|| void0.clone());
                    let a = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_bin(i64 {}, i64 {}, i64 {})\n",
                        r,
                        op_code(op),
                        a,
                        b
                    ));
                    stack.push(r);
                    risky = true;
                }
                Instr::Unary(op) => {
                    if let Some(v) = stack.pop() {
                        let code = match op {
                            Op::Not => 1,
                            Op::BitNot => 2,
                            _ => 0,
                        };
                        let r = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_un(i64 {}, i64 {})\n",
                            r, code, v
                        ));
                        stack.push(r);
                    }
                }
                Instr::Print => {
                    let v = stack.pop().unwrap_or_else(|| void0.clone());
                    out.push_str(&format!("  call void @_lw_print(i64 {})\n", v));
                }
                Instr::Read => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_read()\n", r));
                    stack.push(r);
                }
                Instr::Call(cn, argc) => {
                    let mut args: Vec<String> = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        args.push(stack.pop().unwrap_or_else(|| void0.clone()));
                    }
                    args.reverse();
                    match cn.as_str() {
                        "imprimir" | "print" => {
                            if args.is_empty() {
                                out.push_str("  call void @_lw_print_blank()\n");
                            } else {
                                let mut acc = args[0].clone();
                                for av in args.iter().skip(1) {
                                    let r = format!("%r{}", reg);
                                    reg += 1;
                                    out.push_str(&format!(
                                        "  {} = call i64 @_lw_join(i64 {}, i64 {})\n",
                                        r, acc, av
                                    ));
                                    acc = r;
                                }
                                out.push_str(&format!("  call void @_lw_print(i64 {})\n", acc));
                            }
                            let r = format!("%r{}", reg);
                            reg += 1;
                            out.push_str(&format!("  {} = call i64 @_lw_void()\n", r));
                            stack.push(r);
                        }
                        _ => {
                            let idx = lw_builtin(cn).map(|(i, _)| i);
                            let builtin_name: Option<&str> = match idx {
                                Some(LW_READ) => Some("_lw_read"),
                                Some(LW_ARR_LEN) => Some("_lw_arr_len"),
                                Some(LW_ARR_PUSH) => Some("_lw_arr_push"),
                                Some(LW_TO_TEXT) => Some("_lw_to_text"),
                                Some(LW_TYPEOF) => Some("_lw_typeof"),
                                Some(LW_SUB) => Some("_lw_sub"),
                                Some(LW_CONCAT_LIST) => Some("_lw_concat_list"),
                                Some(LW_ARR_REV) => Some("_lw_arr_rev"),
                                Some(LW_ARR_SORT) => Some("_lw_arr_sort"),
                                Some(LW_MAP_NEW) => Some("_lw_map_new"),
                                Some(LW_MAP_SET) => Some("_lw_map_set"),
                                Some(LW_MAP_GET) => Some("_lw_map_get"),
                                Some(LW_MAP_HAS) => Some("_lw_map_has"),
                                Some(LW_MAP_LEN) => Some("_lw_map_len"),
                                Some(LW_MAP_KEYS) => Some("_lw_map_keys"),
                                Some(LW_ABS) => Some("_lw_abs"),
                                Some(LW_SQRT) => Some("_lw_sqrt"),
                                Some(LW_POW) => Some("_lw_pow"),
                                Some(LW_FLOOR) => Some("_lw_floor"),
                                Some(LW_CEIL) => Some("_lw_ceil"),
                                Some(LW_ROUND) => Some("_lw_round"),
                                _ => None,
                            };
                            if let Some(bn) = builtin_name {
                                let arity = args.len();
                                let mut fargs: Vec<String> = Vec::new();
                                for k in 0..arity {
                                    fargs.push(format!("i64 {}", args[k]));
                                }
                                let r = format!("%r{}", reg);
                                reg += 1;
                                out.push_str(&format!(
                                    "  {} = call i64 @{}({})\n",
                                    r,
                                    bn,
                                    fargs.join(", ")
                                ));
                                stack.push(r);
                                risky = matches!(
                                    idx,
                                    Some(LW_ARR_GET)
                                        | Some(LW_ARR_SET)
                                        | Some(LW_TUP_GET)
                                        | Some(LW_SUB)
                                );
                            } else if program.funcs.contains_key(cn) {
                                // semántica de valores: deep-copy de args
                                let mut fargs: Vec<String> = Vec::new();
                                for a in &args {
                                    let d = format!("%r{}", reg);
                                    reg += 1;
                                    out.push_str(&format!(
                                        "  {} = call i64 @_lw_dcp(i64 {})\n",
                                        d, a
                                    ));
                                    fargs.push(format!("i64 {}", d));
                                }
                                let r = format!("%r{}", reg);
                                reg += 1;
                                out.push_str(&format!(
                                    "  {} = call i64 @lum_{}({})\n",
                                    r,
                                    mangle(cn),
                                    fargs.join(", ")
                                ));
                                stack.push(r);
                                risky = true;
                            } else {
                                record_unsupported_builtin(cn);
                                let r = format!("%r{}", reg);
                                reg += 1;
                                out.push_str(&format!("  {} = call i64 @_lw_void()\n", r));
                                stack.push(r);
                            }
                        }
                    }
                }
                Instr::MakeRef(n) => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    if let Some(cell) = cells.get(n) {
                        let a = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = ptrtoint [{} x i8]* {} to i64\n",
                            a, LW_VAL_SIZE, cell
                        ));
                        out.push_str(&format!("  {} = call i64 @_lw_mkref(i64 {})\n", r, a));
                        stack.push(r);
                    } else {
                        out.push_str(&format!("  {} = call i64 @_lw_void()\n", r));
                        stack.push(r);
                    }
                }
                Instr::ArrayPushVar(n) => {
                    let x = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    if let Some(slot) = find_slot(&scopes, n) {
                        let t = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = load i64, i64* {}\n", t, slot));
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_arr_push_ip(i64 {}, i64 {})\n",
                            r, t, x
                        ));
                        // v3.5.42 (bug fuzz gen_ref): write-through si el slot
                        // es T_PTR (prestado mut) — antes: store directo que
                        // sobreescribía la referencia y perdía la mutación.
                        let a = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = ptrtoint i64* {} to i64\n", a, slot));
                        out.push_str(&format!(
                            "  call void @_lw_store_slot(i64 {}, i64 {})\n",
                            a, r
                        ));
                    } else if let Some(cell) = cells.get(n) {
                        let a = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = ptrtoint [{} x i8]* {} to i64\n",
                            a, LW_VAL_SIZE, cell
                        ));
                        let t = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = call i64 @_lw_load_slot(i64 {})\n", t, a));
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_arr_push_ip(i64 {}, i64 {})\n",
                            r, t, x
                        ));
                        out.push_str(&format!(
                            "  call void @_lw_store_slot(i64 {}, i64 {})\n",
                            a, r
                        ));
                    } else if llvm_global_names.contains(n) {
                        let a = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = ptrtoint [{} x i8]* @lw_glob_{} to i64\n",
                            a,
                            LW_VAL_SIZE,
                            mangle(n)
                        ));
                        let t = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = call i64 @_lw_load_slot(i64 {})\n", t, a));
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_arr_push_ip(i64 {}, i64 {})\n",
                            r, t, x
                        ));
                        out.push_str(&format!(
                            "  call void @_lw_store_slot(i64 {}, i64 {})\n",
                            a, r
                        ));
                    } else {
                        out.push_str(&format!("  {} = call i64 @_lw_void()\n", r));
                    }
                    stack.push(r);
                }
                Instr::ArrayNew(n) => {
                    let mut items: Vec<String> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        items.push(stack.pop().unwrap_or_else(|| void0.clone()));
                    }
                    items.reverse();
                    let mut h = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_arr_new()\n", h));
                    for it in items {
                        let r = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_arr_push(i64 {}, i64 {})\n",
                            r, h, it
                        ));
                        h = r;
                    }
                    stack.push(h);
                }
                Instr::ArrayPush => {
                    let x = stack.pop().unwrap_or_else(|| void0.clone());
                    let a = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_arr_push(i64 {}, i64 {})\n",
                        r, a, x
                    ));
                    stack.push(r);
                }
                Instr::ArrayGet => {
                    let ix = stack.pop().unwrap_or_else(|| void0.clone());
                    let a = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_arr_get(i64 {}, i64 {})\n",
                        r, a, ix
                    ));
                    stack.push(r);
                    risky = true;
                }
                Instr::ArraySet => {
                    let x = stack.pop().unwrap_or_else(|| void0.clone());
                    let ix = stack.pop().unwrap_or_else(|| void0.clone());
                    let a = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_arr_set(i64 {}, i64 {}, i64 {})\n",
                        r, a, ix, x
                    ));
                    stack.push(r);
                    risky = true;
                }
                Instr::ArrayLen => {
                    let a = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_arr_len(i64 {})\n", r, a));
                    stack.push(r);
                }
                Instr::StructNew(_, n) => {
                    let mut names: Vec<String> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        names.push(stack.pop().unwrap_or_else(|| void0.clone()));
                    }
                    names.reverse();
                    let mut vals: Vec<String> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        vals.push(stack.pop().unwrap_or_else(|| void0.clone()));
                    }
                    vals.reverse();
                    let mut h = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_st_new()\n", h));
                    for i in 0..*n {
                        let r = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_st_add(i64 {}, i64 {}, i64 {})\n",
                            r, h, names[i], vals[i]
                        ));
                        h = r;
                    }
                    stack.push(h);
                }
                Instr::StructGet => {
                    let name_h = stack.pop().unwrap_or_else(|| void0.clone());
                    let obj = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_st_get(i64 {}, i64 {})\n",
                        r, obj, name_h
                    ));
                    stack.push(r);
                }
                Instr::StructSet => {
                    let x = stack.pop().unwrap_or_else(|| void0.clone());
                    let name_h = stack.pop().unwrap_or_else(|| void0.clone());
                    let obj = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_st_set(i64 {}, i64 {}, i64 {})\n",
                        r, obj, name_h, x
                    ));
                    stack.push(r);
                }
                Instr::TupleNew(n) => {
                    let mut vals: Vec<String> = Vec::with_capacity(*n);
                    for _ in 0..*n {
                        vals.push(stack.pop().unwrap_or_else(|| void0.clone()));
                    }
                    vals.reverse();
                    let mut h = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_tup_new()\n", h));
                    for v in vals {
                        let r = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_tup_push(i64 {}, i64 {})\n",
                            r, h, v
                        ));
                        h = r;
                    }
                    stack.push(h);
                }
                Instr::TupleAccess(i) => {
                    let t = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_tup_get(i64 {}, i64 {})\n",
                        r, t, i
                    ));
                    stack.push(r);
                    risky = true;
                }
                Instr::OptionSome => {
                    let v = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_some(i64 {})\n", r, v));
                    stack.push(r);
                }
                Instr::OptionNone => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_none()\n", r));
                    stack.push(r);
                }
                Instr::ResultOk => {
                    let v = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_ok(i64 {})\n", r, v));
                    stack.push(r);
                }
                Instr::ResultErr => {
                    let v = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_err(i64 {})\n", r, v));
                    stack.push(r);
                }
                Instr::PushHandler(l) => {
                    out.push_str("  call void @_lw_try_begin()\n");
                    handlers.push(*l);
                }
                Instr::PopHandler => {
                    out.push_str("  call void @_lw_try_end()\n");
                    handlers.pop();
                }
                Instr::TryUnwrap => {
                    let h = stack.pop().unwrap_or_else(|| void0.clone());
                    let t = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_kind(i64 {})\n", t, h));
                    let ie = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = icmp eq i64 {}, 8\n", ie, t));
                    let n_err = reg;
                    reg += 1;
                    let n_cont = reg;
                    reg += 1;
                    out.push_str(&format!(
                        "  br i1 {}, label %tu_err_{}, label %tu_cont_{}\n",
                        ie, n_err, n_cont
                    ));
                    out.push_str(&format!("tu_err_{}:\n", n_err));
                    out.push_str(&format!("  ret i64 {}\n", h));
                    out.push_str(&format!("tu_cont_{}:\n", n_cont));
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_payload(i64 {})\n", r, h));
                    stack.push(r);
                }
                Instr::MatchType(k) => {
                    let h = stack.pop().unwrap_or_else(|| void0.clone());
                    let t = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_kind(i64 {})\n", t, h));
                    let tag = match *k {
                        0 => 9,
                        1 => 7,
                        2 => 8,
                        _ => -1,
                    };
                    let c = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = icmp eq i64 {}, {}\n", c, t, tag));
                    let s = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = zext i1 {} to i64\n", s, c));
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_bool(i64 {})\n", r, s));
                    stack.push(r);
                }
                Instr::MatchPayload => {
                    let h = stack.pop().unwrap_or_else(|| void0.clone());
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_payload(i64 {})\n", r, h));
                    stack.push(r);
                }
                Instr::MatchVariant(vname) => {
                    let h = stack.pop().unwrap_or_else(|| void0.clone());
                    let p = str_ptr(&mut out, &mut reg, &strings, vname);
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_enm_variant_is(i64 {}, i64 {})\n",
                        r, h, p
                    ));
                    stack.push(r);
                }
                Instr::EnumCtor {
                    enum_name,
                    variant,
                    argc,
                } => {
                    let mut vals: Vec<String> = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        vals.push(stack.pop().unwrap_or_else(|| void0.clone()));
                    }
                    vals.reverse();
                    let mut arr = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_arr_new()\n", arr));
                    for v in vals {
                        let r = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_arr_push(i64 {}, i64 {})\n",
                            r, arr, v
                        ));
                        arr = r;
                    }
                    let en = str_ptr(&mut out, &mut reg, &strings, enum_name);
                    let vr = str_ptr(&mut out, &mut reg, &strings, variant);
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_enm_new(i64 {}, i64 {}, i64 {})\n",
                        r, arr, en, vr
                    ));
                    stack.push(r);
                }
                Instr::FuncRef(fn_name) => {
                    let r = format!("%r{}", reg);
                    reg += 1;
                    if program.funcs.contains_key(fn_name) {
                        let nparams = program.funcs[fn_name].params.len();
                        let ptypes = vec!["i64"; nparams].join(", ");
                        let a = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!(
                            "  {} = ptrtoint i64 ({})* @lum_{} to i64\n",
                            a,
                            ptypes,
                            mangle(fn_name)
                        ));
                        let nm = str_ptr(&mut out, &mut reg, &strings, fn_name);
                        out.push_str(&format!(
                            "  {} = call i64 @_lw_fref(i64 {}, i64 {})\n",
                            r, a, nm
                        ));
                        stack.push(r);
                    } else {
                        record_unsupported_builtin(fn_name);
                        out.push_str(&format!("  {} = call i64 @_lw_void()\n", r));
                        stack.push(r);
                    }
                }
                Instr::CallValue(argc) => {
                    let mut args: Vec<String> = Vec::with_capacity(*argc);
                    for _ in 0..*argc {
                        args.push(stack.pop().unwrap_or_else(|| void0.clone()));
                    }
                    args.reverse();
                    let fref = stack.pop().unwrap_or_else(|| void0.clone());
                    let a = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 @_lw_fref_addr(i64 {})\n",
                        a, fref
                    ));
                    let fp = format!("%r{}", reg);
                    reg += 1;
                    let ptypes = vec!["i64"; *argc].join(", ");
                    out.push_str(&format!(
                        "  {} = inttoptr i64 {} to i64 ({})*\n",
                        fp,
                        a,
                        if *argc == 0 { "".to_string() } else { ptypes }
                    ));
                    let mut fargs: Vec<String> = Vec::new();
                    for av in &args {
                        let d = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = call i64 @_lw_dcp(i64 {})\n", d, av));
                        fargs.push(format!("i64 {}", d));
                    }
                    let r = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!(
                        "  {} = call i64 {}({})\n",
                        r,
                        fp,
                        fargs.join(", ")
                    ));
                    stack.push(r);
                    risky = true;
                }
                Instr::ScopePush => {
                    scopes.push(HashMap::new());
                }
                Instr::ScopePop => {
                    if scopes.len() > 1 {
                        scopes.pop();
                    }
                }
                Instr::Return => {
                    let v = stack.pop().unwrap_or_else(|| void0.clone());
                    out.push_str(&format!("  ret i64 {}\n", v));
                    terminated = true;
                    stack.clear();
                }
                Instr::Halt => {
                    let v = stack.pop().unwrap_or_else(|| void0.clone());
                    out.push_str(&format!("  ret i64 {}\n", v));
                    terminated = true;
                    stack.clear();
                }
                Instr::Jmp(t) => {
                    let _mg = store_merge(&stack, &void0, *t);
                    out.push_str(&_mg);
                    out.push_str(&format!("  br label %L_{}\n", t));
                    terminated = true;
                }
                Instr::JmpIf(t) => {
                    if let Some(v) = stack.pop() {
                        let tv = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = call i64 @_lw_truthy_i(i64 {})\n", tv, v));
                        let z = format!("%r{}", reg);
                        reg += 1;
                        out.push_str(&format!("  {} = icmp eq i64 {}, 0\n", z, tv));
                        let fall = reg;
                        reg += 1;
                        let _mg = store_merge(&stack, &void0, *t);
                        out.push_str(&_mg);
                        out.push_str(&format!(
                            "  br i1 {}, label %L_{}, label %jf_{}\n",
                            z, t, fall
                        ));
                        out.push_str(&format!("jf_{}:\n", fall));
                        terminated = false;
                        // el fallthrough continúa con el stack intacto
                    }
                }
                Instr::Phi(..) | Instr::Nop => {}
                _ => {}
            }
            // Chequeo de error tras operación riesgosa (paridad Cranelift/C)
            if risky && !terminated {
                let fl = format!("%r{}", reg);
                reg += 1;
                out.push_str(&format!("  {} = call i64 @_lw_err_active()\n", fl));
                let er = format!("%r{}", reg);
                reg += 1;
                out.push_str(&format!("  {} = icmp ne i64 {}, 0\n", er, fl));
                let n_d = reg;
                reg += 1;
                let n_o = reg;
                reg += 1;
                out.push_str(&format!(
                    "  br i1 {}, label %ec_d_{}, label %ec_o_{}\n",
                    er, n_d, n_o
                ));
                out.push_str(&format!("ec_d_{}:\n", n_d));
                if let Some(&cl) = handlers.last() {
                    let m = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_err_take()\n", m));
                    out.push_str(&format!("  store i64 {}, i64* %mg_{}_0\n", m, cl));
                    out.push_str(&format!("  br label %L_{}\n", cl));
                } else {
                    let vh = format!("%r{}", reg);
                    reg += 1;
                    out.push_str(&format!("  {} = call i64 @_lw_void()\n", vh));
                    out.push_str(&format!("  ret i64 {}\n", vh));
                }
                out.push_str(&format!("ec_o_{}:\n", n_o));
            }
            let _ = open_block;
            open_block = true;
        }
        if !terminated {
            out.push_str(&format!("  ret i64 {}\n", void0));
        }
        out.push_str("}\n\n");
    }

    // Entry point C: main() → entrada del programa (paridad backend Cranelift)
    let entry = if program.funcs.contains_key(&program.entry) {
        program.entry.clone()
    } else if program.funcs.contains_key("main") {
        "main".to_string()
    } else if program.funcs.contains_key("principal") {
        "principal".to_string()
    } else {
        String::new()
    };
    out.push_str("define i32 @main() {\nentry:\n");
    if !entry.is_empty() {
        out.push_str(&format!("  %res = call i64 @lum_{}()\n", mangle(&entry)));
    }
    out.push_str("  ret i32 0\n}\n");

    out
}

pub fn compile_to_object(program: &Program, output: &str) -> Result<(), String> {
    let program = lower_arraysetvar(program);
    let compiler = AotCompiler::new();
    let product = compiler.compile(&program);
    let obj = &product.object;
    let bytes = obj.write().map_err(|e| format!("Write error: {}", e))?;
    std::fs::write(output, &bytes).map_err(|e| format!("IO: {}", e))?;
    Ok(())
}

const C_RUNTIME: &str = include_str!("lumen_rt.h");

/// Runtime `_lw_*` (v3.5.6): capa de HELPERS con HANDLES OPACOS para el
/// backend Cranelift. El código nativo solo ve `i64` (punteros a `Val`);
/// toda la semántica (formato, aritmética mixta entero/decimal, concat de
/// texto, listas, structs, tuplas, mapas, errores) vive aquí y delega en
/// `lumen_rt.h` — paridad con el backend C y la VM sin reimplementar nada.
/// Se escribe en el archivo de link junto a C_RUNTIME (ver lw_shim_source).
const LW_RUNTIME: &str = r#"
/* ══ LÚMEN v3.5.6 — helpers _lw_* (handles opacos) para backend Cranelift ══
   Handle = Val* reservado con malloc. El código Cranelift pasa/recibe i64
   opacos; la semántica completa delega en lumen_rt.h (paridad VM/C). */

/* v3.5.20: ARENA + GC CONSERVADOR (mark-sweep) para las cajas Val.
   - Arena bump TLS: asignación ~10× más barata que malloc.
   - Cuando la asignación acumulada supera LW_GC_THRESHOLD, mark-sweep:
     las RAÍCES son el stack nativo (incluye el de Cranelift: los handles
     i64 viven en slots/spills) + los registros (vía setjmp). Los boxes
     alcanzables se marcan; el resto pasa a una freelist que se reutiliza.
   - Soundness: los boxes NUNCA apuntan a otros boxes (sus campos son
     buffers malloc / strings / punteros a slots), así que el marcado es de
     un solo nivel. En un punto de llamada, los valores vivos del llamador
     están en registros callee-saved (capturados por setjmp) o spilleados al
     stack (escaneados) → no se libera nada vivo.
   - Cada hilo barre SOLO su arena TLS; los valores que cruzan hilos
     (join/canales) viajan como Val por valor, nunca como handle. */
#include <setjmp.h>
#define LW_ARENA_BLOCK (1 << 22)
#define LW_GC_THRESHOLD (8 << 20)
typedef struct LwBlock { char* base; size_t cap; unsigned char* marks; struct LwBlock* next; } LwBlock;
static LW_TLS LwBlock* lw_tls_blocks;
static LW_TLS char* _lw_arena_cur;
static LW_TLS size_t _lw_arena_left;
static LW_TLS Val* lw_tls_free;
static LW_TLS size_t lw_tls_since_gc;
static LW_TLS void* lw_gc_stack_top;
/* v3.5.30: margen sobre el tope capturado en _lw_gc_init — cubre el frame
   del wrapper de entrada (cuyos slots quedan por encima del marcador) y
   holgura de seguridad. Antes el GC escaneaba hasta el fin del mapping del
   stack (8MB) aunque el programa estuviera en un frame somero. */
#define LW_GC_TOP_MARGIN (1 << 20)

void _lw_gc_init(void) { volatile int marker; lw_gc_stack_top = (void*)&marker; }

static unsigned char* lw_gc_mark_of(Val* p) {
  for (LwBlock* b = lw_tls_blocks; b; b = b->next) {
    if ((char*)p >= b->base && (char*)p < b->base + b->cap) {
      return &b->marks[((char*)p - b->base) / sizeof(Val)];
    }
  }
  return NULL;
}
static void lw_gc_scan_range(const char* lo, const char* hi) {
  const char* q;
  for (q = lo; q + 8 <= hi; q += 8) {
    uintptr_t v;
    memcpy(&v, q, 8);
    if ((v & 7) == 0) {
      unsigned char* m = lw_gc_mark_of((Val*)(uintptr_t)v);
      if (m) *m = 1;
    }
  }
}
/* Tope real del stack del hilo actual (fin del mapping): Windows vía TEB,
   Linux vía /proc/self/maps. Evita que el scan lea memoria no mapeada. */
static char* lw_gc_stack_hi(void) {
#ifdef _WIN32
  return (char*)((NT_TIB*)NtCurrentTeb())->StackBase;
#else
  /* TLS: cada hilo tiene su propio mapping de stack. */
  static LW_TLS char* cached_hi = NULL;
  if (cached_hi) return cached_hi;
  {
    FILE* f = fopen("/proc/self/maps", "r");
    if (f) {
      char line[512];
      unsigned long long lo_m = 0, hi_m = 0;
      char perms[8];
      uintptr_t probe = (uintptr_t)(void*)&f;
      while (fgets(line, sizeof line, f)) {
        perms[0] = 0;
        if (sscanf(line, "%llx-%llx %7s", &lo_m, &hi_m, perms) >= 2) {
          if (probe >= (uintptr_t)lo_m && probe < (uintptr_t)hi_m) {
            cached_hi = (char*)(uintptr_t)hi_m;
            break;
          }
        }
      }
      fclose(f);
    }
  }
  return cached_hi;
#endif
}

static void lw_gc_collect(void) {
  jmp_buf jb;
  volatile int canary = 0;
  setjmp(jb);
  (void)canary;
  if (getenv("LUMEN_GC_LOG")) fprintf(stderr, "[gc] collect start blocks=%p\n", (void*)lw_tls_blocks);
  if (getenv("LUMEN_GC_LOG")) fprintf(stderr, "[gc]   memset marks\n");
  for (LwBlock* b = lw_tls_blocks; b; b = b->next)
    memset(b->marks, 0, b->cap / sizeof(Val));
  {
    /* Rango: desde el frame actual hasta el tope capturado en _lw_gc_init
       (frame de arranque ≈ tope real del stack) + margen. Los falsos
       positivos solo retienen cajas (fuga menor), nunca liberan nada vivo.
       v3.5.30: con tope capturado, escanear hasta él (+margen) en vez de
       hasta el fin del mapping del stack (8MB) — el espacio por encima del
       frame de arranque son frames de libc muertos. */
    int here;
    char* lo = (char*)((((uintptr_t)&here) + 7) & ~(uintptr_t)7);
    /* v3.5.30: el tope capturado (+margen) acota el escaneo a los frames
       vivos; el fin del mapping sigue como límite superior para no pisar
       la frontera usuario/kernel (el stack vive pegado al tope). */
    char* hi = lw_gc_stack_hi();
    if (lw_gc_stack_top) {
      char* h2 = (char*)lw_gc_stack_top + LW_GC_TOP_MARGIN;
      if (h2 > (char*)lw_gc_stack_top && (!hi || h2 < hi)) hi = h2;
    }
    if (!hi || hi <= lo) hi = lo + (16 << 10);
    if (getenv("LUMEN_GC_LOG")) fprintf(stderr, "[gc]   scan %p..%p\n", (void*)lo, (void*)hi);
    lw_gc_scan_range(lo, hi);
    if (getenv("LUMEN_GC_LOG")) fprintf(stderr, "[gc]   scan jb\n");
  }
  {
    const char* jb_lo = (const char*)((((uintptr_t)&jb) + 7) & ~(uintptr_t)7);
    lw_gc_scan_range(jb_lo, (const char*)&jb + sizeof(jb));
  }
  if (getenv("LUMEN_GC_LOG")) fprintf(stderr, "[gc]   sweep\n");
  lw_tls_free = NULL;
  size_t freed = 0, total = 0;
  for (LwBlock* b = lw_tls_blocks; b; b = b->next) {
    size_t n = b->cap / sizeof(Val), i;
    for (i = 0; i < n; i++) {
      total++;
      if (!b->marks[i]) {
        Val* p = (Val*)(b->base + i * sizeof(Val));
        *(Val**)p = lw_tls_free;
        lw_tls_free = p;
        freed++;
      }
    }
  }
  if (getenv("LUMEN_GC_LOG")) fprintf(stderr, "[gc] collect done freed=%zu/%zu\n", freed, total);
}
static Val* _lw_box(Val v) {
  if (_lw_arena_left < sizeof(Val)) {
    char* nb = (char*)malloc(LW_ARENA_BLOCK);
    if (!nb) return NULL;
    LwBlock* blk = (LwBlock*)malloc(sizeof(LwBlock));
    if (!blk) { free(nb); return NULL; }
    blk->base = nb;
    blk->cap = LW_ARENA_BLOCK;
    blk->marks = (unsigned char*)calloc(LW_ARENA_BLOCK / sizeof(Val), 1);
    blk->next = lw_tls_blocks;
    lw_tls_blocks = blk;
    _lw_arena_cur = nb;
    _lw_arena_left = LW_ARENA_BLOCK;
  }
  if (lw_tls_since_gc > LW_GC_THRESHOLD) {
    lw_gc_collect();
    lw_tls_since_gc = 0;
  }
  Val* p;
  if (lw_tls_free) {
    p = lw_tls_free;
    lw_tls_free = *(Val**)p;
  } else {
    p = (Val*)_lw_arena_cur;
    _lw_arena_cur += sizeof(Val);
    _lw_arena_left -= sizeof(Val);
  }
  lw_tls_since_gc += sizeof(Val);
  *p = v;
  return p;
}
static Val _lw_unbox(int64_t h) { return h ? *(Val*)(intptr_t)h : _v_void(); }
static Val _lw_u(int64_t h) { return _deref(_lw_unbox(h)); }
static int64_t _lw_h(Val v) { return (int64_t)(intptr_t)_lw_box(v); }
static int64_t _lw_str_take(char* s) { Val v = _v_int(0); v.t = T_STR; v.s = s; return _lw_h(v); }

int64_t _lw_int(int64_t x) { return _lw_h(_v_int(x)); }
int64_t _lw_flt(double x) { return _lw_h(_v_flt(x)); }
int64_t _lw_bool(int64_t x) { return _lw_h(_v_bool((int)x)); }
/* v3.5.30: los literales llegan del .data del objeto (inmortales) — adoptar
   el puntero sin copiar (antes: strlen+arena por literal en cada iteración). */
int64_t _lw_str(const char* s) { return _lw_h(_v_str_lit(s ? s : "")); }
int64_t _lw_void(void) { return _lw_h(_v_void()); }
int64_t _lw_none(void) { return _lw_h(_none()); }

/* imprimir: formatea el handle y añade newline (paridad VM) */
void _lw_print(int64_t h) { char* s = _fmt(_lw_u(h)); printf("%s\n", s); free(s); }
/* imprimir() sin argumentos: línea vacía (paridad VM/backend C) */
void _lw_print_blank(void) { printf("\n"); }

/* unir dos valores como texto (imprimir multi-arg / f-strings) */
int64_t _lw_join(int64_t a, int64_t b) {
  char* xs = _fmt(_lw_u(a)); char* ys = _fmt(_lw_u(b));
  size_t l1 = strlen(xs), l2 = strlen(ys);
  char* m = (char*)malloc(l1 + l2 + 1);
  memcpy(m, xs, l1); memcpy(m + l1, ys, l2 + 1);
  free(xs); free(ys);
  return _lw_str_take(m);
}

/* binarios: mismos códigos que el backend C (1=Add..19=Xor). op 2 = Concat
   (fmt+fmt; _bin no lo cubre). División por cero lanza vía _rt_throw. */
int64_t _lw_bin(int64_t op, int64_t a, int64_t b) {
  Val x = _lw_u(a), y = _lw_u(b);
  if (op == 2) { return _lw_join(a, b); }
  return _lw_h(_bin((int)op, x, y));
}

/* unarios: 0=neg 1=not 2=bitnot */
int64_t _lw_un(int64_t op, int64_t a) {
  Val x = _lw_u(a);
  if (op == 0) return _lw_h(_neg(x));
  if (op == 1) return _lw_h(_not(x));
  return _lw_h(_bnot(x));
}

/* verdad cruda (0/1) para branches — sin boxear */
int64_t _lw_truthy_i(int64_t h) { return _truthy(_lw_u(h)); }

int64_t _lw_arr_new(void) { return _lw_h(_arrn(NULL, 0)); }
int64_t _lw_arr_push(int64_t a, int64_t x) { return _lw_h(_arr_push(_lw_u(a), _lw_unbox(x))); }
/* el índice llega como HANDLE (valor apilado) → desreferenciar a entero */
int64_t _lw_arr_get(int64_t a, int64_t i) { return _lw_h(_arr_get(_lw_u(a), (int64_t)_asf(_lw_u(i)))); }
int64_t _lw_arr_set(int64_t a, int64_t i, int64_t x) { return _lw_h(_arr_set(_lw_u(a), (int64_t)_asf(_lw_u(i)), _lw_unbox(x))); }
int64_t _lw_arr_len(int64_t h) { return _lw_h(_arr_len(_lw_u(h))); }
/* v3.5.30: largo CRUDO (sin box del resultado) para el backend Cranelift —
   `total = total + largo(s)` pasa a ser un iadd nativo. Semántica idéntica
   a _lw_arr_len (misma _arr_len / strlen). */
int64_t _lw_arr_len_i(int64_t h) {
  Val v = _lw_u(h);
  if (v.t == T_STR) return (int64_t)strlen(v.s ? v.s : "");
  return _arr_len(v).i;
}
int64_t _lw_arr_rev(int64_t h) { return _lw_h(_arr_rev(_lw_u(h))); }
int64_t _lw_arr_sort(int64_t h) { return _lw_h(_arr_sort(_lw_u(h))); }

int64_t _lw_st_new(void) {
  Val v = _v_int(0); v.t = T_STT; v.en = ""; v.argc = 0;
  v.items = (Val*)malloc(sizeof(Val) * 2);
  return _lw_h(v);
}
int64_t _lw_st_add(int64_t h, int64_t name_h, int64_t v) {
  Val s = _lw_unbox(h); Val nm = _lw_unbox(name_h);
  int n = s.argc;
  Val* ni = (Val*)malloc(sizeof(Val) * ((size_t)(n + 1) * 2));
  for (int i = 0; i < n * 2; i++) ni[i] = s.items[i];
  ni[2 * n] = nm; ni[2 * n + 1] = _lw_unbox(v);
  s.argc = n + 1; s.items = ni;
  return _lw_h(s);
}
int64_t _lw_st_get(int64_t h, int64_t name_h) {
  Val nm = _lw_unbox(name_h);
  return _lw_h(_st_get(_lw_u(h), nm.s ? nm.s : ""));
}
int64_t _lw_st_set(int64_t h, int64_t name_h, int64_t v) {
  Val nm = _lw_unbox(name_h);
  return _lw_h(_st_set(_lw_u(h), nm.s ? nm.s : "", _lw_unbox(v)));
}

int64_t _lw_tup_new(void) { Val v = _arrn(NULL, 0); v.t = T_TUP; return _lw_h(v); }
int64_t _lw_tup_push(int64_t h, int64_t x) {
  Val t = _lw_unbox(h); t = _arr_push(t, _lw_unbox(x)); t.t = T_TUP;
  return _lw_h(t);
}
int64_t _lw_tup_get(int64_t h, int64_t i) {
  Val t = _lw_u(h);
  if (i < 0 || i >= t.argc) {
    char eb[96];
    snprintf(eb, sizeof eb, "Indice %lld fuera de rango (largo: %d)", (long long)i, t.argc);
    _rt_throw(eb);
  }
  return _lw_h(t.items[i]);
}

int64_t _lw_read(void) { return _lw_h(_read_ln()); }
int64_t _lw_typeof(int64_t h) { return _lw_h(_v_str(_tipo_de_b(_lw_u(h)))); }
int64_t _lw_to_text(int64_t h) { return _lw_h(_to_text(_lw_u(h))); }
/* v3.5.30: a_texto(entero crudo) — itoa directo al arena (sin box del arg,
   sin malloc, sin unbox del handle). Paridad de dígitos con _fmt(T_INT). */
int64_t _lw_to_text_i(int64_t x) {
  char* b = _sa_alloc(32);
  if (!b) { b = (char*)malloc(32); if (!b) return _lw_h(_v_str("")); }
  _itoa_ll(x, b);
  Val v = _v_int(0);
  v.t = T_STR;
  v.s = b;
  return _lw_h(v);
}
/* v3.5.30: fusión `"lit" + X + "lit"` (el patrón de interpolación): una
   arena-alloc + dos memcpy + un box por concatenación triple. */
/* v3.5.30: longitud de "lit" + a_texto(x) + "lit" SIN construir el string
   (patrón `total = total + largo("item-" + a_texto(i) + "-fin")`): solo se
   cuentan dígitos. El Add fusionado de Cranelift lo usa y omite el
   StoreLocal/Load/Call(largo) enteros. */
int64_t _lw_concat3_len_i(const char* p1, int64_t x, const char* p2) {
  int n = 1;
  unsigned long long u = (unsigned long long)x;
  if (x < 0) {
    n = 2;
    u = 0ULL - (unsigned long long)x;
  }
  while (u >= 10) { u /= 10; n++; }
  return (int64_t)(strlen(p1) + (size_t)n + strlen(p2));
}

int64_t _lw_concat3(const char* p1, int64_t mid_h, const char* p2) {
  Val m = _lw_u(mid_h);
  char* ms;
  if (m.t == T_STR) {
    ms = (char*)(m.s ? m.s : "");
  } else {
    ms = _fmt(m);
  }
  size_t l1 = strlen(p1), lm = strlen(ms), l2 = strlen(p2);
  char* out = _sa_alloc(l1 + lm + l2 + 1);
  if (!out) out = (char*)malloc(l1 + lm + l2 + 1);
  memcpy(out, p1, l1);
  memcpy(out + l1, ms, lm);
  memcpy(out + l1 + lm, p2, l2 + 1);
  return _lw_h(_v_str_take(out));
}
/* v3.5.30: "lit" + a_texto(entero crudo) + "lit" — itoa directo al buffer
   final (sin box intermedio ni strlen del medio). */
int64_t _lw_concat3_i(const char* p1, int64_t x, const char* p2) {
  char tmp[24];
  int n = _itoa_ll(x, tmp);
  size_t l1 = strlen(p1), l2 = strlen(p2);
  char* out = _sa_alloc(l1 + (size_t)n + l2 + 1);
  if (!out) out = (char*)malloc(l1 + (size_t)n + l2 + 1);
  memcpy(out, p1, l1);
  memcpy(out + l1, tmp, (size_t)n);
  memcpy(out + l1 + (size_t)n, p2, l2 + 1);
  return _lw_h(_v_str_take(out));
}
int64_t _lw_sub(int64_t h, int64_t a, int64_t b) {
  Val s = _lw_u(h);
  /* a/b llegan como handles → desreferenciar a enteros */
  return _lw_str_take(_sub(s.s ? s.s : "", (int64_t)_asf(_lw_u(a)), (int64_t)_asf(_lw_u(b))));
}
int64_t _lw_concat_list(int64_t h) { return _lw_h(_concat_list(_lw_u(h))); }

int64_t _lw_some(int64_t h) { return _lw_h(_some(_lw_unbox(h))); }
int64_t _lw_ok(int64_t h) { return _lw_h(_res(_lw_unbox(h), 1)); }
int64_t _lw_err(int64_t h) { return _lw_h(_res(_lw_unbox(h), 0)); }

int64_t _lw_map_new(void) { return _lw_h(_map_new()); }
int64_t _lw_map_set(int64_t m, int64_t k, int64_t x) { return _lw_h(_map_set(_lw_u(m), _lw_unbox(k), _lw_unbox(x))); }
int64_t _lw_map_get(int64_t m, int64_t k) { return _lw_h(_map_get(_lw_u(m), _lw_unbox(k))); }
int64_t _lw_map_has(int64_t m, int64_t k) { return _lw_h(_map_has(_lw_u(m), _lw_unbox(k))); }
int64_t _lw_map_len(int64_t m) { return _lw_h(_map_len(_lw_u(m))); }
int64_t _lw_map_keys(int64_t m) { return _lw_h(_map_keys(_lw_u(m))); }

/* ══ Incremento B (v3.5.7) ══════════════════════════════════════════════ */

/* ── intentar/atrapar: el emisor chequea _lw_err_active tras cada operación
      riesgosa y salta al catch abierto (paridad con _ERRCHK del backend C) ── */
void _lw_try_begin(void) { if (_hn < 64) { _h_sp[_hn] = 0; _hn++; } }
void _lw_try_end(void) { if (_hn > 0) _hn--; }
int64_t _lw_err_active(void) { return _err; }
/* Entrada al catch: quita el manejador, limpia el flag, devuelve el mensaje
   como handle texto (la VM lo pushea en el catch; aqui lo bindea Store). */
int64_t _lw_err_take(void) {
  if (_hn > 0) _hn--;
  _err = 0;
  if (_last_err_msg) return _lw_str_take(_last_err_msg);
  char* e = (char*)malloc(1); e[0] = 0;
  return _lw_str_take(e);
}

/* ── enums y matching (elegir) ── */
int64_t _lw_kind(int64_t h) { return (int64_t)_lw_u(h).t; }
/* payload (paridad VM Opcode::MatchPayload): algun/exito/error → interior;
   enum → void / campo único / lista de campos; resto pasa igual */
int64_t _lw_payload(int64_t h) {
  Val u = _lw_u(h);
  if ((u.t == T_SOM || u.t == T_OK || u.t == T_ERR) && u.argc > 0)
    return _lw_h(u.items[0]);
  if (u.t == T_ENM) {
    if (u.argc == 0) return _lw_h(_v_void());
    if (u.argc == 1) return _lw_h(u.items[0]);
    Val* pc = (Val*)malloc(sizeof(Val) * u.argc);
    for (int i = 0; i < u.argc; i++) pc[i] = u.items[i];
    return _lw_h(_arrn(pc, u.argc));
  }
  return _lw_h(u);
}
/* construye T_ENM: args llega como handle de lista (se copia, paridad _enm) */
int64_t _lw_enm_new(int64_t args_h, int64_t en_ptr, int64_t vr_ptr) {
  Val a = _lw_u(args_h);
  Val* xs = (Val*)malloc(sizeof(Val) * (a.argc > 0 ? a.argc : 1));
  for (int i = 0; i < a.argc; i++) xs[i] = a.items[i];
  Val v = _v_int(0); v.t = T_ENM;
  v.en = (const char*)(intptr_t)en_ptr;
  v.vr = (const char*)(intptr_t)vr_ptr;
  v.argc = a.argc; v.items = xs;
  return _lw_h(v);
}
int64_t _lw_enm_variant_is(int64_t h, int64_t vr_ptr) {
  Val v = _lw_u(h);
  int ok = v.t == T_ENM && v.vr && !strcmp(v.vr, (const char*)(intptr_t)vr_ptr);
  return _lw_h(_v_bool(ok));
}

/* ── funciones como valores (FuncRef/CallValue) ── */
int64_t _lw_fref(int64_t addr, int64_t name_ptr) {
  Val v = _v_int(0); v.t = T_FRE;
  v.s = (const char*)(intptr_t)name_ptr;
  union { Val (*fp)(void); int64_t i; } u; u.i = addr; v.fp = u.fp;
  return _lw_h(v);
}
int64_t _lw_fref_addr(int64_t h) {
  Val v = _lw_u(h);
  if (v.t != T_FRE) return 0;
  union { Val (*fp)(void); int64_t i; } u; u.fp = v.fp;
  return u.i;
}
/* ── referencias (prestado mut): celdas Val estables en el stack frame ── */
int64_t _lw_mkref(int64_t addr) {
  Val v = _v_int(0); v.t = T_PTR; v.p = (Val*)(intptr_t)addr;
  return _lw_h(v);
}
/* lee la celda (sigue T_PTR si la celda contiene una referencia) */
int64_t _lw_load_slot(int64_t addr) {
  Val cell = *(Val*)(intptr_t)addr;
  return _lw_h(_deref(cell));
}
/* escribe la celda: si contiene una referencia, write-through al objetivo
   (semantica prestado mut — paridad con Store del backend C sobre gv[]).
   Deep-copy del valor (paridad gv[n]=_dcp(v)): garantiza que dos variables
   nunca compartan buffer y habilita el push in-place amortizado. */
void _lw_store_slot(int64_t addr, int64_t h) {
  Val* cell = (Val*)(intptr_t)addr;
  Val v = _dcp(_lw_unbox(h));
  if (cell->t == T_PTR && cell->p) *cell->p = v; else *cell = v;
}
/* deep copy de un valor (args de llamada: semántica de valores; T_PTR/T_FRE
   pasan tal cual para no romper prestado mut) — paridad _dcp del backend C */
int64_t _lw_dcp(int64_t h) { return _lw_h(_dcp(_lw_unbox(h))); }
/* push in-place amortizado (ArrayPushVar): la celda es dueña exclusiva del
   buffer gracias al deep-copy en stores/llamadas */
int64_t _lw_arr_push_ip(int64_t a, int64_t x) {
  return _lw_h(_arr_push_ip(_lw_unbox(a), _lw_unbox(x)));
}
/* binding de entrada (params/init): SIEMPRE escribe la celda misma, sin
   write-through — la celda puede traer un T_PTR de la llamada anterior y el
   write-through ahi corromperia la variable del llamador (bug v3.5.7) */
void _lw_store_slot_direct(int64_t addr, int64_t h) {
  *(Val*)(intptr_t)addr = _lw_unbox(h);
}
/* ── Matematicas (paridad VM: builtins tienen prioridad sobre funcs usuario) ── */
int64_t _lw_abs(int64_t h) {
  Val v = _lw_u(h);
  if (v.t == T_INT) { long long x = v.i; return _lw_h(_v_int(x < 0 ? -x : x)); }
  return _lw_h(_v_flt(fabs(_asf(v))));
}
int64_t _lw_sqrt(int64_t h) { return _lw_h(_v_flt(sqrt(_asf(_lw_u(h))))); }
int64_t _lw_pow(int64_t ha, int64_t hb) {
  Val a = _lw_u(ha), b = _lw_u(hb);
  if (a.t == T_INT && b.t == T_INT && b.i >= 0) {
    long long r = 1, base = a.i; long long e = b.i;
    while (e > 0) { if (e & 1) r *= base; base *= base; e >>= 1; }
    return _lw_h(_v_int(r));
  }
  return _lw_h(_v_flt(pow(_asf(a), _asf(b))));
}
int64_t _lw_floor(int64_t h) { return _lw_h(_v_int((long long)floor(_asf(_lw_u(h))))); }
int64_t _lw_ceil(int64_t h) { return _lw_h(_v_int((long long)ceil(_asf(_lw_u(h))))); }
int64_t _lw_round(int64_t h) { return _lw_h(_v_int((long long)round(_asf(_lw_u(h))))); }

/* ── v3.5.17: hilos reales (Cranelift/LLVM) ──────────────────────────────
   El runtime pthread/Win32 vive en lumen_rt.h (_lw_thr_spawn/_lw_thr_join);
   aquí la variante basada en handles opacos. El hilo hijo entra por el
   trampolín __lumen_ft_<fn> (objeto Cranelift), que pide cada argumento con
   _lw_thr_arg_handle(k) — deep-copy del Val estagiado en el TLS del hilo. */
const char* _lw_cstr(int64_t h) {
  Val v = _lw_u(h);
  return (v.t == T_STR && v.s) ? v.s : "";
}
int64_t _lw_thr_spawn_h(const char* fn, int64_t* hs, int64_t argc) {
  Val args[8];
  int n = argc > 8 ? 8 : (int)argc;
  if (n < 0) n = 0;
  for (int k = 0; k < n; k++) args[k] = _lw_u(hs[k]);
  return _lw_thr_spawn(fn, args, n);
}
int64_t _lw_thr_join_h(int64_t h) { return _lw_h(_lw_thr_join((int64_t)_asf(_lw_u(h)))); }
int64_t _lw_thr_arg_handle(int64_t k) {
  if (k < 0 || k >= lw_thr_argc) return _lw_h(_v_void());
  return _lw_h(_dcp(lw_thr_args[k]));
}

/* v3.5.17: canales y mutexes — wrappers de handles sobre _rt_*_v */
int64_t _lw_chan_new_h(void) { return _lw_h(_rt_chan_new_v()); }
int64_t _lw_chan_send_h(int64_t cid_h, int64_t v_h) {
  return _lw_h(_rt_chan_send_v(_lw_u(cid_h), _lw_u(v_h)));
}
int64_t _lw_chan_recv_h(int64_t cid_h) { return _lw_h(_rt_chan_recv_v(_lw_u(cid_h))); }
int64_t _lw_mutex_new_h(void) { return _lw_h(_rt_mutex_new_v()); }
int64_t _lw_mutex_lock_call_h(int64_t mid_h, int64_t fn_h, int64_t arg_h) {
  return _lw_h(_rt_mutex_lock_call_v(_lw_u(mid_h), _lw_u(fn_h), _lw_u(arg_h)));
}
int64_t _lw_cal_hijri_h(int64_t t) { return _lw_h(_rt_calendario_hijri((int64_t)_asf(_lw_u(t)))); }
int64_t _lw_cal_persa_h(int64_t t) { return _lw_h(_rt_calendario_persa((int64_t)_asf(_lw_u(t)))); }
int64_t _lw_time_now_h(void) { return _lw_h(_time_now()); }
int64_t _lw_time_fmt_h(int64_t t, int64_t f) {
  Val tv = _lw_u(t), fv = _lw_u(f);
  return _lw_h(_v_str(_time_fmt((int64_t)_asf(tv), (fv.t == T_STR && fv.s) ? fv.s : "")));
}
int64_t _lw_time_diff_h(int64_t t1, int64_t t2) {
  int64_t a = (int64_t)_asf(_lw_u(t1)), b = (int64_t)_asf(_lw_u(t2));
  int64_t d = a - b; if (d < 0) d = -d;
  return _lw_h(_v_int(d));
}
int64_t _lw_time_parse_h(int64_t s) {
  Val v = _lw_u(s);
  return _lw_h(_v_int(_time_parse((v.t == T_STR && v.s) ? v.s : "")));
}
/* v3.5.25: extrae el entero de un handle (slots i64 de Cranelift). */
int64_t _lw_h2i(int64_t h) { return (int64_t)_asf(_lw_u(h)); }
/* v3.5.28: throw de división por cero para Div/Mod nativos de Cranelift. */
void _lw_throw_div(void) { _rt_throw("Error: Division por cero"); }
/* v3.5.29: arrays de enteros SIN boxear (Cranelift): el array vive en
   (ptr,len,cap) nativos; push con crecimiento amortizado, get con bounds. */
void _lw_iarr_push(int64_t ptr_addr, int64_t len_addr, int64_t cap_addr, int64_t v) {
  int64_t* pp = (int64_t*)(intptr_t)ptr_addr;
  int64_t* lp = (int64_t*)(intptr_t)len_addr;
  int64_t* cp = (int64_t*)(intptr_t)cap_addr;
  int64_t len = *lp, cap = *cp;
  if (len == cap) {
    cap = cap ? cap * 2 : 8;
    int64_t* np = (int64_t*)realloc((void*)(intptr_t)*pp, (size_t)cap * sizeof(int64_t));
    if (!np) exit(3);
    *pp = (int64_t)(intptr_t)np;
    *cp = cap;
  }
  ((int64_t*)(intptr_t)*pp)[len] = v;
  *lp = len + 1;
}
int64_t _lw_iarr_get(int64_t ptr, int64_t len, int64_t ix) {
  if (ix < 0 || ix >= len) { _rt_throw("Indice fuera de rango"); return 0; }
  return ((const int64_t*)(intptr_t)ptr)[ix];
}
/* v3.5.18: builtins de string unicode (stress_03 en Cranelift) */
static const char* _lw_cstr_of(Val v) { return (v.t == T_STR && v.s) ? v.s : ""; }
int64_t _lw_str_chars_h(int64_t s) { return _lw_h(_to_chars(_lw_cstr_of(_lw_u(s)))); }
int64_t _lw_str_upper_h(int64_t s) { return _lw_h(_v_str(_case_str(_lw_cstr_of(_lw_u(s)), 1))); }
int64_t _lw_str_lower_h(int64_t s) { return _lw_h(_v_str(_case_str(_lw_cstr_of(_lw_u(s)), 0))); }
int64_t _lw_str_pad_h(int64_t s, int64_t w, int64_t f, int64_t start) {
  Val sv = _lw_u(s), wv = _lw_u(w), fv = _lw_u(f);
  const char* fill = (fv.t == T_STR && fv.s && fv.s[0]) ? fv.s : " ";
  return _lw_h(_v_str(_pad_str(_lw_cstr_of(sv), (int64_t)_asf(wv), fill, (int)start)));
}
"#;

/// Base del shim de link (Cranelift/LLVM): runtime C probado (lumen_rt.h)
/// + helpers `_lw_*` de handles opacos.
fn lw_shim_base() -> String {
    let mut s = String::with_capacity(C_RUNTIME.len() + LW_RUNTIME.len() + 256);
    s.push_str(C_RUNTIME);
    s.push('\n');
    s.push_str(LW_RUNTIME);
    s
}

/// Fuente completa del shim de link para el backend Cranelift/LLVM SIN
/// conocimiento del programa: stubs del runtime de hilos (el dispatch real lo
/// arma `lw_shim_source_for`). Úsala solo el path LLVM por ahora.
pub fn lw_shim_source() -> String {
    let mut s = lw_shim_base();
    s.push_str("\nstatic void _init(void) {}\n");
    s.push_str("static Val _call_by_name_thread(const char* nm) { (void)nm; return _v_void(); }\n");
    // v3.5.30: default de _call_by_name — los shims LLVM/Cranelift no tienen
    // tablas _lfn (el backend C define su propia versión fuerte). Sin esto,
    // el uso en las corutinas deja "declared but not defined" en clang/gcc.
    s.push_str("static Val _call_by_name(const char* nm) { (void)nm; return _v_void(); }\n");
    s
}

/// v3.5.17: shim con hilos REALES para Cranelift. Los trampolines
/// `__lumen_ft_<fn>` viven en el objeto Cranelift (ven los parámetros por
/// handle); aquí solo se arma la tabla de dispatch `_lft` que consume
/// `_call_by_name_thread` (lumen_rt.h) en cada hilo hijo.
pub fn lw_shim_source_for(program: &Program) -> String {
    let program = lower_arraysetvar(program);
    let mut s = lw_shim_base();
    s.push_str("\n/* v3.5.17: hilos reales (Cranelift) — tabla _lft */\n");
    s.push_str("static void _init(void) {}\n");
    // v3.5.30: default de _call_by_name (sin tablas _lfn en este shim).
    s.push_str("static Val _call_by_name(const char* nm) { (void)nm; return _v_void(); }\n");
    let mut fnames: Vec<&String> = program.funcs.keys().collect();
    fnames.sort();
    for n in &fnames {
        s.push_str(&format!("extern int64_t __lumen_ft_{}(void);\n", mangle(n)));
    }
    if fnames.is_empty() {
        s.push_str(
            "static Val _call_by_name_thread(const char* nm) { (void)nm; return _v_void(); }\n",
        );
        return s;
    }
    s.push_str("static int64_t (*_lft_ptrs[])(void) = {\n");
    for n in &fnames {
        s.push_str(&format!("  __lumen_ft_{},\n", mangle(n)));
    }
    s.push_str("};\n");
    s.push_str("static const char* _lft_names[] = {\n");
    for n in &fnames {
        s.push_str(&format!("  \"{}\",\n", esc(n)));
    }
    s.push_str("};\n");
    s.push_str(&format!(
        "static Val _call_by_name_thread(const char* nm) {{\n  for (int _i = 0; _i < {}; _i++) if (!strcmp(_lft_names[_i], nm)) return _lw_unbox(_lft_ptrs[_i]());\n  return _v_void();\n}}\n",
        fnames.len()
    ));
    s
}

pub fn compile_to_c(program: &Program) -> String {
    let program = lower_arraysetvar(program);
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
        // v3.5.15: CAPTURAS — si es función anidada, agrega los params de TODA
        // la cadena de contenedores (padre, abuelo, ...) para que las
        // referencias a variables capturadas resuelvan al slot del ancestro.
        // El ancestro más cercano gana ante colisiones de nombre.
        {
            let mut cur = program.parents.get(fname).cloned();
            while let Some(anc) = cur {
                if let Some(anc_func) = program.funcs.get(&anc) {
                    for p in &anc_func.params {
                        if !m.contains_key(p) {
                            m.insert(p.clone(), format!("{}::{}", anc, p));
                        }
                    }
                }
                cur = program.parents.get(&anc).cloned();
            }
        }
        renames.insert(fname.clone(), m);
    }
    // Traduce un nombre de variable al slot real dentro de la función `fname`
    let resolve_var = |fname: &str, n: &str| -> String {
        renames
            .get(fname)
            .and_then(|m| m.get(n))
            .cloned()
            .unwrap_or_else(|| n.to_string())
    };

    // v3.5.17: CAPTURA DE LOCALES (backend C). Los params capturados ya se
    // resuelven vía renames; las variables locales (`sea`) declaradas en el
    // scope base de un ancestro se capturan igual: la función anidada
    // resuelve la referencia al slot exacto del ancestro (paridad con
    // compute_captures del backend Cranelift y con el scope-chain de la VM).
    //
    // Replica el nombrado de keys de plan_var_keys para una función, y
    // devuelve solo los bindings del scope BASE (params + `sea` de nivel 0).
    fn base_bindings(
        fname: &str,
        func: &LumenFunc,
        param_renames: &HashMap<String, String>,
    ) -> HashMap<String, String> {
        let mut scopes: Vec<HashMap<String, String>> = vec![HashMap::new()];
        for (raw, key) in param_renames {
            scopes[0].insert(raw.clone(), key.clone());
        }
        let mut counter = 0usize;
        for ins in &func.instrs {
            match ins {
                Instr::ScopePush => scopes.push(HashMap::new()),
                Instr::ScopePop => {
                    if scopes.len() > 1 {
                        scopes.pop();
                    }
                }
                Instr::StoreLocal(n) => {
                    let top = scopes.last_mut().expect("scope base siempre presente");
                    if !top.contains_key(n) {
                        counter += 1;
                        top.insert(n.clone(), format!("{}::{}#{}", fname, n, counter));
                    }
                }
                _ => {}
            }
        }
        scopes.into_iter().next().unwrap_or_default()
    }

    // v3.5.17: GLOBALES (paridad Cranelift/LLVM). Variables declaradas con
    // `sea` en la entrada y usadas por otras funciones van a UN slot
    // compartido (key cruda): la declaración en la entrada usa la key cruda
    // y las referencias de otras funciones ya caen en la misma key vía
    // fallback. Sin esto cada función veía su propio slot (mutaciones
    // perdidas, divergencia VM).
    let global_names = program_global_names(&program);
    let entry_name: Option<String> = program_entry_name(&program);
    let is_entry_fn = |fname: &str| entry_name.as_deref() == Some(fname);

    // Plan de slots por función (params renombrados + sombreado de bloques).
    // Orden: ancestros antes que anidados (el seed de captura necesita el
    // plan del ancestro para conocer las keys exactas de sus slots).
    let mut var_plans: HashMap<String, HashMap<usize, String>> = HashMap::new();
    // v3.5.19: keys de ancestros referidas por capturas — NO promovibles a
    // registros C (deben vivir en gv para que la anidada las alcance).
    let mut cap_refs: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut pending: Vec<String> = program.funcs.keys().cloned().collect();
    loop {
        let mut progressed = false;
        let mut next: Vec<String> = Vec::new();
        for fname in pending {
            let ready = match program.parents.get(&fname) {
                Some(p) => var_plans.contains_key(p),
                None => true,
            };
            if !ready {
                next.push(fname.clone());
                continue;
            }
            progressed = true;
            let func = &program.funcs[&fname];
            let pr = renames.get(&fname).cloned().unwrap_or_default();
            // Seed de captura: cadena de ancestros; el más cercano gana.
            let mut seed: HashMap<String, String> = HashMap::new();
            let mut cur = program.parents.get(&fname).cloned();
            while let Some(anc) = cur {
                if let Some(anc_func) = program.funcs.get(&anc) {
                    let anc_pr = renames.get(&anc).cloned().unwrap_or_default();
                    for (var, key) in base_bindings(&anc, anc_func, &anc_pr) {
                        seed.entry(var).or_insert(key);
                    }
                }
                cur = program.parents.get(&anc).cloned();
            }
            cap_refs.extend(seed.values().cloned());
            var_plans.insert(
                fname.clone(),
                plan_var_keys(&fname, func, &pr, &seed, &global_names, is_entry_fn(&fname)),
            );
        }
        pending = next;
        if pending.is_empty() || !progressed {
            // Ciclos/parents ausentes: compilar como antes para lo restante.
            for fname in pending {
                let func = &program.funcs[&fname];
                let pr = renames.get(&fname).cloned().unwrap_or_default();
                var_plans.insert(
                    fname.clone(),
                    plan_var_keys(
                        &fname,
                        func,
                        &pr,
                        &Default::default(),
                        &global_names,
                        is_entry_fn(&fname),
                    ),
                );
            }
            break;
        }
    }

    // v3.5.42 (bug fuzz closure_multi): por cada closure que captura,
    // precalcula las celdas FINALES (keys gv con #N) que su snapshot debe
    // guardar — paridad VM: estado por instanciación del definidor, no una
    // celda global compartida entre closures.
    let (captures, _cap_cells) = compute_captures(&program);
    let mut fref_cells: HashMap<String, Vec<String>> = HashMap::new();
    for (callee, cm) in &captures {
        let mut cells: Vec<String> = cm.values().cloned().collect();
        cells.sort();
        cells.dedup();
        let resolved: Vec<String> = cells
            .iter()
            .filter_map(|cell| {
                let (d, vn) = cell.split_once("::")?;
                let plan_d = var_plans.get(d)?;
                let dfunc = program.funcs.get(d)?;
                for (ii, ins) in dfunc.instrs.iter().enumerate() {
                    if let Instr::StoreLocal(x) = ins {
                        if x == vn {
                            if let Some(k) = plan_d.get(&ii) {
                                return Some(k.clone());
                            }
                        }
                    }
                }
                // param/global del definidor: la key cruda ya es la final
                Some(cell.clone())
            })
            .collect();
        fref_cells.insert(callee.clone(), resolved);
    }

    // v3.5.19: PROMOCIÓN DE REGISTROS (backend C). Los locales propios que
    // no escapan (sin MakeRef, sin captura por anidadas) dejan gv[] y se
    // convierten en variables locales C — GCC las mantiene en registros y
    // optimiza bucles enteros. Los params y globales siguen en gv (ABI de
    // hilos/CallValue/capturas).
    let mut promoted: BTreeMap<String, HashMap<String, String>> = BTreeMap::new();
    for (fname, func) in &program.funcs {
        let plan = match var_plans.get(fname) {
            Some(p) => p,
            None => continue,
        };
        // Keys objetivo de MakeRef en esta función (se dirigen, no se promueven).
        let mut ref_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (i, ins) in func.instrs.iter().enumerate() {
            if let Instr::MakeRef(_) = ins {
                if let Some(k) = plan.get(&i) {
                    ref_keys.insert(k.clone());
                }
            }
        }
        let prefix = format!("{}::", fname);
        let mut map: HashMap<String, String> = HashMap::new();
        let mut keys: Vec<&String> = plan.values().collect();
        keys.sort();
        keys.dedup();
        let mut lv = 0usize;
        for k in keys {
            // Solo LOCALES propios (key `{fn}::var#n`): los params
            // (`{fn}::p`) quedan en gv por el ABI de llamadas/hilos.
            let is_own_local = k.starts_with(&prefix) && k[prefix.len()..].contains('#');
            if is_own_local && !ref_keys.contains(k) && !cap_refs.contains(k) {
                map.insert(k.clone(), format!("_lv{}", lv));
                lv += 1;
            }
        }
        promoted.insert(fname.clone(), map);
    }

    // v3.5.21: PROMOCIÓN DE ENTEROS a `long long` nativos (análisis global).
    // key → (nombre C, expresión de inicialización: None = `0`, Some = copia
    // del gv del parámetro en la entrada).
    let (int_locals_map, int_params_map) = int_promotion_analysis(&program, &var_plans);
    // v3.5.21: solo se promocionan PARÁMETROS de funciones con llamadores
    // estáticos. Las llamadas dinámicas (hilos por nombre, CallValue,
    // FuncRef) pueden pasar cualquier cosa → params quedan en gv.
    let mut direct_callers: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut dyn_named: std::collections::HashSet<String> = std::collections::HashSet::new();
    for func in program.funcs.values() {
        let it = func.instrs.iter().peekable();
        for ins in it {
            match ins {
                Instr::Call(cn, _) if program.funcs.contains_key(cn) => {
                    direct_callers.insert(cn.clone());
                }
                Instr::FuncRef(n) => {
                    dyn_named.insert(n.clone());
                }
                Instr::Call(hl, argc)
                    if matches!(
                        hl.as_str(),
                        "__hilo_lanzar" | "__tarea_lanzar" | "__thread_spawn" | "__task_spawn"
                    ) =>
                {
                    // el primer argumento (nombre de función) es la instrucción
                    // anterior si es ConstStr
                    let _ = argc;
                }
                _ => {}
            }
        }
        // v3.5.23: walk de la pila abstracta para capturar el ConstStr del
        // nombre de función en llamadas dinámicas con CUALQUIER nº de args.
        let mut vstack: Vec<Option<String>> = Vec::new();
        for ins in func.instrs.iter() {
            match ins {
                Instr::ConstStr(s) => vstack.push(Some(s.clone())),
                Instr::Call(hl, argc) => {
                    let name_pos: Option<usize> = match hl.as_str() {
                        "__hilo_lanzar" | "__tarea_lanzar" | "__thread_spawn" | "__task_spawn"
                        | "__coro_crear" | "__coro_create" => Some(0),
                        "__mutex_bloquear" | "__mutex_lock" => Some(1),
                        _ => None,
                    };
                    let mut popped: Vec<Option<String>> = Vec::new();
                    for _ in 0..*argc {
                        popped.push(vstack.pop().unwrap_or(None));
                    }
                    if let Some(p) = name_pos {
                        if p < popped.len() {
                            if let Some(n) = popped[popped.len() - 1 - p].as_ref() {
                                dyn_named.insert(n.clone());
                            }
                        }
                    }
                    vstack.push(None);
                }
                Instr::Label(_) | Instr::Jmp(_) => vstack.clear(),
                Instr::JmpIf(_) => {
                    vstack.pop();
                }
                other => {
                    let d = instr_depth_delta(other);
                    let mut pops = if d < 0 { -d } else { 0 };
                    while pops > 0 {
                        vstack.pop();
                        pops -= 1;
                    }
                    for _ in 0..d.max(0) {
                        vstack.push(None);
                    }
                }
            }
        }
    }
    let mut int_proms: BTreeMap<String, HashMap<String, (String, Option<String>)>> =
        BTreeMap::new();
    // v3.5.26: la promoción entera tiene prioridad sobre la promoción Val:
    // quitar de `promoted` las keys que serán enteras ANTES de construir
    // int_proms (así el filtro own_prom no las excluye).
    for (fnm, lm) in &int_locals_map {
        if let Some(p) = promoted.get_mut(fnm) {
            for (k, b) in lm {
                if *b && !cap_refs.contains(k) {
                    p.remove(k);
                }
            }
        }
    }
    for (fname, func) in &program.funcs {
        let plan = match var_plans.get(fname) {
            Some(p) => p,
            None => continue,
        };
        let mut ref_keys: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (i, ins) in func.instrs.iter().enumerate() {
            if let Instr::MakeRef(_) = ins {
                if let Some(k) = plan.get(&i) {
                    ref_keys.insert(k.clone());
                }
            }
        }
        let own_prom = promoted.get(fname);
        let lm = int_locals_map.get(fname);
        let pm = int_params_map.get(fname);
        let mut cands: Vec<(String, Option<String>)> = Vec::new();
        if let Some(lm) = lm {
            for (k, b) in lm.iter() {
                if *b
                    && !cap_refs.contains(k)
                    && !ref_keys.contains(k)
                    && own_prom.is_none_or(|m| !m.contains_key(k))
                {
                    cands.push((k.clone(), None));
                }
            }
        }
        if let Some(pm) = pm {
            let params_promotable = direct_callers.contains(fname) && !dyn_named.contains(fname);
            for (pos, b) in pm.iter().enumerate() {
                if !*b || !params_promotable {
                    continue;
                }
                let p = match func.params.get(pos) {
                    Some(p) => p,
                    None => continue,
                };
                let k = resolve_var(fname, p);
                if cap_refs.contains(&k) || ref_keys.contains(&k) {
                    continue;
                }
                // se guarda la KEY del parámetro; emit_func resuelve gv_of
                cands.push((k.clone(), Some(k.clone())));
            }
        }
        cands.sort_by(|a, b| a.0.cmp(&b.0));
        let mut m: HashMap<String, (String, Option<String>)> = HashMap::new();
        for (idx, (k, init)) in cands.into_iter().enumerate() {
            m.insert(k, (format!("_lli{}", idx), init));
        }
        int_proms.insert(fname.clone(), m);
    }

    // v3.5.23: ABI DE ARGUMENTOS EN REGISTROS. Las funciones que NUNCA se
    // llaman por nombre (hilos/mutex/corutinas/FuncRef/CallValue quedan
    // fuera vía dyn_named) reciben sus argumentos como parámetros C nativos:
    // sin staging por gv[], sin save/restore de params, recursión nativa.
    // Excepción: si un param está CAPTURADO por una función anidada debe
    // vivir en gv (la anidada lo lee de ahí).
    let reg_abi: std::collections::HashSet<String> = program
        .funcs
        .keys()
        .filter(|k| !dyn_named.contains(*k))
        .filter(|k| {
            !program.funcs[*k]
                .params
                .iter()
                .any(|p| cap_refs.contains(&resolve_var(k, p)))
        })
        .cloned()
        .collect();

    // v3.5.24: funciones que NO lanzan → sin ERRCHK y sin NOINLINE (gcc
    // puede inlinearlas: fib y cía se acercan a velocidad C nativa).
    let no_throw = no_throw_analysis(&program);

    // v3.5.24: funciones que NO mutan → sus args se comparten sin _dcp
    // (paridad con la VM, que comparte vía Arc).
    let no_mutate = no_mutate_analysis(&program);

    // v3.5.24: funciones que siempre devuelven entero → retorno long long.
    let returns_int = returns_int_analysis(&program, &int_proms);

    for (name, func) in &program.funcs {
        let plan = &var_plans[name];
        for p in &func.params {
            add_name(&resolve_var(name, p));
        }
        for (i, ins) in func.instrs.iter().enumerate() {
            match ins {
                Instr::Load(_)
                | Instr::Store(_)
                | Instr::StoreLocal(_)
                | Instr::ArrayPushVar(_)
                | Instr::MakeRef(_) => {
                    if let Some(k) = plan.get(&i) {
                        // v3.5.21: los LOCALES enteros promovidos no viven en
                        // gv (los params promovidos sí: staging/save-restore).
                        let is_prom_local = k.contains('#')
                            && int_proms.get(name).is_some_and(|m| m.contains_key(k));
                        if !is_prom_local {
                            add_name(k);
                        }
                    }
                }
                Instr::FuncRef(n) => {
                    add_name(n);
                }
                Instr::Call(cn, _)
                    if !program.funcs.contains_key(cn) && !unknown.iter().any(|u| u == cn) =>
                {
                    unknown.push(cn.clone());
                    record_unsupported_builtin(cn);
                }
                _ => {}
            }
        }
        let _ = name;
    }

    let mut name_sets: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (name, func) in &program.funcs {
        // Slots reales de la función: params renombrados + keys planificadas
        let mut set: Vec<String> = func.params.iter().map(|p| resolve_var(name, p)).collect();
        let plan = &var_plans[name];
        for k in plan.values() {
            if !set.iter().any(|x| x == k) {
                set.push(k.clone());
            }
        }
        // v3.5.17: solo los slots PROPIOS (`{name}::...`) entran al
        // save/restore alrededor de la llamada. Los globales sin prefijo y
        // los slots capturados de ancestros son estado compartido vivo:
        // restaurarlos deshace mutaciones legítimas (globales y capturas
        // mutables, paridad VM/Cranelift).
        let prefix = format!("{}::", name);
        set.retain(|k| k.starts_with(&prefix));
        // v3.5.19: los locales promovidos a registros C viven en el stack de
        // C (per-call): el save/restore de gv ya no aplica para ellos.
        if let Some(pr) = promoted.get(name) {
            set.retain(|k| !pr.contains_key(k));
        }
        // v3.5.21: tampoco los LOCALES long long promovidos (los params
        // promovidos se quedan: el save/restore protege la recursión).
        if let Some(pi) = int_proms.get(name) {
            set.retain(|k| !(k.contains('#') && pi.contains_key(k)));
        }
        // v3.5.23: con ABI de registros los params ya no viven en gv.
        if reg_abi.contains(name) && !func.params.is_empty() {
            let pkeys: std::collections::HashSet<String> =
                func.params.iter().map(|p| resolve_var(name, p)).collect();
            set.retain(|k| !pkeys.contains(k));
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
    // v3.5.23: firma de una función según su ABI (registros o gv).
    let sig_of = |fname: &str| -> String {
        let func = &program.funcs[fname];
        if !reg_abi.contains(fname) || func.params.is_empty() {
            return String::new();
        }
        let prom = int_proms.get(fname);
        let mut ps: Vec<String> = Vec::new();
        for (pos, p) in func.params.iter().enumerate() {
            let key = resolve_var(fname, p);
            let is_int = prom
                .map(|m| matches!(m.get(&key), Some((_, Some(_)))))
                .unwrap_or(false);
            if is_int {
                let ll = &prom.unwrap()[&key].0;
                ps.push(format!("long long {}", ll));
            } else {
                ps.push(format!("Val _pv{}", pos));
            }
        }
        ps.join(", ")
    };
    for name in program.funcs.keys() {
        let sig = sig_of(name);
        let attr = if no_throw.contains(name) {
            ""
        } else {
            "LUMEN_NOINLINE "
        };
        let ret_t = if returns_int.contains(name) {
            "long long"
        } else {
            "Val"
        };
        out.push_str(&format!(
            "static {}{} _f_{}({});\n",
            attr,
            ret_t,
            mangle(name),
            if sig.is_empty() {
                "void".to_string()
            } else {
                sig
            }
        ));
        if (reg_abi.contains(name) && !program.funcs[name].params.is_empty())
            || returns_int.contains(name)
        {
            out.push_str(&format!("static Val _fw_{}(void);\n", mangle(name)));
        }
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
        let empty_prom: HashMap<String, String> = HashMap::new();
        let prom = promoted.get(name).unwrap_or(&empty_prom);
        let empty_prom_i: HashMap<String, (String, Option<String>)> = HashMap::new();
        let prom_i = int_proms.get(name).unwrap_or(&empty_prom_i);
        out.push_str(&emit_func(
            name,
            func,
            &program,
            &name_sets,
            &gv_of,
            &renames,
            &plan,
            prom,
            &promoted,
            prom_i,
            &cap_refs,
            &reg_abi,
            &int_proms,
            &no_throw,
            &no_mutate,
            &returns_int,
            &arr_vars_by_name(func, &cap_refs),
            &fref_cells,
        ));
    }

    // v3.5.23: wrappers gv→registros para funciones con ABI nativo (los usa
    // _call_by_name, el camino legacy y cualquier llamada dinámica).
    for (name, func) in program.funcs.iter() {
        let need_wrapper =
            (reg_abi.contains(name) && !func.params.is_empty()) || returns_int.contains(name);
        if !need_wrapper {
            continue;
        }
        // returns_int sin params (o con ABI gv): wrapper simple de envoltura.
        if !reg_abi.contains(name) || func.params.is_empty() {
            if returns_int.contains(name) {
                out.push_str(&format!(
                    "static Val _fw_{}(void) {{ return _v_int(_f_{}()); }}\n",
                    mangle(name),
                    mangle(name)
                ));
            }
            continue;
        }
        let mut args: Vec<String> = Vec::new();
        for p in func.params.iter() {
            let key = resolve_var(name, p);
            let is_int = int_proms
                .get(name)
                .map(|m| matches!(m.get(&key), Some((_, Some(_)))))
                .unwrap_or(false);
            if is_int {
                args.push(format!("(long long)_asf(_deref({}))", gv_of(&key)));
            } else {
                args.push(format!("_deref({})", gv_of(&key)));
            }
        }
        if returns_int.contains(name) {
            out.push_str(&format!(
                "static Val _fw_{}(void) {{ return _v_int(_f_{}({})); }}\n",
                mangle(name),
                mangle(name),
                args.join(", ")
            ));
        } else {
            out.push_str(&format!(
                "static Val _fw_{}(void) {{ return _f_{}({}); }}\n",
                mangle(name),
                mangle(name),
                args.join(", ")
            ));
        }
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
        // wrapper: ABI de registros con params, o retorno int (envuelve en Val)
        if (reg_abi.contains(*n) && !program.funcs[*n].params.is_empty())
            || returns_int.contains(*n)
        {
            out.push_str(&format!("  &_fw_{},\n", mangle(n)));
        } else {
            out.push_str(&format!("  &_f_{},\n", mangle(n)));
        }
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

    // v3.5.10: trampolines de hilo — _ft_<fn> copia los args staged
    // (lw_thr_args, TLS) a los slots de params (gv, TLS) y llama a _f_<fn>.
    // gv/gn/gc/_err/_pars son TLS: cada hilo tiene su propio entorno global,
    // igual que la VM (que crea una VM nueva por hilo en __hilo_lanzar).
    for (name, func) in &program.funcs {
        // v3.5.23: función con ABI de registros → el trampoline delega en el
        // wrapper _fw (staging gv → params nativos). Estas funciones no se
        // lanzan por nombre en la práctica (dyn_named las excluye).
        if reg_abi.contains(name) && !func.params.is_empty() {
            out.push_str(&format!(
                "static Val _ft_{}(void) {{ return _fw_{}(); }}\n",
                mangle(name),
                mangle(name)
            ));
            continue;
        }
        let mut body = String::new();
        for (pi, p) in func.params.iter().enumerate() {
            if pi >= 8 {
                break;
            }
            let slot = gv_of(&resolve_var(name, p));
            body.push_str(&format!("  {} = _dcp(lw_thr_args[{}]);\n", slot, pi));
        }
        if returns_int.contains(name) {
            body.push_str(&format!("  return _v_int(_f_{}());\n", mangle(name)));
        } else {
            body.push_str(&format!("  return _f_{}();\n", mangle(name)));
        }
        out.push_str(&format!(
            "static Val _ft_{}(void) {{\n{}}}\n",
            mangle(name),
            body
        ));
    }
    for n in &unknown {
        out.push_str(&format!(
            "static Val _ft_{}(void) {{ return _v_void(); }}\n",
            mangle(n)
        ));
    }
    out.push_str("static Val (*_lft_ptrs[])(void) = {\n");
    for n in &fnames {
        out.push_str(&format!("  &_ft_{},\n", mangle(n)));
    }
    out.push_str("};\n");
    out.push_str("static const char* _lft_names[] = {\n");
    for n in &fnames {
        out.push_str(&format!("  \"{}\",\n", esc(n)));
    }
    out.push_str("};\n");
    out.push_str(&format!(
        "static Val _call_by_name_thread(const char* nm) {{\n  for (int _i = 0; _i < {}; _i++) if (!strcmp(_lft_names[_i], nm)) return _lft_ptrs[_i]();\n  return _v_void();\n}}\n\n",
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
    fname: &str,
    func: &LumenFunc,
    param_renames: &HashMap<String, String>,
    capture_seed: &HashMap<String, String>,
    global_names: &std::collections::HashSet<String>,
    declares_globals: bool,
) -> HashMap<usize, String> {
    let mut plan: HashMap<usize, String> = HashMap::new();
    // v3.5.17: scopes[0] = seed de capturas (slots del ancestro, solo
    // lectura/resolución — un `sea` propio NUNCA lo reutiliza); scopes[1] =
    // scope base propio con los renombres de params.
    let mut scopes: Vec<HashMap<String, String>> = vec![capture_seed.clone(), HashMap::new()];
    for (raw, key) in param_renames {
        scopes[1].insert(raw.clone(), key.clone());
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
                // > 2: los dos scopes base (seed de capturas + propio) no se
                // desapilan nunca.
                if scopes.len() > 2 {
                    scopes.pop();
                }
            }
            Instr::StoreLocal(n) => {
                // Declaración: siempre en el scope ACTUAL. Reusar key solo si
                // ya fue declarada en este mismo nivel; un nombre de un nivel
                // exterior se SOMBREA con key nueva.
                let at_base = scopes.len() == 2; // solo el seed + scope propio
                if declares_globals && at_base && global_names.contains(n) {
                    // v3.5.17: global — slot compartido con key cruda para
                    // que las demás funciones (que refieren el nombre crudo)
                    // vean la misma celda.
                    let top = scopes.last_mut().expect("scope base siempre presente");
                    let key = match top.get(n) {
                        Some(k) => k.clone(),
                        None => {
                            top.insert(n.clone(), n.clone());
                            n.clone()
                        }
                    };
                    plan.insert(i, key);
                } else {
                    let top = scopes.last_mut().expect("scope base siempre presente");
                    let key = match top.get(n) {
                        Some(k) => k.clone(),
                        None => {
                            counter += 1;
                            let k = format!("{}::{}#{}", fname, n, counter);
                            top.insert(n.clone(), k.clone());
                            k
                        }
                    };
                    plan.insert(i, key);
                }
            }
            Instr::Load(n) | Instr::Store(n) | Instr::ArrayPushVar(n) | Instr::MakeRef(n) => {
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
#[allow(dead_code)] // v3.4.6: obsoleto desde save/restore callee-scoped; se elimina en limpieza
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
            Instr::ConstInt(_)
            | Instr::ConstFloat(_)
            | Instr::ConstStr(_)
            | Instr::ConstBool(_)
            | Instr::Load(_)
            | Instr::Read
            | Instr::FuncRef(_) => st.push(None),
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
            Instr::Unary(_)
            | Instr::ArrayLen
            | Instr::TryUnwrap
            | Instr::MatchType(_)
            | Instr::MatchPayload
            | Instr::TupleAccess(_)
            | Instr::MatchVariant(_)
            | Instr::ResultOk
            | Instr::ResultErr
            | Instr::OptionSome
            | Instr::OptionNone => {
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
            // v3.5.40: par canónico ArraySet + Store.
            Instr::ArraySetVar(_) => {
                if !popn(&mut st, 3) {
                    break;
                }
                st.push(None);
                if !popn(&mut st, 1) {
                    break;
                }
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
            Instr::Return
            | Instr::Halt
            | Instr::Jmp(_)
            | Instr::Label(_)
            | Instr::ScopePush
            | Instr::ScopePop
            | Instr::PushHandler(_)
            | Instr::PopHandler
            | Instr::Nop
            | Instr::Phi(_, _) => {}
        }
    }
    out
}

/// v3.5.20: vacía la pila de expresiones a ST[] (con ERRCHK si había riesgo).
/// v3.5.21: INFERENCIA GLOBAL DE ENTEROS PUROS. Punto fijo sobre todo el
/// programa:
///  - un LOCAL es entero si todas sus escrituras son expresiones enteras;
///  - un PARÁMETRO es entero si ningún llamador le pasa algo no-entero y no
///    se reasigna en el cuerpo;
///  - los consumos fuera del mundo entero se cubren convirtiendo `_v_int`
///    en el límite (por eso no hace falta analizar consumidores).
///    Con esto los bucles enteros operan con `long long` nativos (sin tags,
///    sin _bin, sin tráfico de Val de 80B).
#[derive(Clone, Copy, PartialEq)]
enum IKind {
    Int,
    Not,
}

/// v3.5.24: builtins que NUNCA lanzan error (lista conservadora).
fn builtin_no_throw(name: &str) -> bool {
    matches!(
        name,
        "imprimir"
            | "print"
            | "a_texto"
            | "to_texto"
            | "__str_from"
            | "largo"
            | "len"
            | "length"
            | "__str_len"
            | "__str_longitud"
            | "agregar"
            | "push"
            | "abs"
            | "absoluto"
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
            | "__str_mayusculas"
            | "__str_upper"
            | "__str_minusculas"
            | "__str_lower"
            | "__str_contiene"
            | "__str_empieza_con"
            | "__str_starts_with"
            | "__str_concat_list"
            | "__str_concatenar_lista"
            | "__lista_invertir"
            | "__list_reverse"
            | "__map_nuevo"
            | "__map_new"
            | "__map_poner"
            | "__map_set"
            | "__map_obtener"
            | "__map_get"
            | "__map_contiene"
            | "__map_contains"
            | "__map_longitud"
            | "__map_len"
            | "__map_claves"
            | "__map_keys"
            | "__canal_nuevo"
            | "__channel_new"
            | "__canal_enviar"
            | "__channel_send"
            | "__canal_recibir"
            | "__channel_recv"
            | "__mutex_nuevo"
            | "__mutex_new"
            | "__tiempo_ahora"
            | "__time_now"
            | "__tiempo_formatear"
            | "__time_format"
            | "__tiempo_diferencia"
            | "__time_diff"
            | "__tiempo_parsear"
            | "__tiempo_parse"
            | "__calendario_hijri"
            | "__calendar_hijri"
            | "__calendario_persa"
            | "__calendar_persian"
            | "__str_padding_inicio"
            | "__str_padding_fin"
            | "__str_dividir"
            | "__str_split"
            | "__str_recortar"
            | "__str_trim"
            | "__str_codigo"
            | "__str_ord"
            | "__str_caracter"
            | "__str_chr"
            | "__tipo_de"
            | "__typeof"
            | "__unicode_normalizar"
            | "__unicode_normalize"
            | "__hilo_esperar"
            | "__thread_join"
            | "__tarea_esperar"
            | "__task_await"
    )
}

/// v3.5.24: ¿la instrucción puede lanzar error en este contexto? `no_throw`
/// = funciones que (por punto fijo) no lanzan. Lanza: Div/Mod (por cero),
/// ArrayGet/ArraySet/TupleAccess (fuera de rango), TryUnwrap, llamadas a
/// funciones que lanzan y builtins no blanqueados.
fn instr_throws(
    ins: &Instr,
    no_throw: &std::collections::HashSet<String>,
    program: &Program,
) -> bool {
    match ins {
        Instr::Binary(Op::Div) | Instr::Binary(Op::Mod) => true,
        Instr::ArrayGet | Instr::ArraySet | Instr::TupleAccess(_) | Instr::TryUnwrap => true,
        Instr::Call(cn, _) => {
            if program.funcs.contains_key(cn) {
                !no_throw.contains(cn)
            } else {
                !builtin_no_throw(cn)
            }
        }
        Instr::CallValue(_) => true,
        _ => false,
    }
}

/// v3.5.24: punto fijo de funciones que NO lanzan: no contienen ops que
/// lancen fuera de sus propios `intentar` ni llaman a funciones que lancen.
/// v3.5.24: análisis de funciones que NO MUTAN valores (arrays/structs).
/// Mutación directa: ArrayPushVar, ArraySet, StructSet, MakeRef, o recibir
/// un argumento prestado (MakeRef del llamador). Propaga por llamadas: si el
/// callee muta, el caller muta (puede pasarle el argumento). Si una función
/// es no-mutante, compartirle argumentos SIN _dcp es seguro y tiene la misma
/// semántica que la VM (que comparte vía Arc).
/// v3.5.24: funciones que SIEMPRE devuelven entero → retornan `long long`
/// nativo (8B en vez de Val de 80B por llamada: fib y cía se acercan a C).
/// v3.5.25: variables locales que son SIEMPRE enteras (por nombre, sin
/// plan del backend C) — para los slots i64 de Cranelift. Punto fijo
/// decreciente (siembra optimista): se degradan las que reciben algo no
/// entero. Solo locales declarados una única vez (sin sombreado) y no
/// capturados (los captura el llamador vía exclude).
/// v3.5.25: (pops, pushes) EXACTOS por instrucción para la pila abstracta
/// de los análisis (el delta neto solo no basta: ArrayNew deja residuos).
fn instr_pops_pushes(ins: &Instr) -> (usize, usize) {
    match ins {
        Instr::ConstInt(_) | Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_) => {
            (0, 1)
        }
        Instr::Load(_)
        | Instr::Read
        | Instr::MakeRef(_)
        | Instr::FuncRef(_)
        | Instr::OptionNone => (0, 1),
        Instr::Store(_) | Instr::StoreLocal(_) | Instr::Print | Instr::Return | Instr::Halt => {
            (1, 0)
        }
        Instr::Binary(_) => (2, 1),
        Instr::Unary(_) => (1, 1),
        Instr::Call(_, argc) => (*argc, 1),
        Instr::CallValue(argc) => (*argc + 1, 1),
        Instr::ArrayNew(n) => (*n, 1),
        Instr::ArrayPush => (2, 1),
        Instr::ArrayPushVar(_) => (1, 1),
        Instr::ArrayGet => (2, 1),
        Instr::ArraySet => (3, 1),
        // v3.5.40: par canónico ArraySet(3,1) + Store(1,0).
        Instr::ArraySetVar(_) => (4, 1),
        Instr::ArrayLen => (1, 1),
        Instr::StructNew(_, n) => (2 * *n, 1),
        Instr::StructGet => (2, 1),
        Instr::StructSet => (3, 1),
        Instr::TupleNew(n) => (*n, 1),
        Instr::TupleAccess(_) => (1, 1),
        Instr::OptionSome | Instr::ResultOk | Instr::ResultErr | Instr::TryUnwrap => (1, 1),
        Instr::MatchType(_) | Instr::MatchPayload | Instr::MatchVariant(_) => (1, 1),
        Instr::EnumCtor { argc, .. } => (*argc, 1),
        Instr::PushHandler(_)
        | Instr::PopHandler
        | Instr::Jmp(_)
        | Instr::Label(_)
        | Instr::Phi(..)
        | Instr::Nop
        | Instr::ScopePush
        | Instr::ScopePop => (0, 0),
        Instr::JmpIf(_) => (1, 0),
    }
}

/// v3.5.26: locales que son ARRAYS DE ENTEROS puros (solo se les hace
/// agregar/leer/largo con enteros y nunca escapan). Se emiten como arrays
/// nativos `long long*` con crecimiento amortizado en el backend C.
fn arr_vars_by_name(
    func: &LumenFunc,
    exclude: &std::collections::HashSet<String>,
) -> std::collections::HashSet<String> {
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum K {
        Int,
        Arr,
        Not,
    }
    let mut decl_count: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for ins in &func.instrs {
        if let Instr::StoreLocal(n) = ins {
            *decl_count.entry(n.clone()).or_insert(0) += 1;
        }
    }
    // candidatos optimistas
    let mut is_arr: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    let mut is_int: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    for (n, c) in &decl_count {
        if *c == 1 && !exclude.contains(n) {
            is_arr.insert(n.clone(), true);
            is_int.insert(n.clone(), true);
        }
    }
    loop {
        let mut changed = false;
        // pila abstracta: (kind, variable fuente del valor si viene de un Load)
        let mut st: Vec<(K, Option<String>)> = Vec::new();
        let demote_arr = |m: &mut std::collections::HashMap<String, bool>,
                          src: &Option<String>,
                          ch: &mut bool,
                          _why: &str,
                          _iidx: usize| {
            if let Some(v) = src {
                if let Some(b) = m.get_mut(v) {
                    if *b {
                        *b = false;
                        *ch = true;
                    }
                }
            }
        };
        for (iidx, ins) in func.instrs.iter().enumerate() {
            match ins {
                Instr::ConstInt(_) => st.push((K::Int, None)),
                Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_) => {
                    st.push((K::Not, None))
                }
                Instr::Load(n) => {
                    let k = if *is_arr.get(n).unwrap_or(&false) {
                        K::Arr
                    } else if *is_int.get(n).unwrap_or(&false) {
                        K::Int
                    } else {
                        K::Not
                    };
                    st.push((k, Some(n.clone())));
                }
                Instr::StoreLocal(x) | Instr::Store(x) => {
                    let (k, src) = st.pop().unwrap_or((K::Not, None));
                    if let (K::Arr, Some(src)) = (k, src.as_ref()) {
                        if src != x {
                            // alias: el array pasa a otra variable → no promovible
                            demote_arr(
                                &mut is_arr,
                                &Some(src.clone()),
                                &mut changed,
                                "alias-store",
                                iidx,
                            );
                        }
                    }
                    if is_arr.get(x).copied().unwrap_or(false) && k != K::Arr {
                        if let Some(b) = is_arr.get_mut(x) {
                            *b = false;
                            changed = true;
                        }
                    }
                    if is_int.get(x).copied().unwrap_or(false) && k != K::Int {
                        if let Some(b) = is_int.get_mut(x) {
                            *b = false;
                            changed = true;
                        }
                    }
                }
                Instr::ArrayPushVar(x) => {
                    let (k, _src) = st.pop().unwrap_or((K::Not, None));
                    if is_arr.get(x).copied().unwrap_or(false) && k != K::Int {
                        if let Some(b) = is_arr.get_mut(x) {
                            *b = false;
                            changed = true;
                        }
                    }
                    st.push((K::Arr, Some(x.clone())));
                }
                Instr::ArrayGet => {
                    st.pop(); // índice
                    let (ka, _src) = st.pop().unwrap_or((K::Not, None));
                    st.push(if ka == K::Arr {
                        (K::Int, None)
                    } else {
                        (K::Not, None)
                    });
                }
                Instr::ArrayLen => {
                    st.pop();
                    st.push((K::Int, None));
                }
                Instr::ArraySet => {
                    st.pop();
                    st.pop();
                    let (ka, src) = st.pop().unwrap_or((K::Not, None));
                    if ka == K::Arr {
                        demote_arr(&mut is_arr, &src, &mut changed, "uso", iidx);
                    }
                    st.push((K::Not, None));
                }
                Instr::ArrayNew(n) => {
                    let mut all_int = true;
                    for _ in 0..*n {
                        let (k, src) = st.pop().unwrap_or((K::Not, None));
                        if k != K::Int {
                            all_int = false;
                        }
                        // elementos: si fueran arrays, ya no se usan más aquí
                        let _ = src;
                    }
                    // v3.5.29: solo ArrayNew(0) (array vacío) se promociona a
                    // slots nativos. Los literales con elementos (n>0) caen al
                    // camino legacy (Val) — el camino nativo solo inicializa
                    // arrays vacíos; promover un literal lo dejaría sin
                    // elementos (bug: "Indice fuera de rango" en locales).
                    st.push(if all_int && *n == 0 {
                        (K::Arr, None)
                    } else {
                        (K::Not, None)
                    });
                }
                Instr::ArrayPush => {
                    let (kv, _) = st.pop().unwrap_or((K::Not, None));
                    let (ka, src) = st.pop().unwrap_or((K::Not, None));
                    if ka != K::Arr || kv != K::Int {
                        demote_arr(&mut is_arr, &src, &mut changed, "uso", iidx);
                    }
                    st.push((if ka == K::Arr { K::Arr } else { K::Not }, src));
                }
                Instr::Call(_, argc) => {
                    for _ in 0..*argc {
                        let (k, src) = st.pop().unwrap_or((K::Not, None));
                        if k == K::Arr {
                            demote_arr(&mut is_arr, &src, &mut changed, "uso", iidx);
                        }
                    }
                    st.push((K::Not, None));
                }
                Instr::CallValue(argc) => {
                    for _ in 0..=*argc {
                        let (k, src) = st.pop().unwrap_or((K::Not, None));
                        if k == K::Arr {
                            demote_arr(&mut is_arr, &src, &mut changed, "uso", iidx);
                        }
                    }
                    st.push((K::Not, None));
                }
                Instr::Print => {
                    let (k, src) = st.pop().unwrap_or((K::Not, None));
                    if k == K::Arr {
                        demote_arr(&mut is_arr, &src, &mut changed, "uso", iidx);
                    }
                }
                Instr::Binary(op) => {
                    let (kb, sb) = st.pop().unwrap_or((K::Not, None));
                    let (ka, sa) = st.pop().unwrap_or((K::Not, None));
                    if ka == K::Arr {
                        demote_arr(&mut is_arr, &sa, &mut changed, "bin", iidx);
                    }
                    if kb == K::Arr {
                        demote_arr(&mut is_arr, &sb, &mut changed, "bin", iidx);
                    }
                    // propagación entera: aritmética de enteros sigue siendo entera
                    let r = if ka == K::Int
                        && kb == K::Int
                        && matches!(
                            op,
                            Op::Add
                                | Op::Sub
                                | Op::Mul
                                | Op::BitAnd
                                | Op::BitOr
                                | Op::BitXor
                                | Op::ShiftLeft
                                | Op::ShiftRight
                        ) {
                        K::Int
                    } else {
                        K::Not
                    };
                    st.push((r, None));
                }
                Instr::Unary(op) => {
                    let (k, s1) = st.pop().unwrap_or((K::Not, None));
                    if k == K::Arr {
                        demote_arr(&mut is_arr, &s1, &mut changed, "uso", iidx);
                    }
                    let r = if k == K::Int && matches!(op, Op::Negate | Op::BitNot) {
                        K::Int
                    } else {
                        K::Not
                    };
                    st.push((r, None));
                }
                Instr::Return | Instr::Halt => {
                    let (k, s1) = st.pop().unwrap_or((K::Not, None));
                    if k == K::Arr {
                        demote_arr(&mut is_arr, &s1, &mut changed, "uso", iidx);
                    }
                    st.clear();
                }
                Instr::Label(_) | Instr::Jmp(_) => st.clear(),
                Instr::JmpIf(_) => {
                    let (k, s1) = st.pop().unwrap_or((K::Not, None));
                    if k == K::Arr {
                        demote_arr(&mut is_arr, &s1, &mut changed, "uso", iidx);
                    }
                }
                other => {
                    let (pops, pushes) = instr_pops_pushes(other);
                    for _ in 0..pops {
                        let (k, s1) = st.pop().unwrap_or((K::Not, None));
                        if k == K::Arr {
                            demote_arr(&mut is_arr, &s1, &mut changed, "uso", iidx);
                        }
                    }
                    for _ in 0..pushes {
                        st.push((K::Not, None));
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    is_arr
        .into_iter()
        .filter(|(_, b)| *b)
        .map(|(n, _)| n)
        .collect()
}

/// v3.5.28: grafo de llamadas (Cranelift) — réplica de la lógica del backend
/// C: `direct_callers` (Call estáticos) y `dyn_named` (FuncRef + nombres en
/// hilos/corutinas/mutex). Las funciones dyn reciben handles arbitrarios →
/// sus params NUNCA se especializan a entero.
fn cr_call_graph(
    program: &Program,
) -> (
    std::collections::HashSet<String>,
    std::collections::HashSet<String>,
) {
    let mut direct_callers: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut dyn_named: std::collections::HashSet<String> = std::collections::HashSet::new();
    for func in program.funcs.values() {
        for ins in func.instrs.iter() {
            if let Instr::Call(cn, _) = ins {
                if program.funcs.contains_key(cn) {
                    direct_callers.insert(cn.clone());
                }
            }
            if let Instr::FuncRef(n) = ins {
                dyn_named.insert(n.clone());
            }
        }
        // Walk de pila abstracta para capturar el ConstStr con el nombre de
        // función en lanzamientos dinámicos (hilos/corutinas/mutex).
        let mut vstack: Vec<Option<String>> = Vec::new();
        for ins in func.instrs.iter() {
            match ins {
                Instr::ConstStr(s) => vstack.push(Some(s.clone())),
                Instr::Call(hl, argc) => {
                    let name_pos: Option<usize> = match hl.as_str() {
                        "__hilo_lanzar" | "__tarea_lanzar" | "__thread_spawn" | "__task_spawn"
                        | "__coro_crear" | "__coro_create" => Some(0),
                        "__mutex_bloquear" | "__mutex_lock" => Some(1),
                        _ => None,
                    };
                    let mut popped: Vec<Option<String>> = Vec::new();
                    for _ in 0..*argc {
                        popped.push(vstack.pop().unwrap_or(None));
                    }
                    if let Some(p) = name_pos {
                        if p < popped.len() {
                            if let Some(n) = popped[popped.len() - 1 - p].as_ref() {
                                dyn_named.insert(n.clone());
                            }
                        }
                    }
                    vstack.push(None);
                }
                Instr::Label(_) | Instr::Jmp(_) => vstack.clear(),
                Instr::JmpIf(_) => {
                    vstack.pop();
                }
                other => {
                    let (pops, pushes) = instr_pops_pushes(other);
                    for _ in 0..pops {
                        vstack.pop();
                    }
                    for _ in 0..pushes {
                        vstack.push(None);
                    }
                }
            }
        }
    }
    let _ = &direct_callers;
    (direct_callers, dyn_named)
}

/// v3.5.28: params enteros (Cranelift) — punto fijo interprocedural réplica
/// de `int_promotion_analysis` del backend C: un parámetro es entero si
/// NINGÚN llamador estático le pasa algo no-entero y no se reasigna en el
/// cuerpo. Las funciones dinámicas (dyn_named) quedan excluidas de antemano.
/// Los Loads de LOCALES resuelven vía `int_vars_by_name` (con las mismas
/// exclusiones que compile_body) para que `f(k)` con k local entero no
/// despromueva el parámetro del callee.
fn cr_params_int_analysis(
    program: &Program,
    dyn_named: &std::collections::HashSet<String>,
    global_names: &std::collections::HashSet<String>,
    captures: &HashMap<String, HashMap<String, String>>,
    cap_cells: &std::collections::HashSet<String>,
) -> HashMap<String, Vec<bool>> {
    #[derive(Clone, Copy, PartialEq)]
    enum K {
        Int,
        Not,
    }
    let mut params_int: HashMap<String, Vec<bool>> = HashMap::new();
    let mut int_locals: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
    for (fname, func) in &program.funcs {
        params_int.insert(fname.clone(), vec![true; func.params.len()]);
        // conocimiento de locales enteros (para kinds de argumentos)
        let mut excl: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ins in func.instrs.iter() {
            if let Instr::MakeRef(rt) = ins {
                excl.insert(rt.clone());
            }
        }
        for g in global_names {
            excl.insert(g.clone());
        }
        if let Some(cm) = captures.get(fname) {
            for v in cm.keys() {
                excl.insert(v.clone());
            }
        }
        for c in cap_cells {
            if let Some(rest) = c.strip_prefix(&format!("{}::", fname)) {
                excl.insert(rest.to_string());
            }
        }
        let arrs_f = arr_vars_by_name(func, &excl);
        int_locals.insert(fname.clone(), int_vars_by_name(func, &excl, &arrs_f));
    }
    for d in dyn_named {
        if let Some(v) = params_int.get_mut(d) {
            for b in v.iter_mut() {
                *b = false;
            }
        }
    }
    // v3.5.28-FIX: plegar las exclusiones de compile_body DENTRO de
    // params_int para que llamador y llamado decidan EXACTAMENTE lo mismo
    // (antes divergían: un param cuyo nombre colisiona con una global era
    // "int" para el llamador y handle para el llamado → ABI roto → segfault).
    // Criterio idéntico al de compile_body: int_exclude ∪ stored_names.
    for (fname, func) in &program.funcs {
        let mut excl: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ins in func.instrs.iter() {
            if let Instr::MakeRef(rt) = ins {
                excl.insert(rt.clone());
            }
        }
        for g in global_names {
            excl.insert(g.clone());
        }
        if let Some(cm) = captures.get(fname) {
            for v in cm.keys() {
                excl.insert(v.clone());
            }
        }
        for c in cap_cells {
            if let Some(rest) = c.strip_prefix(&format!("{}::", fname)) {
                excl.insert(rest.to_string());
            }
        }
        let mut stored: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ins in func.instrs.iter() {
            if let Instr::Store(sn) | Instr::StoreLocal(sn) = ins {
                stored.insert(sn.clone());
            }
        }
        if let Some(v) = params_int.get_mut(fname) {
            for (j, pn) in func.params.iter().enumerate() {
                if excl.contains(pn) || stored.contains(pn) {
                    if let Some(b) = v.get_mut(j) {
                        *b = false;
                    }
                }
            }
        }
    }
    let mut changed = true;
    let mut guard = 0;
    while changed && guard < 64 {
        changed = false;
        guard += 1;
        for (fname, func) in &program.funcs {
            let pm = params_int.get(fname).cloned().unwrap_or_default();
            let locals = int_locals.get(fname);
            let mut pos_of: HashMap<String, usize> = HashMap::new();
            for (j, p) in func.params.iter().enumerate() {
                pos_of.insert(p.clone(), j);
            }
            let mut st: Vec<K> = Vec::new();
            for ins in func.instrs.iter() {
                match ins {
                    Instr::ConstInt(_) => st.push(K::Int),
                    Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_) => {
                        st.push(K::Not)
                    }
                    Instr::Load(n) => {
                        let k = if let Some(&j) = pos_of.get(n) {
                            if pm.get(j).copied().unwrap_or(false) {
                                K::Int
                            } else {
                                K::Not
                            }
                        } else if locals.map(|s| s.contains(n)).unwrap_or(false) {
                            K::Int
                        } else {
                            K::Not
                        };
                        st.push(k);
                    }
                    Instr::Store(n) | Instr::StoreLocal(n) => {
                        st.pop();
                        if let Some(&j) = pos_of.get(n) {
                            if let Some(b) = params_int.get_mut(fname).and_then(|v| v.get_mut(j)) {
                                if *b {
                                    *b = false;
                                    changed = true;
                                }
                            }
                        }
                    }
                    Instr::MakeRef(n) => {
                        if let Some(&j) = pos_of.get(n) {
                            if let Some(b) = params_int.get_mut(fname).and_then(|v| v.get_mut(j)) {
                                if *b {
                                    *b = false;
                                    changed = true;
                                }
                            }
                        }
                        st.push(K::Not);
                    }
                    Instr::Binary(op) => {
                        let b = st.pop().unwrap_or(K::Not);
                        let a = st.pop().unwrap_or(K::Not);
                        let r = match op {
                            Op::Add
                            | Op::Sub
                            | Op::Mul
                            | Op::BitAnd
                            | Op::BitOr
                            | Op::BitXor
                            | Op::ShiftLeft
                            | Op::ShiftRight => {
                                if a == K::Int && b == K::Int {
                                    K::Int
                                } else {
                                    K::Not
                                }
                            }
                            _ => K::Not,
                        };
                        st.push(r);
                    }
                    Instr::Unary(op) => {
                        let a = st.pop().unwrap_or(K::Not);
                        let r = match op {
                            Op::Negate | Op::BitNot => {
                                if a == K::Int {
                                    K::Int
                                } else {
                                    K::Not
                                }
                            }
                            _ => K::Not,
                        };
                        st.push(r);
                    }
                    Instr::Call(cn, argc) => {
                        let mut argsk: Vec<K> = Vec::new();
                        for _ in 0..*argc {
                            argsk.push(st.pop().unwrap_or(K::Not));
                        }
                        argsk.reverse();
                        if program.funcs.contains_key(cn) && !dyn_named.contains(cn) {
                            for (j, ak) in argsk.iter().enumerate() {
                                if *ak == K::Not {
                                    if let Some(b) =
                                        params_int.get_mut(cn).and_then(|v| v.get_mut(j))
                                    {
                                        if *b {
                                            *b = false;
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                        st.push(K::Not);
                    }
                    Instr::CallValue(argc) => {
                        for _ in 0..=*argc {
                            st.pop();
                        }
                        st.push(K::Not);
                    }
                    Instr::Label(_) | Instr::Jmp(_) => st.clear(),
                    Instr::JmpIf(_) => {
                        st.pop();
                    }
                    Instr::Return | Instr::Halt => {
                        st.pop();
                        st.clear();
                    }
                    Instr::ScopePush | Instr::ScopePop | Instr::Nop | Instr::Phi(..) => {}
                    other => {
                        let (pops, pushes) = instr_pops_pushes(other);
                        for _ in 0..pops {
                            st.pop();
                        }
                        for _ in 0..pushes {
                            st.push(K::Not);
                        }
                    }
                }
            }
        }
    }
    params_int
}

/// v3.5.28: funciones que SIEMPRE devuelven entero (Cranelift) — punto fijo
/// réplica de `returns_int_analysis` del backend C. Requisitos: todos los
/// Return devuelven un valor demostrablemente entero, hay al menos un Return,
/// el flujo no puede salir "por el final" sin Return (alive-scan), y la
/// función no es dinámica (dyn_named: hilos/FuncRef exigen handle).
fn cr_returns_int_analysis(
    program: &Program,
    dyn_named: &std::collections::HashSet<String>,
    params_int: &HashMap<String, Vec<bool>>,
    global_names: &std::collections::HashSet<String>,
    captures: &HashMap<String, HashMap<String, String>>,
    cap_cells: &std::collections::HashSet<String>,
) -> HashMap<String, bool> {
    #[derive(Clone, Copy, PartialEq)]
    enum K {
        Int,
        Not,
    }
    // conocimiento de locales enteros por función (mismas exclusiones que
    // compile_body)
    let mut int_locals: HashMap<String, std::collections::HashSet<String>> = HashMap::new();
    for (fname, func) in &program.funcs {
        let mut excl: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ins in func.instrs.iter() {
            if let Instr::MakeRef(rt) = ins {
                excl.insert(rt.clone());
            }
        }
        for g in global_names {
            excl.insert(g.clone());
        }
        if let Some(cm) = captures.get(fname) {
            for v in cm.keys() {
                excl.insert(v.clone());
            }
        }
        for c in cap_cells {
            if let Some(rest) = c.strip_prefix(&format!("{}::", fname)) {
                excl.insert(rest.to_string());
            }
        }
        let arrs_f = arr_vars_by_name(func, &excl);
        int_locals.insert(fname.clone(), int_vars_by_name(func, &excl, &arrs_f));
    }
    let mut ret: HashMap<String, bool> = HashMap::new();
    let mut has_ret: HashMap<String, bool> = HashMap::new();
    for (fname, func) in &program.funcs {
        // alive-scan: si el flujo puede llegar al final de la lista de
        // instrucciones sin Return/Halt, la función devuelve void implícito.
        let mut alive = true;
        for ins in func.instrs.iter() {
            match ins {
                Instr::Return | Instr::Halt => alive = false,
                Instr::Label(_) => alive = true,
                _ => {}
            }
        }
        let eligible = !alive && !dyn_named.contains(fname);
        ret.insert(fname.clone(), eligible);
        has_ret.insert(fname.clone(), false);
    }
    let mut changed = true;
    let mut guard = 0;
    while changed && guard < 64 {
        changed = false;
        guard += 1;
        for (fname, func) in &program.funcs {
            // ret es monótono decreciente: una función ya descartada nunca
            // vuelve a ser candidata — saltar su recorrido es seguro.
            if !ret.get(fname).copied().unwrap_or(false) {
                continue;
            }
            let pm = params_int.get(fname).cloned().unwrap_or_default();
            let locals = int_locals.get(fname);
            let mut pos_of: HashMap<String, usize> = HashMap::new();
            for (j, p) in func.params.iter().enumerate() {
                pos_of.insert(p.clone(), j);
            }
            let mut st: Vec<K> = Vec::new();
            for ins in func.instrs.iter() {
                match ins {
                    Instr::ConstInt(_) => st.push(K::Int),
                    Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_) => {
                        st.push(K::Not)
                    }
                    Instr::Load(n) => {
                        let k = if let Some(&j) = pos_of.get(n) {
                            if pm.get(j).copied().unwrap_or(false) {
                                K::Int
                            } else {
                                K::Not
                            }
                        } else if locals.map(|s| s.contains(n)).unwrap_or(false) {
                            K::Int
                        } else {
                            K::Not
                        };
                        st.push(k);
                    }
                    Instr::Store(_) | Instr::StoreLocal(_) => {
                        st.pop();
                    }
                    Instr::MakeRef(_) => st.push(K::Not),
                    Instr::Binary(op) => {
                        let b = st.pop().unwrap_or(K::Not);
                        let a = st.pop().unwrap_or(K::Not);
                        let r = match op {
                            Op::Add
                            | Op::Sub
                            | Op::Mul
                            | Op::BitAnd
                            | Op::BitOr
                            | Op::BitXor
                            | Op::ShiftLeft
                            | Op::ShiftRight => {
                                if a == K::Int && b == K::Int {
                                    K::Int
                                } else {
                                    K::Not
                                }
                            }
                            _ => K::Not,
                        };
                        st.push(r);
                    }
                    Instr::Unary(op) => {
                        let a = st.pop().unwrap_or(K::Not);
                        let r = match op {
                            Op::Negate | Op::BitNot => {
                                if a == K::Int {
                                    K::Int
                                } else {
                                    K::Not
                                }
                            }
                            _ => K::Not,
                        };
                        st.push(r);
                    }
                    Instr::Call(cn, argc) => {
                        for _ in 0..*argc {
                            st.pop();
                        }
                        let k = if ret.get(cn).copied().unwrap_or(false) {
                            K::Int
                        } else {
                            K::Not
                        };
                        st.push(k);
                    }
                    Instr::CallValue(argc) => {
                        for _ in 0..=*argc {
                            st.pop();
                        }
                        st.push(K::Not);
                    }
                    Instr::Label(_) | Instr::Jmp(_) => st.clear(),
                    Instr::JmpIf(_) => {
                        st.pop();
                    }
                    Instr::Return => {
                        let v = st.pop().unwrap_or(K::Not);
                        if v == K::Not {
                            if let Some(b) = ret.get_mut(fname) {
                                if *b {
                                    *b = false;
                                    changed = true;
                                }
                            }
                        } else {
                            if let Some(b) = has_ret.get_mut(fname) {
                                *b = true;
                            }
                        }
                        st.clear();
                    }
                    Instr::Halt => {
                        st.pop();
                        if let Some(b) = ret.get_mut(fname) {
                            if *b {
                                *b = false;
                                changed = true;
                            }
                        }
                        st.clear();
                    }
                    Instr::ScopePush | Instr::ScopePop | Instr::Nop | Instr::Phi(..) => {}
                    other => {
                        let (pops, pushes) = instr_pops_pushes(other);
                        for _ in 0..pops {
                            st.pop();
                        }
                        for _ in 0..pushes {
                            st.push(K::Not);
                        }
                    }
                }
            }
        }
    }
    for (f, b) in ret.iter_mut() {
        *b = *b && has_ret.get(f).copied().unwrap_or(false);
    }
    ret
}

fn int_vars_by_name(
    func: &LumenFunc,
    exclude: &std::collections::HashSet<String>,
    arr_int: &std::collections::HashSet<String>,
) -> std::collections::HashSet<String> {
    #[derive(Clone, Copy, PartialEq)]
    enum K {
        Int,
        Not,
    }
    // candidatos: nombres con exactamente un StoreLocal
    let mut decl_count: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for ins in &func.instrs {
        if let Instr::StoreLocal(n) = ins {
            *decl_count.entry(n.clone()).or_insert(0) += 1;
        }
    }
    let mut is_int: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
    for (n, c) in &decl_count {
        if *c == 1 && !exclude.contains(n) {
            is_int.insert(n.clone(), true);
        }
    }
    loop {
        let mut changed = false;
        // v3.5.29: la pila lleva (kind, variable fuente del Load) para que
        // ArrayGet sobre un array de enteros promovible propague Int de forma
        // SONDA (acc = acc + xs[j]). Sin fuente → conservador Not.
        let mut st: Vec<(K, Option<String>)> = Vec::new();
        for ins in &func.instrs {
            match ins {
                Instr::ConstInt(_) => st.push((K::Int, None)),
                Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_) => {
                    st.push((K::Not, None))
                }
                Instr::Load(n) => {
                    let k = if *is_int.get(n).unwrap_or(&false) {
                        K::Int
                    } else {
                        K::Not
                    };
                    st.push((k, Some(n.clone())));
                }
                Instr::StoreLocal(n) | Instr::Store(n) => {
                    let (v, _) = st.pop().unwrap_or((K::Not, None));
                    if v == K::Not {
                        if let Some(b) = is_int.get_mut(n) {
                            if *b {
                                *b = false;
                                changed = true;
                            }
                        }
                    }
                }
                Instr::ArrayGet => {
                    st.pop(); // índice
                    let (_, src) = st.pop().unwrap_or((K::Not, None));
                    let is_int_arr = src.map(|v| arr_int.contains(&v)).unwrap_or(false);
                    st.push(if is_int_arr {
                        (K::Int, None)
                    } else {
                        (K::Not, None)
                    });
                }
                Instr::ArrayLen => {
                    st.pop();
                    st.push((K::Int, None));
                }
                Instr::Binary(op) => {
                    let (b, _) = st.pop().unwrap_or((K::Not, None));
                    let (a, _) = st.pop().unwrap_or((K::Not, None));
                    let r = match op {
                        Op::Add
                        | Op::Sub
                        | Op::Mul
                        | Op::BitAnd
                        | Op::BitOr
                        | Op::BitXor
                        | Op::ShiftLeft
                        | Op::ShiftRight => {
                            if a == K::Int && b == K::Int {
                                K::Int
                            } else {
                                K::Not
                            }
                        }
                        _ => K::Not,
                    };
                    st.push((r, None));
                }
                Instr::Unary(op) => {
                    let (a, _) = st.pop().unwrap_or((K::Not, None));
                    let r = match op {
                        Op::Negate | Op::BitNot => {
                            if a == K::Int {
                                K::Int
                            } else {
                                K::Not
                            }
                        }
                        _ => K::Not,
                    };
                    st.push((r, None));
                }
                Instr::Call(cn, argc) => {
                    for _ in 0..*argc {
                        st.pop();
                    }
                    // v3.5.28: builtins que SIEMPRE devuelven entero propagan
                    // Int (paridad int_promotion_analysis del backend C), así
                    // acumuladores `total = total + largo(s)` se promocionan.
                    let int_ret = matches!(
                        cn.as_str(),
                        "largo" | "len" | "__str_len" | "__str_longitud"
                    );
                    st.push(if int_ret {
                        (K::Int, None)
                    } else {
                        (K::Not, None)
                    });
                }
                Instr::CallValue(argc) => {
                    for _ in 0..=*argc {
                        st.pop();
                    }
                    st.push((K::Not, None));
                }
                Instr::Label(_) | Instr::Jmp(_) => st.clear(),
                Instr::JmpIf(_) => {
                    st.pop();
                }
                Instr::Return | Instr::Halt => {
                    st.pop();
                    st.clear();
                }
                other => {
                    let (pops, pushes) = instr_pops_pushes(other);
                    for _ in 0..pops {
                        st.pop();
                    }
                    for _ in 0..pushes {
                        st.push((K::Not, None));
                    }
                }
            }
        }
        if !changed {
            break;
        }
    }
    is_int
        .into_iter()
        .filter(|(_, b)| *b)
        .map(|(n, _)| n)
        .collect()
}

fn returns_int_analysis(
    program: &Program,
    int_proms: &BTreeMap<String, HashMap<String, (String, Option<String>)>>,
) -> std::collections::HashSet<String> {
    #[derive(Clone, Copy, PartialEq)]
    enum K {
        Int,
        Not,
    }
    let mut ri: std::collections::HashSet<String> = program.funcs.keys().cloned().collect();
    loop {
        let mut changed = false;
        for (name, func) in &program.funcs {
            if !ri.contains(name) {
                continue;
            }
            let plan_map: std::collections::HashMap<usize, String> =
                std::collections::HashMap::new();
            let _ = plan_map;
            let prom = int_proms.get(name);
            let mut st: Vec<K> = Vec::new();
            let mut demote = false;
            let mut has_return = false;
            // sin plan por instrucción aquí: los Loads se tratan de forma
            // conservadora según sean keys de enteros promovidos (params o
            // locales) — se aproxima por nombre vía sufijo "::var".
            let int_keys: std::collections::HashSet<String> = prom
                .map(|m| m.keys().cloned().collect())
                .unwrap_or_default();
            for ins in &func.instrs {
                match ins {
                    Instr::ConstInt(_) => st.push(K::Int),
                    Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_) => {
                        st.push(K::Not)
                    }
                    Instr::Load(n) => {
                        // aproximación: entero si el nombre (sin el índice de
                        // sombreado) aparece entre las keys enteras de la fn
                        let is_int = int_keys
                            .iter()
                            .any(|k| k.ends_with(&format!("::{}", n)) || k == n);
                        st.push(if is_int { K::Int } else { K::Not });
                    }
                    Instr::Binary(op) => {
                        let b = st.pop().unwrap_or(K::Not);
                        let a = st.pop().unwrap_or(K::Not);
                        let r = match op {
                            Op::Add
                            | Op::Sub
                            | Op::Mul
                            | Op::BitAnd
                            | Op::BitOr
                            | Op::BitXor
                            | Op::ShiftLeft
                            | Op::ShiftRight => {
                                if a == K::Int && b == K::Int {
                                    K::Int
                                } else {
                                    K::Not
                                }
                            }
                            _ => K::Not,
                        };
                        st.push(r);
                    }
                    Instr::Unary(op) => {
                        let a = st.pop().unwrap_or(K::Not);
                        let r = match op {
                            Op::Negate | Op::BitNot => {
                                if a == K::Int {
                                    K::Int
                                } else {
                                    K::Not
                                }
                            }
                            _ => K::Not,
                        };
                        st.push(r);
                    }
                    Instr::Call(cn, argc) => {
                        for _ in 0..*argc {
                            st.pop();
                        }
                        st.push(if ri.contains(cn) { K::Int } else { K::Not });
                    }
                    Instr::CallValue(argc) => {
                        for _ in 0..=*argc {
                            st.pop();
                        }
                        st.push(K::Not);
                    }
                    Instr::Return => {
                        has_return = true;
                        match st.pop() {
                            Some(K::Int) => {}
                            _ => {
                                demote = true;
                            }
                        }
                        st.clear();
                    }
                    Instr::Store(_) | Instr::StoreLocal(_) => {
                        st.pop();
                    }
                    Instr::Label(_) | Instr::Jmp(_) => st.clear(),
                    Instr::JmpIf(_) => {
                        st.pop();
                    }
                    Instr::Halt => {
                        demote = true; // no retorna valor entero
                    }
                    other => {
                        let (pops, pushes) = instr_pops_pushes(other);
                        for _ in 0..pops {
                            st.pop();
                        }
                        for _ in 0..pushes {
                            st.push(K::Not);
                        }
                    }
                }
                if demote {
                    break;
                }
            }
            if demote || !has_return {
                ri.remove(name);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    ri
}

fn no_mutate_analysis(program: &Program) -> std::collections::HashSet<String> {
    let mut mutates: std::collections::HashSet<String> = std::collections::HashSet::new();
    // 1) mutación directa + llamados con MakeRef en sus argumentos
    for (name, func) in &program.funcs {
        let mut direct = false;
        let mut vstack: Vec<u8> = Vec::new(); // 1 = valor viene de MakeRef
        for ins in &func.instrs {
            match ins {
                Instr::ArrayPushVar(_) | Instr::ArraySet | Instr::StructSet | Instr::MakeRef(_) => {
                    direct = true;
                }
                _ => {}
            }
            match ins {
                Instr::MakeRef(_) => vstack.push(1),
                Instr::ConstInt(_)
                | Instr::ConstFloat(_)
                | Instr::ConstStr(_)
                | Instr::ConstBool(_)
                | Instr::Read
                | Instr::FuncRef(_) => vstack.push(0),
                Instr::Load(_) => vstack.push(0),
                Instr::Call(cn, argc) => {
                    let mut got_ref = false;
                    for _ in 0..*argc {
                        if vstack.pop() == Some(1) {
                            got_ref = true;
                        }
                    }
                    if got_ref && program.funcs.contains_key(cn) {
                        mutates.insert(cn.clone());
                    }
                    vstack.push(0);
                }
                Instr::CallValue(argc) => {
                    for _ in 0..=*argc {
                        vstack.pop();
                    }
                    vstack.push(0);
                }
                Instr::Label(_) | Instr::Jmp(_) => vstack.clear(),
                Instr::JmpIf(_) => {
                    vstack.pop();
                }
                other => {
                    let d = instr_depth_delta(other);
                    let mut pops = if d < 0 { -d } else { 0 };
                    while pops > 0 {
                        vstack.pop();
                        pops -= 1;
                    }
                    vstack.extend(std::iter::repeat_n(0, d.max(0) as usize));
                }
            }
        }
        if direct {
            mutates.insert(name.clone());
        }
    }
    // 2) punto fijo: quien llama a una mutante, muta (puede delegar el arg)
    loop {
        let mut changed = false;
        for (name, func) in &program.funcs {
            if mutates.contains(name) {
                continue;
            }
            let via_call = func.instrs.iter().any(|ins| {
                if let Instr::Call(cn, _) = ins {
                    program.funcs.contains_key(cn) && mutates.contains(cn)
                } else {
                    matches!(ins, Instr::CallValue(_))
                }
            });
            if via_call {
                mutates.insert(name.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    program
        .funcs
        .keys()
        .filter(|k| !mutates.contains(*k))
        .cloned()
        .collect()
}

fn no_throw_analysis(program: &Program) -> std::collections::HashSet<String> {
    // Punto fijo DECRECIENTE (siembra optimista): todas empiezan como
    // no-lanzantes y se degradan las que demostrablemente lanzan. Así la
    // recursión se resuelve (fib se llama a sí misma y ninguna lanza).
    let mut no_throw: std::collections::HashSet<String> = program.funcs.keys().cloned().collect();
    loop {
        let mut changed = false;
        for (name, func) in &program.funcs {
            if !no_throw.contains(name) {
                continue;
            }
            let mut depth = 0usize;
            let mut throws = false;
            for ins in &func.instrs {
                match ins {
                    Instr::PushHandler(_) => depth += 1,
                    Instr::PopHandler => depth = depth.saturating_sub(1),
                    _ => {}
                }
                if depth == 0 && instr_throws(ins, &no_throw, program) {
                    throws = true;
                    break;
                }
            }
            if throws {
                no_throw.remove(name);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    no_throw
}

#[allow(clippy::type_complexity)]
fn int_promotion_analysis(
    program: &Program,
    var_plans: &HashMap<String, HashMap<usize, String>>,
) -> (
    HashMap<String, HashMap<String, bool>>,
    HashMap<String, Vec<bool>>,
) {
    let mut locals_int: HashMap<String, HashMap<String, bool>> = HashMap::new();
    let mut params_int: HashMap<String, Vec<bool>> = HashMap::new();
    // key de parámetro → (fn, pos)
    let mut pkey_of: HashMap<String, HashMap<String, usize>> = HashMap::new();
    for (fname, func) in &program.funcs {
        let plan = match var_plans.get(fname) {
            Some(p) => p,
            None => continue,
        };
        let mut lm: HashMap<String, bool> = HashMap::new();
        for k in plan.values() {
            lm.insert(k.clone(), k.contains('#'));
        }
        locals_int.insert(fname.clone(), lm);
        params_int.insert(fname.clone(), vec![true; func.params.len()]);
        let mut pk: HashMap<String, usize> = HashMap::new();
        for (pos, p) in func.params.iter().enumerate() {
            pk.insert(format!("{}::{}", fname, p), pos);
        }
        pkey_of.insert(fname.clone(), pk);
    }

    let mut changed = true;
    let mut guard = 0;
    while changed && guard < 64 {
        changed = false;
        guard += 1;
        for (fname, func) in &program.funcs {
            let plan = match var_plans.get(fname) {
                Some(p) => p,
                None => continue,
            };
            let lm = match locals_int.get(fname) {
                Some(m) => m.clone(),
                None => continue,
            };
            let pm = match params_int.get(fname) {
                Some(m) => m.clone(),
                None => continue,
            };
            let pk = &pkey_of[fname];
            let mut st: Vec<IKind> = Vec::new();
            for (i, ins) in func.instrs.iter().enumerate() {
                match ins {
                    Instr::ConstInt(_) => st.push(IKind::Int),
                    Instr::ConstFloat(_) | Instr::ConstStr(_) | Instr::ConstBool(_) => {
                        st.push(IKind::Not)
                    }
                    Instr::Load(n) => {
                        let k = plan.get(&i).cloned().unwrap_or_else(|| n.clone());
                        // Primero el mapa de PARAMS (las claves de param también
                        // están en el mapa de locales con `false` y lo
                        // enmascararían).
                        let kind = if let Some(pos) = pk.get(&k) {
                            if pm.get(*pos).copied().unwrap_or(false) {
                                IKind::Int
                            } else {
                                IKind::Not
                            }
                        } else if let Some(b) = lm.get(&k) {
                            if *b {
                                IKind::Int
                            } else {
                                IKind::Not
                            }
                        } else {
                            IKind::Not
                        };
                        st.push(kind);
                    }
                    Instr::Store(n) | Instr::StoreLocal(n) => {
                        let k = plan.get(&i).cloned().unwrap_or_else(|| n.clone());
                        let v = st.pop().unwrap_or(IKind::Not);
                        if let Some(b) = locals_int.get_mut(fname).and_then(|m| m.get_mut(&k)) {
                            if v == IKind::Not && *b {
                                *b = false;
                                changed = true;
                            }
                        } else if let Some(pos) = pk.get(&k) {
                            // parámetro reasignado → no promocionable (fase 1)
                            if let Some(b) = params_int.get_mut(fname).and_then(|m| m.get_mut(*pos))
                            {
                                if *b {
                                    *b = false;
                                    changed = true;
                                }
                            }
                        }
                    }
                    Instr::Binary(op) => {
                        let b = st.pop().unwrap_or(IKind::Not);
                        let a = st.pop().unwrap_or(IKind::Not);
                        let r = match op {
                            Op::Add
                            | Op::Sub
                            | Op::Mul
                            | Op::BitAnd
                            | Op::BitOr
                            | Op::BitXor
                            | Op::ShiftLeft
                            | Op::ShiftRight => {
                                if a == IKind::Int && b == IKind::Int {
                                    IKind::Int
                                } else {
                                    IKind::Not
                                }
                            }
                            _ => IKind::Not,
                        };
                        st.push(r);
                    }
                    Instr::Unary(op) => {
                        let a = st.pop().unwrap_or(IKind::Not);
                        let r = match op {
                            Op::Negate | Op::BitNot => {
                                if a == IKind::Int {
                                    IKind::Int
                                } else {
                                    IKind::Not
                                }
                            }
                            _ => IKind::Not,
                        };
                        st.push(r);
                    }
                    Instr::Call(cn, argc) => {
                        // args en la pila abstracta: últimos argc (orden directo)
                        let mut args_k: Vec<IKind> = Vec::new();
                        for _ in 0..*argc {
                            args_k.push(st.pop().unwrap_or(IKind::Not));
                        }
                        args_k.reverse();
                        if let (Some(callee), Some(cpm)) =
                            (program.funcs.get(cn), params_int.get_mut(cn))
                        {
                            for (j, ak) in args_k.iter().enumerate() {
                                if j < callee.params.len() && *ak == IKind::Not {
                                    if let Some(b) = cpm.get_mut(j) {
                                        if *b {
                                            *b = false;
                                            changed = true;
                                        }
                                    }
                                }
                            }
                        }
                        // v3.5.27: builtins que SIEMPRE devuelven entero
                        // propagan Int (largo nunca lanza y devuelve entero
                        // para cualquier tipo). Así `total = total + largo(s)`
                        // promociona total a long long nativo.
                        let ret_int = !program.funcs.contains_key(cn)
                            && matches!(
                                cn.as_str(),
                                "largo" | "len" | "__str_len" | "__str_longitud"
                            );
                        st.push(if ret_int { IKind::Int } else { IKind::Not });
                    }
                    Instr::CallValue(argc) => {
                        for _ in 0..=*argc {
                            st.pop();
                        }
                        st.push(IKind::Not);
                    }
                    Instr::Label(_) | Instr::Jmp(_) => st.clear(),
                    Instr::JmpIf(_) => {
                        st.pop();
                    }
                    Instr::Return | Instr::Halt => {
                        st.pop();
                        st.clear();
                    }
                    Instr::ScopePush | Instr::ScopePop | Instr::Nop | Instr::Phi(..) => {}
                    _ => st.clear(),
                }
            }
        }
    }

    // un local promocionable necesita al menos una escritura
    for (fname, func) in &program.funcs {
        let plan = match var_plans.get(fname) {
            Some(p) => p,
            None => continue,
        };
        let mut written: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (i, ins) in func.instrs.iter().enumerate() {
            if let Instr::Store(n) | Instr::StoreLocal(n) = ins {
                written.insert(plan.get(&i).cloned().unwrap_or_else(|| n.clone()));
            }
        }
        if let Some(lm) = locals_int.get_mut(fname) {
            for (k, b) in lm.iter_mut() {
                if *b && !written.contains(k) {
                    *b = false;
                }
            }
        }
    }
    (locals_int, params_int)
}

fn xe_spill(
    s: &mut String,
    estack: &mut Vec<(String, bool, bool, u8, bool)>,
    handler_labels: &[usize],
) {
    let risky = estack.iter().any(|(_, r, _, _, _)| *r);
    for (e, _, _, kind, _) in estack.drain(..) {
        // v3.5.21: los enteros sin tag vuelven al mundo Val al spill-ear.
        let ve = if kind == 0 {
            e
        } else {
            format!("_v_int({})", e)
        };
        s.push_str(&format!("  PUSH({});\n", ve));
    }
    if risky {
        xe_errchk(s, handler_labels);
    }
}

fn xe_errchk(s: &mut String, handler_labels: &[usize]) {
    match handler_labels.last() {
        Some(l) => s.push_str(&format!(
            "  if (__builtin_expect(_err,0)) {{ _hn--; SP = _h_sp[_hn]; PUSH(_v_str(_last_err_msg)); _err = 0; goto L_{}; }}\n",
            l
        )),
        None => s.push_str("  if (__builtin_expect(_err,0)) return _v_void();\n"),
    }
}

#[allow(clippy::too_many_arguments)]
fn emit_func(
    name: &str,
    func: &LumenFunc,
    program: &Program,
    name_sets: &BTreeMap<String, Vec<String>>,
    gv_of: &dyn Fn(&str) -> String,
    renames: &HashMap<String, HashMap<String, String>>,
    plan: &HashMap<usize, String>,
    // v3.5.19: key → registro C local (promoción), y mapa global por función
    // (para filtrar el save/restore del callee en las llamadas).
    promoted: &HashMap<String, String>,
    _promoted_all: &BTreeMap<String, HashMap<String, String>>,
    // v3.5.21: key → (nombre C `long long`, init opcional para params).
    prom_int: &HashMap<String, (String, Option<String>)>,
    // v3.5.22: keys capturadas por funciones anidadas (nunca movibles).
    cap_keys: &std::collections::HashSet<String>,
    // v3.5.23: funciones con ABI de argumentos en registros C.
    reg_abi: &std::collections::HashSet<String>,
    // v3.5.23: mapa global de enteros promovidos (para tipos de params del
    // callee en llamadas con ABI de registros).
    int_proms_all: &BTreeMap<String, HashMap<String, (String, Option<String>)>>,
    // v3.5.24: funciones que no lanzan (sin ERRCHK, inlineables).
    no_throw: &std::collections::HashSet<String>,
    // v3.5.24: funciones que no mutan (args compartibles sin _dcp).
    no_mutate: &std::collections::HashSet<String>,
    // v3.5.24: funciones que siempre devuelven entero (retorno long long).
    returns_int: &std::collections::HashSet<String>,
    // v3.5.26: arrays de enteros promovidos a arrays nativos.
    arr_vars: &std::collections::HashSet<String>,
    // v3.5.42: closure → celdas finales capturadas (snapshot por instancia).
    fref_cells: &HashMap<String, Vec<String>>,
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
    // v3.5.23: ABI de registros — los params llegan como parámetros C.
    let is_reg = reg_abi.contains(name) && !func.params.is_empty();
    let mut pvmap: HashMap<String, String> = HashMap::new();
    let header_params = if is_reg {
        let mut ps: Vec<String> = Vec::new();
        for (pos, p) in func.params.iter().enumerate() {
            let key = format!("{}::{}", name, p);
            match prom_int.get(&key) {
                Some((ll, Some(_))) => ps.push(format!("long long {}", ll)),
                _ => {
                    let pv = format!("_pv{}", pos);
                    pvmap.insert(key, pv.clone());
                    ps.push(format!("Val {}", pv));
                }
            }
        }
        ps.join(", ")
    } else {
        String::new()
    };
    let fn_no_throw = no_throw.contains(name);
    let attr = if fn_no_throw { "" } else { "LUMEN_NOINLINE " };
    let fn_ret_int = returns_int.contains(name);
    let ret_t = if fn_ret_int { "long long" } else { "Val" };
    s.push_str(&format!(
        "static {}{} _f_{}({}) {{\n  int _sb = SP; /* v3.5.13: base para descartar residuos */\n",
        attr,
        ret_t,
        mangle(name),
        if header_params.is_empty() {
            "void".to_string()
        } else {
            header_params
        }
    ));
    // v3.5.19: registros locales promovidos (viven en el stack de C; GCC los
    // mantiene en registros — los bucles enteros se optimizan como C nativo).
    if !promoted.is_empty() {
        let mut kvs: Vec<(&String, &String)> = promoted.iter().collect();
        kvs.sort_by(|a, b| a.1.cmp(b.1));
        for (_, lv) in kvs {
            s.push_str(&format!("  Val {} = _v_void();\n", lv));
        }
    }
    // v3.5.21: enteros promovidos a long long nativo (locales = 0; params
    // = copia del slot gv al entrar).
    if !prom_int.is_empty() {
        let mut kvs: Vec<(&String, &(String, Option<String>))> = prom_int.iter().collect();
        kvs.sort_by(|a, b| (a.1).0.cmp(&(b.1).0));
        for (_, (lv, init)) in kvs {
            match init {
                // v3.5.23: con ABI de registros el int-param YA es el
                // parámetro C (no se lee de gv).
                Some(pk) => {
                    if !is_reg {
                        s.push_str(&format!("  long long {} = {}.i;\n", lv, gv_of(pk)))
                    }
                }
                None => s.push_str(&format!("  long long {} = 0;\n", lv)),
            }
        }
    }
    // v3.5.26: arrays de enteros promovidos a arrays nativos (ptr/len/cap).
    if !arr_vars.is_empty() {
        let mut avs: Vec<&String> = arr_vars.iter().collect();
        avs.sort();
        for v in avs {
            s.push_str(&format!(
                "  long long* {}_d = NULL; long long {}_n = 0; long long {}_c = 0;\n",
                v, v, v
            ));
        }
    }

    // Resolvedor de slot por instrucción (params renombrados + sombreado)
    let var_at =
        |i: usize, n: &str| -> String { plan.get(&i).cloned().unwrap_or_else(|| n.to_string()) };
    // v3.5.19: expresión de slot final: registro C promovido o celda gv[].
    let slot_of = |i: usize, n: &str| -> String {
        let k = var_at(i, n);
        promoted.get(&k).cloned().unwrap_or_else(|| gv_of(&k))
    };
    let mut handler_labels: Vec<usize> = Vec::new();
    // v3.5.20: EMISOR POR PILA DE EXPRESIONES ("ultra"). Los tramos rectos
    // (Const/Load/Binary/Unary/Store/JmpIf...) se acumulan como expresiones
    // C y se materializan en UNA sentencia: gcc mantiene todo en registros
    // y el tráfico de la pila de valores ST[] desaparece de los bucles.
    // Move-semantics: Store y argumentos de llamada ya NO deep-copian
    // (paridad con la VM, que comparte vía Arc). Las instrucciones que
    // necesitan ST[] (llamadas, arrays, structs...) vacían la pila de
    // expresiones primero y usan el camino clásico.

    let instrs_all = &func.instrs;
    // v3.5.22: último uso por key (para move-semantics en args de llamada).
    let mut last_uses: HashMap<String, usize> = HashMap::new();
    for (i2, ins2) in instrs_all.iter().enumerate() {
        match ins2 {
            Instr::Load(n) | Instr::MakeRef(n) | Instr::ArrayPushVar(n) => {
                let k = var_at(i2, n.as_str());
                if !cap_keys.contains(&k) {
                    last_uses.insert(k, i2);
                }
            }
            _ => {}
        }
    }
    // (expr, risky, fresh, kind: 0=Val 1=LL 2=Bool, movable: último uso/fresco)
    let mut estack: Vec<(String, bool, bool, u8, bool)> = Vec::new();
    let mut mvac: usize = 0; // contador de temporales de args de llamada

    let mut i: usize = 0;
    while i < instrs_all.len() {
        // v3.5.26: FUSIÓN de arrays de enteros promovidos:
        //   Load xs; Load j|ConstInt; ArrayGet → lectura nativa con bounds
        //   Load xs; ArrayLen                  → xs_n directo
        if !arr_vars.is_empty() {
            if let Instr::Load(xs) = &instrs_all[i] {
                if arr_vars.contains(xs) {
                    if i + 2 < instrs_all.len() {
                        match (&instrs_all[i + 1], &instrs_all[i + 2]) {
                            (Instr::Load(j), Instr::ArrayGet) => {
                                let kj = var_at(i + 1, j);
                                if let Some((ll, _)) = prom_int.get(&kj) {
                                    let expr = format!(
                                        "({{ long long _ix = {}; if (_ix < 0 || _ix >= {}_n) _rt_throw(\"Indice fuera de rango\"); (_ix >= 0 && _ix < {}_n) ? {}_d[_ix] : 0; }})",
                                        ll, xs, xs, xs
                                    );
                                    estack.push((expr, true, true, 1, false));
                                    i += 3;
                                    continue;
                                }
                            }
                            (Instr::ConstInt(k), Instr::ArrayGet) => {
                                let expr = format!(
                                    "({{ long long _ix = {}; if (_ix < 0 || _ix >= {}_n) _rt_throw(\"Indice fuera de rango\"); (_ix >= 0 && _ix < {}_n) ? {}_d[_ix] : 0; }})",
                                    k, xs, xs, xs
                                );
                                estack.push((expr, true, true, 1, false));
                                i += 3;
                                continue;
                            }
                            _ => {}
                        }
                    }
                    if i + 1 < instrs_all.len() {
                        if let Instr::ArrayLen = &instrs_all[i + 1] {
                            estack.push((format!("{}_n", xs), false, true, 1, false));
                            i += 2;
                            continue;
                        }
                    }
                }
            }
        }
        // ────────── camino de expresiones (hot path) ──────────
        let mut xe_consumed = true;
        match &instrs_all[i] {
            Instr::ConstInt(k) => {
                // v3.5.21: entero SIN tag (kind LL); se convierte a Val solo
                // si un consumidor lo necesita.
                estack.push((format!("{}", k), false, true, 1, true));
            }
            Instr::ConstFloat(f) => {
                estack.push((format!("_v_flt({:.17e})", f), false, true, 0, true));
            }
            Instr::ConstBool(b) => {
                estack.push((
                    format!("_v_bool({})", if *b { 1 } else { 0 }),
                    false,
                    true,
                    0,
                    true,
                ));
            }
            Instr::ConstStr(t) => {
                // v3.5.27: _v_str_lit envuelve el literal SIN copiarlo al arena
                // (los textos son inmutables) — antes: strlen+alloc+memcpy por
                // iteración en bucles con literales.
                estack.push((format!("_v_str_lit(\"{}\")", esc(t)), false, true, 0, true));
            }
            Instr::Load(n) => {
                let k = var_at(i, n);
                // v3.5.26: array de enteros promovido: el valor Val no se
                // usa (las ventanas de fusión consumen el patrón); dummy.
                if arr_vars.contains(n) {
                    estack.push(("_v_void()".to_string(), false, true, 0, true));
                } else if let Some(pv) = pvmap.get(&k) {
                    // v3.5.23: param Val que llega por registro C. _deref es
                    // imprescindible: el param puede ser T_PTR (prestado mut).
                    let movable = last_uses.get(&k).copied() == Some(i);
                    estack.push((format!("_deref({})", pv), false, false, 0, movable));
                } else {
                    if let Some((ll_name, _)) = prom_int.get(&k) {
                        estack.push((ll_name.clone(), false, false, 1, true));
                    } else {
                        let movable = last_uses.get(&k).copied() == Some(i);
                        estack.push((
                            format!("_deref({})", slot_of(i, n)),
                            false,
                            false,
                            0,
                            movable,
                        ));
                    }
                }
            }
            Instr::ArrayPushVar(vname) if arr_vars.contains(vname) && estack.len() >= 2 => {
                // v3.5.26: push nativo sobre array de enteros promovido.
                let (ve, _, _, ke, _) = estack.pop().unwrap();
                let _dummy = estack.pop(); // dummy del Load xs
                xe_spill(&mut s, &mut estack, &handler_labels);
                let val_expr = if ke == 1 {
                    ve
                } else {
                    format!("_lw_h2i({})", ve)
                };
                s.push_str(&format!(
                    "  {{ long long _pv = {}; if ({}_n == {}_c) {{ {}_c = {}_c ? {}_c * 2 : 8; {}_d = (long long*)realloc({}_d, {}_c * 8); }} {}_d[{}_n++] = _pv; }}\n",
                    val_expr, vname, vname, vname, vname, vname, vname, vname, vname, vname, vname
                ));
                // El resultado de agregar() en contexto de statement es
                // residuo: NO se apila (inundaría ST). Si el valor se usara
                // como expresión, el análisis despromueve xs y este brazo no
                // se ejecuta.
            }
            Instr::MakeRef(n) => {
                estack.push((format!("_v_ptr(&{})", slot_of(i, n)), false, true, 0, true));
            }
            Instr::FuncRef(fn_name) => {
                // v3.5.42 (bug fuzz closure_multi): si el closure captura,
                // su FuncRef lleva snapshot de las celdas capturadas.
                let expr = match fref_cells.get(fn_name) {
                    Some(cells) if !cells.is_empty() => {
                        let list = cells
                            .iter()
                            .map(|c| format!("\"{}\"", esc(c)))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!(
                            "_vfref_snap(\"{}\", &_f_{}, (const char*[]){{ {} }}, {})",
                            esc(fn_name),
                            mangle(fn_name),
                            list,
                            cells.len()
                        )
                    }
                    _ => format!("_vfref(\"{}\", &_f_{})", esc(fn_name), mangle(fn_name)),
                };
                estack.push((expr, false, true, 0, true));
            }
            Instr::Binary(op) if estack.len() >= 2 => {
                let (b, rb, _fb, kb, _) = estack.pop().unwrap();
                let (a, ra, _fa, ka, _) = estack.pop().unwrap();
                let risky = ra || rb || matches!(op, Op::Div | Op::Mod);
                // v3.5.21: aritmética/comparación de enteros SIN tag → C puro.
                let arith_ll = matches!(
                    op,
                    Op::Add | Op::Sub | Op::Mul | Op::BitAnd | Op::BitOr | Op::BitXor
                );
                let shift_ll = matches!(op, Op::ShiftLeft | Op::ShiftRight);
                let cmp_ll = matches!(
                    op,
                    Op::Less
                        | Op::LessEqual
                        | Op::Greater
                        | Op::GreaterEqual
                        | Op::Equal
                        | Op::NotEqual
                );
                if ka == 1 && kb == 1 && arith_ll {
                    // wrapping exacto vía aritmética unsigned (paridad VM)
                    let expr = match op {
                        Op::Add => format!(
                            "(long long)((unsigned long long)({}) + (unsigned long long)({}))",
                            a, b
                        ),
                        Op::Sub => format!(
                            "(long long)((unsigned long long)({}) - (unsigned long long)({}))",
                            a, b
                        ),
                        Op::Mul => format!(
                            "(long long)((unsigned long long)({}) * (unsigned long long)({}))",
                            a, b
                        ),
                        Op::BitAnd => format!("({} & {})", a, b),
                        Op::BitOr => format!("({} | {})", a, b),
                        Op::BitXor => format!("({} ^ {})", a, b),
                        _ => unreachable!(),
                    };
                    estack.push((expr, false, true, 1, true));
                } else if ka == 1 && kb == 1 && shift_ll {
                    // wrapping_shl/shr de la VM = máscara de 63 bits
                    let expr = match op {
                        Op::ShiftLeft => {
                            format!("(long long)((unsigned long long)({} << (({}) & 63)))", a, b)
                        }
                        _ => format!("(({}) >> (({}) & 63))", a, b),
                    };
                    estack.push((expr, false, true, 1, true));
                } else if ka == 1 && kb == 1 && cmp_ll {
                    let cexpr = match op {
                        Op::Less => format!("(({}) < ({}))", a, b),
                        Op::LessEqual => format!("(({}) <= ({}))", a, b),
                        Op::Greater => format!("(({}) > ({}))", a, b),
                        Op::GreaterEqual => format!("(({}) >= ({}))", a, b),
                        Op::Equal => format!("(({}) == ({}))", a, b),
                        _ => format!("(({}) != ({}))", a, b),
                    };
                    estack.push((cexpr, false, true, 2, true));
                } else {
                    // mixto o no promocionable: promover LL→Val y usar _bin
                    let va = if ka == 1 { format!("_v_int({})", a) } else { a };
                    let vb = if kb == 1 { format!("_v_int({})", b) } else { b };
                    if matches!(op, Op::Concat) {
                        // v3.5.24: STR+STR directo en el camino de expresiones
                        estack.push((format!("_concat2({}, {})", va, vb), false, true, 0, true));
                    } else {
                        estack.push((
                            format!("_bin({}, {}, {})", op_code(op), va, vb),
                            risky,
                            true,
                            0,
                            true,
                        ));
                    }
                }
            }
            Instr::Unary(op) if !estack.is_empty() => {
                let (a, ra, _fa, ka, _) = estack.pop().unwrap();
                if ka == 1 && matches!(op, Op::Negate | Op::BitNot) {
                    let expr = match op {
                        // negación wrapping vía unsigned (paridad VM)
                        Op::Negate => format!("(long long)(0ULL - (unsigned long long)({}))", a),
                        _ => format!("(~({}))", a),
                    };
                    estack.push((expr, false, true, 1, true));
                } else {
                    let va = if ka == 1 { format!("_v_int({})", a) } else { a };
                    let fname = match op {
                        Op::Negate => "_neg",
                        Op::Not => "_not",
                        Op::BitNot => "_bnot",
                        _ => "_neg",
                    };
                    estack.push((format!("{}({})", fname, va), ra, true, 0, true));
                }
            }
            Instr::Store(n) | Instr::StoreLocal(n) if !estack.is_empty() => {
                let (e, r, fresh, ke, _) = estack.pop().unwrap();
                // v3.5.26: store de array promovido: el array nativo ya está
                // declarado; el valor apilado (ArrayNew/dummy) se descarta.
                if arr_vars.contains(n) {
                    let _ = (e, r, fresh, ke);
                    // nada que emitir (el valor era residuo/dummy)
                } else {
                    let k = var_at(i, n);
                    // v3.5.23: store a param Val en registro C.
                    if let Some(pv) = pvmap.get(&k) {
                        let e2 = if ke != 0 { format!("_v_int({})", e) } else { e };
                        s.push_str(&format!(
                        "  {{ Val _sv_ = ({}); if ({}.t == T_PTR && {}.p) *{}.p = _sv_; else {} = _sv_; }}\n",
                        e2, pv, pv, pv, pv
                    ));
                        if r && ke != 0 {
                            xe_errchk(&mut s, &handler_labels);
                        }
                        i += 1;
                        continue;
                    }
                    // v3.5.21: store a variable entera promovida: long long puro.
                    if let Some((ll_name, _)) = prom_int.get(&k) {
                        let expr = if ke == 1 {
                            e
                        } else {
                            format!("(long long)_asf({})", e)
                        };
                        s.push_str(&format!("  {} = {};\n", ll_name, expr));
                        if r && ke != 1 {
                            xe_errchk(&mut s, &handler_labels);
                        }
                        i += 1;
                        continue;
                    }
                    let e = if ke != 0 { format!("_v_int({})", e) } else { e };
                    let g = slot_of(i, n);
                    // v3.5.20: si el valor es FRESCO (resultado de op/llamada,
                    // sin alias posibles) se asigna sin _dcp — semántica idéntica
                    // (no hay otro dueño) y ahorra la copia profunda de arrays.
                    if fresh {
                        s.push_str(&format!(
                        "  {{ Val _sv_ = ({}); if ({}.t == T_PTR && {}.p) *{}.p = _sv_; else {} = _sv_; }}\n",
                        e, g, g, g, g
                    ));
                    } else {
                        s.push_str(&format!(
                        "  {{ Val _sv_ = _dcp({}); if ({}.t == T_PTR && {}.p) *{}.p = _sv_; else {} = _sv_; }}\n",
                        e, g, g, g, g
                    ));
                    }
                    if r {
                        xe_errchk(&mut s, &handler_labels);
                    }
                }
            }
            Instr::JmpIf(t) if !estack.is_empty() => {
                let (e, r, _, ke, _) = estack.pop().unwrap();
                // v3.5.21: comparación entera sin tag → test C directo.
                if ke == 2 || ke == 1 {
                    s.push_str(&format!("  if (!({})) goto L_{};\n", e, t));
                } else {
                    s.push_str(&format!("  if (!_truthy({})) goto L_{};\n", e, t));
                }
                if r {
                    xe_errchk(&mut s, &handler_labels);
                }
            }
            Instr::Jmp(t) => {
                xe_spill(&mut s, &mut estack, &handler_labels);
                s.push_str(&format!("  goto L_{};\n", t));
            }
            Instr::Label(t) => {
                xe_spill(&mut s, &mut estack, &handler_labels);
                s.push_str(&format!("  L_{}:;\n", t));
            }
            Instr::Return if !estack.is_empty() => {
                let (e, _, _, ke, _) = estack.pop().unwrap();
                if fn_ret_int {
                    // v3.5.24: retorno long long nativo (8B, no Val de 80B)
                    let e2 = match ke {
                        1 => e,
                        2 => format!("(long long)({})", e),
                        _ => format!("(long long)_asf({})", e),
                    };
                    xe_spill(&mut s, &mut estack, &handler_labels);
                    s.push_str(&format!("  {{ SP = _sb; return {}; }}\n", e2));
                } else {
                    let e = if ke != 0 { format!("_v_int({})", e) } else { e };
                    xe_spill(&mut s, &mut estack, &handler_labels);
                    s.push_str(&format!(
                        "  {{ Val _r = ({}); if (_r.t == T_FRE && _r.p && _r.i > 0) {{ for (int _k = 0; _k < (int)_r.i; _k++) ((Val*)_r.en)[_k] = _dcp(*((Val**)_r.p)[_k]); }} SP = _sb; return _r; }}\n",
                        e
                    ));
                }
            }
            Instr::Print if !estack.is_empty() => {
                let (e, r, _, ke, _) = estack.pop().unwrap();
                let e = if ke != 0 { format!("_v_int({})", e) } else { e };
                s.push_str(&format!("  printf(\"%s\\n\", _fmt({}));\n", e));
                if r {
                    xe_errchk(&mut s, &handler_labels);
                }
            }
            Instr::Read => {
                estack.push(("_read_ln()".to_string(), false, true, 0, true));
            }
            Instr::OptionNone => {
                estack.push(("_none()".to_string(), false, true, 0, true));
            }
            Instr::OptionSome if !estack.is_empty() => {
                let (e, r, _, ke, _) = estack.pop().unwrap();
                let e = if ke != 0 { format!("_v_int({})", e) } else { e };
                estack.push((format!("_some({})", e), r, false, 0, true));
            }
            Instr::ResultOk if !estack.is_empty() => {
                let (e, r, _, ke, _) = estack.pop().unwrap();
                let e = if ke != 0 { format!("_v_int({})", e) } else { e };
                estack.push((format!("_res({}, 1)", e), r, false, 0, true));
            }
            Instr::ResultErr if !estack.is_empty() => {
                let (e, r, _, ke, _) = estack.pop().unwrap();
                let e = if ke != 0 { format!("_v_int({})", e) } else { e };
                estack.push((format!("_res({}, 0)", e), r, false, 0, true));
            }
            Instr::ArrayGet if estack.len() >= 2 => {
                let (b, _rb, _, kb, _) = estack.pop().unwrap();
                let (a, _ra, _, ka, _) = estack.pop().unwrap();
                let b = if kb == 1 { b } else { format!("({}).i", b) };
                let a = if ka != 0 { format!("_v_int({})", a) } else { a };
                // elemento del array → NO fresco (alias del contenedor)
                estack.push((format!("_arr_get({}, {})", a, b), true, false, 0, false));
            }
            Instr::ArrayLen if !estack.is_empty() => {
                let (a, ra, _, ka, _) = estack.pop().unwrap();
                let a = if ka != 0 { format!("_v_int({})", a) } else { a };
                estack.push((format!("_arr_len({})", a), ra, true, 0, true));
            }
            Instr::ScopePush | Instr::ScopePop | Instr::Nop | Instr::Phi(..) => {
                // sin efecto en la pila de valores ni en el emisor C
            }
            // v3.5.22: llamada a función de usuario desde el camino de
            // expresiones: argumentos MOVIBLES (último uso demostrable o
            // valor fresco) sin _dcp — semántica idéntica (si la variable no
            // se vuelve a leer, copiar o mover es indistinguible).
            Instr::Call(cn, argc) if program.funcs.contains_key(cn) && estack.len() >= *argc => {
                let callee = &program.funcs[cn];
                let plen = callee.params.len().min(*argc);
                let args_e: Vec<(String, bool, bool, u8, bool)> =
                    estack.split_off(estack.len() - *argc);
                let mv_id = mvac;
                mvac += 1;
                // v3.5.22: el resultado de la llamada va a ST[]; cualquier
                // expresión pendiente debe estar YA en ST (en orden) para que
                // los consumidores legacy posteriores popeen correctamente.
                xe_spill(&mut s, &mut estack, &handler_labels);
                let callee_throws =
                    instr_throws(&Instr::Call(cn.clone(), *argc), no_throw, program);
                // v3.5.23: ABI de registros — llamada directa sin staging gv.
                if reg_abi.contains(cn) && !callee.params.is_empty() {
                    let mut cargs: Vec<String> = Vec::new();
                    for (j, (e, _, _, kind, movable)) in args_e.iter().enumerate() {
                        if j >= plen {
                            if *kind != 0 {
                                s.push_str(&format!("  (void)_v_int({});\n", e));
                            } else {
                                s.push_str(&format!("  (void)({});\n", e));
                            }
                            continue;
                        }
                        let pkey = format!("{}::{}", cn, callee.params[j]);
                        let int_param = int_proms_all
                            .get(cn)
                            .map(|m| matches!(m.get(&pkey), Some((_, Some(_)))))
                            .unwrap_or(false);
                        let expr = match (int_param, *kind) {
                            (true, 1) => e.clone(),
                            (true, 2) => format!("(long long)({})", e),
                            (true, 0) => format!("(long long)_asf({})", e),
                            (false, 0) => {
                                if *movable || no_mutate.contains(cn) {
                                    e.clone()
                                } else {
                                    format!("_dcp({})", e)
                                }
                            }
                            _ => format!("_v_int({})", e),
                        };
                        cargs.push(expr);
                    }
                    // save/restore SOLO de locales gv del callee (los params
                    // van por registros; la recursión es nativa).
                    let callee_slots: Vec<String> = name_sets.get(cn).cloned().unwrap_or_default();
                    if !callee_slots.is_empty() {
                        s.push_str(&format!("  {{ Val _cs[{}];\n", callee_slots.len()));
                        for (k, ck) in callee_slots.iter().enumerate() {
                            s.push_str(&format!("    _cs[{}] = {};\n", k, gv_of(ck)));
                        }
                        if returns_int.contains(cn) {
                            s.push_str(&format!(
                                "    long long _mvr{} = _f_{}({});\n",
                                mv_id,
                                mangle(cn),
                                cargs.join(", ")
                            ));
                        } else {
                            s.push_str(&format!(
                                "    Val _mvr{} = _f_{}({});\n",
                                mv_id,
                                mangle(cn),
                                cargs.join(", ")
                            ));
                        }
                        s.push_str("    ");
                        for (k, ck) in callee_slots.iter().enumerate() {
                            s.push_str(&format!("{} = _cs[{}]; ", gv_of(ck), k));
                        }
                        if returns_int.contains(cn) {
                            s.push_str(&format!("PUSH(_v_int(_mvr{})); }}\n", mv_id));
                        } else {
                            s.push_str(&format!("PUSH(_mvr{}); }}\n", mv_id));
                        }
                    } else {
                        if returns_int.contains(cn) {
                            s.push_str(&format!(
                                "  {{ long long _mvr{} = _f_{}({}); PUSH(_v_int(_mvr{})); }}\n",
                                mv_id,
                                mangle(cn),
                                cargs.join(", "),
                                mv_id
                            ));
                        } else {
                            s.push_str(&format!(
                                "  {{ Val _mvr{} = _f_{}({}); PUSH(_mvr{}); }}\n",
                                mv_id,
                                mangle(cn),
                                cargs.join(", "),
                                mv_id
                            ));
                        }
                    }
                    if callee_throws {
                        xe_errchk(&mut s, &handler_labels);
                    }
                } else {
                    for (j, (e, _, _, kind, movable)) in args_e.iter().enumerate() {
                        if j >= plen {
                            // args sobrantes: se descartan (paridad (void)POP())
                            if *kind != 0 {
                                s.push_str(&format!("  (void)_v_int({});\n", e));
                            } else {
                                s.push_str(&format!("  (void)({});\n", e));
                            }
                            continue;
                        }
                        let expr = if *kind != 0 {
                            format!("_v_int({})", e)
                        } else {
                            e.clone()
                        };
                        if *movable {
                            s.push_str(&format!("  Val _mv{}_{} = {};\n", mv_id, j, expr));
                        } else {
                            s.push_str(&format!("  Val _mv{}_{} = _dcp({});\n", mv_id, j, expr));
                        }
                    }
                    // save/restore de los slots del callee (paridad recursión)
                    let callee_slots: Vec<String> = name_sets.get(cn).cloned().unwrap_or_default();
                    if !callee_slots.is_empty() {
                        s.push_str(&format!("  {{ Val _cs[{}];\n", callee_slots.len()));
                        for (k, ck) in callee_slots.iter().enumerate() {
                            s.push_str(&format!("    _cs[{}] = {};\n", k, gv_of(ck)));
                        }
                        for j in (0..plen).rev() {
                            s.push_str(&format!(
                                "    {} = _mv{}_{};\n",
                                gv_of(&callee_slot_of(cn, &callee.params[j])),
                                mv_id,
                                j
                            ));
                        }
                        s.push_str(&format!("    Val _r = _f_{}(); PUSH(_r);\n", mangle(cn)));
                        s.push_str("    ");
                        for (k, ck) in callee_slots.iter().enumerate() {
                            s.push_str(&format!("{} = _cs[{}]; ", gv_of(ck), k));
                        }
                        s.push_str("}\n");
                    } else {
                        for j in (0..plen).rev() {
                            s.push_str(&format!(
                                "  {} = _mv{}_{};\n",
                                gv_of(&callee_slot_of(cn, &callee.params[j])),
                                mv_id,
                                j
                            ));
                        }
                        s.push_str(&format!(
                            "  {{ Val _r = _f_{}(); PUSH(_r); }}\n",
                            mangle(cn)
                        ));
                    }
                    if callee_throws {
                        xe_errchk(&mut s, &handler_labels);
                    }
                }
            }
            // v3.5.27: builtins de texto puros en el camino de expresiones.
            // Antes cortaban la cadena XE y el bucle caía a ST[] (PUSH/POP de
            // Vals de 80B por iteración). a_texto → _to_text* (itoa directo al
            // arena, sin snprintf ni malloc); largo → _largo_ll (long long
            // nativo, alimenta la promoción entera del consumidor).
            Instr::Call(cn, argc)
                if *argc == 1
                    && !estack.is_empty()
                    && !program.funcs.contains_key(cn)
                    && matches!(
                        cn.as_str(),
                        "a_texto"
                            | "to_texto"
                            | "__str_from"
                            | "largo"
                            | "len"
                            | "__str_len"
                            | "__str_longitud"
                    ) =>
            {
                let (a, ra, _fa, ka, _) = estack.pop().unwrap();
                let nm = cn.as_str();
                if matches!(nm, "a_texto" | "to_texto" | "__str_from") {
                    let expr = match ka {
                        1 => format!("_to_text_ll({})", a),
                        2 => format!("_to_text(_v_bool({}))", a),
                        _ => format!("_to_text({})", a),
                    };
                    estack.push((expr, ra, true, 0, true));
                } else {
                    let expr = match ka {
                        1 => format!("_largo_ll(_v_int({}))", a),
                        2 => format!("_largo_ll(_v_bool({}))", a),
                        _ => format!("_largo_ll({})", a),
                    };
                    estack.push((expr, ra, true, 1, true));
                }
            }
            _ => {
                xe_consumed = false;
            }
        }
        if xe_consumed {
            i += 1;
            continue;
        }
        // ────────── camino clásico (ST[]/PUSH/POP) ──────────
        xe_spill(&mut s, &mut estack, &handler_labels);
        let instr = &instrs_all[i];
        // v3.5.24: ¿puede LANZAR realmente esta instrucción? (Div/Mod,
        // bounds, llamadas a funciones/builtins que lanzan). Las demás ya no
        // pagan el ERRCHK.
        let risky = instr_throws(instr, no_throw, program);
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
                // v3.5.27: literal sin copia al arena (_v_str_lit).
                s.push_str(&format!(
                    "  PUSH(_v_str_lit(\"{}\"));\n",
                    x.replace('\\', "\\\\")
                        .replace('"', "\\\"")
                        .replace('\n', "\\n")
                        .replace('\r', "\\r")
                        .replace('\t', "\\t")
                ));
            }
            Instr::Load(n) => {
                s.push_str(&format!("  PUSH(_deref({}));\n", slot_of(i, n)));
            }
            Instr::Store(n) | Instr::StoreLocal(n) => {
                // Si el slot contiene una referencia (prestado mut), escribir
                // a través del puntero; si no, asignación normal.
                // v3.5.20: _scpy (paridad clone() de la VM) en vez de _dcp.
                let g = slot_of(i, n);
                s.push_str(&format!(
                    "  {{ Val _sv_ = POP(); if ({g}.t == T_PTR && {g}.p) *{g}.p = _dcp(_sv_); else {g} = _dcp(_sv_); }}\n",
                    g = g
                ));
            }
            Instr::Binary(op) => {
                let code = op_code(op);
                if matches!(op, Op::Concat) {
                    // v3.5.24: STR+STR directo (1 malloc, sin _fmt)
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_concat2(_a, _b)); }\n");
                } else {
                    s.push_str(&format!(
                        "  {{ Val _b = POP(); Val _a = POP(); PUSH(_bin({}, _a, _b)); }}\n",
                        code
                    ));
                }
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
                            "  {{ Val _t[{}]; for (int _k = {} - 1; _k >= 0; _k--) _t[_k] = POP(); PUSH(_to_text(_t[0])); }}\n",
                            argc, argc
                        ));
                    } else {
                        s.push_str("  PUSH(_v_str(\"\"));\n");
                    }
                } else if n == "agregar" || n == "push" {
                    s.push_str("  { Val _x = POP(); Val _a = POP(); PUSH(_arr_push(_a, _x)); }\n");
                } else if n == "largo" || n == "len" || n == "__str_len" || n == "__str_longitud" {
                    s.push_str("  { Val _x = POP(); if (_x.t == T_ARR || _x.t == T_TUP || _x.t == T_MAP) PUSH(_v_int(_x.argc)); else if (_x.t == T_STR) PUSH(_v_int((int64_t)_utf8_len(_x.s))); else PUSH(_v_int(0)); }\n");
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
                } else if n == "__calendario_hijri" || n == "__calendar_hijri" {
                    // v3.5.17: calendarios nativos (paridad VM).
                    s.push_str("  { Val _t = POP(); PUSH(_rt_calendario_hijri((int64_t)_asf(_t))); }\n");
                } else if n == "__calendario_persa" || n == "__calendar_persian" {
                    s.push_str("  { Val _t = POP(); PUSH(_rt_calendario_persa((int64_t)_asf(_t))); }\n");
                } else if n == "__tarea_lanzar" || n == "__task_spawn" || n == "__hilo_lanzar" || n == "__thread_spawn" {
                    // v3.5.10: hilos REALES (antes era un shim secuencial falso).
                    // Stack: [fn, a1..a_{argc-1}] → pop args en reversa + nombre.
                    let na = (*argc).saturating_sub(1);
                    s.push_str(&format!(
                        "  {{ Val _ta[8]; int _tn = {}; for (int _k = _tn - 1; _k >= 0; _k--) _ta[_k] = POP(); Val _nm = POP(); PUSH(_v_int((int64_t)_lw_thr_spawn(_nm.s, _ta, _tn))); }}\n",
                        na.min(8)
                    ));
                } else if n == "__tarea_esperar" || n == "__task_await" || n == "__hilo_esperar" || n == "__thread_join" {
                    s.push_str("  { Val _id = POP(); PUSH(_lw_thr_join((int64_t)_asf(_id))); }\n");
                } else if n == "__canal_nuevo" || n == "__channel_new" {
                    // v3.5.17: canales nativos (paridad VM).
                    s.push_str("  PUSH(_rt_chan_new_v());\n");
                } else if n == "__canal_enviar" || n == "__channel_send" {
                    s.push_str("  { Val _v = POP(); Val _c = POP(); PUSH(_rt_chan_send_v(_c, _v)); }\n");
                } else if n == "__canal_recibir" || n == "__channel_recv" {
                    s.push_str("  { Val _c = POP(); PUSH(_rt_chan_recv_v(_c)); }\n");
                } else if n == "__mutex_nuevo" || n == "__mutex_new" {
                    s.push_str("  PUSH(_rt_mutex_new_v());\n");
                } else if n == "__mutex_bloquear" || n == "__mutex_lock" {
                    s.push_str("  { Val _a = POP(); Val _f = POP(); Val _m = POP(); PUSH(_rt_mutex_lock_call_v(_m, _f, _a)); }\n");
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
                } else if n == "abs" || n == "absoluto" {
                    s.push_str("  { Val _a = POP(); PUSH(_m_abs(_a)); }\n");
                } else if n == "raiz" || n == "sqrt" {
                    s.push_str("  { Val _a = POP(); PUSH(_m_sqrt(_a)); }\n");
                } else if n == "piso" || n == "floor" {
                    s.push_str("  { Val _a = POP(); PUSH(_m_floor(_a)); }\n");
                } else if n == "techo" || n == "ceil" {
                    s.push_str("  { Val _a = POP(); PUSH(_m_ceil(_a)); }\n");
                } else if n == "redondear" || n == "round" {
                    s.push_str("  { Val _a = POP(); PUSH(_m_round(_a)); }\n");
                } else if n == "potencia" || n == "pow" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_m_pow(_a, _b)); }\n");
                } else if n == "min" || n == "minimo" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_m_min(_a, _b)); }\n");
                } else if n == "max" || n == "maximo" {
                    s.push_str("  { Val _b = POP(); Val _a = POP(); PUSH(_m_max(_a, _b)); }\n");
                } else if let Some(callee) = program.funcs.get(n) {
                    let plen = callee.params.len().min(*argc);
                    // v3.5.23: los callees con ABI de registros se invocan
                    // vía wrapper _fw (staging gv → params nativos).
                    let use_wrapper =
                        reg_abi.contains(n) && !callee.params.is_empty();
                    // v3.4.6: save/restore SOLO de los slots del CALLEE (params y
                    // locales renombrados `{callee}::...`). El llamador ya no se
                    // guarda nunca (sus slots son únicos por función); el callee
                    // sí se preserva para que la RECURSIÓN vea sus params
                    // originales tras las llamadas anidadas.
                    let callee_slots: Vec<String> =
                        name_sets.get(n).cloned().unwrap_or_default();
                    let mut pre = String::new();
                    let mut post = String::new();
                    if !callee_slots.is_empty() {
                        pre.push_str("  { Val _cs[");
                        pre.push_str(&callee_slots.len().to_string());
                        pre.push_str("];\n");
                        for (k, ck) in callee_slots.iter().enumerate() {
                            pre.push_str(&format!("    _cs[{}] = {};\n", k, gv_of(ck)));
                        }
                        post.push_str("    ");
                        for (k, ck) in callee_slots.iter().enumerate() {
                            post.push_str(&format!("{} = _cs[{}]; ", gv_of(ck), k));
                        }
                        post.push_str("}\n");
                    }
                    s.push_str(&pre);
                    let share_args = no_mutate.contains(n);
                    for i in (0..plen).rev() {
                        if share_args {
                            s.push_str(&format!(
                                "  {} = POP();\n",
                                gv_of(&callee_slot_of(n, &callee.params[i]))
                            ));
                        } else {
                            s.push_str(&format!(
                                "  {} = _dcp(POP());\n",
                                gv_of(&callee_slot_of(n, &callee.params[i]))
                            ));
                        }
                    }
                    for _ in plen..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    let conv = if returns_int.contains(n) { "_v_int" } else { "" };
                    if use_wrapper {
                        if conv.is_empty() {
                            s.push_str(&format!(
                                "  {{ Val _r = _fw_{}(); PUSH(_r); }}\n",
                                mangle(n)
                            ));
                        } else {
                            s.push_str(&format!(
                                "  {{ long long _r = _fw_{}(); PUSH(_v_int(_r)); }}\n",
                                mangle(n)
                            ));
                        }
                    } else if conv.is_empty() {
                        s.push_str(&format!("  {{ Val _r = _f_{}(); PUSH(_r); }}\n", mangle(n)));
                    } else {
                        s.push_str(&format!(
                            "  {{ long long _r = _f_{}(); PUSH(_v_int(_r)); }}\n",
                            mangle(n)
                        ));
                    }
                    s.push_str(&post);
                } else {
                    for _ in 0..*argc {
                        s.push_str("  (void)POP();\n");
                    }
                    s.push_str(&format!("  {{ Val _r = _f_{}(); PUSH(_r); }}\n", mangle(n)));
                }
            }
            Instr::FuncRef(n) => {
                // v3.5.42 (bug fuzz closure_multi): snapshot de celdas
                // capturadas en el Val del closure.
                if let Some(cells) = fref_cells.get(n).filter(|c| !c.is_empty()) {
                    let list = cells
                        .iter()
                        .map(|c| format!("\"{}\"", esc(c)))
                        .collect::<Vec<_>>()
                        .join(", ");
                    s.push_str(&format!(
                        "  PUSH(_vfref_snap(\"{}\", &_f_{}, (const char*[]){{ {} }}, {}));\n",
                        esc(n),
                        mangle(n),
                        list,
                        cells.len()
                    ));
                } else {
                    s.push_str(&format!(
                        "  PUSH(_vfref(\"{}\", &_f_{}));\n",
                        esc(n),
                        mangle(n)
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
                    for i in 0..*argc {
                        s.push_str(&format!("      printf(\"%s\\n\", _fmt(_t[{}]));\n", i));
                    }
                    s.push_str("      PUSH(_v_void());\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"leer\") || !strcmp(_cf.s, \"read\")) {\n      PUSH(_read_ln());\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"a_texto\") || !strcmp(_cf.s, \"to_texto\") || !strcmp(_cf.s, \"__str_from\")) {\n      PUSH(_to_text(_t[0]));\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"agregar\") || !strcmp(_cf.s, \"push\")) {\n      PUSH(_arr_push(_t[0], _t[1]));\n");
                    s.push_str("    } else if (!strcmp(_cf.s, \"largo\") || !strcmp(_cf.s, \"len\") || !strcmp(_cf.s, \"__str_len\") || !strcmp(_cf.s, \"__str_longitud\")) {\n      PUSH(_v_int(_t[0].t == T_STR ? (int64_t)_utf8_len(_t[0].s) : _t[0].argc));\n");
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
            Instr::Return => {
                if fn_ret_int {
                    s.push_str("  { Val _r = POP(); SP = _sb; return (long long)_asf(_r); }\n")
                } else {
                    // v3.5.42: refrescar el snapshot de un closure retornado
                    // con el estado final de sus celdas capturadas.
                    s.push_str(
                        "  { Val _r = POP(); if (_r.t == T_FRE && _r.p && _r.i > 0) { for (int _k = 0; _k < (int)_r.i; _k++) ((Val*)_r.en)[_k] = _dcp(*((Val**)_r.p)[_k]); } SP = _sb; return _r; }\n",
                    )
                }
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
            Instr::MakeRef(vname) => {
                // prestado mut (bug #6): apilar puntero al slot gv[] de la variable.
                // El slot gv es estático → la dirección es estable durante todo el run.
                // (Los objetivos de MakeRef nunca se promueven → siempre gv.)
                s.push_str(&format!("  PUSH(_v_ptr(&{}));\n", slot_of(i, vname)));
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
                // AOT: push in-place amortizado + store directo al slot.
                // El slot es dueño exclusivo del buffer (los demás Stores y
                // args de llamada se deep-copian) → O(n) en bucles de agregar.
                // v3.5.42 (bug fuzz gen_ref): el write-back debe resolver el
                // MISMO binding que Load — si la variable es un param en
                // registro C, puede ser T_PTR (prestado mut) y hay que
                // escribir a través del puntero; el slot gv[] local nunca es
                // T_PTR y la mutación del llamador se perdía.
                let k = var_at(i, vname);
                let vn = pvmap.get(&k).cloned().unwrap_or_else(|| slot_of(i, vname));
                s.push_str(
                    "  { Val _x = POP(); Val _a = POP(); PUSH(_arr_push_ip(_a, _x)); }\n",
                );
                s.push_str(&format!(
                    "  {{ Val _sv_ = POP(); if ({vn}.t == T_PTR && {vn}.p) *{vn}.p = _sv_; else {vn} = _sv_; }}\n",
                    vn = vn
                ));
            }
            Instr::ArrayGet => s.push_str("  { Val _i = POP(); Val _a = POP(); PUSH(_arr_get(_a, _i.i)); }\n"),
            Instr::ArraySet => {
                s.push_str("  { Val _x = POP(); Val _i = POP(); Val _a = POP(); PUSH(_arr_set(_a, _i.i, _x)); }\n");
            }
            Instr::ArraySetVar(vname) => {
                // v3.5.40: equivalente canónico (ArraySet + Store). El IR
                // ya llega canonicalizado (lower_arraysetvar); esta rama es
                // guard de robustez para llamadores internos.
                let k = var_at(i, vname);
                let vn = pvmap.get(&k).cloned().unwrap_or_else(|| slot_of(i, vname));
                s.push_str("  { Val _x = POP(); Val _i = POP(); Val _a = POP(); PUSH(_arr_set(_a, _i.i, _x)); }\n");
                s.push_str(&format!(
                    "  {{ Val _sv_ = POP(); if ({vn}.t == T_PTR && {vn}.p) *{vn}.p = _sv_; else {vn} = _sv_; }}\n",
                    vn = vn
                ));
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
                // Paridad VM (QA bug #3): enums con 1 campo → el campo;
                // con varios → lista de campos; vacio → void.
                s.push_str("  { Val _u = _deref(POP()); if ((_u.t == T_SOM || _u.t == T_OK || _u.t == T_ERR) && _u.argc > 0) { PUSH(_u.items[0]); } else if (_u.t == T_ENM) { if (_u.argc == 0) { PUSH(_v_void()); } else if (_u.argc == 1) { PUSH(_u.items[0]); } else { Val* _pc = (Val*)malloc(sizeof(Val) * _u.argc); for (int _pi = 0; _pi < _u.argc; _pi++) _pc[_pi] = _u.items[_pi]; PUSH(_arrn(_pc, _u.argc)); } } else { PUSH(_u); } }\n");
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
        i += 1;
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
        // v3.5.24: main del sample no lanza → sin NOINLINE; termina con
        // Halt (void), así que el retorno sigue siendo Val.
        assert!(c.contains("static Val _f_main(void)"));
        assert!(c.contains("int main(void)"));
        // v3.5.21: enteros sin tag — la suma 40+2 se emite como aritmética
        // nativa (unsigned long long para wrapping exacto) y solo se
        // convierte a Val (_v_int) en la frontera (Print).
        assert!(c.contains("(unsigned long long)(40)"));
        assert!(c.contains("_v_int("));
        assert!(c.contains("printf(\"%s\\n\", _fmt("));
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
            .args([probe_c.to_str().unwrap(), "-o", probe_exe.to_str().unwrap()])
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
        lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        let c = compile_to_c(&ir);
        // Dir único por corrida: el pid se reusa entre ejecuciones del binario
        // de tests y el antivirus puede bloquear un .exe recién escrito.
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("lumen_aot_test_{}_{}", std::process::id(), nanos));
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
    fn test_c_fuzz_v3541_capturas_y_prestado() {
        // v3.5.42 (bugs fuzz closure_multi + gen_ref): en el backend C,
        // (1) cada instanciación de un definidor con capturas debe dar a su
        // closure un estado propio (snapshot por closure), y (2) `agregar`
        // sobre `prestado mut lista` debe escribir a través del puntero y
        // persistir la mutación en el llamador.
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let probe_dir = std::env::temp_dir().join("lumen_gcc_probe2");
        std::fs::create_dir_all(&probe_dir).unwrap();
        let probe_c = probe_dir.join("p.c");
        let probe_exe = probe_dir.join("p.exe");
        std::fs::write(&probe_c, "int main(void){return 0;}\n").unwrap();
        let _ = std::fs::remove_file(&probe_exe);
        match std::process::Command::new("gcc")
            .args([probe_c.to_str().unwrap(), "-o", probe_exe.to_str().unwrap()])
            .output()
        {
            Ok(o) if o.status.success() && probe_exe.exists() => {}
            _ => return,
        }
        let source = r#"
            funcion entero contador() {
                sea n = 0;
                funcion entero inc() {
                    n = n + 1;
                    retornar n;
                }
                retornar inc;
            }
            funcion vacio tocar(prestado mut lista<entero> xs) { xs.agregar(99); }
            funcion vacio main() {
                sea a = contador();
                sea b = contador();
                imprimir("a1:", a());
                imprimir("b1:", b());
                imprimir("a2:", a());
                imprimir("b2:", b());
                sea xs = [1];
                tocar(xs);
                imprimir(xs[1]);
            }
        "#;
        let tokens = lumen_lexer::Lexer::new(source).tokenize();
        let (mut program, _) = lumen_parser::Parser::new(tokens.0).parse();
        let sem_errors = lumen_sema::SemanticAnalyzer::new().analyze(&mut program);
        assert!(sem_errors.is_empty(), "sema fallo: {:?}", sem_errors);
        lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        let c = compile_to_c(&ir);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("lumen_aot_caps_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).unwrap();
        let c_path = dir.join("test_caps.c");
        let exe_path = dir.join("test_caps.exe");
        std::fs::write(&c_path, c).unwrap();
        let status = std::process::Command::new("gcc")
            .arg(c_path.to_str().unwrap())
            .args(["-O2", "-o", exe_path.to_str().unwrap(), "-lm"])
            .output()
            .unwrap_or_else(|e| panic!("gcc fallo al invocar: {:?}", e));
        if !status.status.success() {
            panic!(
                "gcc fallo al compilar:\n{}",
                String::from_utf8_lossy(&status.stderr)
            );
        }
        let out = loop {
            match std::process::Command::new(&exe_path).output() {
                Ok(o) => break o,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(300)),
            }
        };
        let test_out = String::from_utf8_lossy(&out.stdout)
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            test_out,
            vec!["a1:1", "b1:1", "a2:2", "b2:2", "99"],
            "salida completa: {:?}",
            test_out
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cranelift_runtime_lw() {
        // v3.5.6: backend Cranelift con runtime _lw_* (handles opacos).
        // Requiere gcc disponible (Linux CI o MSYS2 en Windows).
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let probe_dir = std::env::temp_dir().join("lumen_cr_probe");
        std::fs::create_dir_all(&probe_dir).unwrap();
        let probe_c = probe_dir.join("p.c");
        let probe_exe = probe_dir.join("p.exe");
        std::fs::write(&probe_c, "int main(void){return 0;}\n").unwrap();
        let _ = std::fs::remove_file(&probe_exe);
        match std::process::Command::new("gcc")
            .args([probe_c.to_str().unwrap(), "-o", probe_exe.to_str().unwrap()])
            .output()
        {
            Ok(o) if o.status.success() && probe_exe.exists() => {}
            _ => return, // gcc ausente o roto en este entorno: omitir test
        }
        let source = r#"
            estructura Punto { x: entero, y: entero }
            funcion entero fib(n: entero) {
                si (n < 2) { retornar n; }
                retornar fib(n - 1) + fib(n - 2);
            }
            funcion texto color(texto c) {
                si (c == "rojo") { retornar "R"; }
                retornar "?";
            }
            funcion vacio main() {
                sea p = Punto { x: 1, y: 2 };
                imprimir(p.x + p.y);
                p.y = 40;
                imprimir(p.y);
                imprimir(fib(10));
                imprimir(color("rojo"));
                sea s = "abc";
                imprimir(s[1]);
                imprimir(s.largo());
                imprimir("x:", s[2]);
                imprimir("hola" + " mundo");
                sea f = 2.5;
                imprimir(f * 2.0);
                imprimir(7 / 2);
                imprimir(7 % 3);
                imprimir(5 > 3);
                sea xs = [10, 20];
                xs.agregar(30);
                imprimir(xs[2]);
                imprimir(xs.largo());
                sea m = __map_nuevo();
                m = __map_poner(m, "k", 99);
                imprimir(__map_obtener(m, "k"));
                imprimir(algun(7));
                imprimir(exito("ok"));
                imprimir(__tipo_de(s));
                imprimir(a_texto(42));
                entero i = 0;
                mientras (i < 3) { imprimir(i); i = i + 1; }
            }
        "#;
        let tokens = lumen_lexer::Lexer::new(source).tokenize();
        let (mut program, _) = lumen_parser::Parser::new(tokens.0).parse();
        let sem_errors = lumen_sema::SemanticAnalyzer::new().analyze(&mut program);
        assert!(sem_errors.is_empty(), "sema fallo: {:?}", sem_errors);
        lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        let unsupported = cranelift_supported(&ir);
        assert!(
            unsupported.is_empty(),
            "cranelift rechazó el programa de test: {:?}",
            unsupported
        );
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("lumen_cr_test_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).unwrap();
        let obj_path = dir.join("test_cr.obj");
        let shim_path = dir.join("test_cr_rt.c");
        let exe_path = dir.join("test_cr.exe");
        compile_to_object(&ir, obj_path.to_str().unwrap()).expect("compile_to_object fallo");
        std::fs::write(&shim_path, lw_shim_source()).unwrap();
        let status = std::process::Command::new("gcc")
            .arg(obj_path.to_str().unwrap())
            .arg(shim_path.to_str().unwrap())
            .args(["-O2", "-o", exe_path.to_str().unwrap(), "-lm"])
            .output()
            .expect("gcc fallo al invocar");
        assert!(
            status.status.success(),
            "gcc fallo al compilar/link:\n{}",
            String::from_utf8_lossy(&status.stderr)
        );
        let out = loop {
            match std::process::Command::new(&exe_path).output() {
                Ok(o) => break o,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(300)),
            }
        };
        let test_out = String::from_utf8_lossy(&out.stdout)
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            test_out,
            vec![
                "3",
                "40",
                "55",
                "R",
                "b",
                "3",
                "x:c",
                "hola mundo",
                "5",
                "3",
                "1",
                "true",
                "30",
                "3",
                "99",
                "algun(7)",
                "exito(ok)",
                "texto",
                "42",
                "0",
                "1",
                "2"
            ],
            "salida completa: {:?}\nstderr: {}",
            test_out,
            String::from_utf8_lossy(&out.stderr)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cranelift_runtime_prestado_agregar() {
        // v3.5.42 (bug fuzz gen_ref): write-back de ArrayPushVar/ArraySetVar
        // vía `prestado mut` debe escribir a través del puntero (LW_STORE_SLOT)
        // y persistir la mutación en el llamador.
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let probe_dir = std::env::temp_dir().join("lumen_cr_probe3");
        std::fs::create_dir_all(&probe_dir).unwrap();
        let probe_c = probe_dir.join("p.c");
        let probe_exe = probe_dir.join("p.exe");
        std::fs::write(&probe_c, "int main(void){return 0;}\n").unwrap();
        let _ = std::fs::remove_file(&probe_exe);
        match std::process::Command::new("gcc")
            .args([probe_c.to_str().unwrap(), "-o", probe_exe.to_str().unwrap()])
            .output()
        {
            Ok(o) if o.status.success() && probe_exe.exists() => {}
            _ => return,
        }
        let source = r#"
            funcion vacio tocar(prestado mut lista<entero> xs) { xs.agregar(99); }
            funcion vacio main() {
                sea xs = [1];
                tocar(xs);
                imprimir(xs[1]);
                imprimir(xs.largo());
            }
        "#;
        let tokens = lumen_lexer::Lexer::new(source).tokenize();
        let (mut program, _) = lumen_parser::Parser::new(tokens.0).parse();
        let sem_errors = lumen_sema::SemanticAnalyzer::new().analyze(&mut program);
        assert!(sem_errors.is_empty(), "sema fallo: {:?}", sem_errors);
        lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        let unsupported = cranelift_supported(&ir);
        assert!(
            unsupported.is_empty(),
            "cranelift rechazó prestado+agregar: {:?}",
            unsupported
        );
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "lumen_cr_prestado_{}_{}",
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let obj_path = dir.join("test_p.obj");
        let shim_path = dir.join("test_p_rt.c");
        let exe_path = dir.join("test_p.exe");
        compile_to_object(&ir, obj_path.to_str().unwrap()).expect("compile_to_object fallo");
        std::fs::write(&shim_path, lw_shim_source()).unwrap();
        let status = std::process::Command::new("gcc")
            .arg(obj_path.to_str().unwrap())
            .arg(shim_path.to_str().unwrap())
            .args(["-O2", "-o", exe_path.to_str().unwrap(), "-lm"])
            .output()
            .expect("gcc fallo al invocar");
        assert!(
            status.status.success(),
            "gcc fallo al compilar/link:\n{}",
            String::from_utf8_lossy(&status.stderr)
        );
        let out = loop {
            match std::process::Command::new(&exe_path).output() {
                Ok(o) => break o,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(300)),
            }
        };
        let test_out = String::from_utf8_lossy(&out.stdout)
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            test_out,
            vec!["99", "2"],
            "salida completa: {:?}\nstderr: {}",
            test_out,
            String::from_utf8_lossy(&out.stderr)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cranelift_runtime_lw_b() {
        // v3.5.7 (Incremento B): prestado mut (MakeRef write-back),
        // intentar/atrapar, enums + elegir, funciones como valores y
        // sombreado por bloques en el backend Cranelift.
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let probe_dir = std::env::temp_dir().join("lumen_cr_probe_b");
        std::fs::create_dir_all(&probe_dir).unwrap();
        let probe_c = probe_dir.join("p.c");
        let probe_exe = probe_dir.join("p.exe");
        std::fs::write(&probe_c, "int main(void){return 0;}\n").unwrap();
        let _ = std::fs::remove_file(&probe_exe);
        match std::process::Command::new("gcc")
            .args([probe_c.to_str().unwrap(), "-o", probe_exe.to_str().unwrap()])
            .output()
        {
            Ok(o) if o.status.success() && probe_exe.exists() => {}
            _ => return,
        }
        let source = r#"
            enum Color { Rojo, Verde, Azul }

            funcion vacio incrementar(prestado mut entero n) {
                n = n + 1;
            }

            funcion entero doble(entero x) {
                retornar x * 2;
            }

            funcion texto nombrar(Color c) {
                elegir (c) {
                    caso Color::Rojo: { retornar "rojo"; }
                    caso Color::Verde: { retornar "verde"; }
                    defecto: { retornar "otro"; }
                }
            }

            funcion vacio main() {
                // prestado mut: write-back real al llamador
                entero v = 41;
                incrementar(v);
                imprimir(v);
                // intentar/atrapar con mensaje bindeado
                intentar {
                    sea xs = [1];
                    imprimir(xs[5]);
                } atrapar (e) {
                    imprimir("atrapado");
                }
                // enums + elegir (MatchVariant)
                imprimir(nombrar(Color::Rojo));
                imprimir(nombrar(Color::Verde));
                imprimir(nombrar(Color::Azul));
                // funciones como valores (FuncRef + CallValue)
                sea f = funcion(entero x) { retornar x * 3; };
                imprimir(f(10));
                imprimir(doble(21));
                // sombreado por bloques (scopes reales)
                entero x = 1;
                si (verdadero) {
                    entero x = 2;
                    imprimir(x);
                }
                imprimir(x);
            }
        "#;
        let tokens = lumen_lexer::Lexer::new(source).tokenize();
        let (mut program, _) = lumen_parser::Parser::new(tokens.0).parse();
        let sem_errors = lumen_sema::SemanticAnalyzer::new().analyze(&mut program);
        assert!(sem_errors.is_empty(), "sema fallo: {:?}", sem_errors);
        lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        let unsupported = cranelift_supported(&ir);
        assert!(
            unsupported.is_empty(),
            "cranelift rechazó el programa de test B: {:?}",
            unsupported
        );
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir =
            std::env::temp_dir().join(format!("lumen_cr_testb_{}_{}", std::process::id(), nanos));
        std::fs::create_dir_all(&dir).unwrap();
        let obj_path = dir.join("test_b.obj");
        let shim_path = dir.join("test_b_rt.c");
        let exe_path = dir.join("test_b.exe");
        compile_to_object(&ir, obj_path.to_str().unwrap()).expect("compile_to_object fallo");
        std::fs::write(&shim_path, lw_shim_source()).unwrap();
        let status = std::process::Command::new("gcc")
            .arg(obj_path.to_str().unwrap())
            .arg(shim_path.to_str().unwrap())
            .args(["-O2", "-o", exe_path.to_str().unwrap(), "-lm"])
            .output()
            .expect("gcc fallo al invocar");
        assert!(
            status.status.success(),
            "gcc fallo al compilar/link:\n{}",
            String::from_utf8_lossy(&status.stderr)
        );
        let out = loop {
            match std::process::Command::new(&exe_path).output() {
                Ok(o) => break o,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(300)),
            }
        };
        let test_out = String::from_utf8_lossy(&out.stdout)
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            test_out,
            vec!["42", "atrapado", "rojo", "verde", "otro", "30", "42", "2", "1"],
            "salida completa: {:?}\nstderr: {}",
            test_out,
            String::from_utf8_lossy(&out.stderr)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_cranelift_threads() {
        // v3.5.17: hilos reales en Cranelift — spawn/join con argumentos,
        // resultado determinista sin importar el intercalado de hilos.
        // En Windows la cadena de hilos es flaky (pthread vs Win32) → skip si no hay toolchain estable
        if std::process::Command::new("gcc")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            == false
        {
            eprintln!("gcc no disponible, skip test_cranelift_threads");
            return;
        }
        // probe gcc puede compilar
        {
            let pdir = std::env::temp_dir().join("lumen_cr_probe_thr");
            let _ = std::fs::create_dir_all(&pdir);
            let pc = pdir.join("p.c");
            let pe = pdir.join("p.exe");
            let _ = std::fs::write(&pc, "int main(void){return 0;}\n");
            let _ = std::fs::remove_file(&pe);
            let mut pargs = vec![
                pc.to_str().unwrap().to_string(),
                "-o".to_string(),
                pe.to_str().unwrap().to_string(),
            ];
            if cfg!(not(windows)) {
                pargs.push("-lpthread".to_string());
            }
            let ok = std::process::Command::new("gcc")
                .args(&pargs)
                .output()
                .map(|o| o.status.success() && pe.exists())
                .unwrap_or(false);
            if !ok {
                eprintln!("gcc no puede compilar, skip test_cranelift_threads");
                return;
            }
            let _ = std::fs::remove_file(pc);
            let _ = std::fs::remove_file(pe);
        }
        if cfg!(windows) {
            // Windows: el runtime de hilos requiere sincronización precisa, flaky en este entorno → skip para CI local
            // CI Linux sí lo ejecuta (linux-test en ubuntu-latest con pthread)
            eprintln!("skip test_cranelift_threads en Windows");
            return;
        }
        let source = r#"
            funcion entero suma_parcial(entero a, entero b) {
                sea acc = a * b;
                entero i = 0;
                mientras (i < 1000) { acc = acc + i; i = i + 1; }
                retornar acc;
            }

            sea h1 = __hilo_lanzar("suma_parcial", 10, 20);
            sea h2 = __hilo_lanzar("suma_parcial", 30, 40);
            sea r1 = __hilo_esperar(h1);
            sea r2 = __hilo_esperar(h2);
            imprimir(r1 + r2);
        "#;
        let tokens = lumen_lexer::Lexer::new(source).tokenize();
        let (mut program, _) = lumen_parser::Parser::new(tokens.0).parse();
        let sem_errors = lumen_sema::SemanticAnalyzer::new().analyze(&mut program);
        assert!(sem_errors.is_empty(), "sema fallo: {:?}", sem_errors);
        lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        let unsupported = cranelift_supported(&ir);
        assert!(
            unsupported.is_empty(),
            "cranelift rechazó hilos: {:?}",
            unsupported
        );
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(0);
        let dir = std::env::temp_dir().join(format!(
            "lumen_cr_thrhilos_{}_{}",
            std::process::id(),
            nanos
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let obj_path = dir.join("test_h.obj");
        let shim_path = dir.join("test_h_rt.c");
        let exe_path = dir.join("test_h.exe");
        compile_to_object(&ir, obj_path.to_str().unwrap()).expect("compile_to_object fallo");
        std::fs::write(&shim_path, lw_shim_source_for(&ir)).unwrap();
        let mut args = vec![
            obj_path.to_str().unwrap().to_string(),
            shim_path.to_str().unwrap().to_string(),
            "-O2".to_string(),
            "-o".to_string(),
            exe_path.to_str().unwrap().to_string(),
            "-lm".to_string(),
        ];
        if cfg!(not(windows)) {
            args.push("-lpthread".to_string());
        }
        let status = std::process::Command::new("gcc")
            .args(&args)
            .output()
            .expect("gcc fallo al invocar");
        assert!(
            status.status.success(),
            "gcc fallo al compilar/link:\n{}",
            String::from_utf8_lossy(&status.stderr)
        );
        let out = loop {
            match std::process::Command::new(&exe_path).output() {
                Ok(o) => break o,
                Err(_) => std::thread::sleep(std::time::Duration::from_millis(300)),
            }
        };
        let test_out = String::from_utf8_lossy(&out.stdout)
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            test_out,
            vec!["1000400"],
            "salida completa: {:?}\nstderr: {}",
            test_out,
            String::from_utf8_lossy(&out.stderr)
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
        // v3.5.7: funciones de usuario con prefijo lum_, entry C aparte,
        // binarios vía runtime _lw_* (mismo modelo que Cranelift)
        assert!(llvm.contains("define i64 @lum_main("));
        assert!(llvm.contains("define i32 @main()"));
        assert!(llvm.contains("call i64 @_lw_bin(i64 1"));
        assert!(llvm.contains("declare i64 @_lw_int(i64)"));
    }

    #[test]
    fn test_llvm_ir_runtime() {
        // v3.5.7: el LLVM IR textual compila con clang + shim _lw_* y produce
        // la misma salida que la VM (muestra: aritmética, texto, listas).
        // En Windows clang suele no estar instalado → skip en vez de FAIL
        if std::process::Command::new("clang")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
            == false
        {
            eprintln!("clang no disponible, skip test_llvm_ir_runtime");
            return;
        }
        // probe: clang debe poder compilar un hello mínimo
        {
            let pdir = std::env::temp_dir().join("lumen_ll_probe");
            let _ = std::fs::create_dir_all(&pdir);
            let pc = pdir.join("p.c");
            let pe = pdir.join("p.exe");
            let _ = std::fs::write(&pc, "int main(void){return 0;}\n");
            let _ = std::fs::remove_file(&pe);
            let ok = std::process::Command::new("clang")
                .args([pc.to_str().unwrap(), "-o", pe.to_str().unwrap()])
                .output()
                .map(|o| o.status.success() && pe.exists())
                .unwrap_or(false);
            if !ok {
                eprintln!("clang no puede compilar, skip test_llvm_ir_runtime");
                return;
            }
            let _ = std::fs::remove_file(pc);
            let _ = std::fs::remove_file(pe);
        }
        let source = r#"
            funcion entero fib(n: entero) {
                si (n < 2) { retornar n; }
                retornar fib(n - 1) + fib(n - 2);
            }
            funcion vacio tocar(prestado mut lista<entero> xs) { xs.agregar(9); }
            funcion vacio main() {
                imprimir(fib(10));
                imprimir(7 / 2, " ", 7 % 3);
                sea s = "lumen";
                imprimir(s.largo(), ":", s[0]);
                sea xs = [1, 2];
                xs.agregar(3);
                imprimir(xs.largo());
                imprimir(2.5 * 2.0);
                // v3.5.42 (bug fuzz gen_ref): write-through de `prestado mut`
                sea ys = [1];
                tocar(ys);
                imprimir(ys[1]);
            }
        "#;
        let tokens = lumen_lexer::Lexer::new(source).tokenize();
        let (mut program, _) = lumen_parser::Parser::new(tokens.0).parse();
        let sem_errors = lumen_sema::SemanticAnalyzer::new().analyze(&mut program);
        assert!(sem_errors.is_empty(), "sema fallo: {:?}", sem_errors);
        lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
        let ir = lumen_ir::IRBuilder::new().build(&program);
        assert!(
            llvm_supported(&ir).is_empty(),
            "llvm rechazó: {:?}",
            llvm_supported(&ir)
        );
        let llvm = compile_to_llvm_ir(&ir);
        let dir = std::env::temp_dir().join(format!(
            "lumen_ll_test_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let ll_path = dir.join("test.ll");
        let shim_path = dir.join("test_rt.c");
        let exe_path = dir.join("test.exe");
        std::fs::write(&ll_path, llvm).unwrap();
        std::fs::write(&shim_path, lw_shim_source()).unwrap();
        // v3.5.30: en Windows (clang con toolchain MSVC) `-lm` se traduce a
        // `m.lib` y el linker falla con LNK1181 ("cannot open input file
        // 'm.lib'"); libm es parte de la CRT allí.
        let mut args = vec![
            ll_path.to_str().unwrap().to_string(),
            shim_path.to_str().unwrap().to_string(),
            "-O2".to_string(),
            "-o".to_string(),
            exe_path.to_str().unwrap().to_string(),
        ];
        if cfg!(not(windows)) {
            args.push("-lm".to_string());
        }
        let status = std::process::Command::new("clang")
            .args(&args)
            .output()
            .expect("clang fallo al invocar");
        assert!(
            status.status.success(),
            "clang fallo al compilar:\n{}",
            String::from_utf8_lossy(&status.stderr)
        );
        let out = std::process::Command::new(&exe_path)
            .output()
            .expect("exec fallo");
        let test_out = String::from_utf8_lossy(&out.stdout)
            .replace("\r\n", "\n")
            .lines()
            .map(|l| l.trim_end().to_string())
            .collect::<Vec<_>>();
        assert_eq!(
            test_out,
            vec!["55", "3 1", "5:l", "3", "5", "9"],
            "salida completa: {:?}\nstderr: {}",
            test_out,
            String::from_utf8_lossy(&out.stderr)
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
