use crate::value::{FixHasher, Value};
use im::HashMap as ImMap;
use lumen_codegen::bytecode::{Bytecode, FuncMeta, Instruction, Opcode};
use std::collections::HashMap;
#[cfg(feature = "full")]
use std::sync::Arc;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;
use chrono::{TimeZone, Datelike, Timelike, Utc};

pub static JS_EVAL: OnceLock<fn(&str) -> String> = OnceLock::new();
#[cfg(feature = "full")]
use crate::coro_ffi::Coroutine;
#[cfg(feature = "full")]
use crate::crypto_ffi::Bcrypt;
#[cfg(feature = "full")]
use crate::gui_ffi::GuiWindow;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub func_name: String,
    pub return_ip: usize,
}

#[derive(Debug)]
pub enum VmError {
    Runtime(String),
    StackUnderflow,
    UndefinedVariable(String),
    UndefinedFunction(String),
    DivisionByZero,
    TypeError(String),
}

impl std::fmt::Display for VmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VmError::Runtime(s) => write!(f, "Error runtime: {}", s),
            VmError::StackUnderflow => write!(f, "Error: Stack underflow"),
            VmError::UndefinedVariable(s) => write!(f, "Error: Variable '{}' no definida", s),
            VmError::UndefinedFunction(s) => write!(f, "Error: Función '{}' no definida", s),
            VmError::DivisionByZero => write!(f, "Error: División por cero"),
            VmError::TypeError(s) => write!(f, "Error de tipo: {}", s),
        }
    }
}

impl VmError {
    pub fn with_stack(self, stack: &[CallFrame]) -> String {
        let msg = match &self {
            VmError::Runtime(s) => format!("Error: {}", s),
            VmError::StackUnderflow => "Error: Stack underflow".to_string(),
            VmError::UndefinedVariable(s) => format!("Error: Variable '{}' no definida", s),
            VmError::UndefinedFunction(s) => format!("Error: Función '{}' no definida", s),
            VmError::DivisionByZero => "Error: División por cero".to_string(),
            VmError::TypeError(s) => format!("Error de tipo: {}", s),
        };
        if stack.is_empty() {
            msg
        } else {
            let trace: Vec<String> = stack
                .iter()
                .map(|f| format!("  · {}", f.func_name))
                .collect();
            format!("{}\n\nPila de llamadas:\n{}", msg, trace.join("\n"))
        }
    }
}

pub struct VM {
    stack: Vec<Value>,
    locals: Vec<HashMap<String, Value, FixHasher>>,
    ip: usize,
    bytecode: Bytecode,
    output: Vec<String>,
    call_stack: Vec<CallFrame>,
    func_index_cache: HashMap<String, usize>,
    pub debug: bool,
    pub breakpoints: Vec<usize>,
    step_mode: bool,
    last_instr: Option<Instruction>,
    pub instr_count: usize,
    #[cfg(feature = "full")]
    bcrypt: Option<Arc<Bcrypt>>,
    #[cfg(feature = "full")]
    gui_windows: HashMap<String, GuiWindow>,
    #[cfg(feature = "full")]
    coroutines: HashMap<String, Coroutine>,
    #[cfg(feature = "full")]
    current_coro: Option<String>,
    #[cfg(feature = "full")]
    #[allow(clippy::type_complexity)]
    main_saved: Option<(Vec<Value>, Vec<HashMap<String, Value, FixHasher>>, usize)>,
    tcp_listener: Option<std::net::TcpListener>,
    #[cfg(feature = "full")]
    cluster_streams: HashMap<String, std::net::TcpStream>,
    #[cfg(feature = "full")]
    scope_handles: Vec<HashMap<String, Value, FixHasher>>,
    #[cfg(feature = "full")]
    thread_handles: HashMap<String, std::thread::JoinHandle<Value>>,
    #[cfg(feature = "full")]
    channels: HashMap<String, (Option<std::sync::mpsc::Sender<Value>>, Option<std::sync::mpsc::Receiver<Value>>)>,
    #[cfg(feature = "full")]
    mutexes: HashMap<String, std::sync::Mutex<Value>>,
    #[cfg(feature = "full")]
    actors: HashMap<String, (Option<std::sync::mpsc::Sender<Value>>, Option<std::sync::mpsc::Receiver<Value>>)>,
    #[cfg(feature = "full")]
    generators: HashMap<String, String>,
    #[cfg(feature = "full")]
    ffi_libraries: HashMap<String, usize>,
    #[cfg(feature = "full")]
    task_results: HashMap<String, std::sync::mpsc::Receiver<Value>>,
    #[cfg(feature = "full")]
    task_counter: usize,
}

/// Helper to convert VmError into the builtin return type.
fn builtin_err(err: VmError) -> Option<Result<(), VmError>> {
    Some(Err(err))
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        let ip = if bytecode.funcs.is_empty() {
            0
        } else {
            bytecode
                .funcs
                .iter()
                .find(|f| f.name == "__main__")
                .or_else(|| bytecode.funcs.iter().find(|f| f.name == "main"))
                .or_else(|| bytecode.funcs.iter().find(|f| f.name == "principal"))
                .map(|f| f.start)
                .unwrap_or(usize::MAX)
        };
        let mut func_index_cache = HashMap::new();
        for (i, func) in bytecode.funcs.iter().enumerate() {
            func_index_cache.insert(func.name.clone(), i);
        }
        #[cfg(feature = "full")]
        let bcrypt = match Bcrypt::load() {
            Ok(b) => Some(Arc::new(b)),
            Err(_) => None,
        };
        Self {
            stack: Vec::new(),
            locals: vec![HashMap::with_hasher(FixHasher::default())],
            ip,
            bytecode,
            output: Vec::new(),
            call_stack: Vec::new(),
            func_index_cache,
            debug: false,
            breakpoints: Vec::new(),
            step_mode: false,
            last_instr: None,
            instr_count: 0,
            #[cfg(feature = "full")]
            bcrypt,
            #[cfg(feature = "full")]
            gui_windows: HashMap::new(),
            #[cfg(feature = "full")]
            coroutines: HashMap::new(),
            #[cfg(feature = "full")]
            current_coro: None,
            #[cfg(feature = "full")]
            main_saved: None,
            tcp_listener: None,
            #[cfg(feature = "full")]
            cluster_streams: HashMap::new(),
            #[cfg(feature = "full")]
            scope_handles: Vec::new(),
            #[cfg(feature = "full")]
            thread_handles: HashMap::new(),
            #[cfg(feature = "full")]
            channels: HashMap::new(),
            #[cfg(feature = "full")]
            mutexes: HashMap::new(),
            #[cfg(feature = "full")]
            actors: HashMap::new(),
            #[cfg(feature = "full")]
            generators: HashMap::new(),
            #[cfg(feature = "full")]
            ffi_libraries: HashMap::new(),
            #[cfg(feature = "full")]
            task_results: HashMap::new(),
            #[cfg(feature = "full")]
            task_counter: 0,
        }
    }

    fn find_func(&self, name: &str) -> Option<&FuncMeta> {
        self.func_index_cache
            .get(name)
            .and_then(|&idx| self.bytecode.funcs.get(idx))
    }

    fn call_core_builtin(&mut self, name: &str, args: &[Value]) -> Option<Result<(), VmError>> {
        let args = args.to_vec();
        if name == "imprimir" || name == "print" {
            for arg in args {
                let s = format!("{}", arg);
                self.output.push(s);
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "leer" || name == "read" {
            self.push(Value::str(String::new()));
            return Some(Ok(()));
        }

        if name == "a_texto" || name == "to_texto" || name == "__str_from" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::str(s));
            return Some(Ok(()));
        }

        if name == "__str_a_entero" || name == "__texto_a_entero" {
            let mut s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(dot) = s.find('.') {
                s.truncate(dot);
            }
            match s.parse::<i64>() {
                Ok(n) => self.push(Value::Int(n)),
                Err(_) => self.push(Value::Int(0)),
            }
            return Some(Ok(()));
        }

        if name == "largo" || name == "len" {
            match args.clone().into_iter().next() {
                Some(Value::Array(v)) => self.push(Value::Int(v.len() as i64)),
                Some(Value::Str(s)) => self.push(Value::Int(s.chars().count() as i64)),
                Some(other) => {
                    return builtin_err(VmError::TypeError(format!(
                        "'largo' espera lista o texto, no {:?}",
                        other
                    )))
                }
                None => {
                    return builtin_err(VmError::TypeError(
                        "'largo' espera 1 argumento".to_string(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "agregar" || name == "push" {
            let mut iter = args.clone().into_iter();
            let list = iter.next().unwrap_or(Value::arr(vec![]));
            let item = iter.next().unwrap_or(Value::Void);
            match list {
                Value::Array(mut v) => {
                    Arc::make_mut(&mut v).push(item);
                    self.push(Value::Array(v));
                }
                _ => {
                    return builtin_err(VmError::TypeError(
                        "'agregar' espera una lista".to_string(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__str_len" || name == "__str_longitud" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::Int(s.len() as i64));
            return Some(Ok(()));
        }

        if name == "__str_upper" || name == "__str_mayusculas" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::str(s.to_uppercase()));
            return Some(Ok(()));
        }

        if name == "__str_lower" || name == "__str_minusculas" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::str(s.to_lowercase()));
            return Some(Ok(()));
        }

        if name == "__str_trim" || name == "__str_recortar" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::str(s.trim().to_string()));
            return Some(Ok(()));
        }

        if name == "__str_contains" || name == "__str_contiene" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let sub = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::Bool(s.contains(&sub)));
            return Some(Ok(()));
        }

        if name == "__str_split" || name == "__str_dividir" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let delim = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let parts: Vec<Value> = if delim.is_empty() {
                s.chars().map(|c| Value::str(c.to_string())).collect()
            } else {
                s.split(&delim).map(|p| Value::str(p.to_string())).collect()
            };
            self.push(Value::arr(parts));
            return Some(Ok(()));
        }

        if name == "__str_ord" || name == "__str_codigo" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let codes: Vec<Value> = s.chars().map(|c| Value::Int(c as i64)).collect();
            self.push(Value::arr(codes));
            return Some(Ok(()));
        }

        if name == "__str_slice" || name == "__str_subcadena" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let start = args.get(1).and_then(|v| match v {
                Value::Int(i) => Some(*i as usize),
                _ => v.as_num().map(|f| f as usize),
            }).unwrap_or(0);
            let end = args.get(2).and_then(|v| match v {
                Value::Int(i) => {
                    if *i == -1 { Some(s.len()) } else { Some(*i as usize) }
                }
                _ => v.as_num().map(|f| f as usize),
            }).unwrap_or(s.len());
            let start = start.min(s.len());
            let end = end.min(s.len()).max(start);
            let sub: String = s.chars().skip(start).take(end - start).collect();
            self.push(Value::str(sub));
            return Some(Ok(()));
        }

        if name == "__str_concat_list" || name == "__str_concatenar_lista" {
            let list = args.first().cloned().unwrap_or(Value::arr(vec![]));
            match list {
                Value::Array(items) => {
                    let result = items.iter().map(|v| format!("{}", v)).collect::<String>();
                    self.push(Value::str(result));
                }
                _ => {
                    return builtin_err(VmError::TypeError(
                        "__str_concat_list espera una lista".to_string(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__str_starts_with" || name == "__str_empieza_con" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let prefix = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::Bool(s.starts_with(&prefix)));
            return Some(Ok(()));
        }

        if name == "__str_to_chars" || name == "__str_a_caracteres" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let chars: Vec<Value> = s.chars().map(|c| Value::str(c.to_string())).collect();
            self.push(Value::arr(chars));
            return Some(Ok(()));
        }

        if name == "__str_reemplazar" || name == "__str_replace" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let from = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let to = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::str(s.replace(&from, &to)));
            return Some(Ok(()));
        }

        if name == "__str_subcadena_chars" || name == "__str_slice_chars" {
            let cs = match args.first() {
                Some(Value::Array(a)) => a.clone(),
                _ => Arc::new(vec![]),
            };
            let st = args.get(1).and_then(|v| v.as_num()).map(|f| f as i64).unwrap_or(0);
            let en = args.get(2).and_then(|v| v.as_num()).map(|f| f as i64).unwrap_or(-1);
            let n = cs.len() as i64;
            let st = st.max(0).min(n);
            let en = if en < 0 { n } else { en.max(0).min(n) };
            let mut out = String::new();
            for c in cs.iter().skip(st as usize).take((en - st).max(0) as usize) {
                out.push_str(&format!("{}", c));
            }
            self.push(Value::str(out));
            return Some(Ok(()));
        }

        if name == "__str_chr" || name == "__str_caracter" {
            let n = args
                .first()
                .and_then(|v| match v {
                    Value::Int(i) => Some(*i),
                    _ => v.as_num().map(|f| f as i64),
                })
                .unwrap_or(0);
            let c = char::from_u32(n as u32).map(|c| c.to_string()).unwrap_or_default();
            self.push(Value::str(c));
            return Some(Ok(()));
        }

        if name == "__file_read" || name == "__leer_archivo" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::fs::read_to_string(&path) {
                Ok(content) => self.push(Value::Exito(Box::new(Value::str(content)))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__file_write" || name == "__escribir_archivo" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match std::fs::write(&path, &content) {
                Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__file_exists" || name == "__existe_archivo" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::Bool(std::path::Path::new(&path).exists()));
            return Some(Ok(()));
        }

        if name == "__file_size" || name == "__tamano_archivo" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::fs::metadata(&path) {
                Ok(meta) => self.push(Value::Int(meta.len() as i64)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__file_append" || name == "__agregar_archivo" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            use std::io::Write;
            match std::fs::OpenOptions::new().create(true).append(true).open(&path) {
                Ok(mut file) => {
                    match file.write_all(content.as_bytes()) {
                        Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__file_write_binary" || name == "__escribir_archivo_bin" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let bytes = match args.get(1) {
                Some(Value::Array(arr)) => arr.iter().filter_map(|v| match v {
                    Value::Int(n) if *n >= 0 && *n <= 255 => Some(*n as u8),
                    _ => None,
                }).collect::<Vec<u8>>(),
                _ => Vec::new(),
            };
            match std::fs::write(&path, &bytes) {
                Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__file_bytes" || name == "__leer_bytes" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::fs::read(&path) {
                Ok(data) => self.push(Value::arr(
                    data.iter().map(|&b| Value::Int(b as i64)).collect(),
                )),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__a_f64_bytes" || name == "__bytes_a_f64" {
            let mut buf = [0u8; 8];
            if let Some(Value::Array(arr)) = args.first() {
                for (i, b) in arr.iter().take(8).enumerate() {
                    if let Value::Int(n) = b {
                        buf[i] = *n as u8;
                    }
                }
            }
            self.push(Value::Float(f64::from_le_bytes(buf)));
            return Some(Ok(()));
        }

        if name == "__num_a_f64_bytes" || name == "__numero_a_bytes_f64" {
            let n = args.first().map(|v| match v {
                Value::Int(i) => *i as f64,
                Value::Float(f) => *f,
                _ => 0.0,
            }).unwrap_or(0.0);
            let bytes: Vec<Value> = n.to_le_bytes().iter().map(|&b| Value::Int(b as i64)).collect();
            self.push(Value::arr(bytes));
            return Some(Ok(()));
        }

        if name == "__codegen_a_nvc" || name == "__codegen_a_nvc" {
            // Takes a codegen map, returns Array<Int> of .nvc bytes
            let cg = args.first().cloned().unwrap_or(Value::Void);
            let result = self.codegen_to_nvc(cg);
            return match result {
                Ok(bytes) => { self.push(bytes); Some(Ok(())) }
                Err(_) => { self.push(Value::Error(Box::new(Value::str("codegen_to_nvc failed")))); Some(Ok(())) }
            };
        }

        if name == "__compile_nv" || name == "__compilar_nv" {
            // Compile a .nv source file to .nvc bytes using the native Rust pipeline
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let source = match std::fs::read_to_string(&path) {
                Ok(s) => s,
                Err(e) => {
                    self.push(Value::Error(Box::new(Value::str(format!("IO: {}", e)))));
                    return Some(Ok(()));
                }
            };
            let base_path = std::path::Path::new(&path);
            let mut lib_dirs = vec![std::path::PathBuf::from("stdlib")];
            if let Ok(cwd) = std::env::current_dir() {
                lib_dirs.push(cwd.join("stdlib"));
            }
            let mut loader = lumen_sema::loader::ModuleLoader::new(lib_dirs);
            let mut program = match loader.resolve_imports(&source, base_path) {
                Ok(p) => p,
                Err(e) => {
                    self.push(Value::Error(Box::new(Value::str(format!("Loader error: {:?}", e)))));
                    return Some(Ok(()));
                }
            };
            let sema = lumen_sema::sema::SemanticAnalyzer::new();
            let sem_errors = sema.analyze(&mut program);
            if !sem_errors.is_empty() {
                let msg = sem_errors.iter().map(|e| format!("{}", e.message)).collect::<Vec<_>>().join("; ");
                self.push(Value::Error(Box::new(Value::str(format!("Sem error: {}", msg)))));
                return Some(Ok(()));
            }
            let builder = lumen_ir::builder::IRBuilder::new();
            let ir = builder.build(&program);
            let codegen = lumen_codegen::codegen::Codegen::new();
            let (bytecode, _) = codegen.generate(&ir);
            let bytes_vec = bytecode.encode();
            let bytes: Vec<Value> = bytes_vec.into_iter().map(|b| Value::Int(b as i64)).collect();
            self.push(Value::arr(bytes));
            return Some(Ok(()));
        }

        if name == "__time_now" || name == "__tiempo_ahora" {
            use std::time::{SystemTime, UNIX_EPOCH};
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default();
            self.push(Value::Int(now.as_secs() as i64));
            return Some(Ok(()));
        }

        if name == "__list_reverse" || name == "__lista_invertir" {
            let mut arr = match args.clone().into_iter().next() {
                Some(Value::Array(v)) => v,
                Some(other) => {
                    return builtin_err(VmError::TypeError(format!(
                        "__list_reverse espera una lista, no {:?}",
                        other
                    )))
                }
                None => {
                    return builtin_err(VmError::TypeError(
                        "__list_reverse espera 1 argumento".to_string(),
                    ))
                }
            };
            Arc::make_mut(&mut arr).reverse();
            self.push(Value::Array(arr));
            return Some(Ok(()));
        }

        if name == "__list_sort" || name == "__lista_ordenar" {
            let mut arr = match args.clone().into_iter().next() {
                Some(Value::Array(v)) => v,
                Some(other) => {
                    return builtin_err(VmError::TypeError(format!(
                        "__list_sort espera una lista, no {:?}",
                        other
                    )))
                }
                None => {
                    return builtin_err(VmError::TypeError(
                        "__list_sort espera 1 argumento".to_string(),
                    ))
                }
            };
            Arc::make_mut(&mut arr).sort_by(|a, b| {
                let an = a.as_num().unwrap_or(f64::MAX);
                let bn = b.as_num().unwrap_or(f64::MAX);
                an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
            });
            self.push(Value::Array(arr));
            return Some(Ok(()));
        }

        if name == "__map_new" || name == "__map_nuevo" {
            self.push(Value::Map(ImMap::with_hasher(FixHasher::default())));
            return Some(Ok(()));
        }

        if name == "__map_set" || name == "__map_poner" {
            let mut it = args.clone().into_iter();
            let m = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let k = it.next().unwrap_or(Value::Void);
            let v = it.next().unwrap_or(Value::Void);
            match m {
                Value::Map(mut m) => {
                    m.insert(k, v);
                    self.push(Value::Map(m));
                }
                _ => return builtin_err(VmError::TypeError("__map_set espera diccionario".into())),
            }
            return Some(Ok(()));
        }

        if name == "__map_get" || name == "__map_obtener" {
            let mut it = args.clone().into_iter();
            let m = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let k = it.next().unwrap_or(Value::Void);
            match m {
                Value::Map(m) => {
                    self.push(m.get(&k).cloned().unwrap_or(Value::Void));
                }
                _ => return builtin_err(VmError::TypeError("__map_get espera diccionario".into())),
            }
            return Some(Ok(()));
        }

        if name == "__map_len" || name == "__map_longitud" {
            let m = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match m {
                Value::Map(m) => self.push(Value::Int(m.len() as i64)),
                _ => return builtin_err(VmError::TypeError("__map_len espera diccionario".into())),
            }
            return Some(Ok(()));
        }

        if name == "__map_keys" || name == "__map_claves" {
            let m = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match m {
                Value::Map(m) => {
                    let keys: Vec<Value> = m.into_iter().map(|(k, _)| k).collect();
                    self.push(Value::arr(keys));
                }
                _ => {
                    return builtin_err(VmError::TypeError("__map_keys espera diccionario".into()))
                }
            }
            return Some(Ok(()));
        }

        if name == "__map_contains" || name == "__map_contiene" {
            let mut it = args.clone().into_iter();
            let m = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let k = it.next().unwrap_or(Value::Void);
            match m {
                Value::Map(m) => self.push(Value::Bool(m.contains_key(&k))),
                _ => {
                    return builtin_err(VmError::TypeError(
                        "__map_contains espera diccionario".into(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__set_new" || name == "__conjunto_nuevo" {
            self.push(Value::Map(ImMap::with_hasher(FixHasher::default())));
            return Some(Ok(()));
        }

        if name == "__set_add" || name == "__conjunto_agregar" {
            let mut it = args.clone().into_iter();
            let s = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let item = it.next().unwrap_or(Value::Void);
            match s {
                Value::Map(mut m) => {
                    m.insert(item, Value::Bool(true));
                    self.push(Value::Map(m));
                }
                _ => return builtin_err(VmError::TypeError("__set_add espera conjunto".into())),
            }
            return Some(Ok(()));
        }

        if name == "__set_has" || name == "__conjunto_tiene" {
            let mut it = args.clone().into_iter();
            let s = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let item = it.next().unwrap_or(Value::Void);
            match s {
                Value::Map(m) => self.push(Value::Bool(m.contains_key(&item))),
                _ => return builtin_err(VmError::TypeError("__set_has espera conjunto".into())),
            }
            return Some(Ok(()));
        }

        if name == "__set_union" || name == "__conjunto_unir" {
            let mut it = args.clone().into_iter();
            let a = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let b = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match (a, b) {
                (Value::Map(mut m1), Value::Map(m2)) => {
                    for (k, v) in m2 {
                        if !m1.contains_key(&k) { m1.insert(k, v); }
                    }
                    self.push(Value::Map(m1));
                }
                _ => return builtin_err(VmError::TypeError("__set_union espera conjuntos".into())),
            }
            return Some(Ok(()));
        }

        if name == "__set_inter" || name == "__conjunto_interseccion" {
            let mut it = args.clone().into_iter();
            let a = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let b = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match (a, b) {
                (Value::Map(m1), Value::Map(m2)) => {
                    let r: ImMap<Value, Value, FixHasher> = m1
                        .into_iter()
                        .filter(|(k, _)| m2.contains_key(k))
                        .collect();
                    self.push(Value::Map(r));
                }
                _ => return builtin_err(VmError::TypeError("__set_inter espera conjuntos".into())),
            }
            return Some(Ok(()));
        }

        if name == "__set_diff" || name == "__conjunto_diferencia" {
            let mut it = args.clone().into_iter();
            let a = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let b = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match (a, b) {
                (Value::Map(m1), Value::Map(m2)) => {
                    let r: ImMap<Value, Value, FixHasher> = m1
                        .into_iter()
                        .filter(|(k, _)| !m2.contains_key(k))
                        .collect();
                    self.push(Value::Map(r));
                }
                _ => return builtin_err(VmError::TypeError("__set_diff espera conjuntos".into())),
            }
            return Some(Ok(()));
        }

        if name == "__deque_new" || name == "__deque_nuevo" {
            self.push(Value::arr(vec![]));
            return Some(Ok(()));
        }

        if name == "__deque_push_front" || name == "__deque_agregar_frente" {
            let mut it = args.clone().into_iter();
            let d = it.next().unwrap_or(Value::arr(vec![]));
            let item = it.next().unwrap_or(Value::Void);
            match d {
                Value::Array(mut v) => {
                    Arc::make_mut(&mut v).insert(0, item);
                    self.push(Value::Array(v));
                }
                _ => {
                    return builtin_err(VmError::TypeError(
                        "__deque_push_front espera deque".into(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__deque_push_back" || name == "__deque_agregar_final" {
            let mut it = args.clone().into_iter();
            let d = it.next().unwrap_or(Value::arr(vec![]));
            let item = it.next().unwrap_or(Value::Void);
            match d {
                Value::Array(mut v) => {
                    Arc::make_mut(&mut v).push(item);
                    self.push(Value::Array(v));
                }
                _ => {
                    return builtin_err(VmError::TypeError("__deque_push_back espera deque".into()))
                }
            }
            return Some(Ok(()));
        }

        if name == "__deque_pop_front" || name == "__deque_quitar_frente" {
            let d = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match d {
                Value::Array(mut v) => self.push(if v.is_empty() {
                    Value::Void
                } else {
                    Arc::make_mut(&mut v).remove(0)
                }),
                _ => {
                    return builtin_err(VmError::TypeError("__deque_pop_front espera deque".into()))
                }
            }
            return Some(Ok(()));
        }

        if name == "__deque_pop_back" || name == "__deque_quitar_final" {
            let d = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match d {
                Value::Array(mut v) => self.push(if v.is_empty() {
                    Value::Void
                } else {
                    Arc::make_mut(&mut v).pop().unwrap_or(Value::Void)
                }),
                _ => {
                    return builtin_err(VmError::TypeError("__deque_pop_back espera deque".into()))
                }
            }
            return Some(Ok(()));
        }

        if name == "__deque_len" || name == "__deque_longitud" {
            let d = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match d {
                Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                _ => return builtin_err(VmError::TypeError("__deque_len espera deque".into())),
            }
            return Some(Ok(()));
        }

        if name == "__heap_new" || name == "__monticulo_nuevo" {
            self.push(Value::arr(vec![]));
            return Some(Ok(()));
        }

        if name == "__heap_push" || name == "__monticulo_agregar" {
            let mut it = args.clone().into_iter();
            let h = it.next().unwrap_or(Value::arr(vec![]));
            let item = it.next().unwrap_or(Value::Void);
            match h {
                Value::Array(mut v) => {
                    Arc::make_mut(&mut v).push(item);
                    Arc::make_mut(&mut v).sort_by(|a, b| {
                        let an = a.as_num().unwrap_or(f64::MIN);
                        let bn = b.as_num().unwrap_or(f64::MIN);
                        bn.partial_cmp(&an).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    self.push(Value::Array(v));
                }
                _ => return builtin_err(VmError::TypeError("__heap_push espera heap".into())),
            }
            return Some(Ok(()));
        }

        if name == "__heap_pop" || name == "__monticulo_quitar" {
            let h = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match h {
                Value::Array(mut v) => self.push(if v.is_empty() {
                    Value::Void
                } else {
                    Arc::make_mut(&mut v).remove(0)
                }),
                _ => return builtin_err(VmError::TypeError("__heap_pop espera heap".into())),
            }
            return Some(Ok(()));
        }

        if name == "__heap_peek" || name == "__monticulo_ver" {
            let h = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match h {
                Value::Array(v) => self.push(if v.is_empty() {
                    Value::Void
                } else {
                    v[0].clone()
                }),
                _ => return builtin_err(VmError::TypeError("__heap_peek espera heap".into())),
            }
            return Some(Ok(()));
        }

        if name == "__heap_len" || name == "__monticulo_longitud" {
            let h = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match h {
                Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                _ => return builtin_err(VmError::TypeError("__heap_len espera heap".into())),
            }
            return Some(Ok(()));
        }

        if name == "__linked_new" || name == "__enlazada_nuevo" {
            self.push(Value::arr(vec![]));
            return Some(Ok(()));
        }

        if name == "__linked_push_front" || name == "__enlazada_agregar_frente" {
            let mut it = args.clone().into_iter();
            let l = it.next().unwrap_or(Value::arr(vec![]));
            let item = it.next().unwrap_or(Value::Void);
            match l {
                Value::Array(mut v) => {
                    Arc::make_mut(&mut v).insert(0, item);
                    self.push(Value::Array(v));
                }
                _ => {
                    return builtin_err(VmError::TypeError(
                        "__linked_push_front espera linked".into(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__linked_push_back" || name == "__enlazada_agregar_final" {
            let mut it = args.clone().into_iter();
            let l = it.next().unwrap_or(Value::arr(vec![]));
            let item = it.next().unwrap_or(Value::Void);
            match l {
                Value::Array(mut v) => {
                    Arc::make_mut(&mut v).push(item);
                    self.push(Value::Array(v));
                }
                _ => {
                    return builtin_err(VmError::TypeError(
                        "__linked_push_back espera linked".into(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__linked_pop_front" || name == "__enlazada_quitar_frente" {
            let l = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match l {
                Value::Array(mut v) => self.push(if v.is_empty() {
                    Value::Void
                } else {
                    Arc::make_mut(&mut v).remove(0)
                }),
                _ => {
                    return builtin_err(VmError::TypeError(
                        "__linked_pop_front espera linked".into(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__linked_pop_back" || name == "__enlazada_quitar_final" {
            let l = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match l {
                Value::Array(mut v) => self.push(if v.is_empty() {
                    Value::Void
                } else {
                    Arc::make_mut(&mut v).pop().unwrap_or(Value::Void)
                }),
                _ => {
                    return builtin_err(VmError::TypeError(
                        "__linked_pop_back espera linked".into(),
                    ))
                }
            }
            return Some(Ok(()));
        }

        if name == "__linked_len" || name == "__enlazada_longitud" {
            let l = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match l {
                Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                _ => return builtin_err(VmError::TypeError("__linked_len espera linked".into())),
            }
            return Some(Ok(()));
        }

        if name == "__regex_new" || name == "__regex_nuevo" {
            let pat = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match regex::Regex::new(&pat) {
                Ok(_) => self.push(Value::Bool(true)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__regex_is_match" || name == "__regex_coincide" {
            let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match regex::Regex::new(&re_s) {
                Ok(r) => self.push(Value::Bool(r.is_match(&text))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__regex_captures" || name == "__regex_capturar" {
            let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match regex::Regex::new(&re_s) {
                Ok(r) => {
                    if let Some(caps) = r.captures(&text) {
                        let vs: Vec<Value> = caps
                            .iter()
                            .map(|m| {
                                Value::str(m.map(|x| x.as_str().to_string()).unwrap_or_default())
                            })
                            .collect();
                        self.push(Value::arr(vs));
                    } else {
                        self.push(Value::arr(vec![]));
                    }
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__regex_replace" || name == "__regex_reemplazar" {
            let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let rep = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            match regex::Regex::new(&re_s) {
                Ok(r) => self.push(Value::str(r.replace_all(&text, rep.as_str()).to_string())),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__unicode_normalize" || name == "__unicode_normalizar" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let form = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let nf: String = match form.as_str() {
                "NFC" => s.nfc().collect(),
                "NFD" => s.nfd().collect(),
                "NFKC" => s.nfkc().collect(),
                "NFKD" => s.nfkd().collect(),
                _ => s.nfc().collect(),
            };
            self.push(Value::str(nf));
            return Some(Ok(()));
        }

        if name == "__str_pad_start" || name == "__str_padding_inicio" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let ch = args
                .get(2)
                .map(|v| format!("{}", v))
                .unwrap_or_default()
                .chars()
                .next()
                .unwrap_or(' ');
            self.push(Value::str(format!(
                "{}{}",
                ch.to_string().repeat(len.saturating_sub(s.len())),
                s
            )));
            return Some(Ok(()));
        }

        if name == "__str_pad_end" || name == "__str_padding_fin" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let ch = args
                .get(2)
                .map(|v| format!("{}", v))
                .unwrap_or_default()
                .chars()
                .next()
                .unwrap_or(' ');
            self.push(Value::str(format!(
                "{}{}",
                s,
                ch.to_string().repeat(len.saturating_sub(s.len()))
            )));
            return Some(Ok(()));
        }

        if name == "__encoding_utf8" || name == "__codificacion_utf8" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::arr(
                s.bytes().map(|b| Value::Int(b as i64)).collect(),
            ));
            return Some(Ok(()));
        }

        if name == "__encoding_from_utf8" || name == "__desde_utf8" {
            let arr = args
                .clone()
                .into_iter()
                .next()
                .unwrap_or(Value::arr(vec![]));
            match arr {
                Value::Array(v) => {
                    let bytes: Vec<u8> = v
                        .iter()
                        .filter_map(|x| {
                            if let Value::Int(n) = x {
                                Some(*n as u8)
                            } else {
                                None
                            }
                        })
                        .collect();
                    self.push(Value::str(String::from_utf8_lossy(&bytes).to_string()));
                }
                _ => self.push(Value::str(String::new())),
            }
            return Some(Ok(()));
        }

        if name == "__buf_reader" || name == "__lector_buffer" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::fs::read_to_string(&path) {
                Ok(c) => {
                    let lines: Vec<Value> = c.lines().map(|l| Value::str(l.to_string())).collect();
                    self.push(Value::arr(lines));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__buf_writer" || name == "__escritor_buffer" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match std::fs::write(&path, &content) {
                Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__stream_chunks" || name == "__stream_trozos" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let size = args.get(1).and_then(|v| v.as_num()).unwrap_or(4096.0) as usize;
            match std::fs::read(&path) {
                Ok(data) => {
                    let chunks: Vec<Value> = data
                        .chunks(size)
                        .map(|c| Value::arr(c.iter().map(|&b| Value::Int(b as i64)).collect()))
                        .collect();
                    self.push(Value::arr(chunks));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__http_server" || name == "__http_servidor" {
            let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::str(format!("HTTP server on {}", addr)));
            return Some(Ok(()));
        }

        if name == "__serial_open" || name == "__serial_abrir" {
            self.push(Value::Bool(true));
            return Some(Ok(()));
        }

        if name == "__json_parse" || name == "__json_parsear" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match serde_json::from_str::<serde_json::Value>(&s) {
                Ok(v) => self.push(json_value_to_lumen(v)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__json_stringify" || name == "__json_texto" {
            let val = args.first().cloned().unwrap_or(Value::Void);
            let json = lumen_value_to_json(&val);
            self.push(Value::str(serde_json::to_string(&json).unwrap_or_default()));
            return Some(Ok(()));
        }

        if name == "__js_call" || name == "__js_llamar" {
            let fn_name = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let js_args: Vec<String> = args.iter().skip(1).map(|v| format!("{}", v)).collect();
            let js_code = format!(
                "__lumen_call('{}', [{}])",
                fn_name.replace('\'', "\\'"),
                js_args
                    .iter()
                    .map(|a| format!("'{}'", a.replace('\'', "\\'")))
                    .collect::<Vec<_>>()
                    .join(",")
            );
            if let Some(eval) = JS_EVAL.get() {
                let result = eval(&js_code);
                self.push(Value::str(result));
            } else {
                self.push(Value::str(js_code));
            }
            return Some(Ok(()));
        }

        if name == "__js_eval" || name == "__js_evaluar" {
            let js_code = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(eval) = JS_EVAL.get() {
                let result = eval(&js_code);
                self.push(Value::str(result));
            } else {
                self.push(Value::str(js_code));
            }
            return Some(Ok(()));
        }

        None
    }

    #[cfg(feature = "full")]
    fn call_full_builtin(&mut self, name: &str, args: &[Value]) -> Option<Result<(), VmError>> {
        let args = args.to_vec();
        if name == "__tcp_connect" || name == "__tcp_conectar" {
            let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::net::TcpStream::connect(&addr) {
                Ok(_) => self.push(Value::Bool(true)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__tcp_listen" || name == "__tcp_escuchar" {
            let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::net::TcpListener::bind(&addr) {
                Ok(l) => {
                    self.tcp_listener = Some(l);
                    self.push(Value::Bool(true));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__tcp_accept" || name == "__tcp_aceptar" {
            match &self.tcp_listener {
                Some(l) => match l.accept() {
                    Ok((_stream, _)) => {
                        let addr = _stream
                            .peer_addr()
                            .map(|a| a.to_string())
                            .unwrap_or_default();
                        self.push(Value::str(addr));
                    }
                    Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                },
                None => self.push(Value::Error(Box::new(Value::str("Sin listener")))),
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__http_get" || name == "__http_obtener" {
            let url = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match reqwest::blocking::get(&url) {
                Ok(resp) => {
                    let body = resp.text().unwrap_or_default();
                    self.push(Value::str(body));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__http_post" || name == "__http_enviar" {
            let url = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let body = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match reqwest::blocking::Client::new()
                .post(&url)
                .body(body)
                .send()
            {
                Ok(resp) => {
                    let text = resp.text().unwrap_or_default();
                    self.push(Value::str(text));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        // ██ FFI builtins ██
        if name == "__ffi_cargar" || name == "__ffi_load" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match unsafe { libloading::Library::new(&path) } {
                Ok(lib) => {
                    let ptr = Box::into_raw(Box::new(lib)) as usize;
                    let id = format!("lib_{}", self.ffi_libraries.len());
                    self.ffi_libraries.insert(id.clone(), ptr);
                    self.push(Value::str(id));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__ffi_llamar" || name == "__ffi_call" {
            let lib_id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let call_args: Vec<Value> = args.iter().skip(2).cloned().collect();
            let lib_ptr = match self.ffi_libraries.get(&lib_id) {
                Some(&p) => p,
                None => {
                    self.push(Value::Error(Box::new(Value::str(format!(
                        "Biblioteca '{}' no encontrada",
                        lib_id
                    )))));
                    return Some(Ok(()));
                }
            };
            let lib = unsafe { &*(lib_ptr as *const libloading::Library) };
            let result = unsafe {
                let sym: libloading::Symbol<
                    unsafe extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64,
                > = match lib.get(fn_name.as_bytes()) {
                    Ok(s) => s,
                    Err(e) => {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Símbolo '{}' no encontrado: {}",
                            fn_name, e
                        )))));
                        return Some(Ok(()));
                    }
                };
                let mut a = [0i64; 6];
                for (i, arg) in call_args.iter().enumerate().take(6) {
                    a[i] = arg.as_num().unwrap_or(0.0) as i64;
                }
                Value::Int(sym(a[0], a[1], a[2], a[3], a[4], a[5]))
            };
            self.push(result);
            return Some(Ok(()));
        }

        if name == "__ffi_asignar" || name == "__ffi_alloc" {
            let size = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let align = args.get(1).and_then(|v| v.as_num()).unwrap_or(8.0) as usize;
            let layout = match std::alloc::Layout::from_size_align(size, align) {
                Ok(l) => l,
                Err(e) => {
                    self.push(Value::Error(Box::new(Value::str(format!(
                        "Layout inválido: {}",
                        e
                    )))));
                    return Some(Ok(()));
                }
            };
            let ptr = unsafe { std::alloc::alloc(layout) };
            self.push(Value::Int(ptr as i64));
            return Some(Ok(()));
        }

        if name == "__ffi_liberar" || name == "__ffi_free" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let size = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let align = args.get(2).and_then(|v| v.as_num()).unwrap_or(8.0) as usize;
            if ptr_val != 0 {
                let layout = match std::alloc::Layout::from_size_align(size, align) {
                    Ok(l) => l,
                    Err(_) => {
                        self.push(Value::Error(Box::new(Value::str("Layout inválido para liberar",
                        ))));
                        return Some(Ok(()));
                    }
                };
                unsafe {
                    std::alloc::dealloc(ptr_val as *mut u8, layout);
                }
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "__ffi_escribir" || name == "__ffi_write" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let data = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let bytes = data.as_bytes();
            if ptr_val != 0 {
                unsafe {
                    std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr_val as *mut u8, bytes.len());
                }
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "__ffi_leer" || name == "__ffi_read" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            if ptr_val != 0 && len > 0 {
                let mut buf = vec![0u8; len];
                unsafe {
                    std::ptr::copy_nonoverlapping(ptr_val as *const u8, buf.as_mut_ptr(), len);
                }
                self.push(Value::str(String::from_utf8_lossy(&buf).to_string()));
            } else {
                self.push(Value::str(String::new()));
            }
            return Some(Ok(()));
        }

        if name == "__ffi_peek" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            if ptr_val != 0 {
                unsafe {
                    let val = *(ptr_val as *const u32);
                    self.push(Value::Int(val as i64));
                }
            } else {
                self.push(Value::Int(0));
            }
            return Some(Ok(()));
        }

        if name == "__ffi_poke" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let val = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as u32;
            if ptr_val != 0 {
                unsafe {
                    *(ptr_val as *mut u32) = val;
                }
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        // ██ Crypto builtins ██
        if name == "__hash_sha256" {
            let data = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match self.bcrypt.as_ref() {
                Some(bc) => match bc.sha256(data.as_bytes()) {
                    Ok(hash) => {
                        let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
                        self.push(Value::str(hex));
                    }
                    Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
                },
                None => self.push(Value::Error(Box::new(Value::str("Bcrypt no disponible",
                )))),
            }
            return Some(Ok(()));
        }

        if name == "__hash_sha512" {
            let data = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match self.bcrypt.as_ref() {
                Some(bc) => match bc.sha512(data.as_bytes()) {
                    Ok(hash) => {
                        let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
                        self.push(Value::str(hex));
                    }
                    Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
                },
                None => self.push(Value::Error(Box::new(Value::str("Bcrypt no disponible",
                )))),
            }
            return Some(Ok(()));
        }

        if name == "__aes_encriptar" || name == "__aes_encrypt" {
            if self.bcrypt.is_none() {
                match Bcrypt::load() {
                    Ok(b) => self.bcrypt = Some(Arc::new(b)),
                    Err(e) => {
                        self.push(Value::Error(Box::new(Value::str(e))));
                        return Some(Ok(()));
                    }
                }
            }
            let key = args
                .first()
                .map(|v| format!("{}", v).into_bytes())
                .unwrap_or_default();
            let data = args
                .get(1)
                .map(|v| format!("{}", v).into_bytes())
                .unwrap_or_default();
            match self.bcrypt.as_ref().unwrap().aes_encrypt(&key, &data) {
                Ok(ct) => self.push(Value::str(hex::encode(ct))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        if name == "__aes_desencriptar" || name == "__aes_decrypt" {
            if self.bcrypt.is_none() {
                match Bcrypt::load() {
                    Ok(b) => self.bcrypt = Some(Arc::new(b)),
                    Err(e) => {
                        self.push(Value::Error(Box::new(Value::str(e))));
                        return Some(Ok(()));
                    }
                }
            }
            let key = args
                .first()
                .map(|v| format!("{}", v).into_bytes())
                .unwrap_or_default();
            let hex_data = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let data = hex::decode(&hex_data).unwrap_or_default();
            match self.bcrypt.as_ref().unwrap().aes_decrypt(&key, &data) {
                Ok(pt) => self.push(Value::str(String::from_utf8_lossy(&pt).to_string())),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        if name == "__jwt_codificar" || name == "__jwt_encode" {
            let payload = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let secret = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let header = serde_json::json!({"alg": "HS256", "typ": "JWT"});
            let b64_header = base64url_encode(
                serde_json::to_string(&header)
                    .unwrap_or_default()
                    .as_bytes(),
            );
            let b64_payload = base64url_encode(payload.as_bytes());
            let signature_input = format!("{}.{}", b64_header, b64_payload);
            let sig = hmac_sha256(signature_input.as_bytes(), secret.as_bytes());
            let b64_sig = base64url_encode(&sig);
            self.push(Value::str(format!(
                "{}.{}.{}",
                b64_header, b64_payload, b64_sig
            )));
            return Some(Ok(()));
        }

        if name == "__jwt_decodificar" || name == "__jwt_decode" {
            let token = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let secret = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let parts: Vec<&str> = token.split('.').collect();
            if parts.len() != 3 {
                self.push(Value::Error(Box::new(Value::str("JWT inválido: se esperan 3 partes",
                ))));
                return Some(Ok(()));
            }
            let sig_input = format!("{}.{}", parts[0], parts[1]);
            let expected_sig = hmac_sha256(sig_input.as_bytes(), secret.as_bytes());
            let actual_sig = base64url_decode(parts[2]);
            if actual_sig != expected_sig {
                self.push(Value::Error(Box::new(Value::str("Firma JWT inválida",
                ))));
                return Some(Ok(()));
            }
            match base64url_decode_to_string(parts[1]) {
                Ok(payload) => self.push(Value::str(payload)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        // ██ Utility builtins ██
        if name == "__tipo_de" || name == "__typeof" {
            let val = args.first().cloned().unwrap_or(Value::Void);
            let type_name = match &val {
                Value::Int(_) => "entero",
                Value::Float(_) => "decimal",
                Value::Bool(_) => "booleano",
                Value::Str(_) => "texto",
                Value::Array(_) => "lista",
                Value::Map(_) => "diccionario",
                Value::Void => "nulo",
                Value::Func(_) => "funcion",
                Value::Struct { .. } => "estructura",
                Value::Enum { .. } => "enumeracion",
                Value::Tuple(_) => "tupla",
                Value::Exito(_) => "exito",
                Value::Error(_) => "error",
                Value::Opcion(_) => "opcion",
            };
            self.push(Value::str(type_name.to_string()));
            return Some(Ok(()));
        }

        if name == "__fs_listar" || name == "__fs_listdir" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::fs::read_dir(&path) {
                Ok(entries) => {
                    let mut items = Vec::new();
                    for entry in entries.flatten() {
                        items.push(Value::str(entry.file_name().to_string_lossy().to_string()));
                    }
                    self.push(Value::arr(items));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__env_listar" || name == "__env_list" {
            let vars: Vec<Value> = std::env::vars()
                .map(|(k, v)| Value::str(format!("{}={}", k, v)))
                .collect();
            self.push(Value::arr(vars));
            return Some(Ok(()));
        }

        // ██ Date builtins ██
        if name == "__tiempo_formatear" || name == "__time_format" {
            let timestamp = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let fmt = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let formatted = format_timestamp(timestamp, &fmt);
            self.push(Value::str(formatted));
            return Some(Ok(()));
        }

        if name == "__tiempo_parsear" || name == "__time_parse" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match parse_iso8601_to_unix(&s) {
                Ok(ts) => self.push(Value::Int(ts)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        if name == "__tiempo_diferencia" || name == "__time_diff" {
            let t1 = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let t2 = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let diff = (t1 - t2).abs();
            self.push(Value::Int(diff));
            return Some(Ok(()));
        }

        // ██ Coroutine builtins ██
        if name == "__coro_crear" || name == "__coro_create" {
            let fn_name = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(func) = self.find_func(&fn_name) {
                let coro = Coroutine::new(&fn_name, func.start);
                let coro_id = format!("coro_{}", self.coroutines.len());
                self.coroutines.insert(coro_id.clone(), coro);
                self.push(Value::str(coro_id));
            } else {
                self.push(Value::Error(Box::new(Value::str(format!(
                    "Función '{}' no encontrada",
                    fn_name
                )))));
            }
            return Some(Ok(()));
        }

        if name == "__coro_ceder" || name == "__coro_yield" {
            if let Some(ref coro_id) = self.current_coro.clone() {
                if let Some(coro) = self.coroutines.get_mut(coro_id) {
                    coro.stack = self.stack.clone();
                    coro.locals = self.locals.clone();
                    coro.ip = self.ip;
                }
            }
            // Restore main saved state
            if let Some((saved_stack, saved_locals, saved_ip)) = self.main_saved.take() {
                self.stack = saved_stack;
                self.locals = saved_locals;
                self.ip = saved_ip;
            }
            self.current_coro = None;
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "__coro_reanudar" || name == "__coro_resume" {
            let coro_id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(coro) = self.coroutines.get(&coro_id) {
                if coro.is_done {
                    self.push(Value::Error(Box::new(Value::str("Coroutine terminada",
                    ))));
                    return Some(Ok(()));
                }
            }
            // Save main state before resuming coroutine
            self.main_saved = Some((self.stack.clone(), self.locals.clone(), self.ip));
            if let Some(coro) = self.coroutines.get_mut(&coro_id) {
                self.stack = coro.stack.clone();
                self.locals = coro.locals.clone();
                self.ip = coro.ip;
                self.current_coro = Some(coro_id.clone());
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        // ██ GUI builtins ██
        if name == "__gui_ventana" || name == "__gui_window" {
            let title = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let width = args.get(1).and_then(|v| v.as_num()).unwrap_or(800.0) as i32;
            let height = args.get(2).and_then(|v| v.as_num()).unwrap_or(600.0) as i32;
            match GuiWindow::create(&title, width, height) {
                Ok(w) => {
                    let wid = format!("win_{}", self.gui_windows.len());
                    self.gui_windows.insert(wid.clone(), w);
                    self.push(Value::str(wid));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        if name == "__gui_mostrar" || name == "__gui_show" {
            let id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(w) = self.gui_windows.get(&id) {
                w.show();
                self.push(Value::Bool(true));
            } else {
                self.push(Value::Error(Box::new(Value::str(format!(
                    "Ventana '{}' no encontrada",
                    id
                )))));
            }
            return Some(Ok(()));
        }

        if name == "__gui_cerrar" || name == "__gui_close" {
            let id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if self.gui_windows.remove(&id).is_some() {
                self.push(Value::Bool(true));
            } else {
                self.push(Value::Bool(false));
            }
            return Some(Ok(()));
        }

        if name == "__gui_id" || name == "__gui_hwnd" {
            let id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(w) = self.gui_windows.get(&id) {
                self.push(Value::Int(w.hwnd() as i64));
            } else {
                self.push(Value::Error(Box::new(Value::str(format!(
                    "Ventana '{}' no encontrada",
                    id
                )))));
            }
            return Some(Ok(()));
        }

        if name == "__gui_esperar" || name == "__gui_poll" {
            let id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(w) = self.gui_windows.get_mut(&id) {
                match w.poll_event() {
                    Some(_evt) => self.push(Value::Bool(true)),
                    None => self.push(Value::Bool(false)),
                }
            } else {
                self.push(Value::Bool(false));
            }
            return Some(Ok(()));
        }

        // ██ Async/Task builtins ██
        if name == "__tarea_lanzar" || name == "__task_spawn" {
            let fn_name = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_args: Vec<Value> = args.into_iter().skip(1).collect();
            let bc = self.bytecode.clone();
            let (tx, rx) = std::sync::mpsc::channel();
            let id = self.task_counter;
            self.task_counter += 1;
            std::thread::spawn(move || {
                let mut vm = VM::new(bc);
                let result = vm.run_function(&fn_name, fn_args);
                let _ = tx.send(result.unwrap_or(Value::Void));
            });
            let task_id = format!("task_{}", id);
            self.task_results.insert(task_id.clone(), rx);
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        if name == "__tarea_esperar" || name == "__task_await" {
            let task_id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some(rx) = self.task_results.remove(&task_id) {
                match rx.recv() {
                    Ok(val) => self.push(val),
                    Err(_) => self.push(Value::Error(Box::new(Value::str("Task failed")))),
                }
            } else {
                self.push(Value::Error(Box::new(Value::str("Task not found"))));
            }
            return Some(Ok(()));
        }

        // ██ Timezone builtins ██
        if name == "__timezone_info" || name == "__zona_info" {
            let tz = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let offset = match tz.to_lowercase().as_str() {
                "utc" | "gmt" => 0,
                "est" | "et" | "america/new_york" | "us/eastern" => -5,
                "cst" | "ct" | "america/chicago" | "us/central" => -6,
                "mst" | "mt" | "america/denver" | "us/mountain" => -7,
                "pst" | "pt" | "america/los_angeles" | "us/pacific" => -8,
                "cet" | "europe/madrid" | "europe/paris" | "europe/berlin" => 1,
                "eet" | "europe/athens" | "europe/helsinki" => 2,
                "ist" | "asia/kolkata" => 5,
                "jst" | "asia/tokyo" => 9,
                "aest" | "australia/sydney" => 10,
                "nzst" | "pacific/auckland" => 12,
                _ => 0,
            };
            self.push(Value::Int(offset));
            return Some(Ok(()));
        }

        // ██ Duration builtins ██
        if name == "__duration_new" || name == "__duracion_nueva" {
            let secs = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let nanos = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            self.push(Value::Int(secs * 1_000_000_000 + nanos));
            return Some(Ok(()));
        }
        if name == "__duration_secs" || name == "__duracion_segundos" {
            let nanos = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            self.push(Value::Int(nanos / 1_000_000_000));
            return Some(Ok(()));
        }

        // ██ Calendar builtins ██
        if name == "__calendar_hijri" || name == "__calendario_hijri" {
            let timestamp = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let days = timestamp / 86400 + 719163;
            let hijri_year = ((days as f64) / 354.367) as i64 + 1;
            let remaining = days - (((hijri_year - 1) as f64) * 354.367) as i64;
            let hijri_month = (remaining / 30).clamp(1, 12);
            let hijri_day = (remaining % 30 + 1).min(30);
            let result = format!("{}-{:02}-{:02} AH", hijri_year, hijri_month, hijri_day);
            self.push(Value::str(result));
            return Some(Ok(()));
        }

        if name == "__calendar_persian" || name == "__calendario_persa" {
            let timestamp = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let days = timestamp / 86400;
            let persian_year = ((days as f64 - 226899.0) / 365.242) as i64 + 1;
            let persian_month = 1 + ((days % 365) / 31).min(11);
            let persian_day = 1 + (days % 31).min(30);
            let result = format!(
                "{}-{:02}-{:02} AP",
                persian_year, persian_month, persian_day
            );
            self.push(Value::str(result));
            return Some(Ok(()));
        }

        // ██ Async File I/O builtins ██
        if name == "__leer_archivo_async" || name == "__file_read_async" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let (tx, rx) = std::sync::mpsc::channel();
            let id = self.task_counter;
            self.task_counter += 1;
            std::thread::spawn(move || {
                let content = std::fs::read_to_string(&path);
                let _ = tx.send(match content {
                    Ok(s) => Value::str(s),
                    Err(e) => Value::Error(Box::new(Value::str(e.to_string()))),
                });
            });
            let task_id = format!("file_{}", id);
            self.task_results.insert(task_id.clone(), rx);
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        if name == "__escribir_archivo_async" || name == "__file_write_async" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let (tx, rx) = std::sync::mpsc::channel();
            let id = self.task_counter;
            self.task_counter += 1;
            std::thread::spawn(move || {
                let result = std::fs::write(&path, &content);
                let _ = tx.send(match result {
                    Ok(()) => Value::Bool(true),
                    Err(e) => Value::Error(Box::new(Value::str(e.to_string()))),
                });
            });
            let task_id = format!("file_{}", id);
            self.task_results.insert(task_id.clone(), rx);
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        // ██ Async Timer builtins ██
        if name == "__timer_delay" || name == "__temporizador_esperar" {
            let ms = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as u64;
            let (tx, rx) = std::sync::mpsc::channel();
            let id = self.task_counter;
            self.task_counter += 1;
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(ms));
                let _ = tx.send(Value::Bool(true));
            });
            let task_id = format!("timer_{}", id);
            self.task_results.insert(task_id.clone(), rx);
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        // ██ Async TCP connect builtins ██
        if name == "__tcp_connect_async" || name == "__tcp_conectar_async" {
            let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let (tx, rx) = std::sync::mpsc::channel();
            let id = self.task_counter;
            self.task_counter += 1;
            std::thread::spawn(move || match std::net::TcpStream::connect(&addr) {
                Ok(_) => {
                    let _ = tx.send(Value::Bool(true));
                }
                Err(e) => {
                    let _ = tx.send(Value::Error(Box::new(Value::str(e.to_string()))));
                }
            });
            let task_id = format!("tcp_{}", id);
            self.task_results.insert(task_id.clone(), rx);
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        // ██ Concurrency builtins ██
        if name == "__dormir" || name == "__sleep" {
            let ms = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as u64;
            std::thread::sleep(std::time::Duration::from_millis(ms));
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "__hilo_lanzar" || name == "__thread_spawn" {
            let fn_name = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_args: Vec<Value> = args.into_iter().skip(1).collect();
            let bc = self.bytecode.clone();
            let handle = std::thread::spawn(move || {
                let mut vm = VM::new(bc);
                vm.run_function(&fn_name, fn_args)
                    .unwrap_or(Value::Void)
            });
            let hid = format!("thread_{}", self.thread_handles.len());
            self.thread_handles.insert(hid.clone(), handle);
            self.push(Value::str(hid));
            return Some(Ok(()));
        }

        if name == "__hilo_esperar" || name == "__thread_join" {
            let hid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if let Some((_, handle)) = self.thread_handles.remove_entry(&hid) {
                match handle.join() {
                    Ok(val) => self.push(val),
                    Err(_) => self.push(Value::Error(Box::new(Value::str("Thread panicked")))),
                }
            } else {
                self.push(Value::Error(Box::new(Value::str("Thread not found"))));
            }
            return Some(Ok(()));
        }

        if name == "__canal_nuevo" || name == "__channel_new" {
            let (tx, rx) = std::sync::mpsc::channel::<Value>();
            let cid = format!("chan_{}", self.channels.len());
            self.channels.insert(cid.clone(), (Some(tx), Some(rx)));
            self.push(Value::str(cid));
            return Some(Ok(()));
        }

        if name == "__canal_enviar" || name == "__channel_send" {
            let cid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let val = args.get(1).cloned().unwrap_or(Value::Void);
            if let Some((Some(ref tx), _)) = self.channels.get(&cid) {
                match tx.send(val) {
                    Ok(()) => self.push(Value::Bool(true)),
                    Err(_) => self.push(Value::Bool(false)),
                }
            } else {
                self.push(Value::Error(Box::new(Value::str("Channel not found"))));
            }
            return Some(Ok(()));
        }

        if name == "__canal_recibir" || name == "__channel_recv" {
            let cid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            // Need to take the receiver out temporarily
            let rx = if let Some((_, ref mut rx_opt)) = self.channels.get_mut(&cid) {
                rx_opt.take()
            } else {
                None
            };
            match rx {
                Some(rx) => {
                    let val = rx.recv().unwrap_or(Value::Void);
                    // Put receiver back
                    if let Some((_, ref mut rx_opt)) = self.channels.get_mut(&cid) {
                        *rx_opt = Some(rx);
                    }
                    self.push(val);
                }
                None => {
                    self.push(Value::Error(Box::new(Value::str("Channel not found"))));
                }
            }
            return Some(Ok(()));
        }

        if name == "__mutex_nuevo" || name == "__mutex_new" {
            let mid = format!("mutex_{}", self.mutexes.len());
            self.mutexes.insert(mid.clone(), std::sync::Mutex::new(Value::Void));
            self.push(Value::str(mid));
            return Some(Ok(()));
        }

        if name == "__mutex_bloquear" || name == "__mutex_lock" {
            let mid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let fn_arg = args.get(2).cloned().unwrap_or(Value::Void);
            let result = if let Some(mutex) = self.mutexes.get(&mid) {
                let _guard = mutex.lock().unwrap();
                drop(_guard);
                // Execute function while holding lock
                let bc = self.bytecode.clone();
                let mut vm = VM::new(bc);
                vm.run_function(&fn_name, vec![fn_arg]).unwrap_or(Value::Void)
            } else {
                Value::Error(Box::new(Value::str("Mutex not found")))
            };
            self.push(result);
            return Some(Ok(()));
        }

        if name == "__stream_desde" || name == "__stream_from" {
            let source = args.first().cloned().unwrap_or(Value::Void);
            self.push(source);
            return Some(Ok(()));
        }

        if name == "__stream_mapear" || name == "__stream_map" {
            let source = args.first().cloned().unwrap_or(Value::Void);
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match source {
                Value::Array(items) => {
                    let bc = self.bytecode.clone();
                    let mapped: Vec<Value> = items
                        .iter()
                        .map(|item| {
                            let mut vm = VM::new(bc.clone());
                            vm.run_function(&fn_name, vec![item.clone()])
                                .unwrap_or(Value::Void)
                        })
                        .collect();
                    self.push(Value::arr(mapped));
                }
                _ => self.push(Value::Error(Box::new(Value::str("stream_map espera una lista",
                )))),
            }
            return Some(Ok(()));
        }

        if name == "__stream_filtrar" || name == "__stream_filter" {
            let source = args.first().cloned().unwrap_or(Value::Void);
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match source {
                Value::Array(items) => {
                    let bc = self.bytecode.clone();
                    let filtered: Vec<Value> = items
                        .iter()
                        .filter(|item| {
                            let mut vm = VM::new(bc.clone());
                            match vm.run_function(&fn_name, vec![(*item).clone()]) {
                                Ok(Value::Bool(true)) => true,
                                _ => false,
                            }
                        })
                        .cloned()
                        .collect();
                    self.push(Value::arr(filtered));
                }
                _ => self.push(Value::Error(Box::new(Value::str("stream_filter espera una lista",
                )))),
            }
            return Some(Ok(()));
        }

        if name == "__stream_colectar" || name == "__stream_collect" {
            let source = args.first().cloned().unwrap_or(Value::Void);
            self.push(source);
            return Some(Ok(()));
        }

        if name == "__par_mapear" || name == "__par_map" {
            let source = args.first().cloned().unwrap_or(Value::arr(vec![]));
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match source {
                Value::Array(items) => {
                    let bc = self.bytecode.clone();
                    let mut handles = Vec::new();
                    for item in items.iter().cloned() {
                        let bc_clone = bc.clone();
                        let fn_clone = fn_name.clone();
                        handles.push(std::thread::spawn(move || {
                            let mut vm = VM::new(bc_clone);
                            vm.run_function(&fn_clone, vec![item.clone()])
                                .unwrap_or(Value::Void)
                        }));
                    }
                    let results: Vec<Value> = handles
                        .into_iter()
                        .map(|h| h.join().unwrap_or(Value::Void))
                        .collect();
                    self.push(Value::arr(results));
                }
                _ => self.push(Value::Error(Box::new(Value::str("par_map espera una lista",
                )))),
            }
            return Some(Ok(()));
        }

        if name == "__par_unir" || name == "__par_join" {
            let fn1 = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let a1 = args.get(1).cloned().unwrap_or(Value::Void);
            let fn2 = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            let a2 = args.get(3).cloned().unwrap_or(Value::Void);
            let bc = self.bytecode.clone();
            let bc2 = bc.clone();
            let h1 = std::thread::spawn(move || {
                let mut vm = VM::new(bc);
                if fn1.is_empty() {
                    Value::Void
                } else {
                    vm.run_function(&fn1, vec![a1]).unwrap_or(Value::Void)
                }
            });
            let h2 = std::thread::spawn(move || {
                let mut vm = VM::new(bc2);
                if fn2.is_empty() {
                    Value::Void
                } else {
                    vm.run_function(&fn2, vec![a2]).unwrap_or(Value::Void)
                }
            });
            let r1 = h1.join().unwrap_or(Value::Void);
            let r2 = h2.join().unwrap_or(Value::Void);
            self.push(Value::arr(vec![r1, r2]));
            return Some(Ok(()));
        }

        if name == "__actor_nuevo" || name == "__actor_new" {
            let aid = format!("actor_{}", self.actors.len());
            // Actor is just a mailbox: a channel
            let (tx, rx) = std::sync::mpsc::channel::<Value>();
            self.actors.insert(aid.clone(), (Some(tx), Some(rx)));
            self.push(Value::str(aid));
            return Some(Ok(()));
        }

        if name == "__actor_enviar" || name == "__actor_send" {
            let aid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let msg = args.get(1).cloned().unwrap_or(Value::Void);
            if let Some((Some(ref tx), _)) = self.actors.get(&aid) {
                match tx.send(msg) {
                    Ok(()) => self.push(Value::Bool(true)),
                    Err(_) => self.push(Value::Bool(false)),
                }
            } else {
                self.push(Value::Error(Box::new(Value::str("Actor not found"))));
            }
            return Some(Ok(()));
        }

        if name == "__actor_recibir" || name == "__actor_recv" {
            let aid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let rx = if let Some((_, ref mut rx_opt)) = self.actors.get_mut(&aid) {
                rx_opt.take()
            } else {
                None
            };
            match rx {
                Some(rx) => {
                    let val = rx.recv().unwrap_or(Value::Void);
                    if let Some((_, ref mut rx_opt)) = self.actors.get_mut(&aid) {
                        *rx_opt = Some(rx);
                    }
                    self.push(val);
                }
                None => {
                    self.push(Value::Error(Box::new(Value::str("Actor not found"))));
                }
            }
            return Some(Ok(()));
        }

        if name == "__generador_nuevo" || name == "__generator_new" {
            let gid = format!("gen_{}", self.generators.len());
            let fn_name = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.generators.insert(gid.clone(), fn_name);
            self.push(Value::str(gid));
            return Some(Ok(()));
        }

        if name == "__generador_siguiente" || name == "__generator_next" {
            let gid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let val = args.get(1).cloned().unwrap_or(Value::Void);
            let fn_name = if let Some(fn_name) = self.generators.get(&gid) {
                fn_name.clone()
            } else {
                self.push(Value::Error(Box::new(Value::str("Generator not found"))));
                return Some(Ok(()));
            };
            let mut vm = VM::new(self.bytecode.clone());
            let result = vm.run_function(&fn_name, vec![val]).unwrap_or(Value::Void);
            self.push(result);
            return Some(Ok(()));
        }

        // Stubs for remaining concurrency builtins
        if name == "__seleccionar" || name == "__select" {
            // Return first available; stub returns "select_stub"
            self.push(Value::str("select_stub"));
            return Some(Ok(()));
        }
        if name == "__scope_nuevo" || name == "__scope_new" {
            self.push(Value::str("scope_0"));
            return Some(Ok(()));
        }
        if name == "__scope_lanzar" || name == "__scope_spawn" {
            self.push(Value::str("scope_task_0"));
            return Some(Ok(()));
        }
        if name == "__scope_cancelar" || name == "__scope_cancel" {
            self.push(Value::Void);
            return Some(Ok(()));
        }
        if name == "__supervisor_nuevo" || name == "__supervisor_new" {
            self.push(Value::str("sup_0"));
            return Some(Ok(()));
        }
        if name == "__supervisor_agregar" || name == "__supervisor_add" {
            self.push(Value::Void);
            return Some(Ok(()));
        }
        if name == "__supervisor_iniciar" || name == "__supervisor_start" {
            self.push(Value::Void);
            return Some(Ok(()));
        }
        if name == "__cluster_conectar" || name == "__cluster_connect" {
            self.push(Value::str("cluster_0"));
            return Some(Ok(()));
        }
        if name == "__cluster_enviar" || name == "__cluster_send" {
            self.push(Value::Bool(false));
            return Some(Ok(()));
        }
        if name == "__rwlock_nuevo" || name == "__rwlock_new" {
            self.push(Value::str("rwlock_0"));
            return Some(Ok(()));
        }
        if name == "__rwlock_leer" || name == "__rwlock_read" {
            self.push(Value::Void);
            return Some(Ok(()));
        }
        if name == "__rwlock_escribir" || name == "__rwlock_write" {
            self.push(Value::Void);
            return Some(Ok(()));
        }
        if name == "__arc_nuevo" || name == "__arc_new" {
            self.push(Value::str("arc_0"));
            return Some(Ok(()));
        }
        if name == "__arc_obtener" || name == "__arc_get" {
            self.push(args.get(1).cloned().unwrap_or(Value::Void));
            return Some(Ok(()));
        }
        if name == "__arc_asignar" || name == "__arc_set" {
            self.push(Value::Void);
            return Some(Ok(()));
        }

        None
    }
    pub fn run(&mut self) -> Result<(), VmError> {
        let profile = !std::env::var("LUMEN_PROFILE").unwrap_or_default().is_empty();
        let mut prof: std::collections::HashMap<String, (u64, f64)> = std::collections::HashMap::new();
        let prof_start = std::time::Instant::now();
        loop {
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }
            let instr = self.bytecode.instructions[self.ip].clone();
            self.ip += 1;
            self.instr_count += 1;
            self.last_instr = Some(instr.clone());
            if self.debug && (self.breakpoints.contains(&self.ip) || self.step_mode) {
                println!(
                    "\n[DEBUG] ip={} instr={:?} stack={} vars={}",
                    self.ip,
                    instr,
                    self.stack.len(),
                    self.locals.last().map_or(0, |l| l.len())
                );
                if self.step_mode {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).ok();
                    match input.trim() {
                        "c" | "continue" => self.step_mode = false,
                        "q" | "quit" => return Ok(()),
                        _ => {}
                    }
                }
            }
            if profile {
                let t0 = std::time::Instant::now();
                let opname = match &instr {
                    Instruction::WithIdx(op @ Opcode::Call, nidx) => {
                        format!("Call:{}", self.bytecode.names.get(*nidx).cloned().unwrap_or_default())
                    }
                    Instruction::Simple(op) => format!("{:?}", op),
                    Instruction::WithNum(op, _) => format!("{:?}", op),
                    Instruction::WithStr(op, _) => format!("{:?}", op),
                    Instruction::WithBool(op, _) => format!("{:?}", op),
                    Instruction::WithIdx(op, _) => format!("{:?}", op),
                };
                let res = self.execute(&instr);
                let e = t0.elapsed().as_secs_f64();
                let ent = prof.entry(opname).or_insert((0, 0.0));
                ent.0 += 1;
                ent.1 += e;
                res?;
            } else {
                self.execute(&instr)?;
            }
        }
        if profile {
            let total = prof_start.elapsed().as_secs_f64();
            let mut v: Vec<(String, u64, f64)> = prof
                .into_iter()
                .map(|(k, (c, t))| (k, c, t))
                .collect();
            v.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            eprintln!("=== PROFILE total={:.1}s instrs={} ===", total, self.instr_count);
            for (k, c, t) in v {
                eprintln!("{:20} {:>12} calls {:>10.3}s ({:>5.1}%)", k, c, t, t / total * 100.0);
            }
        }
        Ok(())
    }

    pub fn set_breakpoint(&mut self, ip: usize) {
        self.breakpoints.push(ip);
    }
    pub fn step(&mut self) -> Result<(), VmError> {
        self.step_mode = true;
        self.debug = true;
        if self.ip >= self.bytecode.instructions.len() {
            return Ok(());
        }
        let instr = self.bytecode.instructions[self.ip].clone();
        self.ip += 1;
        self.instr_count += 1;
        self.last_instr = Some(instr.clone());
        self.execute(&instr)
    }
    pub fn stack_top(&self) -> Option<&Value> {
        self.stack.last()
    }
    pub fn current_locals(&self) -> Option<&HashMap<String, Value, FixHasher>> {
        self.locals.last()
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    /// Run a specific function by name with given args, returning its result.
    /// Used by spawned task threads to execute a function in isolation.
    pub fn run_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        if let Some(func) = self.find_func(name) {
            let func_start = func.start;
            let func_params = func.params.clone();
            let mut scope = HashMap::with_capacity_and_hasher(func_params.len(), FixHasher::default());
            for (i, param_name) in func_params.iter().enumerate() {
                if let Some(arg) = args.get(i) {
                    scope.insert(param_name.clone(), arg.clone());
                }
            }
            self.locals.push(scope);
            self.call_stack.push(CallFrame {
                func_name: name.to_string(),
                return_ip: self.bytecode.instructions.len(), // Past end → run() loop breaks
            });
            self.ip = func_start;
            self.run()?;
            Ok(self.pop().unwrap_or(Value::Void))
        } else {
            Err(VmError::UndefinedFunction(name.to_string()))
        }
    }

    fn execute(&mut self, instr: &Instruction) -> Result<(), VmError> {
        match instr {
            Instruction::Simple(op) => self.execute_simple(*op),
            Instruction::WithNum(op, n) => self.execute_with_num(*op, *n),
            Instruction::WithStr(op, s) => self.execute_with_str(*op, s),
            Instruction::WithBool(op, b) => self.execute_with_bool(*op, *b),
            Instruction::WithIdx(op, idx) => self.execute_with_idx(*op, *idx),
        }
    }

    fn execute_simple(&mut self, op: Opcode) -> Result<(), VmError> {
        match op {
            Opcode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a + b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(*a as f64 + b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a + *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a + b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::str(format!("{}{}", a, b))),
                    (Value::Str(a), Value::Int(b)) => self.push(Value::str(format!("{}{}", a, b))),
                    (Value::Str(a), Value::Float(b)) => {
                        self.push(Value::str(format!("{}{}", a, b)))
                    }
                    (Value::Int(a), Value::Str(b)) => self.push(Value::str(format!("{}{}", a, b))),
                    (Value::Float(a), Value::Str(b)) => {
                        self.push(Value::str(format!("{}{}", a, b)))
                    }
                    (Value::Str(a), Value::Bool(b)) => self.push(Value::str(format!("{}{}", a, b))),
                    (Value::Bool(a), Value::Str(b)) => self.push(Value::str(format!("{}{}", a, b))),
                    _ => {
                        return Err(VmError::TypeError(
                            "Add requires numbers or strings".to_string(),
                        ))
                    }
                }
            }
            Opcode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a - b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(*a as f64 - b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a - *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a - b)),
                    _ => return Err(VmError::TypeError("Sub requires numbers".to_string())),
                }
            }
            Opcode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a * b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(*a as f64 * b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a * *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a * b)),
                    _ => return Err(VmError::TypeError("Mul requires numbers".to_string())),
                }
            }
            Opcode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(_), Value::Int(0)) => return Err(VmError::DivisionByZero),
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a / b)),
                    (Value::Int(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(*a as f64 / b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if *b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a / *b as f64))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a / b))
                    }
                    _ => return Err(VmError::TypeError("Div requires numbers".to_string())),
                }
            }
            Opcode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(_), Value::Int(0)) => return Err(VmError::DivisionByZero),
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a.rem_euclid(*b))),
                    (Value::Int(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(*a as f64 % b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if *b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a % *b as f64))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a % b))
                    }
                    _ => return Err(VmError::TypeError("Mod requires numbers".to_string())),
                }
            }
            Opcode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => a == b,
                    (Value::Int(a), Value::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
                    (Value::Float(a), Value::Int(b)) => (a - *b as f64).abs() < f64::EPSILON,
                    (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
                    (Value::Str(a), Value::Str(b)) => a == b,
                    (Value::Bool(a), Value::Bool(b)) => a == b,
                    (
                        Value::Struct {
                            name: an,
                            fields: af,
                        },
                        Value::Struct {
                            name: bn,
                            fields: bf,
                        },
                    ) => an == bn && af == bf,
                    (Value::Opcion(a), Value::Opcion(b)) => a == b,
                    (
                        Value::Enum {
                            name: an,
                            variant: av,
                            fields: af,
                        },
                        Value::Enum {
                            name: bn,
                            variant: bv,
                            fields: bf,
                        },
                    ) => an == bn && av == bv && af == bf,
                    _ => false,
                };
                self.push(Value::Bool(result));
            }
            Opcode::Neq => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => a != b,
                    (Value::Int(a), Value::Float(b)) => (*a as f64 - b).abs() >= f64::EPSILON,
                    (Value::Float(a), Value::Int(b)) => (a - *b as f64).abs() >= f64::EPSILON,
                    (Value::Float(a), Value::Float(b)) => (a - b).abs() >= f64::EPSILON,
                    (Value::Str(a), Value::Str(b)) => a != b,
                    (Value::Bool(a), Value::Bool(b)) => a != b,
                    (
                        Value::Struct {
                            name: an,
                            fields: af,
                        },
                        Value::Struct {
                            name: bn,
                            fields: bf,
                        },
                    ) => an != bn || af != bf,
                    (Value::Opcion(a), Value::Opcion(b)) => a != b,
                    (
                        Value::Enum {
                            name: an,
                            variant: av,
                            fields: af,
                        },
                        Value::Enum {
                            name: bn,
                            variant: bv,
                            fields: bf,
                        },
                    ) => an != bn || av != bv || af != bf,
                    _ => true,
                };
                self.push(Value::Bool(result));
            }
            Opcode::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a < b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) < *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a < *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a < b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a < b)),
                    _ => return Err(VmError::TypeError("Lt requires numbers or strings".to_string())),
                }
            }
            Opcode::Le => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a <= b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) <= *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a <= *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a <= b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a <= b)),
                    _ => return Err(VmError::TypeError("Le requires numbers or strings".to_string())),
                }
            }
            Opcode::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a > b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) > *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a > *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a > b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a > b)),
                    _ => return Err(VmError::TypeError("Gt requires numbers or strings".to_string())),
                }
            }
            Opcode::Ge => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a >= b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) >= *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a >= *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a >= b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a >= b)),
                    _ => return Err(VmError::TypeError("Ge requires numbers or strings".to_string())),
                }
            }
            Opcode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Value::Bool(a.is_truthy() && b.is_truthy()));
            }
            Opcode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Value::Bool(a.is_truthy() || b.is_truthy()));
            }
            Opcode::BitOr => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a | b)),
                    _ => return Err(VmError::TypeError("BitOr requires integers".to_string())),
                }
            }
            Opcode::BitAnd => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a & b)),
                    _ => return Err(VmError::TypeError("BitAnd requires integers".to_string())),
                }
            }
            Opcode::ShiftLeft => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => {
                        if *b < 0 || *b > 63 {
                            return Err(VmError::Runtime(format!(
                                "Desplazamiento {} fuera de rango (0-63)",
                                b
                            )));
                        }
                        self.push(Value::Int(a << *b as u32));
                    }
                    _ => return Err(VmError::TypeError("ShiftLeft requires integers".to_string())),
                }
            }
            Opcode::ShiftRight => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => {
                        if *b < 0 || *b > 63 {
                            return Err(VmError::Runtime(format!(
                                "Desplazamiento {} fuera de rango (0-63)",
                                b
                            )));
                        }
                        self.push(Value::Int(a >> *b as u32));
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "ShiftRight requires integers".to_string(),
                        ))
                    }
                }
            }
            Opcode::Concat => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Str(a), Value::Str(b)) => {
                        let mut s = a.to_string();
                        s.push_str(b);
                        self.push(Value::str(s));
                    }
                    (Value::Array(a), Value::Array(b)) => {
                        let mut v = a.as_ref().clone();
                        v.extend(b.as_ref().clone());
                        self.push(Value::arr(v));
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "Concat requires strings or lists".to_string(),
                        ))
                    }
                }
            }
            Opcode::Neg => {
                let a = self.pop()?;
                match a {
                    Value::Int(n) => self.push(Value::Int(-n)),
                    Value::Float(n) => self.push(Value::Float(-n)),
                    _ => return Err(VmError::TypeError("Neg requires number".to_string())),
                }
            }
            Opcode::Not => {
                let a = self.pop()?;
                self.push(Value::Bool(!a.is_truthy()));
            }
            Opcode::Ret => {
                let ret_val = self.pop().unwrap_or(Value::Void);
                if let Some(coro_id) = self.current_coro.clone() {
                    if let Some(coro) = self.coroutines.get_mut(&coro_id) {
                        coro.is_done = true;
                    }
                    self.current_coro = None;
                    if let Some((saved_stack, saved_locals, saved_ip)) = self.main_saved.take() {
                        self.stack = saved_stack;
                        self.locals = saved_locals;
                        self.ip = saved_ip;
                    } else {
                        self.ip = usize::MAX;
                    }
                    self.push(ret_val);
                } else if let Some(frame) = self.call_stack.pop() {
                    self.locals.pop();
                    self.ip = frame.return_ip;
                    self.push(ret_val);
                } else {
                    self.ip = usize::MAX;
                    self.push(ret_val);
                }
            }
            Opcode::Print => {
                let val = self.pop()?;
                let s = format!("{}", val);
                self.output.push(s);
            }
            Opcode::Halt => {
                self.ip = usize::MAX;
            }
            Opcode::StructNew => {
                // handled in execute_with_idx
            }
            Opcode::StructGet => {
                let field_name = self.pop()?;
                let struct_val = self.pop()?;
                let field = match &field_name {
                    Value::Str(s) => s.to_string(),
                    _ => {
                        return Err(VmError::TypeError(
                            "StructGet requires string field name".to_string(),
                        ))
                    }
                };
                match struct_val {
                    Value::Struct { fields, .. } => {
                        let val = fields.iter().find(|(name, _)| name.as_str() == field.as_str());
                        match val {
                            Some((_, v)) => self.push(v.clone()),
                            None => {
                                return Err(VmError::Runtime(format!(
                                    "Campo '{}' no encontrado en struct",
                                    field
                                )))
                            }
                        }
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "StructGet requires struct value".to_string(),
                        ))
                    }
                }
            }
            Opcode::StructSet => {
                let new_val = self.pop()?;
                let field_name = self.pop()?;
                let struct_val = self.pop()?;
                let field = match &field_name {
                    Value::Str(s) => s.to_string(),
                    _ => {
                        return Err(VmError::TypeError(
                            "StructGet requires string field name".to_string(),
                        ))
                    }
                };
                match struct_val {
                    Value::Struct { name, mut fields } => {
                        let pos = fields.iter().position(|(n, _)| n.as_str() == field.as_str());
                        match pos {
                            Some(i) => {
                                fields[i] = (field.to_string(), new_val);
                                self.push(Value::Struct { name, fields });
                            }
                            None => {
                                return Err(VmError::Runtime(format!(
                                    "Campo '{}' no encontrado en struct",
                                    field
                                )))
                            }
                        }
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "StructSet requires struct value".to_string(),
                        ))
                    }
                }
            }
            Opcode::FuncRef => {
                // handled in execute_with_idx
            }
            Opcode::CallValue => {
                // handled in execute_with_idx
            }
            Opcode::ArrayNew => {
                // handled in execute_with_idx
            }
            Opcode::ArrayGet => {
                let index = self.pop()?;
                let container = self.pop()?;
                match container {
                    Value::Array(arr) => {
                        let idx = match &index {
                            Value::Int(i) => *i,
                            _ => {
                                return Err(VmError::TypeError(
                                    "ArrayGet requires integer index for arrays".to_string(),
                                ))
                            }
                        };
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango (largo: {})",
                                idx,
                                arr.len()
                            )));
                        }
                        self.push(arr[idx as usize].clone());
                    }
                    Value::Str(s) => {
                        let idx = match &index {
                            Value::Int(i) => *i,
                            _ => {
                                return Err(VmError::TypeError(
                                    "ArrayGet requires integer index for strings".to_string(),
                                ))
                            }
                        };
                        if idx < 0 {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango", idx
                            )));
                        }
                        match s.chars().nth(idx as usize) {
                            Some(c) => self.push(Value::str(c.to_string())),
                            None => return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango (largo: {})", idx, s.chars().count()
                            ))),
                        }
                    }
                    Value::Map(map) => {
                        let val = map.get(&index).cloned().unwrap_or(Value::Int(0));
                        self.push(val);
                    }
                    Value::Tuple(items) => {
                        let idx = match &index {
                            Value::Int(i) => *i,
                            _ => {
                                return Err(VmError::TypeError(
                                    "ArrayGet requires integer index for tuples".to_string(),
                                ))
                            }
                        };
                        if idx < 0 || idx as usize >= items.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango (largo: {})",
                                idx,
                                items.len()
                            )));
                        }
                        self.push(items[idx as usize].clone());
                    }
                    Value::Struct { fields, .. } => {
                        let field = match &index {
                            Value::Str(s) => s.to_string(),
                            _ => "".to_string(),
                        };
                        let val = fields.iter().find(|(n, _)| n == &field).map(|(_, v)| v.clone()).unwrap_or(Value::Int(0));
                        self.push(val);
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "ArrayGet requires array, string, map or struct".to_string(),
                        ))
                    }
                }
            }
            Opcode::ArraySet => {
                let value = self.pop()?;
                let index = self.pop()?;
                let array = self.pop()?;
                match (array, index, value) {
                    (Value::Array(mut arr), Value::Int(idx), val) => {
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango (largo: {})",
                                idx,
                                arr.len()
                            )));
                        }
                        Arc::make_mut(&mut arr)[idx as usize] = val;
                        self.push(Value::Array(arr));
                    }
                    (Value::Map(mut map), key, val) => {
                        map.insert(key, val);
                        self.push(Value::Map(map));
                    }
                    (Value::Struct { name, mut fields }, key, val) => {
                        let field_str = match &key {
                            Value::Str(s) => s.to_string(),
                            _ => "".to_string(),
                        };
                        if let Some(pos) = fields.iter().position(|(n, _)| n == &field_str) {
                            fields[pos].1 = val;
                        } else {
                            fields.push((field_str, val));
                        }
                        self.push(Value::Struct { name, fields });
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "ArraySet requires array, map or struct".to_string(),
                        ))
                    }
                }
            }
            Opcode::ArrayLen => {
                let array = self.pop()?;
                match array {
                    Value::Array(arr) => self.push(Value::Int(arr.len() as i64)),
                    Value::Str(s) => self.push(Value::Int(s.chars().count() as i64)),
                    _ => {
                        return Err(VmError::TypeError(
                            "ArrayLen requires array or string".to_string(),
                        ))
                    }
                }
            }
            Opcode::ArrayPush => {
                let value = self.pop()?;
                let array = self.pop()?;
                match array {
                    Value::Array(mut arr) => {
                        Arc::make_mut(&mut arr).push(value);
                        self.push(Value::Array(arr));
                    }
                    other => {
                        self.push(other);
                        self.push(value);
                        return Err(VmError::TypeError(
                            "ArrayPush requires array as receiver".to_string(),
                        ));
                    }
                }
            }
            Opcode::ResultOk => {
                let val = self.pop()?;
                self.push(Value::Exito(Box::new(val)));
            }
            Opcode::ResultErr => {
                let val = self.pop()?;
                self.push(Value::Error(Box::new(val)));
            }
            Opcode::TryUnwrap => {
                let val = self.pop()?;
                match val {
                    Value::Exito(inner) => {
                        self.push(*inner);
                    }
                    Value::Error(inner) => {
                        let err_wrapper = Value::Error(inner);
                        if let Some(frame) = self.call_stack.pop() {
                            self.locals.pop();
                            self.ip = frame.return_ip;
                        }
                        self.push(err_wrapper);
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "TryUnwrap requires a result value".to_string(),
                        ))
                    }
                }
            }
            Opcode::OptionSome => {
                let val = self.pop()?;
                self.push(Value::Opcion(Some(Box::new(val))));
            }
            Opcode::OptionNone => {
                self.push(Value::Opcion(None));
            }
            Opcode::TupleNew => {
                // handled in execute_with_idx
            }
            Opcode::TupleAccess => {
                // handled in execute_with_idx
            }
            Opcode::EnumCtor => {
                // handled in execute_with_idx
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_with_num(&mut self, op: Opcode, n: f64) -> Result<(), VmError> {
        match op {
            Opcode::PushNum => {
                self.push(Value::Float(n));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn execute_with_str(&mut self, op: Opcode, s: &str) -> Result<(), VmError> {
        match op {
            Opcode::PushStr => {
                self.push(Value::str(s.to_string()));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn execute_with_bool(&mut self, op: Opcode, b: bool) -> Result<(), VmError> {
        match op {
            Opcode::PushBool => {
                self.push(Value::Bool(b));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn execute_with_idx(&mut self, op: Opcode, idx: usize) -> Result<(), VmError> {
        match op {
            Opcode::PushInt => {
                let n = self.bytecode.ints.get(idx).copied().unwrap_or(0);
                self.push(Value::Int(n));
            }
            Opcode::PushNum => {
                let n = self.bytecode.nums.get(idx).copied().unwrap_or(0.0);
                self.push(Value::Float(n));
            }
            Opcode::PushStr => {
                let s = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                self.push(Value::str(s));
            }
            Opcode::PushBool => {
                self.push(Value::Bool(idx != 0));
            }
            Opcode::Load => {
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let val = self.lookup(&name)?;
                self.push(val);
            }
            Opcode::Store => {
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let val = self.pop()?;
                let n = self.locals.len();
                if n > 0 {
                    let cur = n - 1;
                    let write_idx = if self.locals[cur].contains_key(&name) {
                        cur
                    } else if n >= 2 && self.locals[0].contains_key(&name) {
                        0
                    } else {
                        cur
                    };
                    self.locals[write_idx].insert(name, val);
                }
            }            Opcode::Call => {
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let argc_idx = self.ip;
                self.ip += 1;
                let argc = if argc_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, nidx) = &self.bytecode.instructions[argc_idx] {
                        self.bytecode.nums.get(*nidx).copied().unwrap_or(0.0) as usize
                    } else {
                        0
                    }
                } else {
                    0
                };
                let mut args: Vec<Value> = Vec::new();
                for _ in 0..argc {
                    args.push(self.pop()?);
                }
                args.reverse();
                if let Some(result) = self.call_core_builtin(&name, &args) {
                    return result;
                }
                #[cfg(feature = "full")]
                if let Some(result) = self.call_full_builtin(&name, &args) {
                    return result;
                }
                if let Some(func) = self.find_func(&name) {
                    let func_start = func.start;
                    let func_params = func.params.clone();
                    self.call_stack.push(CallFrame {
                        func_name: name.clone(),
                        return_ip: self.ip,
                    });
                    let mut scope = HashMap::with_capacity_and_hasher(func_params.len(), FixHasher::default());
                    for (i, param_name) in func_params.iter().enumerate() {
                        if let Some(arg) = args.get(i) {
                            scope.insert(param_name.clone(), arg.clone());
                        }
                    }
                    self.locals.push(scope);
                    self.ip = func_start;
                } else {
                    return Err(VmError::UndefinedFunction(name));
                }
            }
            Opcode::FuncRef => {
                let name = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                self.push(Value::Func(name));
            }
            Opcode::CallValue => {
                let argc = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let mut args = Vec::new();
                for _ in 0..argc {
                    args.push(self.pop()?);
                }
                args.reverse();
                let callee = self.pop()?;
                let name = match &callee {
                    Value::Func(n) => n.clone(),
                    _ => {
                        return Err(VmError::TypeError(
                            "Se esperaba una función para llamar".to_string(),
                        ))
                    }
                };
                if name == "imprimir" || name == "print" {
                    for arg in args {
                        let s = format!("{}", arg);
                        self.output.push(s);
                    }
                    self.push(Value::Void);
                } else if name == "leer" || name == "read" {
                    self.push(Value::str(String::new()));
                } else if name == "a_texto" || name == "to_texto" || name == "__str_from" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::str(s));
                } else if name == "__str_a_entero" || name == "__texto_a_entero" {
                    let mut s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    if let Some(dot) = s.find('.') {
                        s.truncate(dot);
                    }
                    match s.parse::<i64>() {
                        Ok(n) => self.push(Value::Int(n)),
                        Err(_) => self.push(Value::Int(0)),
                    }
                } else if name == "__str_len" || name == "__str_longitud" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Int(s.len() as i64));
                } else if name == "__str_upper" || name == "__str_mayusculas" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::str(s.to_uppercase()));
                } else if name == "__str_lower" || name == "__str_minusculas" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::str(s.to_lowercase()));
                } else if name == "__str_trim" || name == "__str_recortar" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::str(s.trim().to_string()));
                } else if name == "__str_contains" || name == "__str_contiene" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let sub = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Bool(s.contains(&sub)));
                } else if name == "__str_split" || name == "__str_dividir" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let delim = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let parts: Vec<Value> = if delim.is_empty() {
                        s.chars().map(|c| Value::str(c.to_string())).collect()
                    } else {
                        s.split(&delim).map(|p| Value::str(p.to_string())).collect()
                    };
                    self.push(Value::arr(parts));
                } else if name == "__str_ord" || name == "__str_codigo" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let codes: Vec<Value> = s.chars().map(|c| Value::Int(c as i64)).collect();
                    self.push(Value::arr(codes));
                } else if name == "__str_chr" || name == "__str_caracter" {
                    let n = args
                        .first()
                        .and_then(|v| match v {
                            Value::Int(i) => Some(*i),
                            _ => v.as_num().map(|f| f as i64),
                        })
                        .unwrap_or(0);
                    let c = char::from_u32(n as u32).map(|c| c.to_string()).unwrap_or_default();
                    self.push(Value::str(c));
                } else if name == "__str_slice" || name == "__str_subcadena" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let start = args.get(1).and_then(|v| match v {
                        Value::Int(i) => Some(*i as usize),
                        _ => v.as_num().map(|f| f as usize),
                    }).unwrap_or(0);
                    let end = args.get(2).and_then(|v| match v {
                        Value::Int(i) => {
                            if *i == -1 { Some(s.len()) } else { Some(*i as usize) }
                        }
                        _ => v.as_num().map(|f| f as usize),
                    }).unwrap_or(s.len());
                    let start = start.min(s.len());
                    let end = end.min(s.len()).max(start);
                    let sub: String = s.chars().skip(start).take(end - start).collect();
                    self.push(Value::str(sub));
                } else if name == "__str_concat_list" || name == "__str_concatenar_lista" {
                    let list = args.first().cloned().unwrap_or(Value::arr(vec![]));
                    match list {
                        Value::Array(items) => {
                            let result = items.iter().map(|v| format!("{}", v)).collect::<String>();
                            self.push(Value::str(result));
                        }
                        _ => self.push(Value::str(String::new())),
                    }
                } else if name == "__str_starts_with" || name == "__str_empieza_con" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let prefix = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Bool(s.starts_with(&prefix)));
                } else if name == "__str_to_chars" || name == "__str_a_caracteres" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let chars: Vec<Value> = s.chars().map(|c| Value::str(c.to_string())).collect();
                    self.push(Value::arr(chars));
                } else if name == "__str_reemplazar" || name == "__str_replace" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let from = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let to = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::str(s.replace(&from, &to)));
                } else if name == "__str_subcadena_chars" || name == "__str_slice_chars" {
                    let cs = match args.first() {
                        Some(Value::Array(a)) => a.clone(),
                _ => Arc::new(vec![]),
                    };
                    let st = args.get(1).and_then(|v| v.as_num()).map(|f| f as i64).unwrap_or(0);
                    let en = args.get(2).and_then(|v| v.as_num()).map(|f| f as i64).unwrap_or(-1);
                    let n = cs.len() as i64;
                    let st = st.max(0).min(n);
                    let en = if en < 0 { n } else { en.max(0).min(n) };
                    let mut out = String::new();
                    for c in cs.iter().skip(st as usize).take((en - st).max(0) as usize) {
                        out.push_str(&format!("{}", c));
                    }
                    self.push(Value::str(out));
                } else if name == "__map_new" || name == "__map_nuevo" {
                    self.push(Value::Map(ImMap::with_hasher(FixHasher::default())));
                } else if name == "__map_set" || name == "__map_poner" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let k = it.next().unwrap_or(Value::Void);
                    let v = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(mut m) => {
                            m.insert(k, v);
                            self.push(Value::Map(m));
                        }
                        _ => return Err(VmError::TypeError("__map_set espera diccionario".into())),
                    }
                } else if name == "__map_get" || name == "__map_obtener" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let k = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(m) => {
                            self.push(m.get(&k).cloned().unwrap_or(Value::Void));
                        }
                        _ => return Err(VmError::TypeError("__map_get espera diccionario".into())),
                    }
                } else if name == "__map_len" || name == "__map_longitud" {
                    let m = args.into_iter().next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match m {
                        Value::Map(m) => self.push(Value::Int(m.len() as i64)),
                        _ => return Err(VmError::TypeError("__map_len espera diccionario".into())),
                    }
                } else if name == "__map_keys" || name == "__map_claves" {
                    let m = args.into_iter().next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match m {
                        Value::Map(m) => {
                            self.push(Value::arr(m.into_iter().map(|(k, _)| k).collect()));
                        }
                        _ => {
                            return Err(VmError::TypeError("__map_keys espera diccionario".into()))
                        }
                    }
                } else if name == "__map_contains" || name == "__map_contiene" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let k = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(m) => self.push(Value::Bool(m.contains_key(&k))),
                        _ => {
                            return Err(VmError::TypeError(
                                "__map_contains espera diccionario".into(),
                            ))
                        }
                    }
                } else if name == "__set_new" || name == "__conjunto_nuevo" {
                    self.push(Value::Map(ImMap::with_hasher(FixHasher::default())));
                } else if name == "__set_add" || name == "__conjunto_agregar" {
                    let mut it = args.into_iter();
                    let s = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let item = it.next().unwrap_or(Value::Void);
                    match s {
                        Value::Map(mut m) => {
                            m.insert(item, Value::Bool(true));
                            self.push(Value::Map(m));
                        }
                        _ => return Err(VmError::TypeError("__set_add espera conjunto".into())),
                    }
                } else if name == "__set_has" || name == "__conjunto_tiene" {
                    let mut it = args.into_iter();
                    let s = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let item = it.next().unwrap_or(Value::Void);
                    match s {
                        Value::Map(m) => self.push(Value::Bool(m.contains_key(&item))),
                        _ => return Err(VmError::TypeError("__set_has espera conjunto".into())),
                    }
                } else if name == "__set_union" || name == "__conjunto_unir" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let b = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match (a, b) {
                        (Value::Map(mut m1), Value::Map(m2)) => {
                            for (k, v) in m2 {
                                if !m1.contains_key(&k) { m1.insert(k, v); }
                            }
                            self.push(Value::Map(m1));
                        }
                        _ => return Err(VmError::TypeError("__set_union espera conjuntos".into())),
                    }
                } else if name == "__set_inter" || name == "__conjunto_interseccion" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let b = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match (a, b) {
                        (Value::Map(m1), Value::Map(m2)) => {
                            let r: ImMap<Value, Value, FixHasher> = m1
                                .into_iter()
                                .filter(|(k, _)| m2.contains_key(k))
                                .collect();
                            self.push(Value::Map(r));
                        }
                        _ => return Err(VmError::TypeError("__set_inter espera conjuntos".into())),
                    }
                } else if name == "__set_diff" || name == "__conjunto_diferencia" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let b = it.next().unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match (a, b) {
                        (Value::Map(m1), Value::Map(m2)) => {
                            let r: ImMap<Value, Value, FixHasher> = m1
                                .into_iter()
                                .filter(|(k, _)| !m2.contains_key(k))
                                .collect();
                            self.push(Value::Map(r));
                        }
                        _ => return Err(VmError::TypeError("__set_diff espera conjuntos".into())),
                    }
                } else if name == "__deque_new" || name == "__deque_nuevo" {
                    self.push(Value::arr(vec![]));
                } else if name == "__deque_push_front" || name == "__deque_agregar_frente" {
                    let mut it = args.into_iter();
                    let d = it.next().unwrap_or(Value::arr(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match d {
                        Value::Array(mut v) => {
                            Arc::make_mut(&mut v).insert(0, item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__deque_push_front espera deque".into(),
                            ))
                        }
                    }
                } else if name == "__deque_push_back" || name == "__deque_agregar_final" {
                    let mut it = args.into_iter();
                    let d = it.next().unwrap_or(Value::arr(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match d {
                        Value::Array(mut v) => {
                            Arc::make_mut(&mut v).push(item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError("__deque_push_back espera deque".into()))
                        }
                    }
                } else if name == "__deque_pop_front" || name == "__deque_quitar_frente" {
                    let d = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match d {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            Arc::make_mut(&mut v).remove(0)
                        }),
                        _ => {
                            return Err(VmError::TypeError("__deque_pop_front espera deque".into()))
                        }
                    }
                } else if name == "__deque_pop_back" || name == "__deque_quitar_final" {
                    let d = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match d {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            Arc::make_mut(&mut v).pop().unwrap_or(Value::Void)
                        }),
                        _ => {
                            return Err(VmError::TypeError("__deque_pop_back espera deque".into()))
                        }
                    }
                } else if name == "__deque_len" || name == "__deque_longitud" {
                    let d = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match d {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__deque_len espera deque".into())),
                    }
                } else if name == "__heap_new" || name == "__monticulo_nuevo" {
                    self.push(Value::arr(vec![]));
                } else if name == "__heap_push" || name == "__monticulo_agregar" {
                    let mut it = args.into_iter();
                    let h = it.next().unwrap_or(Value::arr(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match h {
                        Value::Array(mut v) => {
                            Arc::make_mut(&mut v).push(item);
                            Arc::make_mut(&mut v).sort_by(|a, b| {
                                let an = a.as_num().unwrap_or(f64::MIN);
                                let bn = b.as_num().unwrap_or(f64::MIN);
                                bn.partial_cmp(&an).unwrap_or(std::cmp::Ordering::Equal)
                            });
                            self.push(Value::Array(v));
                        }
                        _ => return Err(VmError::TypeError("__heap_push espera heap".into())),
                    }
                } else if name == "__heap_pop" || name == "__monticulo_quitar" {
                    let h = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match h {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            Arc::make_mut(&mut v).remove(0)
                        }),
                        _ => return Err(VmError::TypeError("__heap_pop espera heap".into())),
                    }
                } else if name == "__heap_peek" || name == "__monticulo_ver" {
                    let h = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match h {
                        Value::Array(v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v[0].clone()
                        }),
                        _ => return Err(VmError::TypeError("__heap_peek espera heap".into())),
                    }
                } else if name == "__heap_len" || name == "__monticulo_longitud" {
                    let h = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match h {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__heap_len espera heap".into())),
                    }
                } else if name == "__linked_new" || name == "__enlazada_nuevo" {
                    self.push(Value::arr(vec![]));
                } else if name == "__linked_push_front" || name == "__enlazada_agregar_frente" {
                    let mut it = args.into_iter();
                    let l = it.next().unwrap_or(Value::arr(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match l {
                        Value::Array(mut v) => {
                            Arc::make_mut(&mut v).insert(0, item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_push_front espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_push_back" || name == "__enlazada_agregar_final" {
                    let mut it = args.into_iter();
                    let l = it.next().unwrap_or(Value::arr(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match l {
                        Value::Array(mut v) => {
                            Arc::make_mut(&mut v).push(item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_push_back espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_pop_front" || name == "__enlazada_quitar_frente" {
                    let l = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match l {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            Arc::make_mut(&mut v).remove(0)
                        }),
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_pop_front espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_pop_back" || name == "__enlazada_quitar_final" {
                    let l = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match l {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            Arc::make_mut(&mut v).pop().unwrap_or(Value::Void)
                        }),
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_pop_back espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_len" || name == "__enlazada_longitud" {
                    let l = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match l {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__linked_len espera linked".into())),
                    }
                } else if name == "__regex_new" || name == "__regex_nuevo" {
                    let pat = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&pat) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__regex_is_match" || name == "__regex_coincide" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => self.push(Value::Bool(r.is_match(&text))),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__regex_captures" || name == "__regex_capturar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => {
                            if let Some(caps) = r.captures(&text) {
                                let vs: Vec<Value> = caps
                                    .iter()
                                    .map(|m| {
                                        Value::str(
                                            m.map(|x| x.as_str().to_string()).unwrap_or_default(),
                                        )
                                    })
                                    .collect();
                                self.push(Value::arr(vs));
                            } else {
                                self.push(Value::arr(vec![]));
                            }
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__regex_replace" || name == "__regex_reemplazar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let rep = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => {
                            self.push(Value::str(r.replace_all(&text, rep.as_str()).to_string()))
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__unicode_normalize" || name == "__unicode_normalizar" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let form = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let nf: String = match form.as_str() {
                        "NFC" => s.nfc().collect(),
                        "NFD" => s.nfd().collect(),
                        "NFKC" => s.nfkc().collect(),
                        "NFKD" => s.nfkd().collect(),
                        _ => s.nfc().collect(),
                    };
                    self.push(Value::str(nf));
                } else if name == "__str_pad_start" || name == "__str_padding_inicio" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
                    let ch = args
                        .get(2)
                        .map(|v| format!("{}", v))
                        .unwrap_or_default()
                        .chars()
                        .next()
                        .unwrap_or(' ');
                    self.push(Value::str(format!(
                        "{}{}",
                        ch.to_string().repeat(len.saturating_sub(s.len())),
                        s
                    )));
                } else if name == "__str_pad_end" || name == "__str_padding_fin" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
                    let ch = args
                        .get(2)
                        .map(|v| format!("{}", v))
                        .unwrap_or_default()
                        .chars()
                        .next()
                        .unwrap_or(' ');
                    self.push(Value::str(format!(
                        "{}{}",
                        s,
                        ch.to_string().repeat(len.saturating_sub(s.len()))
                    )));
                } else if name == "__encoding_utf8" || name == "__codificacion_utf8" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::arr(
                        s.bytes().map(|b| Value::Int(b as i64)).collect(),
                    ));
                } else if name == "__encoding_from_utf8" || name == "__desde_utf8" {
                    let arr = args.into_iter().next().unwrap_or(Value::arr(vec![]));
                    match arr {
                        Value::Array(v) => {
                            let bytes: Vec<u8> = v
                                .iter()
                                .filter_map(|x| {
                                    if let Value::Int(n) = x {
                                        Some(*n as u8)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            self.push(Value::str(String::from_utf8_lossy(&bytes).to_string()));
                        }
                        _ => self.push(Value::str(String::new())),
                    }
                } else if name == "__buf_reader" || name == "__lector_buffer" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::read_to_string(&path) {
                        Ok(c) => {
                            let lines: Vec<Value> =
                                c.lines().map(|l| Value::str(l.to_string())).collect();
                            self.push(Value::arr(lines));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__buf_writer" || name == "__escritor_buffer" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::write(&path, &content) {
                        Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__stream_chunks" || name == "__stream_trozos" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let size = args.get(1).and_then(|v| v.as_num()).unwrap_or(4096.0) as usize;
                    match std::fs::read(&path) {
                        Ok(data) => {
                            let chunks: Vec<Value> = data
                                .chunks(size)
                                .map(|c| {
                                    Value::arr(c.iter().map(|&b| Value::Int(b as i64)).collect())
                                })
                                .collect();
                            self.push(Value::arr(chunks));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__tcp_connect" || name == "__tcp_conectar" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::net::TcpStream::connect(&addr) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__tcp_listen" || name == "__tcp_escuchar" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::net::TcpListener::bind(&addr) {
                        Ok(l) => {
                            self.tcp_listener = Some(l);
                            self.push(Value::Bool(true));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__tcp_accept" || name == "__tcp_aceptar" {
                    match &self.tcp_listener {
                        Some(l) => match l.accept() {
                            Ok((_stream, _)) => {
                                let addr = _stream
                                    .peer_addr()
                                    .map(|a| a.to_string())
                                    .unwrap_or_default();
                                self.push(Value::str(addr));
                            }
                            Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                        },
                        None => {
                            self.push(Value::Error(Box::new(Value::str("Sin listener"))))
                        }
                    }
                } else if name == "__http_server" || name == "__http_servidor" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::str(format!("HTTP server on {}", addr)));
                } else if name == "__serial_open" || name == "__serial_abrir" {
                    self.push(Value::Bool(true));
                } else if name == "__json_parse" || name == "__json_parsear" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match serde_json::from_str::<serde_json::Value>(&s) {
                        Ok(v) => self.push(json_value_to_lumen(v)),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__json_stringify" || name == "__json_texto" {
                    let val = args.first().cloned().unwrap_or(Value::Void);
                    let json = lumen_value_to_json(&val);
                    self.push(Value::str(serde_json::to_string(&json).unwrap_or_default()));
                } else if let Some(func) = self.find_func(&name) {
                    let func_start = func.start;
                    let func_params = func.params.clone();
                    self.call_stack.push(CallFrame {
                        func_name: name.clone(),
                        return_ip: self.ip,
                    });
                    let mut scope = HashMap::with_capacity_and_hasher(func_params.len(), FixHasher::default());
                    for (i, param_name) in func_params.iter().enumerate() {
                        if let Some(arg) = args.get(i) {
                            scope.insert(param_name.clone(), arg.clone());
                        }
                    }
                    self.locals.push(scope);
                    self.ip = func_start;
                } else {
                    return Err(VmError::UndefinedFunction(name));
                }
            }
            Opcode::ArrayNew => {
                let n = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(Value::arr(items));
            }
            Opcode::StructNew => {
                let struct_name = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                let argc_idx = self.ip;
                self.ip += 1;
                let count = if argc_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, nidx) = &self.bytecode.instructions[argc_idx] {
                        self.bytecode.nums.get(*nidx).copied().unwrap_or(0.0) as usize
                    } else {
                        0
                    }
                } else {
                    0
                };
                let mut field_names = Vec::with_capacity(count);
                for _ in 0..count {
                    field_names.push(self.pop()?);
                }
                field_names.reverse();
                let mut field_values = Vec::with_capacity(count);
                for _ in 0..count {
                    field_values.push(self.pop()?);
                }
                field_values.reverse();
                let fields: Vec<(String, Value)> = field_names
                    .into_iter()
                    .zip(field_values)
                    .map(|(name, val)| {
                        let n = match name {
                            Value::Str(s) => s.to_string(),
                            _ => "?".to_string(),
                        };
                        (n, val)
                    })
                    .collect();
                self.push(Value::Struct {
                    name: struct_name,
                    fields,
                });
            }
            Opcode::Jmp => {
                let target = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                self.ip = target;
            }
            Opcode::JmpIf => {
                let val = self.pop()?;
                if !val.is_truthy() {
                    let target = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                    self.ip = target;
                }
            }
            Opcode::TupleNew => {
                let n = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(Value::Tuple(items));
            }
            Opcode::TupleAccess => {
                let index = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let tuple_val = self.pop()?;
                match tuple_val {
                    Value::Tuple(items) => {
                        if index >= items.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango para tupla de {} elementos",
                                index,
                                items.len()
                            )));
                        }
                        self.push(items[index].clone());
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "TupleAccess requires a tuple value".to_string(),
                        ));
                    }
                }
            }
            Opcode::EnumCtor => {
                let enum_name = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                let var_idx = self.ip;
                self.ip += 1;
                let variant = if var_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, vidx) = &self.bytecode.instructions[var_idx] {
                        self.bytecode
                            .strings
                            .get(*vidx)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let argc_idx = self.ip;
                self.ip += 1;
                let argc = if argc_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, nidx) = &self.bytecode.instructions[argc_idx] {
                        self.bytecode.nums.get(*nidx).copied().unwrap_or(0.0) as usize
                    } else {
                        0
                    }
                } else {
                    0
                };
                let mut fields = Vec::with_capacity(argc);
                for _ in 0..argc {
                    fields.push(self.pop()?);
                }
                fields.reverse();
                self.push(Value::Enum {
                    name: enum_name,
                    variant,
                    fields,
                });
            }
            _ => {}
        }
        Ok(())
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn lookup(&self, name: &str) -> Result<Value, VmError> {
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }
        Err(VmError::UndefinedVariable(name.to_string()))
    }

    fn codegen_to_nvc(&self, cg: Value) -> Result<Value, VmError> {
        let cg_map = match &cg {
            Value::Map(m) => m.clone(),
            _ => return Err(VmError::TypeError("codegen must be a map".into())),
        };

        let map_get = |map: &ImMap<Value, Value, FixHasher>, key: &str| -> Option<Value> {
            map.get(&Value::str(key.to_string())).cloned()
        };

        let map_get_i64 = |map: &ImMap<Value, Value, FixHasher>, key: &str| -> Option<i64> {
            match map_get(map, key)? {
                Value::Int(n) => Some(n),
                _ => None,
            }
        };

        let str_cnt = map_get_i64(&cg_map, "str_cnt").unwrap_or(0) as usize;
        let int_cnt = map_get_i64(&cg_map, "int_cnt").unwrap_or(0) as usize;
        let pc = map_get_i64(&cg_map, "pos").unwrap_or(0) as usize;

        let strs_map = match map_get(&cg_map, "strings") {
            Some(Value::Map(m)) => m,
            _ => ImMap::with_hasher(FixHasher::default()),
        };
        let ints_map = match map_get(&cg_map, "ints") {
            Some(Value::Map(m)) => m,
            _ => ImMap::with_hasher(FixHasher::default()),
        };
        let instrs_map = match map_get(&cg_map, "instrs") {
            Some(Value::Map(m)) => m,
            _ => ImMap::with_hasher(FixHasher::default()),
        };

        let get_str = |idx: usize| -> String {
            match strs_map.get(&Value::str(idx.to_string())) {
                Some(Value::Str(s)) => s.to_string(),
                _ => String::new(),
            }
        };

        let get_int = |idx: usize| -> i64 {
            match ints_map.get(&Value::str(idx.to_string())) {
                Some(Value::Int(n)) => *n,
                Some(Value::Float(f)) => *f as i64,
                Some(Value::Str(s)) => s.parse::<i64>().unwrap_or(0),
                _ => 0,
            }
        };

        let get_instr = |idx: usize| -> Option<(i64, i64)> {
            match instrs_map.get(&Value::str(idx.to_string())) {
                Some(Value::Map(m)) => {
                    let op = m.get(&Value::str("op"))
                        .and_then(|v| if let Value::Int(o) = v { Some(*o) } else { None })
                        .unwrap_or(0);
                    let arg = m.get(&Value::str("arg"))
                        .and_then(|v| if let Value::Int(a) = v { Some(*a) } else { None })
                        .unwrap_or(0);
                    Some((op, arg))
                }
                _ => None,
            }
        };

        // Build names table from LOAD(21)/STORE(20)/CALL(24) instructions
        let mut names: Vec<String> = Vec::new();
        for ip in 0..pc {
            if let Some((op, arg)) = get_instr(ip) {
                if op == 20 || op == 21 || op == 24 {
                    let name = get_str(arg as usize);
                    if !name.is_empty() && !names.contains(&name) {
                        names.push(name);
                    }
                }
            }
        }

        // Collect nums (argc values from CALL pairs)
        let mut nums_bytes: Vec<u8> = Vec::new();
        let mut num_cnt = 0u32;
        let mut instr_bytes: Vec<u8> = Vec::new();

        // Opcode mapping: codegen.nv opcodes → Rust VM Opcode::to_u8()
        // codegen: 1=PushStr, 2=PushStr, 4=PushBool, 5=Add... 27=Halt
        // Rust VM: Nop=0, PushInt=1, PushNum=2, PushStr=3, PushBool=4,
        //          Load=5, Store=6, Add=7, ... Halt=27, Mod=46
        let cg_to_vm = |cg_op: i64| -> u8 {
            match cg_op {
                1 => 2,       // PushNum → VM PushNum (f64)
                2 => 3,       // PushStr
                3 => 1,       // PushInt
                4 => 4,       // PushBool
                5 => 7,       // Add
                6 => 8,       // Sub
                7 => 9,       // Mul
                8 => 10,      // Div
                9 => 46,      // Mod
                10 => 11,     // Eq
                11 => 12,     // Neq
                12 => 13,     // Lt
                13 => 14,     // Le
                14 => 15,     // Gt
                15 => 16,     // Ge
                16 => 17,     // And
                17 => 18,     // Or
                18 => 20,     // Not
                19 => 19,     // Neg
                20 => 6,      // Store
                21 => 5,      // Load
                22 => 25,     // Jmp
                23 => 26,     // JmpIf
                24 => 21,     // Call
                25 => 22,     // Ret
                26 => 23,     // Print
                27 => 27,     // Halt
                 28 => 28,     // ArrayNew
                 29 => 29,     // ArrayGet
                 30 => 30,     // ArraySet
                 31 => 31,     // ArrayLen
                 38 => 38,     // ResultOk
                 39 => 39,     // ResultErr
                 40 => 40,     // TryUnwrap
                 41 => 41,     // OptionSome
                 42 => 42,     // OptionNone
                 47 => 44,     // TupleNew
                 48 => 45,     // TupleAccess
                 46 => 47,     // BitOr
                 49 => 48,     // BitAnd
                  50 => 49,     // ShiftLeft
                  51 => 50,     // ShiftRight
                  52 => 51,     // Concat
                  _ => 0,       // Nop
            }
        };

        let mut i = 0usize;
        while i < pc {
            if let Some((op, arg)) = get_instr(i) {
                if op == 24 && i + 1 < pc {
                    if let Some((next_op, next_arg)) = get_instr(i + 1) {
                        if next_op == 24 {
                            // CALL pair
                            let name = get_str(arg as usize);
                            let name_idx = names.iter().position(|n| n == &name).unwrap_or(0);
                            // Call instruction: tag 0x04, opcode 21, u32 name_idx
                            instr_bytes.push(4);
                            instr_bytes.push(21);
                            instr_bytes.extend_from_slice(&(name_idx as u32).to_le_bytes());
                            // Get argc from ints table
                            let argc = get_int(next_arg as usize);
                            nums_bytes.extend_from_slice(&(argc as f64).to_le_bytes());
                            // Nop instruction: tag 0x04, opcode 0, u32 num_idx
                            instr_bytes.push(4);
                            instr_bytes.push(0);
                            instr_bytes.extend_from_slice(&num_cnt.to_le_bytes());
                            num_cnt += 1;
                            i += 2;
                            continue;
                        }
                    }
                }

                let vm_op = cg_to_vm(op);

                // Simple ops (5-19 except 4) plus array ops (29-31; 28 needs WithIdx)
                if (op >= 5 && op <= 19 && op != 4) || (op == 29 || op == 30 || op == 31) {
                    instr_bytes.push(0);
                    instr_bytes.push(vm_op);
                } else if op == 28 || op == 47 || op == 48 {
                    // ArrayNew/TupleNew/TupleAccess: VM expects arg = nums idx (value lives in nums)
                    let val = get_int(arg as usize) as f64;
                    nums_bytes.extend_from_slice(&val.to_le_bytes());
                    instr_bytes.push(4);
                    instr_bytes.push(vm_op);
                    instr_bytes.extend_from_slice(&num_cnt.to_le_bytes());
                    num_cnt += 1;
                } else if op == 4 {
                    // PushBool: value is ints[arg]
                    let val = get_int(arg as usize);
                    instr_bytes.push(4);
                    instr_bytes.push(4);
                    instr_bytes.extend_from_slice(&(val as u32).to_le_bytes());
                } else if op == 1 || op == 3 {
                    // PushNum/PushInt: arg is ints index → VM PushInt (1) / PushNum (2)
                    if op == 1 {
                        let val = match ints_map.get(&Value::str(arg.to_string())) {
                            Some(Value::Float(f)) => *f,
                            Some(Value::Int(n)) => *n as f64,
                            Some(Value::Str(s)) => s.parse::<f64>().unwrap_or(0.0),
                            _ => 0.0,
                        };
                        nums_bytes.extend_from_slice(&val.to_le_bytes());
                        instr_bytes.push(4);
                        instr_bytes.push(2);
                        instr_bytes.extend_from_slice(&num_cnt.to_le_bytes());
                        num_cnt += 1;
                    } else {
                        instr_bytes.push(4);
                        instr_bytes.push(1);
                        instr_bytes.extend_from_slice(&(arg as u32).to_le_bytes());
                    }
                } else if op == 2 {
                    // PushStr: arg is strings index
                    instr_bytes.push(4);
                    instr_bytes.push(3);
                    instr_bytes.extend_from_slice(&(arg as u32).to_le_bytes());
                } else if op == 20 || op == 21 {
                    // Store/Load: arg is strings index (variable name) → names table
                    let name = get_str(arg as usize);
                    let name_idx = names.iter().position(|n| n == &name).unwrap_or(0);
                    instr_bytes.push(4);
                    instr_bytes.push(vm_op);
                    instr_bytes.extend_from_slice(&(name_idx as u32).to_le_bytes());
                } else if op == 22 || op == 23 {
                    // Jmp/JmpIf: VM expects arg = nums idx; target value lives in nums
                    let target = get_int(arg as usize) as f64;
                    nums_bytes.extend_from_slice(&target.to_le_bytes());
                    instr_bytes.push(4);
                    instr_bytes.push(vm_op);
                    instr_bytes.extend_from_slice(&num_cnt.to_le_bytes());
                    num_cnt += 1;
                } else if op == 43 || op == 44 || op == 45 {
                    // EnumCtor triplet: 43 = WithIdx(EnumCtor, name@strings),
                    // 44 = WithIdx(Nop, variant@strings), 45 = WithIdx(Nop, argc@nums)
                    if op == 43 {
                        instr_bytes.push(4);
                        instr_bytes.push(43);
                        instr_bytes.extend_from_slice(&(arg as u32).to_le_bytes());
                    } else if op == 44 {
                        instr_bytes.push(4);
                        instr_bytes.push(0);
                        instr_bytes.extend_from_slice(&(arg as u32).to_le_bytes());
                    } else {
                        let argc = get_int(arg as usize) as f64;
                        nums_bytes.extend_from_slice(&argc.to_le_bytes());
                        instr_bytes.push(4);
                        instr_bytes.push(0);
                        instr_bytes.extend_from_slice(&num_cnt.to_le_bytes());
                        num_cnt += 1;
                    }
                } else if op == 25 || op == 26 || op == 27 {
                    // Ret, Print, Halt
                    instr_bytes.push(0);
                    instr_bytes.push(vm_op);
                } else {
                    instr_bytes.push(0);
                    instr_bytes.push(vm_op);
                }
            }
            i += 1;
        }

        // Count logical instructions for .nvc header
        let mut logical_cnt = 0u32;
        let mut bi = 0usize;
        while bi < instr_bytes.len() {
            let tag = instr_bytes[bi];
            logical_cnt += 1;
            match tag {
                0 => bi += 2,
                3 => bi += 3,
                _ => bi += 6,
            }
        }

        // Build final .nvc byte array
        let mut buf = Vec::new();

        // Magic: LUMN
        buf.extend_from_slice(b"LUMN");

        // Version
        buf.extend_from_slice(&6u32.to_le_bytes());

        // Strings table
        buf.extend_from_slice(&(str_cnt as u32).to_le_bytes());
        for si in 0..str_cnt {
            let s = get_str(si);
            let utf8 = s.as_bytes();
            buf.extend_from_slice(&(utf8.len() as u32).to_le_bytes());
            buf.extend_from_slice(utf8);
        }

        // Ints table
        buf.extend_from_slice(&(int_cnt as u32).to_le_bytes());
        for ii in 0..int_cnt {
            let val = get_int(ii);
            buf.extend_from_slice(&val.to_le_bytes());
        }

        // Nums table
        buf.extend_from_slice(&num_cnt.to_le_bytes());
        buf.extend_from_slice(&nums_bytes);

        // Names table
        buf.extend_from_slice(&(names.len() as u32).to_le_bytes());
        for name in &names {
            let utf8 = name.as_bytes();
            buf.extend_from_slice(&(utf8.len() as u32).to_le_bytes());
            buf.extend_from_slice(utf8);
        }

        // Funcs table
        let func_cnt = map_get_i64(&cg_map, "func_cnt").unwrap_or(0) as usize;
        let funcs_map = match map_get(&cg_map, "funcs") {
            Some(Value::Map(m)) => m,
            _ => ImMap::with_hasher(FixHasher::default()),
        };
        let mut func_bytes: Vec<u8> = Vec::new();
        func_bytes.extend_from_slice(&(func_cnt as u32).to_le_bytes());
        for fi in 0..func_cnt {
            let f = match funcs_map.get(&Value::str(fi.to_string())) {
                Some(Value::Map(m)) => m.clone(),
                _ => ImMap::with_hasher(FixHasher::default()),
            };
            let fname = match f.get(&Value::str("nombre")) {
                Some(Value::Str(s)) => s.to_string(),
                _ => String::new(),
            };
            func_bytes.extend_from_slice(&(fname.len() as u32).to_le_bytes());
            func_bytes.extend_from_slice(fname.as_bytes());
            let fparams = match f.get(&Value::str("params")) {
                Some(Value::Map(m)) => m.clone(),
                _ => ImMap::with_hasher(FixHasher::default()),
            };
            let pcount = match fparams.get(&Value::str("cnt")) {
                Some(Value::Int(n)) => *n as usize,
                _ => 0,
            };
            func_bytes.extend_from_slice(&(pcount as u32).to_le_bytes());
            for pi in 0..pcount {
                let pname = match fparams.get(&Value::str(pi.to_string())) {
                    Some(Value::Str(s)) => s.to_string(),
                _ => String::new(),
                };
                func_bytes.extend_from_slice(&(pname.len() as u32).to_le_bytes());
                func_bytes.extend_from_slice(pname.as_bytes());
            }
            let fstart = match f.get(&Value::str("start")) {
                Some(Value::Int(n)) => *n as u64,
                Some(Value::Float(n)) => *n as u64,
                _ => 0,
            };
            func_bytes.extend_from_slice(&fstart.to_le_bytes());
        }
        buf.extend_from_slice(&func_bytes);

        // Instruction count
        buf.extend_from_slice(&logical_cnt.to_le_bytes());

        // Instructions
        buf.extend_from_slice(&instr_bytes);

        // Return as Array<Int>
        let result: Vec<Value> = buf.iter().map(|&b| Value::Int(b as i64)).collect();
        Ok(Value::arr(result))
    }
}

fn json_value_to_lumen(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Void,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::str(s),
        serde_json::Value::Array(arr) => {
            Value::arr(arr.into_iter().map(json_value_to_lumen).collect())
        }
        serde_json::Value::Object(map) => {
            let mut m = ImMap::with_hasher(FixHasher::default());
            for (k, v) in map {
                m.insert(Value::str(k), json_value_to_lumen(v));
            }
            Value::Map(m)
        }
    }
}

fn lumen_value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Int(n) => serde_json::json!(*n),
        Value::Float(n) => serde_json::json!(*n),
        Value::Str(s) => serde_json::json!(s.as_ref()),
        Value::Bool(b) => serde_json::json!(*b),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(lumen_value_to_json).collect())
        }
        Value::Map(map) => {
            let mut obj = serde_json::Map::new();
            for (k, v) in map {
                if let Value::Str(key) = k {
                    obj.insert(key.to_string(), lumen_value_to_json(v));
                }
            }
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

// ── FFI dynamic call helper ─────────────────────────────────────────────
#[allow(dead_code)]
fn ffi_call_dynamic(
    func_ptr: *const (),
    _arg_types: &[&str],
    args: &[Value],
    ret_type: &str,
) -> Result<Value, String> {
    unsafe {
        match ret_type {
            "void" | "" => {
                let f: extern "C" fn() = std::mem::transmute(func_ptr);
                f();
                Ok(Value::Void)
            }
            "int" | "i32" | "i64" => {
                let f: unsafe extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64 =
                    std::mem::transmute(func_ptr);
                let mut a = [0i64; 6];
                for (i, arg) in args.iter().enumerate().take(6) {
                    a[i] = arg.as_num().unwrap_or(0.0) as i64;
                }
                Ok(Value::Int(f(a[0], a[1], a[2], a[3], a[4], a[5])))
            }
            "float" | "f64" | "double" => {
                let f: unsafe extern "C" fn(f64, f64, f64, f64, f64, f64) -> f64 =
                    std::mem::transmute(func_ptr);
                let mut a = [0.0f64; 6];
                for (i, arg) in args.iter().enumerate().take(6) {
                    a[i] = arg.as_num().unwrap_or(0.0);
                }
                Ok(Value::Float(f(a[0], a[1], a[2], a[3], a[4], a[5])))
            }
            "ptr" | "pointer" => {
                let f: unsafe extern "C" fn(i64, i64, i64, i64, i64, i64) -> i64 =
                    std::mem::transmute(func_ptr);
                let mut a = [0i64; 6];
                for (i, arg) in args.iter().enumerate().take(6) {
                    a[i] = arg.as_num().unwrap_or(0.0) as i64;
                }
                Ok(Value::Int(f(a[0], a[1], a[2], a[3], a[4], a[5])))
            }
            _ => Err(format!("unsupported return type: {}", ret_type)),
        }
    }
}

// ── JWT helpers ─────────────────────────────────────────────────────────
#[cfg(feature = "full")]
fn base64url_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

#[cfg(feature = "full")]
fn base64url_decode(data: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .unwrap_or_default()
}

#[cfg(feature = "full")]
fn base64url_decode_to_string(data: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 error: {}", e))
}

#[cfg(feature = "full")]
fn hmac_sha256(data: &[u8], key: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length OK");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

// ── Date helpers ─────────────────────────────────
fn format_timestamp(timestamp: i64, fmt: &str) -> String {
    if let Some(dt) = Utc.timestamp_opt(timestamp, 0).single() {
        if fmt.is_empty() {
            return dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        }
        let fmt = fmt
            .replace("%Y", &format!("{:04}", dt.year()))
            .replace("%m", &format!("{:02}", dt.month()))
            .replace("%d", &format!("{:02}", dt.day()))
            .replace("%H", &format!("{:02}", dt.hour()))
            .replace("%M", &format!("{:02}", dt.minute()))
            .replace("%S", &format!("{:02}", dt.second()))
            .replace("%A", &dt.format("%A").to_string())
            .replace("%B", &dt.format("%B").to_string())
            .replace("%W", &format!("{:02}", dt.iso_week().week()))
            .replace("%I", &format!("{:02}", dt.hour12().1))
            .replace("%p", &dt.format("%p").to_string());
        fmt
    } else {
        unix_timestamp_to_iso8601(timestamp)
    }
}

fn unix_timestamp_to_iso8601(timestamp: i64) -> String {
    // Simple algorithm: convert seconds since epoch to yyyy-mm-ddThh:mm:ssZ
    let secs = if timestamp >= 0 {
        timestamp
    } else {
        // Approximate negative
        return format!("{}T00:00:00Z", timestamp);
    };

    let mut remaining = secs;
    let sec = remaining % 60;
    remaining /= 60;
    let min = remaining % 60;
    remaining /= 60;
    let hour = remaining % 24;
    remaining /= 24; // days since epoch

    // Convert days since epoch to year/month/day
    let mut y = 1970i64;
    let mut d = remaining;

    // Simple leap-year-aware day count
    loop {
        let days_in_year = if is_leap(y) { 366 } else { 365 };
        if d < days_in_year {
            break;
        }
        d -= days_in_year;
        y += 1;
    }

    let months_days = if is_leap(y) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 1u32;
    for &md in months_days.iter() {
        if d < md {
            break;
        }
        d -= md;
        m += 1;
    }

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y,
        m,
        d + 1,
        hour,
        min,
        sec
    )
}

fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn parse_iso8601_to_unix(s: &str) -> Result<i64, String> {
    // Accept: "2024-01-15T10:30:00Z" or "2024-01-15T10:30:00" or "2024-01-15"
    let s = s.trim();
    let s: String = if s.contains(' ') && !s.contains('T') {
        s.replacen(' ', "T", 1)
    } else {
        s.to_string()
    };

    // Remove trailing Z
    let s = s.strip_suffix('Z').unwrap_or(&s);

    let (date_part, time_part) = if let Some(idx) = s.find('T') {
        let (d, t) = s.split_at(idx);
        (d, Some(&t[1..]))
    } else {
        (s, None)
    };

    // Parse date: YYYY-MM-DD
    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 {
        return Err(format!("Formato de fecha inválido: {}", s));
    }
    let year: i64 = parts[0]
        .parse()
        .map_err(|_| format!("Año inválido: {}", parts[0]))?;
    let month: u32 = parts[1]
        .parse()
        .map_err(|_| format!("Mes inválido: {}", parts[1]))?;
    let day: u32 = parts[2]
        .parse()
        .map_err(|_| format!("Día inválido: {}", parts[2]))?;

    let (hour, min, sec) = if let Some(t) = time_part {
        let tparts: Vec<&str> = t.split(':').collect();
        if tparts.len() != 3 {
            return Err(format!("Formato de hora inválido: {}", t));
        }
        let h: u32 = tparts[0]
            .parse()
            .map_err(|_| format!("Hora inválida: {}", tparts[0]))?;
        let m: u32 = tparts[1]
            .parse()
            .map_err(|_| format!("Minuto inválido: {}", tparts[1]))?;
        let s: u32 = tparts[2]
            .parse()
            .map_err(|_| format!("Segundo inválido: {}", tparts[2]))?;
        (h, m, s)
    } else {
        (0, 0, 0)
    };

    // Compute days since epoch
    let days = days_since_epoch(year, month, day);
    let total_secs = days * 86400 + hour as i64 * 3600 + min as i64 * 60 + sec as i64;
    Ok(total_secs)
}

fn days_since_epoch(year: i64, month: u32, day: u32) -> i64 {
    let mut total = 0i64;
    let mut y = 1970;
    while y < year {
        total += if is_leap(y) { 366 } else { 365 };
        y += 1;
    }
    while y > year {
        y -= 1;
        total -= if is_leap(y) { 366 } else { 365 };
    }

    let months_days = if is_leap(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    for (i, &md) in months_days.iter().enumerate() {
        let m = (i + 1) as u32;
        if m < month {
            total += md as i64;
        } else if m == month {
            total += (day - 1) as i64;
            break;
        }
    }

    total
}

// ── String ordinal helper ───────────────────────────────────────────────
fn __str_ord(s: &str) -> Vec<i64> {
    s.chars().map(|c| c as i64).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_codegen::bytecode::{Bytecode, FuncMeta, Instruction, Opcode};

    fn make_bc(instrs: Vec<Instruction>) -> Bytecode {
        Bytecode {
            instructions: instrs,
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        }
    }

    #[test]
    fn test_halt() {
        let bc = make_bc(vec![Instruction::Simple(Opcode::Halt)]);
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_push_num() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_add() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![2.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_print() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["hola".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["hola"]);
    }

    #[test]
    fn test_division_by_zero() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Div),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![1.0, 0.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_store_and_load() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::Store, 0),
                Instruction::WithIdx(Opcode::Load, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec!["x".to_string()],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["42"]);
    }

    #[test]
    fn test_comparisons() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Lt),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![1.0, 2.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_conditional_jump() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushBool, 0),
                Instruction::WithIdx(Opcode::JmpIf, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![4.0, 42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_neg() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Neg),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["-42"]);
    }

    #[test]
    fn test_not() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::Simple(Opcode::Not),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["false"]);
    }

    #[test]
    fn test_jmp() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::Jmp, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        // Should skip PushNum and Print, output is empty
        assert!(vm.output().is_empty());
    }

    #[test]
    fn test_jmpif_true_does_not_jump() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::WithIdx(Opcode::JmpIf, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![4.0, 42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["42"]);
    }

    #[test]
    fn test_call_builtin_print() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 0),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["test".to_string()],
            ints: vec![],
            nums: vec![1.0],
            names: vec!["imprimir".to_string()],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["test"]);
    }

    #[test]
    fn test_call_builtin_read() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 0),
                Instruction::WithIdx(Opcode::Call, 1),
                Instruction::WithIdx(Opcode::Nop, 1),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![0.0, 1.0],
            names: vec!["leer".to_string(), "imprimir".to_string()],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        // leer pushes empty string, then imprimir prints it
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &[""]);
    }

    #[test]
    fn test_call_user_function() {
        let bc = Bytecode {
            instructions: vec![
                // __main__: push 3, push 4, call sum, print result, halt
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 0), // same num
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 1),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
                // sum function at offset 6: load a, load b, add, ret
                Instruction::WithIdx(Opcode::Load, 1),
                Instruction::WithIdx(Opcode::Load, 2),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Ret),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 2.0],
            names: vec!["sum".to_string(), "a".to_string(), "b".to_string()],
            funcs: vec![
                FuncMeta {
                    name: "__main__".to_string(),
                    params: vec![],
                    start: 0,
                },
                FuncMeta {
                    name: "sum".to_string(),
                    params: vec!["a".to_string(), "b".to_string()],
                    start: 6,
                },
            ],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["6"]);
    }

    #[test]
    fn test_call_undefined_function() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 0),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![0.0],
            names: vec!["nonexistent".to_string()],
            funcs: vec![FuncMeta {
                name: "__main__".to_string(),
                params: vec![],
                start: 0,
            }],
        };
        let mut vm = VM::new(bc);
        let result = vm.run();
        assert!(result.is_err());
        match result.unwrap_err() {
            VmError::UndefinedFunction(_) => {}
            other => panic!("Expected UndefinedFunction, got {:?}", other),
        }
    }

    #[test]
    fn test_ret_without_call() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Ret),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec![],
            funcs: vec![FuncMeta {
                name: "__main__".to_string(),
                params: vec![],
                start: 0,
            }],
        };
        let mut vm = VM::new(bc);
        // Ret without call should just push the value and continue (no call_stack to pop)
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_arithmetic_mul() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Mul),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![6.0, 7.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["42"]);
    }

    #[test]
    fn test_arithmetic_sub() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Sub),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![10.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["7"]);
    }

    #[test]
    fn test_arithmetic_div() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Div),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![10.0, 2.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["5"]);
    }

    #[test]
    fn test_comparison_eq() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Eq),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_comparison_neq() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Neq),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_logical_and() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::Simple(Opcode::And),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_logical_or() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, false),
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::Simple(Opcode::Or),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_add_str_num_concatenates() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["x".to_string()],
            ints: vec![],
            nums: vec![1.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["x1"]);
    }

    #[test]
    fn test_type_error_on_sub_str() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::Simple(Opcode::Sub),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["x".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_type_error_on_mul_str() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::Simple(Opcode::Mul),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["x".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_ge_comparison() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Ge),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_le_comparison() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Le),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 5.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_gt_comparison() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Gt),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_string_add_in_vm() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushStr, 1),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["hola ".to_string(), "mundo".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["hola mundo"]);
    }

    #[test]
    fn test_sub_negative_result() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Sub),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 7.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["-4"]);
    }
}
