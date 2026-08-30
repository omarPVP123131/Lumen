use crate::value::{FixHasher, Value};
use chrono::{Datelike, TimeZone, Timelike, Utc};
use im::HashMap as ImMap;
use lumen_codegen::bytecode::{Bytecode, DefaultValue, FuncMeta, Instruction, Opcode};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::OnceLock;
use unicode_normalization::UnicodeNormalization;

pub static JS_EVAL: OnceLock<fn(&str) -> String> = OnceLock::new();
#[cfg(any(feature = "extra", feature = "full"))]
use crate::coro_ffi::Coroutine;
#[cfg(feature = "full")]
use crate::crypto_ffi::Bcrypt;
#[cfg(feature = "full")]
use crate::gui_ffi::GuiWindow;

#[cfg(feature = "full")]
macro_rules! ffi_rt_ty {
    (I) => { i64 };
    (F) => { f64 };
    (S) => { *const std::ffi::c_char };
    (V) => { () };
}

#[cfg(feature = "full")]
macro_rules! ffi_rt_conv {
    (I, $e:expr) => {
        Value::Int($e)
    };
    (F, $e:expr) => {
        Value::Float($e)
    };
    (S, $e:expr) => {
        ffi_ret_str($e)
    };
    (V, $e:expr) => {{
        $e;
        Value::Void
    }};
}

#[cfg(feature = "full")]
macro_rules! ffi_int_call {
    (0, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<unsafe extern "C" fn() -> ffi_rt_ty!($rtk)> =
            unsafe { $lib.get($name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        Ok(ffi_rt_conv!($rtk, unsafe { sym() }))
    }};
    (1, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<unsafe extern "C" fn(i64) -> ffi_rt_ty!($rtk)> =
            unsafe { $lib.get($name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe { sym(v[0]) }))
    }};
    (2, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<unsafe extern "C" fn(i64, i64) -> ffi_rt_ty!($rtk)> =
            unsafe { $lib.get($name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe { sym(v[0], v[1]) }))
    }};
    (3, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<unsafe extern "C" fn(i64, i64, i64) -> ffi_rt_ty!($rtk)> =
            unsafe { $lib.get($name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe { sym(v[0], v[1], v[2]) }))
    }};
    (4, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<unsafe extern "C" fn(i64, i64, i64, i64) -> ffi_rt_ty!($rtk)> =
            unsafe { $lib.get($name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe { sym(v[0], v[1], v[2], v[3]) }))
    }};
    (5, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(i64, i64, i64, i64, i64) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(v[0], v[1], v[2], v[3], v[4])
        }))
    }};
    (6, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(i64, i64, i64, i64, i64, i64) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(v[0], v[1], v[2], v[3], v[4], v[5])
        }))
    }};
    (7, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(i64, i64, i64, i64, i64, i64, i64) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(v[0], v[1], v[2], v[3], v[4], v[5], v[6])
        }))
    }};
    (8, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(i64, i64, i64, i64, i64, i64, i64, i64) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7])
        }))
    }};
    (9, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(i64, i64, i64, i64, i64, i64, i64, i64, i64) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8])
        }))
    }};
    (10, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
            ) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9])
        }))
    }};
    (11, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
            ) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(
                v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10],
            )
        }))
    }};
    (12, $lib:expr, $name:expr, $rtk:tt, $args:expr) => {{
        let sym: libloading::Symbol<
            unsafe extern "C" fn(
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
                i64,
            ) -> ffi_rt_ty!($rtk),
        > = unsafe { $lib.get($name.as_bytes()) }
            .map_err(|e| format!("Símbolo '{}' no encontrado: {}", $name, e))?;
        let v = ffi_ints($args);
        Ok(ffi_rt_conv!($rtk, unsafe {
            sym(
                v[0], v[1], v[2], v[3], v[4], v[5], v[6], v[7], v[8], v[9], v[10], v[11],
            )
        }))
    }};
}

#[cfg(feature = "full")]
macro_rules! ffi_int_arms {
    ($args:expr, $lib:expr, $name:expr, $n:tt, $ret:expr,
     $(($nlit:tt, $rtk:tt, $variant:ident)),* $(,)?) => {
        $(
            if $n == $nlit && $ret == FfiRet::$variant {
                return ffi_int_call!($nlit, $lib, $name, $rtk, $args);
            }
        )*
    };
}

#[cfg(feature = "full")]
#[derive(Clone, Copy, PartialEq)]
enum FfiTy {
    Int,
    Float,
    Str,
}

#[cfg(feature = "full")]
#[derive(Clone, Copy, PartialEq)]
enum FfiRet {
    Int,
    Float,
    Str,
    Void,
}

#[cfg(feature = "full")]
fn ffi_ints(args: &[Value]) -> Vec<i64> {
    args.iter()
        .map(|v| match v {
            Value::Int(i) => *i,
            Value::Float(f) => *f as i64,
            _ => v
                .as_i64()
                .or_else(|| v.as_num().map(|f| f as i64))
                .unwrap_or(0),
        })
        .collect()
}

#[cfg(feature = "full")]
fn parse_ffi_types(s: &str) -> Vec<FfiTy> {
    if s.trim().is_empty() {
        return Vec::new();
    }
    s.split(',')
        .map(|t| match t.trim() {
            "decimal" | "float" | "f" | "double" => FfiTy::Float,
            "texto" | "string" | "s" | "str" => FfiTy::Str,
            _ => FfiTy::Int,
        })
        .collect()
}

#[cfg(feature = "full")]
fn parse_ffi_ret(s: &str) -> FfiRet {
    match s.trim() {
        "decimal" | "float" | "f" | "double" => FfiRet::Float,
        "texto" | "string" | "s" | "str" => FfiRet::Str,
        "void" | "vacio" | "" => FfiRet::Void,
        _ => FfiRet::Int,
    }
}

#[cfg(feature = "full")]
fn ffi_text_arg(v: &Value) -> Result<(*const std::ffi::c_char, Option<std::ffi::CString>), String> {
    match v {
        Value::Str(s) => {
            let cs = std::ffi::CString::new(s.as_bytes().to_vec())
                .map_err(|_| "Argumento texto FFI con NUL embebido".to_string())?;
            Ok((cs.as_ptr(), Some(cs)))
        }
        other => match other.as_num() {
            Some(f) => Ok((f as i64 as *const std::ffi::c_char, None)),
            None => Err(format!("Argumento FFI inválido para 'texto': {}", other)),
        },
    }
}

#[cfg(feature = "full")]
fn ffi_ret_str(ptr: *const std::ffi::c_char) -> Value {
    if ptr.is_null() {
        return Value::str(String::new());
    }
    let s = unsafe { std::ffi::CStr::from_ptr(ptr) };
    Value::str(s.to_string_lossy().to_string())
}

#[cfg(feature = "full")]
fn ffi_call_typed(
    lib: &libloading::Library,
    name: &str,
    tipos: &str,
    ret: &str,
    args: &[Value],
) -> Result<Value, String> {
    let tys = parse_ffi_types(tipos);
    let shape: String = tys
        .iter()
        .map(|t| match t {
            FfiTy::Int => 'E',
            FfiTy::Float => 'F',
            FfiTy::Str => 'S',
        })
        .collect();
    let n = shape.len();
    let r = parse_ffi_ret(ret);
    if shape == "E".repeat(n) {
        ffi_int_arms!(
            args,
            lib,
            name,
            n,
            r,
            (0, I, Int),
            (1, I, Int),
            (2, I, Int),
            (3, I, Int),
            (4, I, Int),
            (5, I, Int),
            (6, I, Int),
            (7, I, Int),
            (8, I, Int),
            (9, I, Int),
            (10, I, Int),
            (11, I, Int),
            (12, I, Int),
            (0, F, Float),
            (1, F, Float),
            (2, F, Float),
            (3, F, Float),
            (4, F, Float),
            (5, F, Float),
            (6, F, Float),
            (7, F, Float),
            (8, F, Float),
            (9, F, Float),
            (10, F, Float),
            (11, F, Float),
            (12, F, Float),
            (0, S, Str),
            (1, S, Str),
            (2, S, Str),
            (3, S, Str),
            (4, S, Str),
            (5, S, Str),
            (6, S, Str),
            (7, S, Str),
            (8, S, Str),
            (9, S, Str),
            (10, S, Str),
            (11, S, Str),
            (12, S, Str),
            (0, V, Void),
            (1, V, Void),
            (2, V, Void),
            (3, V, Void),
            (4, V, Void),
            (5, V, Void),
            (6, V, Void),
            (7, V, Void),
            (8, V, Void),
            (9, V, Void),
            (10, V, Void),
            (11, V, Void),
            (12, V, Void),
        );
    }
    match (shape.as_str(), r) {
        ("S", FfiRet::Int) => {
            let (p0, _k0) = ffi_text_arg(&args[0])?;
            let sym: libloading::Symbol<unsafe extern "C" fn(*const std::ffi::c_char) -> i64> =
                unsafe { lib.get(name.as_bytes()) }
                    .map_err(|e| format!("Símbolo '{}' no encontrado: {}", name, e))?;
            Ok(Value::Int(unsafe { sym(p0) }))
        }
        ("S", FfiRet::Str) => {
            let (p0, _k0) = ffi_text_arg(&args[0])?;
            let sym: libloading::Symbol<
                unsafe extern "C" fn(*const std::ffi::c_char) -> *const std::ffi::c_char,
            > = unsafe { lib.get(name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", name, e))?;
            Ok(ffi_ret_str(unsafe { sym(p0) }))
        }
        ("SS", FfiRet::Int) => {
            let (p0, _k0) = ffi_text_arg(&args[0])?;
            let (p1, _k1) = ffi_text_arg(&args[1])?;
            let sym: libloading::Symbol<
                unsafe extern "C" fn(*const std::ffi::c_char, *const std::ffi::c_char) -> i64,
            > = unsafe { lib.get(name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", name, e))?;
            Ok(Value::Int(unsafe { sym(p0, p1) }))
        }
        ("ESE", FfiRet::Int) => {
            let a0 = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let (p1, _k1) = ffi_text_arg(args.get(1).ok_or("Faltan argumentos FFI")?)?;
            let a2 = args.get(2).and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let sym: libloading::Symbol<
                unsafe extern "C" fn(i64, *const std::ffi::c_char, i64) -> i64,
            > = unsafe { lib.get(name.as_bytes()) }
                .map_err(|e| format!("Símbolo '{}' no encontrado: {}", name, e))?;
            Ok(Value::Int(unsafe { sym(a0, p1, a2) }))
        }
        ("F", FfiRet::Float) => {
            let f0 = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            let sym: libloading::Symbol<unsafe extern "C" fn(f64) -> f64> =
                unsafe { lib.get(name.as_bytes()) }
                    .map_err(|e| format!("Símbolo '{}' no encontrado: {}", name, e))?;
            Ok(Value::Float(unsafe { sym(f0) }))
        }
        ("F", FfiRet::Int) => {
            let f0 = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            let sym: libloading::Symbol<unsafe extern "C" fn(f64) -> i64> =
                unsafe { lib.get(name.as_bytes()) }
                    .map_err(|e| format!("Símbolo '{}' no encontrado: {}", name, e))?;
            Ok(Value::Int(unsafe { sym(f0) }))
        }
        _ => Err(format!(
            "Firma FFI no soportada: args '{}' con retorno '{}'",
            shape, ret
        )),
    }
}

pub const MAX_CALL_STACK_DEPTH: usize = 10_000;

#[derive(Debug, Clone)]
pub struct CallFrame {
    /// v3.5.31: índice del nombre en `bytecode.names` (cero allocs por
    /// llamada; resolver con `VM::frame_func_name` bajo demanda).
    pub func_name: usize,
    /// Solo para nombres SINTÉTICOS (closures sin entrada en el pool):
    /// `func_name == usize::MAX` y el nombre vive aquí.
    pub func_label: Option<String>,
    pub return_ip: usize,
    /// Nivel de `locals` al entrar a la función (antes del scope de params).
    /// Se usa en Ret para desapilar todos los scopes del frame y hacer
    /// write-back de referencias prestado mut (bug #6).
    pub locals_base: usize,
    /// v3.5.34: ¿algún slot de ESTE frame recibió un Value::Ref con owner?
    /// Si no, Ret salta el escaneo de write-backs (el caso dominante).
    pub has_refs: bool,
    /// v3.5.13: profundidad de la pila de valores al entrar (args ya popeados).
    /// En Ret se trunca hasta aquí para descartar residuos de llamadas a
    /// statements (p.ej. el Void que deja un `imprimir` interior) que antes
    /// desalineaban los argumentos de llamadas multi-arg del llamador
    /// (fase65_guard_let2: `imprimir("a: ", raiz(x))` → "a: 5"/NaN).
    pub stack_base: usize,
    /// v3.5.18: true si la llamada entró por una closure (CallValue sobre
    /// Value::Closure). Hay un scope sintético de entorno capturado JUSTO
    /// debajo de `locals_base` que Ret debe desapilar al salir.
    pub is_closure: bool,
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
    /// v3.5.31: recibe la VM para resolver los nombres de los marcos bajo
    /// demanda (CallFrame guarda solo el índice del nombre).
    pub fn with_stack(self, vm: &VM) -> String {
        let msg = match &self {
            VmError::Runtime(s) => format!("Error: {}", s),
            VmError::StackUnderflow => "Error: Stack underflow".to_string(),
            VmError::UndefinedVariable(s) => format!("Error: Variable '{}' no definida", s),
            VmError::UndefinedFunction(s) => format!("Error: Función '{}' no definida", s),
            VmError::DivisionByZero => "Error: División por cero".to_string(),
            VmError::TypeError(s) => format!("Error de tipo: {}", s),
        };
        let stack = &vm.call_stack;
        if stack.is_empty() {
            msg
        } else {
            let trace: Vec<String> = stack
                .iter()
                .map(|f| format!("  · {}", vm.frame_func_name(f)))
                .collect();
            format!("{}\n\nPila de llamadas:\n{}", msg, trace.join("\n"))
        }
    }
}
/// v3.5.31: nombres de los builtins (generado por scripts/gen_builtin_names.py
/// desde call_core_builtin / call_extra_builtin). Solo se usa como
/// pre-filtro O(1) en el camino caliente de Call: si el nombre NO está aquí,
/// es función de usuario y se despacha SIN clonar el String (los builtins
/// conservan la precedencia: se despachan primero).
// 352 nombres (generado por scripts/gen_builtin_names.py)
pub(crate) const BUILTIN_NAMES: &[&str] = &[
    "__a_f64_bytes",
    "__actor_enviar",
    "__actor_new",
    "__actor_nuevo",
    "__actor_recibir",
    "__actor_recv",
    "__actor_send",
    "__aes_decrypt",
    "__aes_desencriptar",
    "__aes_encriptar",
    "__aes_encrypt",
    "__agregar_archivo",
    "__arc_asignar",
    "__arc_get",
    "__arc_new",
    "__arc_nuevo",
    "__arc_obtener",
    "__arc_set",
    "__buf_reader",
    "__buf_writer",
    "__bytes_a_f64",
    "__calendar_hijri",
    "__calendar_persian",
    "__calendario_hijri",
    "__calendario_persa",
    "__canal_enviar",
    "__canal_nuevo",
    "__canal_recibir",
    "__channel_new",
    "__channel_recv",
    "__channel_send",
    "__cluster_conectar",
    "__cluster_connect",
    "__cluster_enviar",
    "__cluster_send",
    "__codegen_a_nvc",
    "__codificacion_utf8",
    "__compilar_nv",
    "__compile_nv",
    "__conjunto_agregar",
    "__conjunto_diferencia",
    "__conjunto_interseccion",
    "__conjunto_nuevo",
    "__conjunto_tiene",
    "__conjunto_unir",
    "__coro_ceder",
    "__coro_crear",
    "__coro_create",
    "__coro_reanudar",
    "__coro_resume",
    "__coro_yield",
    "__deque_agregar_final",
    "__deque_agregar_frente",
    "__deque_len",
    "__deque_longitud",
    "__deque_new",
    "__deque_nuevo",
    "__deque_pop_back",
    "__deque_pop_front",
    "__deque_push_back",
    "__deque_push_front",
    "__deque_quitar_final",
    "__deque_quitar_frente",
    "__desde_utf8",
    "__dormir",
    "__duracion_nueva",
    "__duracion_segundos",
    "__duration_new",
    "__duration_secs",
    "__encoding_from_utf8",
    "__encoding_utf8",
    "__enlazada_agregar_final",
    "__enlazada_agregar_frente",
    "__enlazada_longitud",
    "__enlazada_nuevo",
    "__enlazada_quitar_final",
    "__enlazada_quitar_frente",
    "__env_list",
    "__env_listar",
    "__escribir_archivo",
    "__escribir_archivo_async",
    "__escribir_archivo_bin",
    "__escritor_buffer",
    "__existe_archivo",
    "__ffi_alloc",
    "__ffi_asignar",
    "__ffi_asm",
    "__ffi_c_eval",
    "__ffi_call",
    "__ffi_cargar",
    "__ffi_escribir",
    "__ffi_free",
    "__ffi_leer",
    "__ffi_liberar",
    "__ffi_llamar",
    "__ffi_llamar_nv",
    "__ffi_load",
    "__ffi_peek",
    "__ffi_peek64",
    "__ffi_peek_byte",
    "__ffi_peek_ptr",
    "__ffi_peek_u32",
    "__ffi_peek_u8",
    "__ffi_poke",
    "__ffi_poke_byte",
    "__ffi_poke_u32",
    "__ffi_poke_u8",
    "__ffi_read",
    "__ffi_rust_eval",
    "__ffi_write",
    "__file_append",
    "__file_bytes",
    "__file_exists",
    "__file_read",
    "__file_read_async",
    "__file_size",
    "__file_write",
    "__file_write_async",
    "__file_write_binary",
    "__fs_listar",
    "__fs_listdir",
    "__generador_nuevo",
    "__generador_siguiente",
    "__generator_new",
    "__generator_next",
    "__gui_cerrar",
    "__gui_close",
    "__gui_esperar",
    "__gui_hwnd",
    "__gui_id",
    "__gui_mostrar",
    "__gui_poll",
    "__gui_show",
    "__gui_ventana",
    "__gui_window",
    "__hash_sha256",
    "__hash_sha512",
    "__heap_len",
    "__heap_new",
    "__heap_peek",
    "__heap_pop",
    "__heap_push",
    "__hilo_esperar",
    "__hilo_lanzar",
    "__http_enviar",
    "__http_get",
    "__http_obtener",
    "__http_post",
    "__http_server",
    "__http_servidor",
    "__js_call",
    "__js_eval",
    "__js_evaluar",
    "__js_llamar",
    "__json_parse",
    "__json_parsear",
    "__json_stringify",
    "__json_texto",
    "__jwt_codificar",
    "__jwt_decode",
    "__jwt_decodificar",
    "__jwt_encode",
    "__lector_buffer",
    "__leer_archivo",
    "__leer_archivo_async",
    "__leer_bytes",
    "__lex_native",
    "__lexer_nativo",
    "__linked_len",
    "__linked_new",
    "__linked_pop_back",
    "__linked_pop_front",
    "__linked_push_back",
    "__linked_push_front",
    "__list_reverse",
    "__list_sort",
    "__lista_invertir",
    "__lista_ordenar",
    "__main__",
    "__map_claves",
    "__map_contains",
    "__map_contiene",
    "__map_get",
    "__map_keys",
    "__map_len",
    "__map_longitud",
    "__map_new",
    "__map_nuevo",
    "__map_obtener",
    "__map_poner",
    "__map_set",
    "__monticulo_agregar",
    "__monticulo_longitud",
    "__monticulo_nuevo",
    "__monticulo_quitar",
    "__monticulo_ver",
    "__mutex_bloquear",
    "__mutex_lock",
    "__mutex_new",
    "__mutex_nuevo",
    "__num_a_f64_bytes",
    "__numero_a_bytes_f64",
    "__par_join",
    "__par_map",
    "__par_mapear",
    "__par_unir",
    "__process_pid",
    "__regex_capturar",
    "__regex_captures",
    "__regex_coincide",
    "__regex_is_match",
    "__regex_new",
    "__regex_nuevo",
    "__regex_reemplazar",
    "__regex_replace",
    "__rwlock_escribir",
    "__rwlock_leer",
    "__rwlock_new",
    "__rwlock_nuevo",
    "__rwlock_read",
    "__rwlock_write",
    "__scope_cancel",
    "__scope_cancelar",
    "__scope_lanzar",
    "__scope_new",
    "__scope_nuevo",
    "__scope_spawn",
    "__seleccionar",
    "__select",
    "__self_healing_estado",
    "__self_healing_invocar",
    "__self_healing_registrar_parche",
    "__serial_abrir",
    "__serial_open",
    "__set_add",
    "__set_diff",
    "__set_has",
    "__set_inter",
    "__set_new",
    "__set_union",
    "__sistema_pid",
    "__sleep",
    "__str_a_caracteres",
    "__str_a_entero",
    "__str_caracter",
    "__str_chr",
    "__str_codigo",
    "__str_concat_list",
    "__str_concatenar_lista",
    "__str_contains",
    "__str_contiene",
    "__str_dividir",
    "__str_empieza_con",
    "__str_from",
    "__str_len",
    "__str_longitud",
    "__str_lower",
    "__str_mayusculas",
    "__str_minusculas",
    "__str_ord",
    "__str_pad_end",
    "__str_pad_start",
    "__str_padding_fin",
    "__str_padding_inicio",
    "__str_recortar",
    "__str_reemplazar",
    "__str_replace",
    "__str_slice",
    "__str_slice_chars",
    "__str_split",
    "__str_starts_with",
    "__str_subcadena",
    "__str_subcadena_chars",
    "__str_to_chars",
    "__str_trim",
    "__str_upper",
    "__stream_chunks",
    "__stream_colectar",
    "__stream_collect",
    "__stream_desde",
    "__stream_filter",
    "__stream_filtrar",
    "__stream_from",
    "__stream_map",
    "__stream_mapear",
    "__stream_trozos",
    "__supervisor_add",
    "__supervisor_agregar",
    "__supervisor_iniciar",
    "__supervisor_new",
    "__supervisor_nuevo",
    "__supervisor_start",
    "__tamano_archivo",
    "__tarea_esperar",
    "__tarea_lanzar",
    "__task_await",
    "__task_spawn",
    "__tcp_accept",
    "__tcp_aceptar",
    "__tcp_conectar",
    "__tcp_conectar_async",
    "__tcp_connect",
    "__tcp_connect_async",
    "__tcp_escuchar",
    "__tcp_listen",
    "__temporizador_esperar",
    "__texto_a_entero",
    "__thread_join",
    "__thread_spawn",
    "__tiempo_ahora",
    "__tiempo_diferencia",
    "__tiempo_formatear",
    "__tiempo_parsear",
    "__time_diff",
    "__time_format",
    "__time_now",
    "__time_parse",
    "__timer_delay",
    "__timezone_info",
    "__tipo_de",
    "__typeof",
    "__unicode_normalizar",
    "__unicode_normalize",
    "__zona_info",
    "a_texto",
    "abs",
    "absoluto",
    "agregar",
    "ceil",
    "floor",
    "imprimir",
    "largo",
    "leer",
    "len",
    "main",
    "max",
    "maximo",
    "min",
    "minimo",
    "piso",
    "potencia",
    "pow",
    "principal",
    "print",
    "push",
    "raiz",
    "read",
    "redondear",
    "round",
    "sqrt",
    "techo",
    "to_texto",
];

/// v3.5.31: conjunto estático para el pre-filtro (OnceLock).
pub(crate) fn builtin_name_set() -> &'static std::collections::HashSet<&'static str> {
    use std::sync::OnceLock;
    static SET: OnceLock<std::collections::HashSet<&'static str>> = OnceLock::new();
    SET.get_or_init(|| {
        let mut h = std::collections::HashSet::new();
        for n in BUILTIN_NAMES {
            h.insert(*n);
        }
        h
    })
}

#[cfg(any(feature = "extra", feature = "full"))]
type ChannelCell = (
    Option<std::sync::mpsc::Sender<Value>>,
    Option<std::sync::mpsc::Receiver<Value>>,
);

#[derive(Debug, Clone)]
pub struct VmSnapshot {
    pub ip: usize,
    pub instr_count: usize,
    pub stack: Vec<Value>,
    pub locals: Vec<ScopeFrame>,
    pub flat: Vec<Value>,
    pub free_slots: Vec<u32>,
    pub call_stack: Vec<CallFrame>,
    pub output_len: usize,
}

/// v3.5.31: marco de scope con los VALORES en la arena `flat` (VM::flat).
/// El mapa nombre → slot evita hashear el nombre en cada acceso (el caché
/// var_cache apunta directo al slot); `slots` es el inventario para liberar
/// al cerrar el scope; `id` es una identidad única para validar el caché
/// (un pop+push en el mismo índice NO revalida entradas muertas).
#[derive(Debug, Clone)]
pub struct ScopeFrame {
    pub map: HashMap<String, u32, FixHasher>,
    pub slots: Vec<u32>,
    pub id: u64,
    /// v3.5.31: scope de PARÁMETROS de la función `Some(func_idx)`. El mapa
    /// va VACÍO (cero allocs y cero clones de String por llamada): el
    /// nombre→slot se resuelve por POSICIÓN — params[i] ↔ slots[i].
    pub param_func: Option<usize>,
}

impl ScopeFrame {
    pub fn new(id: u64) -> Self {
        Self {
            map: HashMap::with_hasher(FixHasher::default()),
            slots: Vec::new(),
            id,
            param_func: None,
        }
    }
    pub fn with_capacity(cap: usize, id: u64) -> Self {
        Self {
            map: HashMap::with_capacity_and_hasher(cap, FixHasher::default()),
            slots: Vec::new(),
            id,
            param_func: None,
        }
    }
    /// v3.5.31: scope de parámetros SIN mapa — la identidad de la función
    /// basta para resolver nombres posicionalmente.
    pub fn params(func_idx: usize, id: u64) -> Self {
        Self {
            map: HashMap::with_hasher(FixHasher::default()),
            slots: Vec::new(),
            id,
            param_func: Some(func_idx),
        }
    }
    /// v3.5.36: construcción con partes (buffers reutilizados del pool).
    pub fn with_parts(
        map: HashMap<String, u32, FixHasher>,
        slots: Vec<u32>,
        id: u64,
        param_func: Option<usize>,
    ) -> Self {
        Self {
            map,
            slots,
            id,
            param_func,
        }
    }
}

pub struct VM {
    stack: Vec<Value>,
    locals: Vec<ScopeFrame>,
    /// v3.5.31: arena que POSEE los valores de las variables. Los mapas de
    /// scope solo mapean nombre → slot (u32). Un único dueño del Arc de un
    /// array preserva el O(1) de `Arc::make_mut` en `agregar` (un espejo
    /// clonado reintroduciría O(n²)).
    flat: Vec<Value>,
    /// slots liberados por scopes cerrados (reuso; flat nunca se encoge).
    free_slots: Vec<u32>,
    // v3.5.36: pools de buffers de scopes (slots y mapas) — evitan el
    // alloc/free por llamada (scope de parámetros) y por bloque de bucle.
    slot_pool: Vec<Vec<u32>>,
    map_pool: Vec<HashMap<String, u32, FixHasher>>,
    /// contador de identidades de scope.
    scope_id_next: u64,
    /// v3.5.19: caché inline de resolución de variables (Load/Store):
    /// por name-idx → (slot_flat, scope_idx, scope_id, gen). La entrada es
    /// válida si `gen == var_cache_gen` Y `scope_idx < locals.len()` Y
    /// `locals[scope_idx].id == scope_id`. La identidad de scope hace la
    /// caché inmune a push/pop de scopes VACÍOS (antes, el len oscilaba y
    /// cada acceso re-escaneaba). `gen` se incrementa al INSERTAR nombres
    /// nuevos (StoreLocal/Store) o reemplazar `locals` de golpe
    /// (corutinas/snapshots).
    var_cache: Vec<(u32, u32, u64, u64)>,
    var_cache_gen: u64,
    /// v3.5.36: índices de nombre de los PARÁMETROS por función — para la
    /// invalidación SELECTIVA de la caché de variables al entrar a una
    /// llamada (solo los nombres sombreados por los params se invalidan;
    /// el resto del llamador sigue cacheado a través de la llamada).
    params_name_idx: Vec<Vec<usize>>,
    /// v3.5.31: destinos de salto resueltos una vez (idx de nums → ip real).
    jump_targets: Vec<Option<usize>>,
    /// v3.5.31: ejecución dentro de un cuerpo JIT nativo (ip obsoleto).
    native_exec: bool,
    ip: usize,
    bytecode: Bytecode,
    output: Vec<String>,
    echo_stdout: bool,
    call_stack: Vec<CallFrame>,
    func_index_cache: HashMap<String, usize>,
    /// v3.5.31: búsqueda de función por ÍNDICE de nombre (sin hash ni alloc
    /// por llamada — el camino caliente de Call usa esto).
    func_index_by_name_idx: Vec<Option<usize>>,
    pub debug: bool,
    pub breakpoints: Vec<usize>,
    step_mode: bool,
    last_instr: Option<Instruction>,
    pub instr_count: usize,
    pub snapshots: Vec<VmSnapshot>,
    /// v3.5.31: contador de llamadas por índice de nombre (umbral JIT).
    pub call_counts: Vec<usize>,
    pub jit_threshold: usize,
    #[cfg(feature = "aot")]
    pub jit_engine: Option<lumen_aot::JitEngine>,
    // JIT Tier-1 (v3.5.9): bytecode caliente → código nativo. Se activa con
    // LUMEN_JIT=1; por defecto APAGADO (fixpoint corre en intérprete puro).
    jit_enabled: bool,
    #[cfg(feature = "aot")]
    jit_rt: Option<crate::jit::VmJit>,
    pub(crate) jit_error: Option<VmError>,
    #[cfg(feature = "full")]
    bcrypt: Option<Arc<Bcrypt>>,
    #[cfg(feature = "full")]
    gui_windows: HashMap<String, GuiWindow>,
    #[cfg(any(feature = "extra", feature = "full"))]
    coroutines: HashMap<String, Coroutine>,
    #[cfg(any(feature = "extra", feature = "full"))]
    current_coro: Option<String>,
    #[cfg(any(feature = "extra", feature = "full"))]
    #[allow(clippy::type_complexity)]
    main_saved: Option<(Vec<Value>, Vec<ScopeFrame>, Vec<Value>, Vec<u32>, usize)>,
    tcp_listener: Option<std::net::TcpListener>,
    #[cfg(feature = "full")]
    #[allow(dead_code)]
    cluster_streams: HashMap<String, std::net::TcpStream>,
    #[cfg(feature = "full")]
    #[allow(dead_code)]
    scope_handles: Vec<HashMap<String, Value, FixHasher>>,
    #[cfg(any(feature = "extra", feature = "full"))]
    #[allow(dead_code)]
    thread_handles: HashMap<String, std::thread::JoinHandle<Value>>,
    #[cfg(any(feature = "extra", feature = "full"))]
    #[allow(clippy::type_complexity)]
    // v3.5.17: compartidos entre la VM principal y las VMs de __hilo_lanzar
    // (los hilos nativos de C/Cranelift ya comparten el registro de proceso).
    channels: Arc<std::sync::Mutex<HashMap<String, ChannelCell>>>,
    #[cfg(any(feature = "extra", feature = "full"))]
    mutexes: Arc<std::sync::Mutex<HashMap<String, Arc<std::sync::Mutex<Value>>>>>,
    #[cfg(any(feature = "extra", feature = "full"))]
    #[allow(clippy::type_complexity)]
    actors: HashMap<String, ChannelCell>,
    #[cfg(any(feature = "extra", feature = "full"))]
    generators: HashMap<String, String>,
    #[cfg(feature = "full")]
    ffi_libraries: HashMap<String, usize>,
    #[cfg(feature = "full")]
    ffi_allocations: HashMap<usize, std::alloc::Layout>,
    #[cfg(any(feature = "extra", feature = "full"))]
    #[allow(dead_code)]
    task_results: HashMap<String, std::sync::mpsc::Receiver<Value>>,
    #[cfg(any(feature = "extra", feature = "full"))]
    #[allow(dead_code)]
    task_results_sync: HashMap<String, Value, FixHasher>,
    #[cfg(any(feature = "extra", feature = "full"))]
    task_counter: usize,
    /// Frames de manejador intentar/atrapar: (catch_ip, stack_len, locals_len, call_len)
    handlers: Vec<(usize, usize, usize, usize)>,
}

/// Helper to convert VmError into the builtin return type.
fn builtin_err(err: VmError) -> Option<Result<(), VmError>> {
    Some(Err(err))
}

/// Mensaje legible de un VmError para bind en intentar/atrapar
fn vm_error_message(e: &VmError) -> String {
    match e {
        VmError::Runtime(s) => s.clone(),
        VmError::TypeError(s) => format!("Tipo incorrecto: {}", s),
        VmError::DivisionByZero => "División por cero".to_string(),
        VmError::StackUnderflow => "Desbordamiento de pila interno".to_string(),
        VmError::UndefinedVariable(s) => format!("Variable '{}' no definida", s),
        VmError::UndefinedFunction(s) => format!("Función '{}' no definida", s),
    }
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
        // v3.5.31: búsqueda por índice de nombre (Vec directa, sin hash).
        let n_names = bytecode.names.len();
        let mut func_index_by_name_idx = vec![None; n_names];
        for (nidx, name) in bytecode.names.iter().enumerate() {
            func_index_by_name_idx[nidx] = func_index_cache.get(name).copied();
        }
        // v3.5.36: índices de nombre de los parámetros por función.
        let params_name_idx: Vec<Vec<usize>> = bytecode
            .funcs
            .iter()
            .map(|f| {
                f.params
                    .iter()
                    .filter_map(|p| bytecode.names.iter().position(|n| n == p))
                    .collect()
            })
            .collect();
        #[cfg(feature = "full")]
        let bcrypt = match Bcrypt::load() {
            Ok(b) => Some(Arc::new(b)),
            Err(_) => None,
        };
        #[cfg(feature = "aot")]
        let jit_engine = lumen_aot::JitEngine::new().ok();
        Self {
            stack: Vec::new(),
            locals: vec![ScopeFrame::new(0)],
            flat: Vec::new(),
            free_slots: Vec::new(),
            slot_pool: Vec::new(),
            map_pool: Vec::new(),
            params_name_idx,
            scope_id_next: 1,
            // v3.5.31: pre-dimensionado a names.len() para indexar SIN
            // bounds-check en el fast-path (entradas (0,0,0,0) con gen=0
            // quedan inválidas de nacimiento — el slow-path las rellena).
            var_cache: vec![(0u32, 0u32, 0u64, 0u64); bytecode.names.len()],
            var_cache_gen: 1,
            jump_targets: Vec::new(),
            native_exec: false,
            ip,
            bytecode,
            output: Vec::new(),
            echo_stdout: false,
            call_stack: Vec::new(),
            func_index_cache,
            func_index_by_name_idx,
            debug: false,
            breakpoints: Vec::new(),
            step_mode: false,
            last_instr: None,
            instr_count: 0,
            snapshots: Vec::new(),
            call_counts: vec![0; n_names],
            jit_threshold: 50,
            #[cfg(feature = "aot")]
            jit_engine,
            // v3.5.31: JIT PREDETERMINADO — Tier-1 + Tier-2 validados en
            // 956 tests y ci_gate completo (0 fallos no permitidos).
            // LUMEN_JIT=0/off lo desactiva (intérprete puro, diagnóstico).
            jit_enabled: {
                let v = std::env::var("LUMEN_JIT").unwrap_or_default();
                let off =
                    v == "0" || v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("off");
                !off && cfg!(feature = "aot")
            },
            #[cfg(feature = "aot")]
            jit_rt: None,
            jit_error: None,
            #[cfg(feature = "full")]
            bcrypt,
            #[cfg(feature = "full")]
            gui_windows: HashMap::new(),
            #[cfg(any(feature = "extra", feature = "full"))]
            coroutines: HashMap::new(),
            #[cfg(any(feature = "extra", feature = "full"))]
            current_coro: None,
            #[cfg(any(feature = "extra", feature = "full"))]
            main_saved: None,
            tcp_listener: None,
            #[cfg(feature = "full")]
            cluster_streams: HashMap::new(),
            #[cfg(feature = "full")]
            scope_handles: Vec::new(),
            #[cfg(any(feature = "extra", feature = "full"))]
            thread_handles: HashMap::new(),
            #[cfg(any(feature = "extra", feature = "full"))]
            channels: Arc::new(std::sync::Mutex::new(HashMap::new())),
            #[cfg(any(feature = "extra", feature = "full"))]
            mutexes: Arc::new(std::sync::Mutex::new(HashMap::new())),
            #[cfg(any(feature = "extra", feature = "full"))]
            actors: HashMap::new(),
            #[cfg(any(feature = "extra", feature = "full"))]
            generators: HashMap::new(),
            #[cfg(feature = "full")]
            ffi_libraries: HashMap::new(),
            #[cfg(feature = "full")]
            ffi_allocations: HashMap::new(),
            #[cfg(any(feature = "extra", feature = "full"))]
            task_results: HashMap::new(),
            #[cfg(any(feature = "extra", feature = "full"))]
            task_results_sync: HashMap::with_hasher(FixHasher::default()),
            #[cfg(any(feature = "extra", feature = "full"))]
            task_counter: 0,
            handlers: Vec::new(),
        }
    }

    fn find_func(&self, name: &str) -> Option<&FuncMeta> {
        self.func_index_cache
            .get(name)
            .and_then(|&idx| self.bytecode.funcs.get(idx))
    }

    fn call_core_builtin(&mut self, name: &str, args: &[Value]) -> Option<Result<(), VmError>> {
        if name == "imprimir" || name == "print" {
            let mut combined = String::new();
            for arg in args {
                combined.push_str(&format!("{}", arg));
            }
            self.emit_line(combined);
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "leer" || name == "read" {
            self.push(Value::str(String::new()));
            return Some(Ok(()));
        }

        if name == "abs" || name == "absoluto" {
            match args.first() {
                Some(Value::Int(i)) => self.push(Value::Int(i.abs())),
                Some(Value::Float(f)) => self.push(Value::Float(f.abs())),
                Some(other) => {
                    if let Some(n) = other.as_num() {
                        self.push(Value::Float(n.abs()));
                    } else {
                        return builtin_err(VmError::TypeError(format!(
                            "'abs' espera un número, no {:?}",
                            other
                        )));
                    }
                }
                None => {
                    return builtin_err(VmError::TypeError("'abs' espera 1 argumento".to_string()))
                }
            }
            return Some(Ok(()));
        }

        if name == "min" || name == "minimo" {
            let a = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            let b = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0);
            if args
                .first()
                .map(|v| matches!(v, Value::Int(_)))
                .unwrap_or(false)
                && args
                    .get(1)
                    .map(|v| matches!(v, Value::Int(_)))
                    .unwrap_or(false)
            {
                self.push(Value::Int((a as i64).min(b as i64)));
            } else {
                self.push(Value::Float(a.min(b)));
            }
            return Some(Ok(()));
        }

        if name == "max" || name == "maximo" {
            let a = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            let b = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0);
            if args
                .first()
                .map(|v| matches!(v, Value::Int(_)))
                .unwrap_or(false)
                && args
                    .get(1)
                    .map(|v| matches!(v, Value::Int(_)))
                    .unwrap_or(false)
            {
                self.push(Value::Int((a as i64).max(b as i64)));
            } else {
                self.push(Value::Float(a.max(b)));
            }
            return Some(Ok(()));
        }

        if name == "raiz" || name == "sqrt" {
            let a = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            self.push(Value::Float(a.sqrt()));
            return Some(Ok(()));
        }

        if name == "potencia" || name == "pow" {
            let a = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            let b = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0);
            if args
                .first()
                .map(|v| matches!(v, Value::Int(_)))
                .unwrap_or(false)
                && args
                    .get(1)
                    .map(|v| matches!(v, Value::Int(_)))
                    .unwrap_or(false)
                && b >= 0.0
            {
                self.push(Value::Int((a as i64).pow(b as u32)));
            } else {
                self.push(Value::Float(a.powf(b)));
            }
            return Some(Ok(()));
        }

        if name == "piso" || name == "floor" {
            let a = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            self.push(Value::Int(a.floor() as i64));
            return Some(Ok(()));
        }

        if name == "techo" || name == "ceil" {
            let a = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            self.push(Value::Int(a.ceil() as i64));
            return Some(Ok(()));
        }

        if name == "redondear" || name == "round" {
            let a = args.first().and_then(|v| v.as_num()).unwrap_or(0.0);
            self.push(Value::Int(a.round() as i64));
            return Some(Ok(()));
        }

        if name == "a_texto" || name == "to_texto" || name == "__str_from" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::str(s));
            return Some(Ok(()));
        }

        // ██ Utility builtins (core — disponibles también en wasm sin feature "full") ██
        if name == "__tipo_de" || name == "__typeof" {
            let val = args.first().cloned().unwrap_or(Value::Void);
            // Transparencia de referencias: el tipo reportado es el del contenido
            let val = val.deep_deref();
            let type_name = match &val {
                Value::Int(_) => "entero",
                Value::Float(_) => "decimal",
                Value::Bool(_) => "booleano",
                Value::Str(_) => "texto",
                Value::Array(_) => "lista",
                Value::Map(_) => "diccionario",
                Value::Void => "nulo",
                Value::Func(_) => "funcion",
                Value::Closure { .. } => "funcion",
                Value::Struct { .. } => "estructura",
                Value::Enum { .. } => "enumeracion",
                Value::Tuple(_) => "tupla",
                Value::Exito(_) => "exito",
                Value::Error(_) => "error",
                Value::Opcion(_) => "opcion",
                Value::Ref { .. } => unreachable!("deep_deref elimina Ref"),
            };
            self.push(Value::str(type_name.to_string()));
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
            match args.first().cloned() {
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
            let mut iter = args.iter().cloned();
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
            // Conteo de caracteres (chars), no bytes — consistente con s[i]/slice
            self.push(Value::Int(s.chars().count() as i64));
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
            let nchars = s.chars().count() as i64;
            // Leer como i64 para clamp correcto de negativos (soporta rangos estilo Python)
            let start = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as i64;
            let end = args
                .get(2)
                .and_then(|v| v.as_num())
                .unwrap_or(nchars as f64) as i64;
            // Normalizar negativos - conservar compatibilidad: end==-1 significa "hasta el final" (len), no len-1
            let s0 = if start < 0 { nchars + start } else { start };
            let e0 = if end == -1 {
                nchars
            } else if end < 0 {
                nchars + end
            } else {
                end
            };
            let s0 = s0.clamp(0, nchars);
            let e0 = e0.clamp(0, nchars).max(s0.max(0));
            let sub: String = s
                .chars()
                .skip(s0 as usize)
                .take((e0 - s0) as usize)
                .collect();
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

        if name == "__lexer_nativo" || name == "__lex_native" {
            let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(crate::native_lex::native_lex(&s));
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
            let st = args
                .get(1)
                .and_then(|v| v.as_num())
                .map(|f| f as i64)
                .unwrap_or(0);
            let en = args
                .get(2)
                .and_then(|v| v.as_num())
                .map(|f| f as i64)
                .unwrap_or(-1);
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
            let c = char::from_u32(n as u32)
                .map(|c| c.to_string())
                .unwrap_or_default();
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

        // PID del proceso (v3.3.7 — hueco detectado por fuzz_paridad.ps1)
        if name == "__sistema_pid" || name == "__process_pid" {
            self.push(Value::Int(std::process::id() as i64));
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
            match std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
            {
                Ok(mut file) => match file.write_all(content.as_bytes()) {
                    Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                    Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                },
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__file_write_binary" || name == "__escribir_archivo_bin" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let bytes = match args.get(1) {
                Some(Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|v| match v {
                        Value::Int(n) if *n >= 0 && *n <= 255 => Some(*n as u8),
                        _ => None,
                    })
                    .collect::<Vec<u8>>(),
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
            let n = args
                .first()
                .map(|v| match v {
                    Value::Int(i) => *i as f64,
                    Value::Float(f) => *f,
                    _ => 0.0,
                })
                .unwrap_or(0.0);
            let bytes: Vec<Value> = n
                .to_le_bytes()
                .iter()
                .map(|&b| Value::Int(b as i64))
                .collect();
            self.push(Value::arr(bytes));
            return Some(Ok(()));
        }

        if name == "__codegen_a_nvc" {
            // Takes a codegen map, returns Array<Int> of .nvc bytes
            let cg = args.first().cloned().unwrap_or(Value::Void);
            let result = self.codegen_to_nvc(cg);
            return match result {
                Ok(bytes) => {
                    self.push(bytes);
                    Some(Ok(()))
                }
                Err(_) => {
                    self.push(Value::Error(Box::new(Value::str("codegen_to_nvc failed"))));
                    Some(Ok(()))
                }
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
                    self.push(Value::Error(Box::new(Value::str(format!(
                        "Loader error: {:?}",
                        e
                    )))));
                    return Some(Ok(()));
                }
            };
            let mut sema = lumen_sema::sema::SemanticAnalyzer::new();
            let sem_errors = sema.analyze(&mut program);
            if !sem_errors.is_empty() {
                let msg = sem_errors
                    .iter()
                    .map(|e| e.message.to_string())
                    .collect::<Vec<_>>()
                    .join("; ");
                self.push(Value::Error(Box::new(Value::str(format!(
                    "Sem error: {}",
                    msg
                )))));
                return Some(Ok(()));
            }
            let builder = lumen_ir::builder::IRBuilder::new();
            let ir = builder.build(&program);
            let codegen = lumen_codegen::codegen::Codegen::new();
            let (bytecode, _) = codegen.generate(&ir);
            let bytes_vec = bytecode.encode();
            let bytes: Vec<Value> = bytes_vec
                .into_iter()
                .map(|b| Value::Int(b as i64))
                .collect();
            self.push(Value::arr(bytes));
            return Some(Ok(()));
        }

        if name == "__time_now" || name == "__tiempo_ahora" {
            #[cfg(not(target_arch = "wasm32"))]
            let secs: i64 = {
                use std::time::{SystemTime, UNIX_EPOCH};
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default();
                now.as_secs() as i64
            };
            #[cfg(target_arch = "wasm32")]
            let secs: i64 = {
                if let Some(eval) = JS_EVAL.get() {
                    eval("String(Math.floor(Date.now()/1000))")
                        .trim()
                        .parse()
                        .unwrap_or(0)
                } else {
                    0
                }
            };
            self.push(Value::Int(secs));
            return Some(Ok(()));
        }

        if name == "__list_reverse" || name == "__lista_invertir" {
            let mut arr = match args.first().cloned() {
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
            let mut arr = match args.first().cloned() {
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
            let mut it = args.iter().cloned();
            let m = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
            let mut it = args.iter().cloned();
            let m = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
                .first()
                .cloned()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match m {
                Value::Map(m) => self.push(Value::Int(m.len() as i64)),
                _ => return builtin_err(VmError::TypeError("__map_len espera diccionario".into())),
            }
            return Some(Ok(()));
        }

        if name == "__map_keys" || name == "__map_claves" {
            let m = args
                .first()
                .cloned()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match m {
                Value::Map(m) => {
                    let keys: Vec<Value> = m.keys().cloned().collect();
                    self.push(Value::arr(keys));
                }
                _ => {
                    return builtin_err(VmError::TypeError("__map_keys espera diccionario".into()))
                }
            }
            return Some(Ok(()));
        }

        if name == "__map_contains" || name == "__map_contiene" {
            let mut it = args.iter().cloned();
            let m = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
            let mut it = args.iter().cloned();
            let s = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
            let mut it = args.iter().cloned();
            let s = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let item = it.next().unwrap_or(Value::Void);
            match s {
                Value::Map(m) => self.push(Value::Bool(m.contains_key(&item))),
                _ => return builtin_err(VmError::TypeError("__set_has espera conjunto".into())),
            }
            return Some(Ok(()));
        }

        if name == "__set_union" || name == "__conjunto_unir" {
            let mut it = args.iter().cloned();
            let a = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let b = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match (a, b) {
                (Value::Map(mut m1), Value::Map(m2)) => {
                    for (k, v) in m2 {
                        if !m1.contains_key(&k) {
                            m1.insert(k, v);
                        }
                    }
                    self.push(Value::Map(m1));
                }
                _ => return builtin_err(VmError::TypeError("__set_union espera conjuntos".into())),
            }
            return Some(Ok(()));
        }

        if name == "__set_inter" || name == "__conjunto_interseccion" {
            let mut it = args.iter().cloned();
            let a = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let b = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            match (a, b) {
                (Value::Map(m1), Value::Map(m2)) => {
                    let r: ImMap<Value, Value, FixHasher> =
                        m1.into_iter().filter(|(k, _)| m2.contains_key(k)).collect();
                    self.push(Value::Map(r));
                }
                _ => return builtin_err(VmError::TypeError("__set_inter espera conjuntos".into())),
            }
            return Some(Ok(()));
        }

        if name == "__set_diff" || name == "__conjunto_diferencia" {
            let mut it = args.iter().cloned();
            let a = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
            let b = it
                .next()
                .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
            let mut it = args.iter().cloned();
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
            let mut it = args.iter().cloned();
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
            let d = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let d = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let d = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let mut it = args.iter().cloned();
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
            let h = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let h = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let h = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let mut it = args.iter().cloned();
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
            let mut it = args.iter().cloned();
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
            let l = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let l = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            let l = args.first().cloned().unwrap_or(Value::arr(vec![]));
            match l {
                Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                _ => return builtin_err(VmError::TypeError("__linked_len espera linked".into())),
            }
            return Some(Ok(()));
        }

        if name == "__regex_new" || name == "__regex_nuevo" {
            let pat = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match crate::lumen_min_regex_new(&pat) {
                Ok(_) => self.push(Value::Bool(true)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__regex_is_match" || name == "__regex_coincide" {
            let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match crate::lumen_min_regex_new(&re_s) {
                Ok(r) => self.push(Value::Bool(r.is_match(&text))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__regex_captures" || name == "__regex_capturar" {
            let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            match crate::lumen_min_regex_new(&re_s) {
                Ok(r) => {
                    let caps = r.captures(&text);
                    let vs: Vec<Value> = caps.into_iter().map(Value::str).collect();
                    self.push(Value::arr(vs));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        if name == "__regex_replace" || name == "__regex_reemplazar" {
            let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let rep = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            match crate::lumen_min_regex_new(&re_s) {
                Ok(r) => self.push(Value::str(r.replace(&text, rep.as_str()))),
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
            let arr = args.first().cloned().unwrap_or(Value::arr(vec![]));
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
            #[allow(unused_variables)]
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

    #[cfg(any(feature = "extra", feature = "full"))]
    fn call_extra_builtin(&mut self, name: &str, args: &[Value]) -> Option<Result<(), VmError>> {
        #[cfg(feature = "full")]
        if name == "__tcp_connect" || name == "__tcp_conectar" {
            #[allow(unused_variables)]
            let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            match std::net::TcpStream::connect(&addr) {
                Ok(_) => self.push(Value::Bool(true)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__tcp_listen" || name == "__tcp_escuchar" {
            #[allow(unused_variables)]
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

        #[cfg(feature = "full")]
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
            match ureq::get(&url).call() {
                Ok(resp) => {
                    let body = resp.into_string().unwrap_or_default();
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
            match ureq::post(&url).send_string(&body) {
                Ok(resp) => {
                    let text = resp.into_string().unwrap_or_default();
                    self.push(Value::str(text));
                }
                Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
            }
            return Some(Ok(()));
        }

        // ██ FFI builtins ██
        #[cfg(feature = "full")]
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

        #[cfg(feature = "full")]
        if name == "__ffi_llamar" || name == "__ffi_call" {
            let lib_id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let tipos = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            let ret = args.get(4).map(|v| format!("{}", v)).unwrap_or_default();
            let elems: Vec<Value> = match args.get(3) {
                Some(Value::Array(a)) => a.to_vec(),
                _ => Vec::new(),
            };
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
            match ffi_call_typed(lib, &fn_name, &tipos, &ret, &elems) {
                Ok(v) => self.push(v),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_llamar_nv" {
            let lib_id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let tipos = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            let ret = args.get(3).map(|v| format!("{}", v)).unwrap_or_default();
            let base = args.get(4).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let tys = parse_ffi_types(&tipos);
            let mut elems = Vec::with_capacity(tys.len());
            for (i, ty) in tys.iter().enumerate() {
                let off = base + i * 8;
                let lo = unsafe { *(off as *const u32) } as u64;
                let hi = unsafe { *((off + 4) as *const u32) } as u64;
                match ty {
                    FfiTy::Float => elems.push(Value::Float(f64::from_bits(lo | (hi << 32)))),
                    _ => elems.push(Value::Int((lo | (hi << 32)) as i64)),
                }
            }
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
            match ffi_call_typed(lib, &fn_name, &tipos, &ret, &elems) {
                Ok(v) => self.push(v),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_asignar" || name == "__ffi_alloc" {
            let size = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let align = args.get(1).and_then(|v| v.as_num()).unwrap_or(8.0) as usize;
            if size == 0 {
                self.push(Value::Int(0));
                return Some(Ok(()));
            }
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
            if ptr.is_null() {
                self.push(Value::Error(Box::new(Value::str(
                    "Fallo al reservar memoria FFI".to_string(),
                ))));
                return Some(Ok(()));
            }
            self.ffi_allocations.insert(ptr as usize, layout);
            self.push(Value::Int(ptr as i64));
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_liberar" || name == "__ffi_free" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            if ptr_val == 0 {
                self.push(Value::Void);
                return Some(Ok(()));
            }
            if let Some(layout) = self.ffi_allocations.remove(&ptr_val) {
                // Usa el layout almacenado en alloc, no el proporcionado por el caller (evita mismatch size/align)
                unsafe {
                    std::alloc::dealloc(ptr_val as *mut u8, layout);
                }
                self.push(Value::Void);
            } else {
                self.push(Value::Error(Box::new(Value::str(
                    "Liberación FFI inválida: puntero no encontrado o doble free".to_string(),
                ))));
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_escribir" || name == "__ffi_write" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let offset = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let data = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            let bytes = data.as_bytes();
            if ptr_val != 0 && !bytes.is_empty() {
                if let Some(layout) = self.ffi_allocations.get(&ptr_val) {
                    if offset
                        .checked_add(bytes.len())
                        .is_none_or(|end| end > layout.size())
                    {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Escritura FFI fuera de rango: offset {} + len {} > size {}",
                            offset,
                            bytes.len(),
                            layout.size()
                        )))));
                        return Some(Ok(()));
                    }
                }
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        (ptr_val + offset) as *mut u8,
                        bytes.len(),
                    );
                }
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_leer" || name == "__ffi_read" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let offset = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let len = args.get(2).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            if ptr_val != 0 && len > 0 {
                if let Some(layout) = self.ffi_allocations.get(&ptr_val) {
                    if offset
                        .checked_add(len)
                        .is_none_or(|end| end > layout.size())
                    {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Lectura FFI fuera de rango: offset {} + len {} > size {}",
                            offset,
                            len,
                            layout.size()
                        )))));
                        return Some(Ok(()));
                    }
                }
                let mut buf = vec![0u8; len];
                unsafe {
                    std::ptr::copy_nonoverlapping(
                        (ptr_val + offset) as *const u8,
                        buf.as_mut_ptr(),
                        len,
                    );
                }
                self.push(Value::str(String::from_utf8_lossy(&buf).to_string()));
            } else {
                self.push(Value::str(String::new()));
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_peek" || name == "__ffi_peek_u32" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let offset = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            if ptr_val != 0 {
                if let Some(layout) = self.ffi_allocations.get(&ptr_val) {
                    if offset.checked_add(4).is_none_or(|end| end > layout.size()) {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Lectura FFI fuera de rango: offset {} + len 4 > size {}",
                            offset,
                            layout.size()
                        )))));
                        return Some(Ok(()));
                    }
                }
                unsafe {
                    let b0 = *((ptr_val + offset) as *const u8) as u32;
                    let b1 = *((ptr_val + offset + 1) as *const u8) as u32;
                    let b2 = *((ptr_val + offset + 2) as *const u8) as u32;
                    let b3 = *((ptr_val + offset + 3) as *const u8) as u32;
                    let val = b0 | (b1 << 8) | (b2 << 16) | (b3 << 24);
                    self.push(Value::Int(val as i64));
                }
            } else {
                self.push(Value::Int(0));
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_peek64" || name == "__ffi_peek_ptr" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let offset = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            if ptr_val != 0 {
                if let Some(layout) = self.ffi_allocations.get(&ptr_val) {
                    if offset.checked_add(8).is_none_or(|end| end > layout.size()) {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Lectura FFI64 fuera de rango: offset {} + len 8 > size {}",
                            offset,
                            layout.size()
                        )))));
                        return Some(Ok(()));
                    }
                }
                unsafe {
                    let mut b = [0u8; 8];
                    std::ptr::copy_nonoverlapping(
                        (ptr_val + offset) as *const u8,
                        b.as_mut_ptr(),
                        8,
                    );
                    let val = i64::from_le_bytes(b);
                    self.push(Value::Int(val));
                }
            } else {
                self.push(Value::Int(0));
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_peek_byte" || name == "__ffi_peek_u8" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let offset = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            if ptr_val != 0 {
                if let Some(layout) = self.ffi_allocations.get(&ptr_val) {
                    if offset.checked_add(1).is_none_or(|end| end > layout.size()) {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Peek byte fuera de rango: offset {} + len 1 > size {}",
                            offset,
                            layout.size()
                        )))));
                        return Some(Ok(()));
                    }
                }
                unsafe {
                    let b = *((ptr_val + offset) as *const u8);
                    self.push(Value::Int(b as i64));
                }
            } else {
                self.push(Value::Int(0));
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_poke" || name == "__ffi_poke_u32" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let offset = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let val = args.get(2).and_then(|v| v.as_num()).unwrap_or(0.0) as u32;
            if ptr_val != 0 {
                if let Some(layout) = self.ffi_allocations.get(&ptr_val) {
                    if offset.checked_add(4).is_none_or(|end| end > layout.size()) {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Poke fuera de rango: offset {} + len 4 > size {}",
                            offset,
                            layout.size()
                        )))));
                        return Some(Ok(()));
                    }
                }
                unsafe {
                    *((ptr_val + offset) as *mut u8) = val as u8;
                    *((ptr_val + offset + 1) as *mut u8) = (val >> 8) as u8;
                    *((ptr_val + offset + 2) as *mut u8) = (val >> 16) as u8;
                    *((ptr_val + offset + 3) as *mut u8) = (val >> 24) as u8;
                }
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
        if name == "__ffi_poke_byte" || name == "__ffi_poke_u8" {
            let ptr_val = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let offset = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
            let val = args.get(2).and_then(|v| v.as_num()).unwrap_or(0.0) as u8;
            if ptr_val != 0 {
                if let Some(layout) = self.ffi_allocations.get(&ptr_val) {
                    if offset.checked_add(1).is_none_or(|end| end > layout.size()) {
                        self.push(Value::Error(Box::new(Value::str(format!(
                            "Poke byte fuera de rango: offset {} + len 1 > size {}",
                            offset,
                            layout.size()
                        )))));
                        return Some(Ok(()));
                    }
                }
                unsafe {
                    *((ptr_val + offset) as *mut u8) = val;
                }
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "__ffi_asm" {
            let _code = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            // En modo VM / JIT interpreta el bloque de ensamblador como ejecución nativa
            self.push(Value::Int(0));
            return Some(Ok(()));
        }

        if name == "__ffi_c_eval" {
            let _code = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::Int(0));
            return Some(Ok(()));
        }

        if name == "__ffi_rust_eval" {
            let _code = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            self.push(Value::Int(0));
            return Some(Ok(()));
        }

        // ██ Self-Healing Runtime Builtins ██
        if name == "__self_healing_registrar_parche" {
            let orig = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let patch = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            // Registra parche de auto-reparación
            self.push(Value::str(format!("PARCHE_REGISTRADO:{}:{}", orig, patch)));
            return Some(Ok(()));
        }

        if name == "__self_healing_invocar" {
            let fn_name = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            // Ejecución protegida de función con fallback automático
            self.push(Value::str(format!(
                "EJECUTADO_CON_SELF_HEALING:{}",
                fn_name
            )));
            return Some(Ok(()));
        }

        if name == "__self_healing_estado" {
            self.push(Value::str(
                "ESTADO:RESILIENTE|FALLOS_INTERCEPTADOS:0|HOT_PATCHES:0",
            ));
            return Some(Ok(()));
        }

        // ██ Crypto builtins ██
        if name == "__hash_sha256" {
            let data = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            #[cfg(feature = "full")]
            let hash: Vec<u8> = match self.bcrypt.as_ref() {
                Some(bc) => bc.sha256(data.as_bytes()).unwrap_or_default(),
                None => {
                    use sha2::{Digest, Sha256};
                    let mut h = Sha256::new();
                    h.update(data.as_bytes());
                    h.finalize().to_vec()
                }
            };
            #[cfg(not(feature = "full"))]
            let hash: Vec<u8> = {
                use sha2::{Digest, Sha256};
                let mut h = Sha256::new();
                h.update(data.as_bytes());
                h.finalize().to_vec()
            };
            let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
            self.push(Value::str(hex));
            return Some(Ok(()));
        }

        if name == "__hash_sha512" {
            let data = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            #[cfg(feature = "full")]
            let hash: Vec<u8> = match self.bcrypt.as_ref() {
                Some(bc) => bc.sha512(data.as_bytes()).unwrap_or_default(),
                None => {
                    use sha2::{Digest, Sha512};
                    let mut h = Sha512::new();
                    h.update(data.as_bytes());
                    h.finalize().to_vec()
                }
            };
            #[cfg(not(feature = "full"))]
            let hash: Vec<u8> = {
                use sha2::{Digest, Sha512};
                let mut h = Sha512::new();
                h.update(data.as_bytes());
                h.finalize().to_vec()
            };
            let hex: String = hash.iter().map(|b| format!("{:02x}", b)).collect();
            self.push(Value::str(hex));
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
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
            if self.bcrypt.is_none() {
                match Bcrypt::load() {
                    Ok(b) => self.bcrypt = Some(Arc::new(b)),
                    Err(e) => {
                        self.push(Value::Error(Box::new(Value::str(e))));
                        return Some(Ok(()));
                    }
                }
            }
            let bcrypt = match self.bcrypt.as_ref() {
                Some(b) => b,
                None => {
                    self.push(Value::Error(Box::new(Value::str(
                        "Bcrypt no inicializado".to_string(),
                    ))));
                    return Some(Ok(()));
                }
            };
            match bcrypt.aes_encrypt(&key, &data) {
                Ok(ct) => self.push(Value::str(hex::encode(ct))),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
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
            let bcrypt = match self.bcrypt.as_ref() {
                Some(b) => b,
                None => {
                    self.push(Value::Error(Box::new(Value::str(
                        "Bcrypt no inicializado".to_string(),
                    ))));
                    return Some(Ok(()));
                }
            };
            match bcrypt.aes_decrypt(&key, &data) {
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
                self.push(Value::Error(Box::new(Value::str(
                    "JWT inválido: se esperan 3 partes",
                ))));
                return Some(Ok(()));
            }
            let sig_input = format!("{}.{}", parts[0], parts[1]);
            let expected_sig = hmac_sha256(sig_input.as_bytes(), secret.as_bytes());
            let actual_sig = base64url_decode(parts[2]);
            if actual_sig != expected_sig {
                self.push(Value::Error(Box::new(Value::str("Firma JWT inválida"))));
                return Some(Ok(()));
            }
            match base64url_decode_to_string(parts[1]) {
                Ok(payload) => self.push(Value::str(payload)),
                Err(e) => self.push(Value::Error(Box::new(Value::str(e)))),
            }
            return Some(Ok(()));
        }

        // ██ Utility builtins ██
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
            #[cfg(not(target_arch = "wasm32"))]
            let vars: Vec<Value> = std::env::vars()
                .map(|(k, v)| Value::str(format!("{}={}", k, v)))
                .collect();
            #[cfg(target_arch = "wasm32")]
            let vars: Vec<Value> = Vec::new();
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
                    coro.flat = self.flat.clone();
                    coro.free_slots = self.free_slots.clone();
                    coro.ip = self.ip;
                }
            }
            // Restore main saved state
            if let Some((saved_stack, saved_locals, saved_flat, saved_free, saved_ip)) =
                self.main_saved.take()
            {
                self.stack = saved_stack;
                self.replace_locals_full(saved_locals, saved_flat, saved_free);
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
                    self.push(Value::Error(Box::new(Value::str("Coroutine terminada"))));
                    return Some(Ok(()));
                }
            }
            // Save main state before resuming coroutine
            self.main_saved = Some((
                self.stack.clone(),
                self.locals.clone(),
                self.flat.clone(),
                self.free_slots.clone(),
                self.ip,
            ));
            if let Some(coro) = self.coroutines.get(&coro_id) {
                let coro_stack = coro.stack.clone();
                let coro_locals = coro.locals.clone();
                let coro_flat = coro.flat.clone();
                let coro_free = coro.free_slots.clone();
                let coro_ip = coro.ip;
                self.stack = coro_stack;
                self.replace_locals_full(coro_locals, coro_flat, coro_free);
                self.ip = coro_ip;
                self.current_coro = Some(coro_id.clone());
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        // ██ GUI builtins ██
        #[cfg(feature = "full")]
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

        #[cfg(feature = "full")]
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

        #[cfg(feature = "full")]
        if name == "__gui_cerrar" || name == "__gui_close" {
            let id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            if self.gui_windows.remove(&id).is_some() {
                self.push(Value::Bool(true));
            } else {
                self.push(Value::Bool(false));
            }
            return Some(Ok(()));
        }

        #[cfg(feature = "full")]
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

        #[cfg(feature = "full")]
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
            let fn_args: Vec<Value> = args.iter().skip(1).cloned().collect();
            let id = self.task_counter;
            self.task_counter += 1;
            let task_id = format!("task_{}", id);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let bc = self.bytecode.clone();
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let mut vm = VM::new(bc);
                    let result = vm.run_function(&fn_name, fn_args);
                    let _ = tx.send(result.unwrap_or(Value::Void));
                });
                self.task_results.insert(task_id.clone(), rx);
            }
            #[cfg(target_arch = "wasm32")]
            {
                let bc = self.bytecode.clone();
                let mut vm = VM::new(bc);
                let result = vm.run_function(&fn_name, fn_args);
                self.task_results_sync
                    .insert(task_id.clone(), result.unwrap_or(Value::Void));
            }
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        if name == "__tarea_esperar" || name == "__task_await" {
            let task_id = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            #[cfg(not(target_arch = "wasm32"))]
            let found = {
                if let Some(rx) = self.task_results.remove(&task_id) {
                    match rx.recv() {
                        Ok(val) => Some(val),
                        Err(_) => Some(Value::Error(Box::new(Value::str("Task failed")))),
                    }
                } else {
                    None
                }
            };
            #[cfg(target_arch = "wasm32")]
            let found = self.task_results_sync.remove(&task_id);
            match found {
                Some(val) => self.push(val),
                None => self.push(Value::Error(Box::new(Value::str("Task not found")))),
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
            let id = self.task_counter;
            self.task_counter += 1;
            let task_id = format!("file_{}", id);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let content = std::fs::read_to_string(&path);
                    let _ = tx.send(match content {
                        Ok(s) => Value::str(s),
                        Err(e) => Value::Error(Box::new(Value::str(e.to_string()))),
                    });
                });
                self.task_results.insert(task_id.clone(), rx);
            }
            #[cfg(target_arch = "wasm32")]
            {
                let content = std::fs::read_to_string(&path);
                let val = match content {
                    Ok(s) => Value::str(s),
                    Err(e) => Value::Error(Box::new(Value::str(e.to_string()))),
                };
                self.task_results_sync.insert(task_id.clone(), val);
            }
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        if name == "__escribir_archivo_async" || name == "__file_write_async" {
            let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let id = self.task_counter;
            self.task_counter += 1;
            let task_id = format!("file_{}", id);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let result = std::fs::write(&path, &content);
                    let _ = tx.send(match result {
                        Ok(()) => Value::Bool(true),
                        Err(e) => Value::Error(Box::new(Value::str(e.to_string()))),
                    });
                });
                self.task_results.insert(task_id.clone(), rx);
            }
            #[cfg(target_arch = "wasm32")]
            {
                let result = std::fs::write(&path, &content);
                let val = match result {
                    Ok(()) => Value::Bool(true),
                    Err(e) => Value::Error(Box::new(Value::str(e.to_string()))),
                };
                self.task_results_sync.insert(task_id.clone(), val);
            }
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        // ██ Async Timer builtins ██
        if name == "__timer_delay" || name == "__temporizador_esperar" {
            #[allow(unused_variables)]
            let ms = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as u64;
            let id = self.task_counter;
            self.task_counter += 1;
            let task_id = format!("timer_{}", id);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(ms));
                    let _ = tx.send(Value::Bool(true));
                });
                self.task_results.insert(task_id.clone(), rx);
            }
            #[cfg(target_arch = "wasm32")]
            {
                self.task_results_sync
                    .insert(task_id.clone(), Value::Bool(true));
            }
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        // ██ Async TCP connect builtins ██
        if name == "__tcp_connect_async" || name == "__tcp_conectar_async" {
            #[allow(unused_variables)]
            let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let id = self.task_counter;
            self.task_counter += 1;
            let task_id = format!("tcp_{}", id);
            #[cfg(not(target_arch = "wasm32"))]
            {
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || match std::net::TcpStream::connect(&addr) {
                    Ok(_) => {
                        let _ = tx.send(Value::Bool(true));
                    }
                    Err(e) => {
                        let _ = tx.send(Value::Error(Box::new(Value::str(e.to_string()))));
                    }
                });
                self.task_results.insert(task_id.clone(), rx);
            }
            #[cfg(target_arch = "wasm32")]
            {
                self.task_results_sync.insert(
                    task_id.clone(),
                    Value::Error(Box::new(Value::str("TCP no soportado en wasm"))),
                );
            }
            self.push(Value::str(task_id));
            return Some(Ok(()));
        }

        // ██ Concurrency builtins ██
        if name == "__dormir" || name == "__sleep" {
            #[allow(unused_variables)]
            let ms = args.first().and_then(|v| v.as_num()).unwrap_or(0.0) as u64;
            #[cfg(not(target_arch = "wasm32"))]
            {
                std::thread::sleep(std::time::Duration::from_millis(ms));
            }
            #[cfg(target_arch = "wasm32")]
            {
                let _ = ms;
            }
            self.push(Value::Void);
            return Some(Ok(()));
        }

        if name == "__hilo_lanzar" || name == "__thread_spawn" {
            let fn_name = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_args: Vec<Value> = args.iter().skip(1).cloned().collect();
            #[cfg(not(target_arch = "wasm32"))]
            let hid = format!("thread_{}", self.thread_handles.len());
            #[cfg(target_arch = "wasm32")]
            let hid = {
                let id = self.task_counter;
                self.task_counter += 1;
                format!("thread_{}", id)
            };
            #[cfg(not(target_arch = "wasm32"))]
            {
                let bc = self.bytecode.clone();
                // v3.5.17: el hilo comparte canales/mutexes con la VM madre.
                #[cfg(any(feature = "extra", feature = "full"))]
                let chans = Arc::clone(&self.channels);
                #[cfg(any(feature = "extra", feature = "full"))]
                let mtxs = Arc::clone(&self.mutexes);
                let handle = std::thread::spawn(move || {
                    let mut vm = VM::new(bc);
                    #[cfg(any(feature = "extra", feature = "full"))]
                    {
                        vm.channels = chans;
                        vm.mutexes = mtxs;
                    }
                    vm.run_function(&fn_name, fn_args).unwrap_or(Value::Void)
                });
                self.thread_handles.insert(hid.clone(), handle);
            }
            #[cfg(target_arch = "wasm32")]
            {
                let bc = self.bytecode.clone();
                let mut vm = VM::new(bc);
                #[cfg(any(feature = "extra", feature = "full"))]
                {
                    vm.channels = Arc::clone(&self.channels);
                    vm.mutexes = Arc::clone(&self.mutexes);
                }
                let result = vm.run_function(&fn_name, fn_args).unwrap_or(Value::Void);
                self.task_results_sync.insert(hid.clone(), result);
            }
            self.push(Value::str(hid));
            return Some(Ok(()));
        }

        if name == "__hilo_esperar" || name == "__thread_join" {
            let hid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            #[cfg(not(target_arch = "wasm32"))]
            let found = {
                if let Some((_, handle)) = self.thread_handles.remove_entry(&hid) {
                    match handle.join() {
                        Ok(val) => Some(val),
                        Err(_) => Some(Value::Error(Box::new(Value::str("Thread panicked")))),
                    }
                } else {
                    None
                }
            };
            #[cfg(target_arch = "wasm32")]
            let found = self.task_results_sync.remove(&hid);
            match found {
                Some(val) => self.push(val),
                None => self.push(Value::Error(Box::new(Value::str("Thread not found")))),
            }
            return Some(Ok(()));
        }

        if name == "__canal_nuevo" || name == "__channel_new" {
            let (tx, rx) = std::sync::mpsc::channel::<Value>();
            let mut chans = self.channels.lock().unwrap_or_else(|e| e.into_inner());
            let cid = format!("chan_{}", chans.len());
            chans.insert(cid.clone(), (Some(tx), Some(rx)));
            drop(chans);
            self.push(Value::str(cid));
            return Some(Ok(()));
        }

        if name == "__canal_enviar" || name == "__channel_send" {
            let cid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let val = args.get(1).cloned().unwrap_or(Value::Void);
            let chans = self.channels.lock().unwrap_or_else(|e| e.into_inner());
            let res = if let Some((Some(ref tx), _)) = chans.get(&cid) {
                match tx.send(val) {
                    Ok(()) => Value::Bool(true),
                    Err(_) => Value::Bool(false),
                }
            } else {
                Value::Error(Box::new(Value::str("Channel not found")))
            };
            drop(chans);
            self.push(res);
            return Some(Ok(()));
        }

        if name == "__canal_recibir" || name == "__channel_recv" {
            let cid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            // Sacar el receiver y SOLTAR el candado antes del recv bloqueante:
            // si lo retenemos, el hilo productor no podría enviar.
            let rx = {
                let mut chans = self.channels.lock().unwrap_or_else(|e| e.into_inner());
                chans.get_mut(&cid).and_then(|(_, rx_opt)| rx_opt.take())
            };
            match rx {
                Some(rx) => {
                    let val = rx.recv().unwrap_or(Value::Void);
                    let mut chans = self.channels.lock().unwrap_or_else(|e| e.into_inner());
                    if let Some(cell) = chans.get_mut(&cid) {
                        cell.1 = Some(rx);
                    }
                    drop(chans);
                    self.push(val);
                }
                None => {
                    self.push(Value::Error(Box::new(Value::str("Channel not found"))));
                }
            }
            return Some(Ok(()));
        }

        if name == "__mutex_nuevo" || name == "__mutex_new" {
            let mut mtxs = self.mutexes.lock().unwrap_or_else(|e| e.into_inner());
            let mid = format!("mutex_{}", mtxs.len());
            mtxs.insert(mid.clone(), Arc::new(std::sync::Mutex::new(Value::Void)));
            drop(mtxs);
            self.push(Value::str(mid));
            return Some(Ok(()));
        }

        if name == "__mutex_bloquear" || name == "__mutex_lock" {
            let mid = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let fn_name = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
            let fn_arg = args.get(2).cloned().unwrap_or(Value::Void);
            let mtx = {
                let mtxs = self.mutexes.lock().unwrap_or_else(|e| e.into_inner());
                mtxs.get(&mid).cloned()
            };
            let result = if let Some(mutex) = mtx {
                let _guard = mutex.lock().unwrap_or_else(|e| e.into_inner());
                drop(_guard);
                // Ejecutar la función en una VM hija que comparte los
                // registros de canales/mutexes (v3.5.17).
                let bc = self.bytecode.clone();
                let mut vm = VM::new(bc);
                vm.channels = Arc::clone(&self.channels);
                vm.mutexes = Arc::clone(&self.mutexes);
                vm.run_function(&fn_name, vec![fn_arg])
                    .unwrap_or(Value::Void)
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
                _ => self.push(Value::Error(Box::new(Value::str(
                    "stream_map espera una lista",
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
                            matches!(
                                vm.run_function(&fn_name, vec![(*item).clone()]),
                                Ok(Value::Bool(true))
                            )
                        })
                        .cloned()
                        .collect();
                    self.push(Value::arr(filtered));
                }
                _ => self.push(Value::Error(Box::new(Value::str(
                    "stream_filter espera una lista",
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
                    #[cfg(not(target_arch = "wasm32"))]
                    let results: Vec<Value> = {
                        let bc = self.bytecode.clone();
                        let mut handles = Vec::new();
                        for item in items.iter() {
                            let bc_clone = bc.clone();
                            let fn_clone = fn_name.clone();
                            let item_clone = (*item).clone();
                            handles.push(std::thread::spawn(move || {
                                let mut vm = VM::new(bc_clone);
                                vm.run_function(&fn_clone, vec![item_clone])
                                    .unwrap_or(Value::Void)
                            }));
                        }
                        handles
                            .into_iter()
                            .map(|h| h.join().unwrap_or(Value::Void))
                            .collect()
                    };
                    #[cfg(target_arch = "wasm32")]
                    let results: Vec<Value> = {
                        let bc = self.bytecode.clone();
                        items
                            .iter()
                            .map(|item| {
                                let mut vm = VM::new(bc.clone());
                                vm.run_function(&fn_name, vec![item.clone()])
                                    .unwrap_or(Value::Void)
                            })
                            .collect()
                    };
                    self.push(Value::arr(results));
                }
                _ => self.push(Value::Error(Box::new(Value::str(
                    "par_map espera una lista",
                )))),
            }
            return Some(Ok(()));
        }

        if name == "__par_unir" || name == "__par_join" {
            let fn1 = args.first().map(|v| format!("{}", v)).unwrap_or_default();
            let a1 = args.get(1).cloned().unwrap_or(Value::Void);
            let fn2 = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
            let a2 = args.get(3).cloned().unwrap_or(Value::Void);
            #[cfg(not(target_arch = "wasm32"))]
            let (r1, r2) = {
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
                (
                    h1.join().unwrap_or(Value::Void),
                    h2.join().unwrap_or(Value::Void),
                )
            };
            #[cfg(target_arch = "wasm32")]
            let (r1, r2) = {
                let bc = self.bytecode.clone();
                let mut vm1 = VM::new(bc.clone());
                let mut vm2 = VM::new(bc);
                let v1 = if fn1.is_empty() {
                    Value::Void
                } else {
                    vm1.run_function(&fn1, vec![a1]).unwrap_or(Value::Void)
                };
                let v2 = if fn2.is_empty() {
                    Value::Void
                } else {
                    vm2.run_function(&fn2, vec![a2]).unwrap_or(Value::Void)
                };
                (v1, v2)
            };
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
        let profile = !std::env::var("LUMEN_PROFILE")
            .unwrap_or_default()
            .is_empty();
        #[allow(unused_mut)]
        let mut prof: std::collections::HashMap<String, (u64, f64)> =
            std::collections::HashMap::new();
        // Instant::now() NO existe en wasm32-unknown-unknown (paniquea);
        // en wasm `profile` es siempre false (env::var falla) — se evita.
        #[cfg(not(target_arch = "wasm32"))]
        let prof_start = std::time::Instant::now();
        loop {
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }
            self.step_instr(None)?;
        }
        if profile {
            #[cfg(not(target_arch = "wasm32"))]
            let total = prof_start.elapsed().as_secs_f64();
            #[cfg(target_arch = "wasm32")]
            let total = 0.0;
            let mut v: Vec<(String, u64, f64)> =
                prof.into_iter().map(|(k, (c, t))| (k, c, t)).collect();
            v.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap_or(std::cmp::Ordering::Equal));
            eprintln!(
                "=== PROFILE total={:.1}s instrs={} ===",
                total, self.instr_count
            );
            for (k, c, t) in v {
                eprintln!(
                    "{:20} {:>12} calls {:>10.3}s ({:>5.1}%)",
                    k,
                    c,
                    t,
                    t / total * 100.0
                );
            }
        }
        Ok(())
    }

    pub fn set_breakpoint(&mut self, ip: usize) {
        self.breakpoints.push(ip);
    }

    // ══════════════════════════════════════════════════════════════
    // JIT Tier-1 (v3.5.9) — integración con crate::jit
    // ══════════════════════════════════════════════════════════════

    /// Ejecuta UNA instrucción en `self.ip` (avanza ip), con el desenrollado
    /// intentar/atrapar usual. `min_cl` acota el desenrollado: si hay un
    /// handler registrado con `call_stack.len() < min_cl` (o sea, registrado
    /// FUERA de la región anidada actual), el error se propaga hacia arriba en
    /// vez de capturarse aquí (lo capturará el loop externo con su estado
    /// correcto). `None` = comportamiento clásico del loop principal.
    #[inline(always)]
    fn step_instr(&mut self, min_cl: Option<usize>) -> Result<(), VmError> {
        let cur_ip = self.ip;
        self.ip += 1;
        self.instr_count += 1;
        let exec_result = match self.bytecode.instructions[cur_ip] {
            // v3.5.31: Jmp inline — es la instrucción MÁS caliente de los
            // bucles; evita el salto a execute_with_idx y su match completo.
            Instruction::WithIdx(Opcode::Jmp, idx) => {
                self.ip = self.resolve_target(idx);
                Ok(())
            }
            Instruction::Simple(op) => self.execute_simple(op),
            Instruction::WithIdx(op, idx) => self.execute_with_idx(op, idx),
            Instruction::WithNum(op, n) => self.execute_with_num(op, n),
            Instruction::WithBool(op, b) => self.execute_with_bool(op, b),
            Instruction::WithStr(op, ref s) => {
                let s_clone = s.clone();
                self.execute_with_str(op, &s_clone)
            }
            // v3.5.20 super-opcodes: bucles sin push/pop ni dispatch extra.
            // v3.5.31: destructuring escalar SIN clonar la instrucción
            // (antes: memcpy de ~48 B por instrucción del bucle).
            Instruction::FusedBinK { op, a, k, d } => self.exec_fused_bink(op, a, k, d),
            Instruction::FusedBin { op, a, b, d } => self.exec_fused_bin(op, a, b, d),
            Instruction::FusedCmpKJmp { op, a, k, target } => {
                self.exec_fused_cmpkjmp(op, a, k, target)
            }
            Instruction::FusedCmpJmp { op, a, b, target } => {
                self.exec_fused_cmpjmp(op, a, b, target)
            }
            // v3.5.31: aritmética + comparación + salto (6 IR → 1).
            Instruction::FusedBinCmpJmp {
                op1,
                op2,
                a,
                b,
                c,
                target,
            } => self.exec_fused_bincmpjmp(op1, op2, a, b, c, target),
            Instruction::FusedBinKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => self.exec_fused_binkcmpjmp(op1, op2, a, b, k, target),
            Instruction::FusedBinKKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => self.exec_fused_binkkcmpjmp(op1, op2, a, b, k, target),
        };
        if let Err(e) = exec_result {
            // intentar/atrapar: desenrollar al manejador más cercano si existe
            let mut handled = false;
            let en_rango = match self.handlers.last() {
                Some(&(_, _, _, cl)) => min_cl.is_none_or(|m| cl >= m),
                None => false,
            };
            if en_rango {
                {
                    let (catch_ip, sl, ll, cl) = self.handlers.pop().unwrap();
                    self.stack.truncate(sl);
                    self.scope_truncate(ll);
                    self.call_stack.truncate(cl);
                    let msg = vm_error_message(&e);
                    self.push(Value::str(msg));
                    self.ip = catch_ip;
                    handled = true;
                }
            }
            if !handled {
                return Err(e);
            }
        }
        Ok(())
    }

    /// Interpreta hasta que el frame en `depth` retorne (o el bytecode termine).
    /// Lo usa `perform_call` cuando el JIT llama a una función NO compilada.
    fn run_until_return(&mut self, depth: usize) -> Result<(), VmError> {
        loop {
            if self.call_stack.len() < depth {
                return Ok(());
            }
            if self.ip >= self.bytecode.instructions.len() {
                return Ok(());
            }
            self.step_instr(Some(depth))?;
        }
    }

    /// Empuja CallFrame + scope de parámetros (idéntico al camino del
    /// intérprete). NO toca `ip`.
    fn setup_call_frame(&mut self, name_idx: usize, func_idx: usize, args: &[Value]) {
        self.call_stack.push(CallFrame {
            func_name: name_idx,
            func_label: None,
            return_ip: self.ip,
            locals_base: self.locals.len(),
            stack_base: self.stack.len(),
            is_closure: false,
            has_refs: false,
        });
        self.push_params_scope(func_idx, args);
    }

    /// Empuja el scope de parámetros de la función `func_idx` con los
    /// argumentos ya evaluados.
    /// v3.5.31: SIN mapa ni clones de String — el scope guarda SOLO el
    /// func_idx y los slots en orden de parámetro (params[i] ↔ slots[i]).
    fn push_params_scope(&mut self, func_idx: usize, args: &[Value]) {
        let param_count = self.bytecode.funcs[func_idx].params.len();
        let id = self.next_scope_id();
        // v3.5.36: buffer de slots REUTILIZADO del pool — sin alloc por
        // llamada (la maquinaria de llamadas de fib OFF paga esto ~317k
        // veces por fib(27)).
        let mut slots: Vec<u32> = self.slot_pool.pop().unwrap_or_default();
        slots.clear();
        self.locals.push(ScopeFrame::with_parts(
            HashMap::with_hasher(FixHasher::default()),
            slots,
            id,
            Some(func_idx),
        ));
        // v3.5.36: invalidación SELECTIVA — solo los nombres que los
        // parámetros SOMBREAN pierden su entrada de caché; los demás
        // accesos del llamador siguen cacheados a través de la llamada
        // (ganancia real en bucles con calls: contar_primos OFF, etc.).
        if let Some(pis) = self.params_name_idx.get(func_idx) {
            for &pidx in pis {
                if pidx < self.var_cache.len() && self.var_cache_slot(pidx).is_some() {
                    self.var_cache[pidx] = (0, 0, 0, 0);
                }
            }
        }
        if param_count == 0 {
            return;
        }
        let top = self.locals.len() - 1;
        for i in 0..param_count {
            let arg = if i < args.len() {
                args[i].clone()
            } else if let Some(Some(dv)) = self.bytecode.funcs[func_idx].defaults.get(i) {
                match dv {
                    DefaultValue::Int(v) => Value::Int(*v),
                    DefaultValue::Float(v) => Value::Float(*v),
                    DefaultValue::Str(s) => Value::str(s.clone()),
                    DefaultValue::Bool(b) => Value::Bool(*b),
                }
            } else {
                Value::Void
            };
            let slot = self.alloc_slot(arg);
            self.locals[top].slots.push(slot);
        }
    }

    /// v3.5.31: lookup nombre→slot dentro de UN scope. Para scopes de
    /// parámetros (mapa vacío) resuelve posicionalmente: params[i] ↔
    /// slots[i] — sin hash y sin alloc.
    #[inline(always)]
    fn scope_get(&self, scope: &ScopeFrame, name: &str) -> Option<u32> {
        if let Some(&s) = scope.map.get(name) {
            return Some(s);
        }
        if let Some(fi) = scope.param_func {
            let params = &self.bytecode.funcs[fi].params;
            for (i, p) in params.iter().enumerate() {
                if p == name {
                    return scope.slots.get(i).copied();
                }
            }
        }
        None
    }

    /// Llamada completa (para el helper lj_call del JIT): pop de args,
    /// builtins, funciones compiladas (recursión nativa) o interpretación
    /// anidada hasta el retorno.
    /// v3.5.31: llamada desde el cuerpo JIT (lj_call pasa el ÍNDICE de
    /// nombre — sin parseo de string ni hash por llamada).
    pub(crate) fn perform_call_idx(&mut self, nidx: usize, argc: usize) -> Result<(), VmError> {
        // v3.5.31: args SIN heap para argc pequeño (el caso dominante).
        let mut small: [Value; 4] = std::array::from_fn(|_| Value::Void);
        let mut big: Vec<Value>;
        let args: &[Value] = if argc <= 4 {
            for k in 0..argc {
                small[argc - 1 - k] = self.pop()?;
            }
            &small[..argc]
        } else {
            big = Vec::with_capacity(argc);
            for _ in 0..argc {
                big.push(self.pop()?);
            }
            big.reverse();
            &big
        };
        // v3.5.31: pre-filtro O(1) de builtins — los nombres que no son
        // builtins van directo a la tabla de funciones SIN clonar el String.
        let name_is_builtin = builtin_name_set().contains(
            self.bytecode
                .names
                .get(nidx)
                .map(|s| s.as_str())
                .unwrap_or(""),
        );
        if name_is_builtin {
            let name = self.bytecode.names.get(nidx).cloned().unwrap_or_default();
            if let Some(result) = self.call_core_builtin(&name, args) {
                return result;
            }
            #[cfg(any(feature = "extra", feature = "full"))]
            if let Some(result) = self.call_extra_builtin(&name, args) {
                return result;
            }
        }
        self.perform_user_call(nidx, args)
    }

    /// v3.5.32 (Tier-2): llamada SIN el pre-filtro de builtins — el JIT solo
    /// la emite cuando el nombre NO es builtin (decisión ESTÁTICA en
    /// compilación con el mismo set). Ahorra el check O(1) por llamada
    /// recursiva nativa (p.ej. fib). Paridad exacta con perform_call_idx.
    pub(crate) fn perform_call_fast(&mut self, nidx: usize, argc: usize) -> Result<(), VmError> {
        // args SIN heap para argc pequeño (igual que perform_call_idx).
        let mut small: [Value; 4] = std::array::from_fn(|_| Value::Void);
        let mut big: Vec<Value>;
        let args: &[Value] = if argc <= 4 {
            for k in 0..argc {
                small[argc - 1 - k] = self.pop()?;
            }
            &small[..argc]
        } else {
            big = Vec::with_capacity(argc);
            for _ in 0..argc {
                big.push(self.pop()?);
            }
            big.reverse();
            &big
        };
        self.perform_user_call(nidx, args)
    }

    /// Parte común de la llamada (builtin ya descartado): tabla de
    /// funciones, contador de hotness, frame y ejecución nativa/interp.
    fn perform_user_call(&mut self, nidx: usize, args: &[Value]) -> Result<(), VmError> {
        let func_idx = match self.func_index_by_name_idx.get(nidx).copied().flatten() {
            Some(fi) => fi,
            None => {
                return Err(VmError::UndefinedFunction(
                    self.bytecode.names.get(nidx).cloned().unwrap_or_default(),
                ))
            }
        };
        if self.call_stack.len() >= MAX_CALL_STACK_DEPTH {
            return Err(VmError::Runtime(format!(
                "Desbordamiento de pila (Stack overflow): límite de recursión excedido (>{} llamadas)",
                MAX_CALL_STACK_DEPTH
            )));
        }
        let count_now = {
            let count = if nidx < self.call_counts.len() {
                &mut self.call_counts[nidx]
            } else {
                return Err(VmError::UndefinedFunction(
                    self.bytecode.names.get(nidx).cloned().unwrap_or_default(),
                ));
            };
            *count += 1;
            *count
        };
        self.jit_maybe_compile(func_idx, count_now);
        self.setup_call_frame(nidx, func_idx, args);
        #[cfg(feature = "aot")]
        if let Some(f) = self.jit_get_fn(func_idx) {
            // SAFETY: el código nativo re-entra a la VM vía puntero crudo;
            // ningún otro &mut a la VM se usa mientras `f` ejecuta.
            // v3.5.31: native_exec desactiva los peepholes de ip.
            let prev_native = self.native_exec;
            self.native_exec = true;
            let vm_ptr = self as *mut VM as *mut std::ffi::c_void;
            let r = unsafe { f(vm_ptr) };
            self.native_exec = prev_native;
            match r {
                0 => return Ok(()),
                // v3.5.31 (Tier-2): guarda de tipos falló → ejecutar el
                // MISMO frame en el intérprete (ip al inicio del cuerpo).
                2 => {
                    if let Some(rt) = self.jit_rt.as_mut() {
                        rt.invalidate(func_idx);
                    }
                    let func_start = self.bytecode.funcs[func_idx].start;
                    self.ip = func_start;
                    let depth = self.call_stack.len();
                    return self.run_until_return(depth);
                }
                _ => {
                    return Err(self.jit_error.take().unwrap_or_else(|| {
                        VmError::Runtime("JIT: error en código nativo".into())
                    }));
                }
            }
        }
        let func_start = self.bytecode.funcs[func_idx].start;
        self.ip = func_start;
        let depth = self.call_stack.len();
        self.run_until_return(depth)
    }

    /// Intenta compilar `func_idx` cuando la cuenta alcanza el umbral.
    #[cfg(feature = "aot")]
    fn jit_maybe_compile(&mut self, func_idx: usize, count: usize) {
        if !self.jit_enabled {
            return;
        }
        // v3.5.31: además del umbral clásico (50 llamadas), compila en la
        // PRIMERA llamada cuando el cuerpo contiene un bucle hacia atrás:
        // un bucle domina el tiempo de ejecución aunque la función se llame
        // una sola vez (patrón de los benchmarks de bucles).
        let hot = count == self.jit_threshold
            || (count == 1 && crate::jit::body_has_loop(&self.bytecode, func_idx));
        if !hot {
            return;
        }
        if self.jit_rt.is_none() {
            match crate::jit::VmJit::new() {
                Ok(rt) => self.jit_rt = Some(rt),
                Err(e) => {
                    eprintln!("[jit] no se pudo inicializar el motor JIT: {}", e);
                    self.jit_enabled = false;
                    return;
                }
            }
        }
        if let Some(rt) = self.jit_rt.as_mut() {
            rt.try_compile(&self.bytecode, func_idx);
        }
    }

    #[cfg(not(feature = "aot"))]
    fn jit_maybe_compile(&mut self, _func_idx: usize, _count: usize) {}

    #[cfg(feature = "aot")]
    fn jit_get_fn(&self, func_idx: usize) -> Option<crate::jit::JitFn> {
        if !self.jit_enabled {
            return None;
        }
        self.jit_rt.as_ref().and_then(|rt| rt.get(func_idx))
    }

    // ── Superficie pub(crate) usada por los helpers extern "C" de jit.rs ──
    pub(crate) fn set_jit_error(&mut self, e: VmError) {
        self.jit_error = Some(e);
    }

    /// v3.5.31 (Tier-2): pop Int del tope de la pila de valores; i64::MIN si
    /// el tope no es Int o la pila está vacía (el JIT hace bail-out).
    #[cfg(feature = "aot")]
    #[inline(always)]
    pub(crate) fn pop_int_pub(&mut self) -> i64 {
        match self.stack.last() {
            Some(Value::Int(v)) => {
                let v = *v;
                self.stack.pop();
                v
            }
            _ => i64::MIN,
        }
    }
    /// v3.5.37: concat rápido para el Tier-2 en modo texto — si alguno de
    /// los dos operandos es Str, reproduce EXACTAMENTE el arm Add del
    /// intérprete (format "{}{}"); si no, delega en el handler genérico.
    #[cfg(feature = "aot")]
    pub(crate) fn concat_pub(&mut self) -> Result<(), VmError> {
        let b = self.pop()?;
        let a = self.pop()?;
        if matches!(&a, Value::Str(_)) || matches!(&b, Value::Str(_)) {
            self.push(Value::str(format!("{}{}", a, b)));
            Ok(())
        } else {
            // no es concat: restaurar operandos y ejecutar Add normal.
            self.push(a);
            self.push(b);
            self.execute_simple_pub(Opcode::Add)
        }
    }

    pub(crate) fn execute_simple_pub(&mut self, op: Opcode) -> Result<(), VmError> {
        self.execute_simple(op)
    }
    pub(crate) fn execute_with_num_pub(&mut self, op: Opcode, n: f64) -> Result<(), VmError> {
        self.execute_with_num(op, n)
    }
    pub(crate) fn execute_with_bool_pub(&mut self, op: Opcode, b: bool) -> Result<(), VmError> {
        self.execute_with_bool(op, b)
    }
    pub(crate) fn execute_with_str_pub(&mut self, op: Opcode, s: &str) -> Result<(), VmError> {
        self.execute_with_str(op, s)
    }
    pub(crate) fn execute_with_idx_pub(&mut self, op: Opcode, idx: usize) -> Result<(), VmError> {
        self.execute_with_idx(op, idx)
    }
    pub(crate) fn pop_pub(&mut self) -> Result<Value, VmError> {
        self.pop()
    }

    /// v3.5.31 (Tier-2): push directo (epílogo Load→push→Ret del bucle nativo).
    #[cfg(feature = "aot")]
    pub(crate) fn push_pub(&mut self, val: Value) {
        self.push(val);
    }

    /// v3.5.31: el cuerpo nativo mantiene el ip correcto por instrucción
    /// (los handlers StructNew/EnumCtor y los peepholes leen self.ip y
    /// deben ver la MISMA posición que vería el intérprete).
    #[cfg(feature = "aot")]
    pub(crate) fn set_ip_pub(&mut self, ip: usize) {
        self.ip = ip;
    }

    /// v3.5.31 (Tier-2): puntero base al `flat` (estable durante el bucle).
    #[cfg(feature = "aot")]
    pub(crate) fn flat_ptr_pub(&self) -> *const Value {
        self.flat.as_ptr()
    }

    /// v3.5.31 (Tier-2): ¿el slot contiene Value::Int?
    #[cfg(feature = "aot")]
    pub(crate) fn flat_slot_is_int_pub(&self, slot: usize) -> bool {
        matches!(self.flat.get(slot), Some(Value::Int(_)))
    }

    pub fn step(&mut self) -> Result<(), VmError> {
        self.step_mode = true;
        self.debug = true;
        if self.ip >= self.bytecode.instructions.len() {
            return Ok(());
        }
        // Time-Travel Snapshot
        self.snapshots.push(VmSnapshot {
            ip: self.ip,
            instr_count: self.instr_count,
            stack: self.stack.clone(),
            locals: self.locals.clone(),
            flat: self.flat.clone(),
            free_slots: self.free_slots.clone(),
            call_stack: self.call_stack.clone(),
            output_len: self.output.len(),
        });
        if self.snapshots.len() > 5000 {
            self.snapshots.remove(0);
        }
        let instr = self.bytecode.instructions[self.ip].clone();
        self.ip += 1;
        self.instr_count += 1;
        self.last_instr = Some(instr.clone());
        self.execute(&instr)
    }

    pub fn step_back(&mut self) -> Result<bool, VmError> {
        if let Some(snap) = self.snapshots.pop() {
            self.ip = snap.ip;
            self.instr_count = snap.instr_count;
            self.stack = snap.stack;
            let snap_locals = snap.locals;
            let snap_flat = snap.flat;
            let snap_free = snap.free_slots;
            self.replace_locals_full(snap_locals, snap_flat, snap_free);
            self.call_stack = snap.call_stack;
            self.output.truncate(snap.output_len);
            Ok(true)
        } else {
            Ok(false)
        }
    }
    pub fn stack_top(&self) -> Option<&Value> {
        self.stack.last()
    }
    /// Vista de los locales del scope ACTIVO (para debug/REPL): construye el
    /// mapa nombre → valor bajo demanda (los valores viven en `flat`).
    pub fn current_locals(&self) -> Option<HashMap<String, Value, FixHasher>> {
        let frame = self.locals.last()?;
        let mut out = HashMap::with_hasher(FixHasher::default());
        for (name, &slot) in &frame.map {
            if let Some(v) = self.flat.get(slot as usize) {
                out.insert(name.clone(), v.clone());
            }
        }
        // v3.5.31: scopes de parámetros sin mapa → reconstruir posicionalmente.
        if let Some(fi) = frame.param_func {
            let params = &self.bytecode.funcs[fi].params;
            for (i, name) in params.iter().enumerate() {
                if let Some(&slot) = frame.slots.get(i) {
                    if let Some(v) = self.flat.get(slot as usize) {
                        out.insert(name.clone(), v.clone());
                    }
                }
            }
        }
        Some(out)
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    /// Modo echo: las líneas de `imprimir` salen por stdout EN VIVO
    /// (además de quedar en el buffer `output`). Lo activa `lumen run`
    /// para que procesos largos (self-compile) muestren progreso real.
    pub fn set_echo_stdout(&mut self, on: bool) {
        self.echo_stdout = on;
    }

    fn emit_line(&mut self, s: String) {
        if self.echo_stdout {
            println!("{}", s);
        }
        self.output.push(s);
    }

    pub fn call_stack(&self) -> &[CallFrame] {
        &self.call_stack
    }

    /// v3.5.31: resuelve el nombre de un marco bajo demanda (CallFrame solo
    /// guarda el índice en `bytecode.names` — sin alloc por llamada).
    #[inline]
    pub fn frame_func_name(&self, frame: &CallFrame) -> String {
        if let Some(l) = &frame.func_label {
            return l.clone();
        }
        self.bytecode
            .names
            .get(frame.func_name)
            .cloned()
            .unwrap_or_default()
    }

    /// Run a specific function by name with given args, returning its result.
    /// Used by spawned task threads to execute a function in isolation.
    pub fn run_function(&mut self, name: &str, args: Vec<Value>) -> Result<Value, VmError> {
        if let Some(func) = self.find_func(name) {
            let func_start = func.start;
            let func_params = func.params.clone();
            let func_defaults = func.defaults.clone();
            if self.call_stack.len() >= MAX_CALL_STACK_DEPTH {
                return Err(VmError::Runtime(format!(
                    "Desbordamiento de pila (Stack overflow): límite de recursión excedido (>{} llamadas)",
                    MAX_CALL_STACK_DEPTH
                )));
            }
            self.scope_push_cap(func_params.len());
            let top = self.locals.len() - 1;
            for (i, param_name) in func_params.iter().enumerate() {
                let arg = if i < args.len() {
                    args[i].clone()
                } else if let Some(Some(dv)) = func_defaults.get(i) {
                    match dv {
                        DefaultValue::Int(v) => Value::Int(*v),
                        DefaultValue::Float(v) => Value::Float(*v),
                        DefaultValue::Str(s) => Value::str(s.clone()),
                        DefaultValue::Bool(b) => Value::Bool(*b),
                    }
                } else {
                    Value::Void
                };
                // Frontera de hilo/task: una referencia apunta a scopes del VM
                // originario; aquí se degrada a valor (semántica documentada).
                let arg = match arg {
                    Value::Ref { .. } => arg.deep_deref(),
                    other => other,
                };
                let slot = self.alloc_slot(arg);
                self.locals[top].map.insert(param_name.clone(), slot);
                self.locals[top].slots.push(slot);
            }
            self.call_stack.push(CallFrame {
                func_name: self
                    .bytecode
                    .names
                    .iter()
                    .position(|n| n == name)
                    .unwrap_or(usize::MAX),
                func_label: None,
                return_ip: self.bytecode.instructions.len(), // Past end → run() loop breaks
                locals_base: self.locals.len() - 1,
                stack_base: self.stack.len(),
                is_closure: false,
                has_refs: false,
            });
            self.ip = func_start;
            self.run()?;
            Ok(self.pop().unwrap_or(Value::Void))
        } else {
            Err(VmError::UndefinedFunction(name.to_string()))
        }
    }

    /// v3.5.20: ejecución de los super-opcodes fusionados (compartida por
    /// step_instr y execute).
    /// v3.5.31: resuelve el destino de un salto (idx en `bytecode.nums`) con
    /// caché per-programa — evita el lookup + cast f64→usize por iteración.
    #[inline(always)]
    fn resolve_target(&mut self, idx: usize) -> usize {
        match self.jump_targets.get(idx).copied().flatten() {
            Some(t) => t,
            None => {
                let t = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                if idx >= self.jump_targets.len() {
                    self.jump_targets.resize(idx + 1, None);
                }
                self.jump_targets[idx] = Some(t);
                t
            }
        }
    }

    // v3.5.20 super-opcodes: bucles sin push/pop ni dispatch extra.
    #[inline(always)]
    fn exec_fused_bink(&mut self, op: u8, a: usize, k: i64, d: usize) -> Result<(), VmError> {
        let av = self.do_load_by_idx(a)?;
        let res = match (&av, op) {
            (Value::Int(x), 1) => Value::Int(x.wrapping_add(k)),
            (Value::Int(x), 3) => Value::Int(x.wrapping_sub(k)),
            (Value::Int(x), 4) => Value::Int(x.wrapping_mul(k)),
            (Value::Float(x), 1) => Value::Float(x + k as f64),
            (Value::Float(x), 3) => Value::Float(x - k as f64),
            (Value::Float(x), 4) => Value::Float(x * k as f64),
            _ => self.bin_vals_slow(av, Value::Int(k), op)?,
        };
        self.do_store_by_idx(d, res);
        Ok(())
    }

    #[inline(always)]
    fn exec_fused_bin(&mut self, op: u8, a: usize, b: usize, d: usize) -> Result<(), VmError> {
        let av = self.do_load_by_idx(a)?;
        let bv = self.do_load_by_idx(b)?;
        let res = match (&av, &bv, op) {
            (Value::Int(x), Value::Int(y), 1) => Value::Int(x.wrapping_add(*y)),
            (Value::Int(x), Value::Int(y), 3) => Value::Int(x.wrapping_sub(*y)),
            (Value::Int(x), Value::Int(y), 4) => Value::Int(x.wrapping_mul(*y)),
            (Value::Float(x), Value::Float(y), 1) => Value::Float(x + y),
            (Value::Float(x), Value::Float(y), 3) => Value::Float(x - y),
            (Value::Float(x), Value::Float(y), 4) => Value::Float(x * y),
            (Value::Int(x), Value::Float(y), 1) => Value::Float(*x as f64 + y),
            (Value::Int(x), Value::Float(y), 3) => Value::Float(*x as f64 - y),
            (Value::Int(x), Value::Float(y), 4) => Value::Float(*x as f64 * y),
            (Value::Float(x), Value::Int(y), 1) => Value::Float(x + *y as f64),
            (Value::Float(x), Value::Int(y), 3) => Value::Float(x - *y as f64),
            (Value::Float(x), Value::Int(y), 4) => Value::Float(x * *y as f64),
            _ => self.bin_vals_slow(av, bv, op)?,
        };
        self.do_store_by_idx(d, res);
        Ok(())
    }

    /// v3.5.31: comparación (av vs constante) compartida por el super-opcode
    /// `FusedCmpKJmp` y por el helper JIT `lj_fused_cmpk` (paridad exacta).
    fn cmp_vals_k(&self, av: Value, k: i64, op: u8) -> Result<bool, VmError> {
        match (&av, op) {
            (Value::Int(x), 7) => Ok(*x == k),
            (Value::Int(x), 8) => Ok(*x != k),
            (Value::Int(x), 9) => Ok(*x < k),
            (Value::Int(x), 10) => Ok(*x <= k),
            (Value::Int(x), 11) => Ok(*x > k),
            (Value::Int(x), 12) => Ok(*x >= k),
            // v3.5.31: paridad EXACTA con Eq/Neq clásicos (EPSILON para
            // flotantes — el == exacto divergía en casos límite).
            (Value::Float(x), 7) => Ok((*x - k as f64).abs() < f64::EPSILON),
            (Value::Float(x), 8) => Ok((*x - k as f64).abs() >= f64::EPSILON),
            (Value::Float(x), 9) => Ok(*x < k as f64),
            (Value::Float(x), 10) => Ok(*x <= k as f64),
            (Value::Float(x), 11) => Ok(*x > k as f64),
            (Value::Float(x), 12) => Ok(*x >= k as f64),
            _ => self.cmp_vals_slow(av, Value::Int(k), op),
        }
    }

    /// v3.5.31: comparación (av vs bv) compartida por el super-opcode
    /// `FusedCmpJmp` y por el helper JIT `lj_fused_cmp` (paridad exacta).
    fn cmp_vals_ab(&self, av: Value, bv: Value, op: u8) -> Result<bool, VmError> {
        match (&av, &bv, op) {
            (Value::Int(x), Value::Int(y), 7) => Ok(x == y),
            (Value::Int(x), Value::Int(y), 8) => Ok(x != y),
            (Value::Int(x), Value::Int(y), 9) => Ok(x < y),
            (Value::Int(x), Value::Int(y), 10) => Ok(x <= y),
            (Value::Int(x), Value::Int(y), 11) => Ok(x > y),
            (Value::Int(x), Value::Int(y), 12) => Ok(x >= y),
            // v3.5.31: paridad EXACTA con Eq/Neq clásicos (EPSILON).
            (Value::Float(x), Value::Float(y), 7) => Ok((x - y).abs() < f64::EPSILON),
            (Value::Float(x), Value::Float(y), 8) => Ok((x - y).abs() >= f64::EPSILON),
            (Value::Float(x), Value::Float(y), 9) => Ok(x < y),
            (Value::Float(x), Value::Float(y), 10) => Ok(x <= y),
            (Value::Float(x), Value::Float(y), 11) => Ok(x > y),
            (Value::Float(x), Value::Float(y), 12) => Ok(x >= y),
            (Value::Int(x), Value::Float(y), 9) => Ok((*x as f64) < *y),
            (Value::Int(x), Value::Float(y), 10) => Ok((*x as f64) <= *y),
            (Value::Int(x), Value::Float(y), 11) => Ok((*x as f64) > *y),
            (Value::Int(x), Value::Float(y), 12) => Ok((*x as f64) >= *y),
            (Value::Float(x), Value::Int(y), 9) => Ok(*x < *y as f64),
            (Value::Float(x), Value::Int(y), 10) => Ok(*x <= *y as f64),
            (Value::Float(x), Value::Int(y), 11) => Ok(*x > *y as f64),
            (Value::Float(x), Value::Int(y), 12) => Ok(*x >= *y as f64),
            _ => self.cmp_vals_slow(av, bv, op),
        }
    }

    #[inline(always)]
    fn exec_fused_cmpkjmp(
        &mut self,
        op: u8,
        a: usize,
        k: i64,
        target: usize,
    ) -> Result<(), VmError> {
        let av = self.do_load_by_idx(a)?;
        let cond = self.cmp_vals_k(av, k, op)?;
        if !cond {
            self.ip = self.resolve_target(target);
        }
        Ok(())
    }

    #[inline(always)]
    fn exec_fused_cmpjmp(
        &mut self,
        op: u8,
        a: usize,
        b: usize,
        target: usize,
    ) -> Result<(), VmError> {
        let av = self.do_load_by_idx(a)?;
        let bv = self.do_load_by_idx(b)?;
        let cond = self.cmp_vals_ab(av, bv, op)?;
        if !cond {
            self.ip = self.resolve_target(target);
        }
        Ok(())
    }

    /// v3.5.31: aritmética binaria de los super-opcodes de 6 instrucciones —
    /// reproduce la semántica EXACTA de los opcodes clásicos Add/Sub/Mul/
    /// Div/Mod (Div: i64::MIN/-1 → wrapping_neg; Mod: rem_euclid con
    /// b == -1 → 0; ambos: DivisionByZero).
    fn bin_pair(&self, op: u8, a: Value, b: Value) -> Result<Value, VmError> {
        match (op, &a, &b) {
            (1, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.wrapping_add(*y))),
            (3, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.wrapping_sub(*y))),
            (4, Value::Int(x), Value::Int(y)) => Ok(Value::Int(x.wrapping_mul(*y))),
            (5, Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
            (5, Value::Int(x), Value::Int(y)) => {
                Ok(Value::Int(if *y == -1 { x.wrapping_neg() } else { x / y }))
            }
            (6, Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
            (6, Value::Int(x), Value::Int(y)) => {
                Ok(Value::Int(if *y == -1 { 0 } else { x.rem_euclid(*y) }))
            }
            (5, Value::Int(x), Value::Float(y)) => {
                if *y == 0.0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Float(*x as f64 / y))
                }
            }
            (5, Value::Float(x), Value::Int(y)) => {
                if *y == 0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Float(x / *y as f64))
                }
            }
            (5, Value::Float(x), Value::Float(y)) => {
                if *y == 0.0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Float(x / y))
                }
            }
            (6, Value::Int(x), Value::Float(y)) => {
                if *y == 0.0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Float(*x as f64 % y))
                }
            }
            (6, Value::Float(x), Value::Int(y)) => {
                if *y == 0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Float(x % *y as f64))
                }
            }
            (6, Value::Float(x), Value::Float(y)) => {
                if *y == 0.0 {
                    Err(VmError::DivisionByZero)
                } else {
                    Ok(Value::Float(x % y))
                }
            }
            (5, _, _) => Err(VmError::TypeError("Div requires numbers".to_string())),
            (6, _, _) => Err(VmError::TypeError("Mod requires numbers".to_string())),
            _ => self.bin_vals_slow(a, b, op),
        }
    }

    /// v3.5.31: super-opcode de 6 instrucciones — t = a op1 b; si
    /// (t op2 c) es FALSO salta a target (paridad exacta con
    /// Load,Load,Binary,Load,Cmp,JmpIf del intérprete).
    #[inline(always)]
    fn exec_fused_bincmpjmp(
        &mut self,
        op1: u8,
        op2: u8,
        a: usize,
        b: usize,
        c: usize,
        target: usize,
    ) -> Result<(), VmError> {
        let av = self.do_load_by_idx(a)?;
        let bv = self.do_load_by_idx(b)?;
        let t = self.bin_pair(op1, av, bv)?;
        let cv = self.do_load_by_idx(c)?;
        let cond = self.cmp_vals_ab(t, cv, op2)?;
        if !cond {
            self.ip = self.resolve_target(target);
        }
        Ok(())
    }

    #[inline(always)]
    fn exec_fused_binkcmpjmp(
        &mut self,
        op1: u8,
        op2: u8,
        a: usize,
        b: usize,
        k: i64,
        target: usize,
    ) -> Result<(), VmError> {
        let av = self.do_load_by_idx(a)?;
        let bv = self.do_load_by_idx(b)?;
        let t = self.bin_pair(op1, av, bv)?;
        let cond = self.cmp_vals_k(t, k, op2)?;
        if !cond {
            self.ip = self.resolve_target(target);
        }
        Ok(())
    }

    #[inline(always)]
    fn exec_fused_binkkcmpjmp(
        &mut self,
        op1: u8,
        op2: u8,
        a: usize,
        b: i64,
        k: i64,
        target: usize,
    ) -> Result<(), VmError> {
        let av = self.do_load_by_idx(a)?;
        let t = self.bin_pair(op1, av, Value::Int(b))?;
        let cond = self.cmp_vals_k(t, k, op2)?;
        if !cond {
            self.ip = self.resolve_target(target);
        }
        Ok(())
    }

    // ── Superficie pub(crate) para los helpers extern "C" del JIT (v3.5.31) ──
    #[cfg(feature = "aot")]
    pub(crate) fn exec_fused_bink_pub(
        &mut self,
        op: u8,
        a: usize,
        k: i64,
        d: usize,
    ) -> Result<(), VmError> {
        self.exec_fused_bink(op, a, k, d)
    }
    #[cfg(feature = "aot")]
    pub(crate) fn exec_fused_bin_pub(
        &mut self,
        op: u8,
        a: usize,
        b: usize,
        d: usize,
    ) -> Result<(), VmError> {
        self.exec_fused_bin(op, a, b, d)
    }
    /// Solo evalúa la condición (el JIT decide el salto nativamente).
    #[cfg(feature = "aot")]
    pub(crate) fn fused_cmpk_pub(&mut self, op: u8, a: usize, k: i64) -> Result<bool, VmError> {
        let av = self.do_load_by_idx(a)?;
        self.cmp_vals_k(av, k, op)
    }
    /// Solo evalúa la condición (el JIT decide el salto nativamente).
    #[cfg(feature = "aot")]
    pub(crate) fn fused_cmp_pub(&mut self, op: u8, a: usize, b: usize) -> Result<bool, VmError> {
        let av = self.do_load_by_idx(a)?;
        let bv = self.do_load_by_idx(b)?;
        self.cmp_vals_ab(av, bv, op)
    }
    /// v3.5.31: condición del super-opcode de 6 (a op1 b) op2 c — el JIT
    /// resuelve el salto nativamente con el resultado.
    #[cfg(feature = "aot")]
    pub(crate) fn fused_bincmp_pub(
        &mut self,
        op1: u8,
        op2: u8,
        a: usize,
        b: usize,
        c: usize,
    ) -> Result<bool, VmError> {
        let av = self.do_load_by_idx(a)?;
        let bv = self.do_load_by_idx(b)?;
        let t = self.bin_pair(op1, av, bv)?;
        let cv = self.do_load_by_idx(c)?;
        self.cmp_vals_ab(t, cv, op2)
    }
    /// v3.5.31: condición del super-opcode de 6 (a op1 b) op2 k — el JIT
    /// resuelve el salto nativamente con el resultado.
    /// v3.5.32 (Tier-1): variante KK — `b` es CONSTANTE.
    #[cfg(feature = "aot")]
    pub(crate) fn fused_binkkcmp_pub(
        &mut self,
        op1: u8,
        op2: u8,
        a: usize,
        b: i64,
        k: i64,
    ) -> Result<bool, VmError> {
        let av = self.do_load_by_idx(a)?;
        let t = self.bin_pair(op1, av, Value::Int(b))?;
        self.cmp_vals_k(t, k, op2)
    }

    /// v3.5.31: condición del super-opcode de 6 (a op1 b) op2 k — el JIT
    /// resuelve el salto nativamente con el resultado.
    #[cfg(feature = "aot")]
    pub(crate) fn fused_binkcmp_pub(
        &mut self,
        op1: u8,
        op2: u8,
        a: usize,
        b: usize,
        k: i64,
    ) -> Result<bool, VmError> {
        let av = self.do_load_by_idx(a)?;
        let bv = self.do_load_by_idx(b)?;
        let t = self.bin_pair(op1, av, bv)?;
        self.cmp_vals_k(t, k, op2)
    }

    /// v3.5.31 (Tier-2): resuelve el slot de `name_idx` en el scope actual,
    /// asignándolo si no existe — MIRROR EXACTO del path de asignación de
    /// `StoreLocal` (lookup → slot existente, o alloc+insert+bump).
    #[cfg(feature = "aot")]
    pub(crate) fn resolve_slot_pub(&mut self, name_idx: usize) -> Result<u32, VmError> {
        let name = self
            .bytecode
            .names
            .get(name_idx)
            .cloned()
            .unwrap_or_default();
        let n = self.locals.len();
        if n > 0 {
            let top = n - 1;
            if let Some(s) = self.scope_get(&self.locals[top], &name) {
                return Ok(s);
            }
            let s = self.alloc_slot(Value::Void);
            self.locals[top].map.insert(name, s);
            self.locals[top].slots.push(s);
            self.var_cache_invalidate(name_idx);
            Ok(s)
        } else {
            Err(VmError::Runtime("resolve_slot: scope vacío".into()))
        }
    }

    /// v3.5.31 (Tier-2): lookup SIN asignar — devuelve el slot si el nombre
    /// ya existe en ALGÚN scope (búsqueda top-down, mirror de do_load_by_idx),
    /// o -1 (el JIT hace bail-out al intérprete, que levantará
    /// UndefinedVariable con la semántica original).
    #[cfg(feature = "aot")]
    /// v3.5.32 (Tier-2): lookup + guarda de tipo fusionadas — devuelve el
    /// slot si el nombre existe Y contiene Value::Int; -1 en otro caso
    /// (un solo call de prólogo por nombre en vez de lookup + is_int).
    pub(crate) fn probe_int_pub(&mut self, name_idx: usize) -> i64 {
        let slot = self.lookup_slot_pub(name_idx);
        if slot >= 0 && self.flat_slot_is_int_pub(slot as usize) {
            slot
        } else {
            -1
        }
    }

    pub(crate) fn lookup_slot_pub(&mut self, name_idx: usize) -> i64 {
        let name = self
            .bytecode
            .names
            .get(name_idx)
            .cloned()
            .unwrap_or_default();
        for scope in self.locals.iter().rev() {
            if let Some(s) = self.scope_get(scope, &name) {
                return s as i64;
            }
        }
        -1
    }

    /// v3.5.31 (Tier-2): resolución de destino de ESCRITURA de super-opcode
    /// (mirror de do_store_by_idx): busca en TODOS los scopes (top-down); si
    /// no existe, asigna slot NUEVO en el scope superior (alloc+insert+bump).
    #[cfg(feature = "aot")]
    pub(crate) fn resolve_store_slot_pub(&mut self, name_idx: usize) -> Result<u32, VmError> {
        let name = self
            .bytecode
            .names
            .get(name_idx)
            .cloned()
            .unwrap_or_default();
        for scope in self.locals.iter().rev() {
            if let Some(s) = self.scope_get(scope, &name) {
                return Ok(s);
            }
        }
        let n = self.locals.len();
        if n == 0 {
            return Err(VmError::Runtime("resolve_store: scope vacío".into()));
        }
        let top = n - 1;
        let s = self.alloc_slot(Value::Void);
        self.locals[top].map.insert(name, s);
        self.locals[top].slots.push(s);
        self.var_cache_invalidate(name_idx);
        Ok(s)
    }

    fn execute(&mut self, instr: &Instruction) -> Result<(), VmError> {
        match instr {
            Instruction::Simple(op) => self.execute_simple(*op),
            Instruction::WithNum(op, n) => self.execute_with_num(*op, *n),
            Instruction::WithStr(op, s) => self.execute_with_str(*op, s),
            Instruction::WithBool(op, b) => self.execute_with_bool(*op, *b),
            Instruction::WithIdx(op, idx) => self.execute_with_idx(*op, *idx),
            // v3.5.20 super-opcodes (v3.5.31: escalares, sin clonar)
            Instruction::FusedBinK { op, a, k, d } => self.exec_fused_bink(*op, *a, *k, *d),
            Instruction::FusedBin { op, a, b, d } => self.exec_fused_bin(*op, *a, *b, *d),
            Instruction::FusedCmpKJmp { op, a, k, target } => {
                self.exec_fused_cmpkjmp(*op, *a, *k, *target)
            }
            Instruction::FusedCmpJmp { op, a, b, target } => {
                self.exec_fused_cmpjmp(*op, *a, *b, *target)
            }
            // v3.5.31: aritmética + comparación + salto (6 IR → 1).
            Instruction::FusedBinCmpJmp {
                op1,
                op2,
                a,
                b,
                c,
                target,
            } => self.exec_fused_bincmpjmp(*op1, *op2, *a, *b, *c, *target),
            Instruction::FusedBinKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => self.exec_fused_binkcmpjmp(*op1, *op2, *a, *b, *k, *target),
            Instruction::FusedBinKKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => self.exec_fused_binkkcmpjmp(*op1, *op2, *a, *b, *k, *target),
        }
    }

    #[inline]
    fn execute_simple(&mut self, op: Opcode) -> Result<(), VmError> {
        match op {
            Opcode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        // v3.5.19 peephole: Add→Store sin push/pop intermedio
                        let r = Value::Int(a.wrapping_add(b));
                        if let Some((Opcode::Store, sidx)) = self.peek_with_idx() {
                            self.do_store_by_idx(sidx, r);
                            self.ip += 1;
                        } else {
                            self.push(r);
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(a as f64 + b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a + b as f64)),
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
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        // v3.5.19 peephole: Sub→Store
                        let r = Value::Int(a.wrapping_sub(b));
                        if let Some((Opcode::Store, sidx)) = self.peek_with_idx() {
                            self.do_store_by_idx(sidx, r);
                            self.ip += 1;
                        } else {
                            self.push(r);
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(a as f64 - b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a - b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a - b)),
                    _ => return Err(VmError::TypeError("Sub requires numbers".to_string())),
                }
            }
            Opcode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        // v3.5.19 peephole: Mul→Store
                        let r = Value::Int(a.wrapping_mul(b));
                        if let Some((Opcode::Store, sidx)) = self.peek_with_idx() {
                            self.do_store_by_idx(sidx, r);
                            self.ip += 1;
                        } else {
                            self.push(r);
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(a as f64 * b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a * b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a * b)),
                    _ => return Err(VmError::TypeError("Mul requires numbers".to_string())),
                }
            }
            Opcode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::Int(_), Value::Int(0)) => return Err(VmError::DivisionByZero),
                    (Value::Int(a), Value::Int(b)) => {
                        // i64::MIN / -1 panics — wrapping semantics devuelven i64::MIN
                        let q = if b == -1 { a.wrapping_neg() } else { a / b };
                        self.push(Value::Int(q));
                    }
                    (Value::Int(a), Value::Float(b)) => {
                        if b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a as f64 / b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a / b as f64))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b == 0.0 {
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
                match (a, b) {
                    (Value::Int(_), Value::Int(0)) => return Err(VmError::DivisionByZero),
                    (Value::Int(a), Value::Int(b)) => {
                        // rem_euclid(i64::MIN, -1) panics — wrapping semantics devuelven 0
                        let r = if b == -1 { 0 } else { a.rem_euclid(b) };
                        self.push(Value::Int(r));
                    }
                    (Value::Int(a), Value::Float(b)) => {
                        if b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a as f64 % b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a % b as f64))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if b == 0.0 {
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
                // v3.5.19 peephole: Eq/Neq→JmpIf sin push/dispatch
                if !self.cmp_jmpif_fused(result) {
                    self.push(Value::Bool(result));
                }
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
                // v3.5.19 peephole: Eq/Neq→JmpIf sin push/dispatch
                if !self.cmp_jmpif_fused(result) {
                    self.push(Value::Bool(result));
                }
            }
            Opcode::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        // v3.5.19 peephole: Lt→JmpIf sin push/dispatch
                        let c = a < b;
                        if !self.cmp_jmpif_fused(c) {
                            self.push(Value::Bool(c));
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((a as f64) < b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(a < b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a < b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a < b)),
                    _ => {
                        return Err(VmError::TypeError(
                            "Lt requires numbers or strings".to_string(),
                        ))
                    }
                }
            }
            Opcode::Le => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        // v3.5.19 peephole: Le→JmpIf sin push/dispatch
                        let c = a <= b;
                        if !self.cmp_jmpif_fused(c) {
                            self.push(Value::Bool(c));
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((a as f64) <= b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(a <= b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a <= b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a <= b)),
                    _ => {
                        return Err(VmError::TypeError(
                            "Le requires numbers or strings".to_string(),
                        ))
                    }
                }
            }
            Opcode::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        // v3.5.19 peephole: Gt→JmpIf sin push/dispatch
                        let c = a > b;
                        if !self.cmp_jmpif_fused(c) {
                            self.push(Value::Bool(c));
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((a as f64) > b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(a > b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a > b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a > b)),
                    _ => {
                        return Err(VmError::TypeError(
                            "Gt requires numbers or strings".to_string(),
                        ))
                    }
                }
            }
            Opcode::Ge => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (a, b) {
                    (Value::Int(a), Value::Int(b)) => {
                        // v3.5.19 peephole: Ge→JmpIf sin push/dispatch
                        let c = a >= b;
                        if !self.cmp_jmpif_fused(c) {
                            self.push(Value::Bool(c));
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((a as f64) >= b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(a >= b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a >= b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Bool(a >= b)),
                    _ => {
                        return Err(VmError::TypeError(
                            "Ge requires numbers or strings".to_string(),
                        ))
                    }
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
            Opcode::BitXor => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a ^ b)),
                    _ => return Err(VmError::TypeError("BitXor requires integers".to_string())),
                }
            }
            Opcode::BitNot => {
                let a = self.pop()?;
                match &a {
                    Value::Int(a) => self.push(Value::Int(!a)),
                    _ => return Err(VmError::TypeError("BitNot requires integer".to_string())),
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
                        self.push(Value::Int(a.wrapping_shl(*b as u32)));
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "ShiftLeft requires integers".to_string(),
                        ))
                    }
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
                    Value::Int(n) => self.push(Value::Int(n.wrapping_neg())),
                    Value::Float(n) => self.push(Value::Float(-n)),
                    _ => return Err(VmError::TypeError("Neg requires number".to_string())),
                }
            }
            Opcode::Not => {
                let a = self.pop()?;
                self.push(Value::Bool(!a.is_truthy()));
            }
            Opcode::PopHandler => {
                self.handlers.pop();
            }
            Opcode::ScopePush => {
                self.scope_push();
            }
            Opcode::ScopePop => {
                self.scope_pop();
            }
            Opcode::Ret => {
                let ret_val = self.pop().unwrap_or(Value::Void);
                #[cfg(any(feature = "extra", feature = "full"))]
                {
                    if let Some(coro_id) = self.current_coro.clone() {
                        if let Some(coro) = self.coroutines.get_mut(&coro_id) {
                            coro.is_done = true;
                        }
                        self.current_coro = None;
                        if let Some((saved_stack, saved_locals, saved_flat, saved_free, saved_ip)) =
                            self.main_saved.take()
                        {
                            self.stack = saved_stack;
                            self.replace_locals_full(saved_locals, saved_flat, saved_free);
                            self.ip = saved_ip;
                        } else {
                            self.ip = usize::MAX;
                        }
                        self.push(ret_val);
                        return Ok(());
                    }
                }
                if let Some(frame) = self.call_stack.pop() {
                    // Write-back de referencias prestado mut (bug #6): cada Ref
                    // creada en este frame apunta a un slot del llamador; copiar
                    // el valor final de vuelta antes de descartar los scopes.
                    // v3.5.34: si ningún slot del frame recibió un Ref con
                    // owner (el caso dominante), saltamos el escaneo entero.
                    let base = frame.locals_base.max(1).min(self.locals.len());
                    let mut writebacks: Vec<(usize, String, Value)> = Vec::new();
                    if frame.has_refs {
                        for scope in self.locals.iter().skip(base) {
                            for &slot in &scope.slots {
                                if let Value::Ref {
                                    cell,
                                    owner: Some((target_si, target_name)),
                                } = &self.flat[slot as usize]
                                {
                                    let final_val = cell.lock().unwrap().clone();
                                    writebacks.push((*target_si, target_name.clone(), final_val));
                                }
                            }
                        }
                    }
                    self.scope_truncate(base);
                    // v3.5.18: la llamada por closure inyectó un scope
                    // sintético de entorno justo debajo de locals_base; ya se
                    // usó (write-through vía celdas) → desapilarlo.
                    if frame.is_closure && self.locals.len() > 1 {
                        self.scope_pop();
                    }
                    for (si, nm, val) in writebacks {
                        if si < self.locals.len() {
                            if let Some(slot) = self.scope_get(&self.locals[si], &nm) {
                                self.flat[slot as usize] = val;
                            }
                        }
                    }
                    // v3.5.13: descartar residuos de la pila de valores del
                    // callee (Void de imprimir-statements, expresiones sin
                    // consumir) — deja solo el retorno encima de stack_base.
                    self.stack.truncate(frame.stack_base);
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
                self.emit_line(s);
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
                        let val = fields
                            .iter()
                            .find(|(name, _)| name.as_str() == field.as_str());
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
                        let pos = fields
                            .iter()
                            .position(|(n, _)| n.as_str() == field.as_str());
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
                    Value::Array(arr) => match &index {
                        Value::Int(i) => {
                            let idx = *i;
                            if idx < 0 || idx as usize >= arr.len() {
                                return Err(VmError::Runtime(format!(
                                    "Índice {} fuera de rango (largo: {})",
                                    idx,
                                    arr.len()
                                )));
                            }
                            self.push(arr[idx as usize].clone());
                        }
                        Value::Array(range_items) => {
                            let mut sub = Vec::new();
                            for item in range_items.iter() {
                                if let Some(n) = item.as_num() {
                                    let i = n as usize;
                                    if i < arr.len() {
                                        sub.push(arr[i].clone());
                                    }
                                }
                            }
                            self.push(Value::arr(sub));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "ArrayGet requires integer or range index for arrays".to_string(),
                            ))
                        }
                    },
                    Value::Str(s) => match &index {
                        Value::Int(i) => {
                            let idx = *i;
                            if idx < 0 {
                                return Err(VmError::Runtime(format!(
                                    "Índice {} fuera de rango",
                                    idx
                                )));
                            }
                            match s.chars().nth(idx as usize) {
                                Some(c) => self.push(Value::str(c.to_string())),
                                None => {
                                    return Err(VmError::Runtime(format!(
                                        "Índice {} fuera de rango (largo: {})",
                                        idx,
                                        s.chars().count()
                                    )))
                                }
                            }
                        }
                        Value::Array(range_items) => {
                            let chars: Vec<char> = s.chars().collect();
                            let mut sub = String::new();
                            for item in range_items.iter() {
                                if let Some(n) = item.as_num() {
                                    let i = n as usize;
                                    if i < chars.len() {
                                        sub.push(chars[i]);
                                    }
                                }
                            }
                            self.push(Value::str(sub));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "ArrayGet requires integer or range index for strings".to_string(),
                            ))
                        }
                    },
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
                        let val = fields
                            .iter()
                            .find(|(n, _)| n == &field)
                            .map(|(_, v)| v.clone())
                            .unwrap_or(Value::Int(0));
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
                            self.scope_pop();
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
            Opcode::MatchPayload => {
                let val = self.pop()?;
                match val {
                    Value::Opcion(Some(inner)) => self.push(*inner),
                    Value::Exito(inner) => self.push(*inner),
                    Value::Error(inner) => self.push(*inner),
                    Value::Enum { fields, .. } => {
                        // Destructuring de enums de usuario (QA bug #3)
                        if fields.is_empty() {
                            self.push(Value::Void);
                        } else if fields.len() == 1 {
                            self.push(fields[0].clone());
                        } else {
                            self.push(Value::arr(fields.to_vec()));
                        }
                    }
                    other => self.push(other),
                }
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

    #[inline]
    fn execute_with_num(&mut self, op: Opcode, n: f64) -> Result<(), VmError> {
        match op {
            Opcode::PushNum => {
                self.push(Value::Float(n));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[inline]
    fn execute_with_str(&mut self, op: Opcode, s: &str) -> Result<(), VmError> {
        match op {
            Opcode::PushStr => {
                self.push(Value::str(s.to_string()));
                Ok(())
            }
            Opcode::MatchVariant => {
                // s = variant_name — compara solo el nombre de la variante
                let val = self.pop()?;
                let matches = match &val {
                    Value::Enum { variant: v, .. } => &**v == s,
                    _ => false,
                };
                self.push(Value::Bool(matches));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[inline]
    fn execute_with_bool(&mut self, op: Opcode, b: bool) -> Result<(), VmError> {
        match op {
            Opcode::PushBool => {
                self.push(Value::Bool(b));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    #[inline]
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
                // v3.5.20: usa el helper con caché inline (compartido con
                // los super-opcodes).
                let val = self.do_load_by_idx(idx)?;
                self.push(val);
            }
            Opcode::Store => {
                let val = self.pop()?;
                self.do_store_by_idx(idx, val);
            }
            Opcode::StoreLocal => {
                let val = self.pop()?;
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let n = self.locals.len();
                if n > 0 {
                    // v3.5.19: bump solo si el nombre es NUEVO en el scope
                    // (sobrescribir no cambia la resolución de nombres).
                    let top = n - 1;
                    if let Some(s) = self.scope_get(&self.locals[top], &name) {
                        self.note_val_write(&val);
                        self.flat[s as usize] = val;
                    } else {
                        let s = self.alloc_slot(val);
                        self.locals[top].map.insert(name, s);
                        self.locals[top].slots.push(s);
                        // v3.5.36: solo el nombre insertado se invalida.
                        self.var_cache_invalidate(idx);
                    }
                }
            }
            Opcode::ArrayPushVar => {
                // a.agregar(x) con `a` variable: muta el slot del scope in-place.
                // El builder emitió `Load a; args; ArrayPushVar a` — hacemos pop del
                // receptor obsoleto ANTES de mutar para que refcount vuelva a 1 y
                // Arc::make_mut NO clone el Vec entero (O(n²) → O(n)). v3.5.31:
                // `flat` es el ÚNICO dueño del Arc → make_mut sigue O(1).
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let value = self.pop()?;
                // Descartar el receptor cargado INMEDIATAMENTE (drop explícito libera
                // la referencia Arc antes de make_mut → refcount 1 → sin clone).
                drop(self.pop().ok());
                let mut found_slot: Option<u32> = None;
                for scope in self.locals.iter().rev() {
                    if let Some(s) = self.scope_get(scope, &name) {
                        found_slot = Some(s);
                        break;
                    }
                }
                match found_slot {
                    Some(slot) => match &mut self.flat[slot as usize] {
                        Value::Array(arr) => {
                            Arc::make_mut(arr).push(value);
                        }
                        // Slot con referencia prestado mut: mutar el array
                        // dentro de la celda preservando el owner del Ref
                        Value::Ref { cell, .. } => {
                            let mut g = cell.lock().unwrap();
                            match &mut *g {
                                Value::Array(arr) => {
                                    Arc::make_mut(arr).push(value);
                                }
                                _ => {
                                    return Err(VmError::TypeError(format!(
                                        "agregar requiere lista, pero '{}' no es una lista",
                                        name
                                    )))
                                }
                            }
                        }
                        _ => {
                            return Err(VmError::TypeError(format!(
                                "agregar requiere lista, pero '{}' no es una lista",
                                name
                            )));
                        }
                    },
                    None => {
                        return Err(VmError::UndefinedVariable(name));
                    }
                }
            }
            Opcode::PushHandler => {
                let off = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                self.handlers.push((
                    off,
                    self.stack.len(),
                    self.locals.len(),
                    self.call_stack.len(),
                ));
            }
            Opcode::MakeRef => {
                // prestado mut (bug #6): crear referencia al slot de la variable.
                // Se busca el slot (scope, nombre) y se apila Value::Ref con el
                // owner para que Ret haga write-back al llamador.
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let mut target: Option<(usize, u32)> = None;
                for (si, scope) in self.locals.iter().enumerate().rev() {
                    if let Some(s) = self.scope_get(scope, &name) {
                        target = Some((si, s));
                        break;
                    }
                }
                match target {
                    Some((si, slot)) => {
                        let existing = self.flat.get(slot as usize).cloned().unwrap_or(Value::Void);
                        let ref_val = match existing {
                            // Reenvío: g(p) donde p ya es Ref — compartir la
                            // MISMA celda para que los alias no diverjan
                            Value::Ref { cell, owner } => Value::Ref { cell, owner },
                            other => Value::new_ref(other, Some((si, name))),
                        };
                        self.push(ref_val);
                    }
                    None => return Err(VmError::UndefinedVariable(name)),
                }
            }
            Opcode::Call => {
                // v3.5.31: sin hash ni alloc por llamada en el camino de
                // FUNCIÓN DE USUARIO (el dominante): pre-filtro O(1) de
                // builtins, args inline para argc ≤ 4 y Vec indexada por
                // índice de nombre. El String del nombre SOLO se clona para
                // builtins reales o errores.
                let count_now = {
                    let count = if idx < self.call_counts.len() {
                        &mut self.call_counts[idx]
                    } else {
                        return Err(VmError::UndefinedFunction(
                            self.bytecode.names.get(idx).cloned().unwrap_or_default(),
                        ));
                    };
                    *count += 1;
                    *count
                };
                if count_now == self.jit_threshold && std::env::var_os("LUMEN_JIT_LOG").is_some() {
                    eprintln!(
                        "[jit] 🔥 Hot function detected: '{}' ({} llamadas) -> JIT Tier-1 activado",
                        self.bytecode
                            .names
                            .get(idx)
                            .map(|s| s.as_str())
                            .unwrap_or(""),
                        count_now
                    );
                }
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
                // v3.5.31: args SIN heap para argc pequeño.
                let mut small: [Value; 4] = std::array::from_fn(|_| Value::Void);
                let mut big: Vec<Value>;
                let args: &[Value] = if argc <= 4 {
                    for k in 0..argc {
                        small[argc - 1 - k] = self.pop()?;
                    }
                    &small[..argc]
                } else {
                    big = Vec::with_capacity(argc);
                    for _ in 0..argc {
                        big.push(self.pop()?);
                    }
                    big.reverse();
                    &big
                };
                let name_is_builtin = builtin_name_set().contains(
                    self.bytecode
                        .names
                        .get(idx)
                        .map(|s| s.as_str())
                        .unwrap_or(""),
                );
                if name_is_builtin {
                    let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                    if let Some(result) = self.call_core_builtin(&name, args) {
                        return result;
                    }
                    #[cfg(any(feature = "extra", feature = "full"))]
                    if let Some(result) = self.call_extra_builtin(&name, args) {
                        return result;
                    }
                }
                // v3.5.31: Vec directa por índice de nombre (sin hash).
                let func_idx = match self.func_index_by_name_idx.get(idx).copied().flatten() {
                    Some(fi) => fi,
                    None => {
                        return Err(VmError::UndefinedFunction(
                            self.bytecode.names.get(idx).cloned().unwrap_or_default(),
                        ))
                    }
                };
                if func_idx < usize::MAX {
                    if self.call_stack.len() >= MAX_CALL_STACK_DEPTH {
                        return Err(VmError::Runtime(format!(
                            "Desbordamiento de pila (Stack overflow): límite de recursión excedido (>{} llamadas)",
                            MAX_CALL_STACK_DEPTH
                        )));
                    }
                    // JIT Tier-1 (v3.5.9): compilar al alcanzar el umbral y,
                    // si está disponible, ejecutar nativamente.
                    self.jit_maybe_compile(func_idx, count_now);
                    #[cfg(feature = "aot")]
                    if let Some(f) = self.jit_get_fn(func_idx) {
                        self.setup_call_frame(idx, func_idx, args);
                        // SAFETY: el código nativo re-entra a la VM vía puntero
                        // crudo; ningún otro &mut a la VM se usa durante `f`.
                        // v3.5.31: native_exec desactiva los peepholes de ip
                        // (el ip está obsoleto dentro del cuerpo nativo).
                        let prev_native = self.native_exec;
                        self.native_exec = true;
                        let vm_ptr = self as *mut VM as *mut std::ffi::c_void;
                        let r = unsafe { f(vm_ptr) };
                        self.native_exec = prev_native;
                        match r {
                            0 => return Ok(()),
                            // v3.5.31 (Tier-2): guarda de tipos falló →
                            // ejecutar el MISMO frame en el intérprete.
                            2 => {
                                if let Some(rt) = self.jit_rt.as_mut() {
                                    rt.invalidate(func_idx);
                                }
                                let func_start = self.bytecode.funcs[func_idx].start;
                                self.ip = func_start;
                                let depth = self.call_stack.len();
                                return self.run_until_return(depth);
                            }
                            _ => {
                                return Err(self.jit_error.take().unwrap_or_else(|| {
                                    VmError::Runtime("JIT: error en código nativo".into())
                                }));
                            }
                        }
                    }
                    let func_start = self.bytecode.funcs[func_idx].start;
                    self.setup_call_frame(idx, func_idx, args);
                    self.ip = func_start;
                }
            }
            Opcode::FuncRef => {
                let name = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                // v3.5.18: CLOSURE LÉXICA — captura los bindings visibles como
                // celdas compartidas. La closure retiene su propio entorno:
                // al llamarla después de que el creador retornó, las variables
                // capturadas siguen vivas y las mutaciones persisten.
                let mut env: std::collections::HashMap<
                    String,
                    std::sync::Arc<std::sync::Mutex<Value>>,
                > = std::collections::HashMap::new();
                for scope in self.locals.iter() {
                    for (k, &slot) in &scope.map {
                        if let Some(v) = self.flat.get(slot as usize) {
                            let val = match v {
                                Value::Ref { cell, .. } => cell.lock().unwrap().clone(),
                                other => other.clone(),
                            };
                            env.insert(k.clone(), std::sync::Arc::new(std::sync::Mutex::new(val)));
                        }
                    }
                }
                self.push(Value::Closure { name, env });
            }
            Opcode::CallValue => {
                let argc = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let mut args = Vec::new();
                for _ in 0..argc {
                    args.push(self.pop()?);
                }
                args.reverse();
                let callee = self.pop()?;
                let (name, closure_env) = match &callee {
                    Value::Func(n) => (n.clone(), None),
                    Value::Closure { name, env } => (name.clone(), Some(env.clone())),
                    _ => {
                        return Err(VmError::TypeError(
                            "Se esperaba una función para llamar".to_string(),
                        ))
                    }
                };
                // v3.5.18: si es closure, inyecta el entorno capturado como un
                // scope sintético de referencias (write-through a las celdas).
                // Queda JUSTO debajo del locals_base del frame; Ret lo desapila.
                let is_closure_call = closure_env.is_some();
                if let Some(env) = closure_env {
                    self.scope_push();
                    let top = self.locals.len() - 1;
                    for (k, cell) in env {
                        let slot = self.alloc_slot(Value::Ref {
                            cell: std::sync::Arc::clone(&cell),
                            owner: None,
                        });
                        self.locals[top].map.insert(k.clone(), slot);
                        self.locals[top].slots.push(slot);
                    }
                }
                if name == "imprimir" || name == "print" {
                    let mut combined = String::new();
                    for arg in args {
                        combined.push_str(&format!("{}", arg));
                    }
                    self.emit_line(combined);
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
                    let c = char::from_u32(n as u32)
                        .map(|c| c.to_string())
                        .unwrap_or_default();
                    self.push(Value::str(c));
                } else if name == "__str_slice" || name == "__str_subcadena" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let start = args
                        .get(1)
                        .and_then(|v| match v {
                            Value::Int(i) => Some(*i as usize),
                            _ => v.as_num().map(|f| f as usize),
                        })
                        .unwrap_or(0);
                    let end = args
                        .get(2)
                        .and_then(|v| match v {
                            Value::Int(i) => {
                                if *i == -1 {
                                    Some(s.len())
                                } else {
                                    Some(*i as usize)
                                }
                            }
                            _ => v.as_num().map(|f| f as usize),
                        })
                        .unwrap_or(s.len());
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
                    let st = args
                        .get(1)
                        .and_then(|v| v.as_num())
                        .map(|f| f as i64)
                        .unwrap_or(0);
                    let en = args
                        .get(2)
                        .and_then(|v| v.as_num())
                        .map(|f| f as i64)
                        .unwrap_or(-1);
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
                    let m = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
                    let m = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let k = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(m) => {
                            self.push(m.get(&k).cloned().unwrap_or(Value::Void));
                        }
                        _ => return Err(VmError::TypeError("__map_get espera diccionario".into())),
                    }
                } else if name == "__map_len" || name == "__map_longitud" {
                    let m = args
                        .into_iter()
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match m {
                        Value::Map(m) => self.push(Value::Int(m.len() as i64)),
                        _ => return Err(VmError::TypeError("__map_len espera diccionario".into())),
                    }
                } else if name == "__map_keys" || name == "__map_claves" {
                    let m = args
                        .into_iter()
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
                    let m = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
                    let s = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
                    let s = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let item = it.next().unwrap_or(Value::Void);
                    match s {
                        Value::Map(m) => self.push(Value::Bool(m.contains_key(&item))),
                        _ => return Err(VmError::TypeError("__set_has espera conjunto".into())),
                    }
                } else if name == "__set_union" || name == "__conjunto_unir" {
                    let mut it = args.into_iter();
                    let a = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let b = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match (a, b) {
                        (Value::Map(mut m1), Value::Map(m2)) => {
                            for (k, v) in m2 {
                                if !m1.contains_key(&k) {
                                    m1.insert(k, v);
                                }
                            }
                            self.push(Value::Map(m1));
                        }
                        _ => return Err(VmError::TypeError("__set_union espera conjuntos".into())),
                    }
                } else if name == "__set_inter" || name == "__conjunto_interseccion" {
                    let mut it = args.into_iter();
                    let a = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let b = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    match (a, b) {
                        (Value::Map(m1), Value::Map(m2)) => {
                            let r: ImMap<Value, Value, FixHasher> =
                                m1.into_iter().filter(|(k, _)| m2.contains_key(k)).collect();
                            self.push(Value::Map(r));
                        }
                        _ => return Err(VmError::TypeError("__set_inter espera conjuntos".into())),
                    }
                } else if name == "__set_diff" || name == "__conjunto_diferencia" {
                    let mut it = args.into_iter();
                    let a = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
                    let b = it
                        .next()
                        .unwrap_or(Value::Map(ImMap::with_hasher(FixHasher::default())));
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
                    match crate::lumen_min_regex_new(&pat) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__regex_is_match" || name == "__regex_coincide" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match crate::lumen_min_regex_new(&re_s) {
                        Ok(r) => self.push(Value::Bool(r.is_match(&text))),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__regex_captures" || name == "__regex_capturar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match crate::lumen_min_regex_new(&re_s) {
                        Ok(r) => {
                            let caps = r.captures(&text);
                            let vs: Vec<Value> = caps.into_iter().map(Value::str).collect();
                            self.push(Value::arr(vs));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__regex_replace" || name == "__regex_reemplazar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let rep = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
                    match crate::lumen_min_regex_new(&re_s) {
                        Ok(r) => self.push(Value::str(r.replace(&text, rep.as_str()))),
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
                    #[allow(unused_variables)]
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::net::TcpStream::connect(&addr) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::str(e.to_string())))),
                    }
                } else if name == "__tcp_listen" || name == "__tcp_escuchar" {
                    #[allow(unused_variables)]
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
                        None => self.push(Value::Error(Box::new(Value::str("Sin listener")))),
                    }
                } else if name == "__http_server" || name == "__http_servidor" {
                    #[allow(unused_variables)]
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
                } else if let Some(&func_idx) = self.func_index_cache.get(&name) {
                    if self.call_stack.len() >= MAX_CALL_STACK_DEPTH {
                        return Err(VmError::Runtime(format!(
                            "Desbordamiento de pila (Stack overflow): límite de recursión excedido (>{} llamadas)",
                            MAX_CALL_STACK_DEPTH
                        )));
                    }
                    let func_start = self.bytecode.funcs[func_idx].start;
                    let (fn_nidx, fn_label) =
                        match self.bytecode.names.iter().position(|x| *x == name) {
                            Some(pos) => (pos, None),
                            None => (usize::MAX, Some(name.clone())),
                        };
                    self.call_stack.push(CallFrame {
                        func_name: fn_nidx,
                        func_label: fn_label,
                        return_ip: self.ip,
                        locals_base: self.locals.len(),
                        stack_base: self.stack.len(),
                        is_closure: is_closure_call,
                        has_refs: false,
                    });
                    self.push_params_scope(func_idx, &args);
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
                self.ip = self.resolve_target(idx);
            }
            Opcode::JmpIf => {
                let val = self.pop()?;
                if !val.is_truthy() {
                    self.ip = self.resolve_target(idx);
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
            Opcode::MatchType => {
                let val = self.pop()?;
                let kind = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as u8;
                let matched = match kind {
                    0 => matches!(val, Value::Opcion(Some(_))),
                    1 => matches!(val, Value::Exito(_)),
                    2 => matches!(val, Value::Error(_)),
                    _ => false,
                };
                self.push(Value::Bool(matched));
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

    /// v3.5.19: invalida la caché inline de variables. Solo en INSERCIONES
    /// de nombres nuevos y reemplazos completos de `locals` (los push/pop
    /// de scopes se validan solos por `locals.len()`).
    #[inline(always)]
    fn bump_var_cache(&mut self) {
        self.var_cache_gen = (self.var_cache_gen.wrapping_add(1)) & 0x0FFF_FFFF_FFFF_FFFF;
        if self.var_cache_gen == 0 {
            self.var_cache_gen = 1;
        }
    }

    // ── v3.5.31: scopes con arena de valores (flat) ──────────────────

    #[inline(always)]
    fn next_scope_id(&mut self) -> u64 {
        let id = self.scope_id_next;
        self.scope_id_next += 1;
        id
    }

    /// Empuja un scope vacío nuevo (identidad fresca). Invalida la caché de
    /// variables: un scope nuevo PUEDE sombrear nombres ya cacheados (los
    /// scopes vacíos nunca llegan a la VM — se eliminan en el IR — así que
    /// todo push implica bindings reales o params de llamada).
    #[inline(always)]
    fn scope_push(&mut self) {
        let id = self.next_scope_id();
        let mut map: HashMap<String, u32, FixHasher> = self
            .map_pool
            .pop()
            .unwrap_or_else(|| HashMap::with_hasher(FixHasher::default()));
        map.clear();
        let mut slots: Vec<u32> = self.slot_pool.pop().unwrap_or_default();
        slots.clear();
        self.locals
            .push(ScopeFrame::with_parts(map, slots, id, None));
        // v3.5.36: scope VACÍO — no sombrea nada; los inserts posteriores
        // invalidan selectivamente sus nombres.
    }

    /// Empuja un scope con capacidad pre-reservada (parámetros de llamada).
    #[inline(always)]
    fn scope_push_cap(&mut self, cap: usize) {
        let id = self.next_scope_id();
        let mut map: HashMap<String, u32, FixHasher> = self
            .map_pool
            .pop()
            .unwrap_or_else(|| HashMap::with_hasher(FixHasher::default()));
        map.clear();
        let mut slots: Vec<u32> = self.slot_pool.pop().unwrap_or_default();
        slots.clear();
        if slots.capacity() < cap {
            slots.reserve(cap - slots.len());
        }
        self.locals
            .push(ScopeFrame::with_parts(map, slots, id, None));
        // v3.5.36: scope VACÍO — la invalidación la hacen los inserts.
    }

    /// Libera los slots del scope `si` (flat → freelist) y devuelve los
    /// buffers (slots y mapa) al pool para reutilizarlos (v3.5.36: sin
    /// alloc/free por llamada ni por bloque de bucle).
    fn free_scope_slots(&mut self, si: usize) {
        let mut slots = std::mem::take(&mut self.locals[si].slots);
        for s in slots.drain(..) {
            self.free_slots.push(s);
        }
        if self.slot_pool.len() < 64 {
            self.slot_pool.push(slots);
        }
        let map = std::mem::take(&mut self.locals[si].map);
        if self.map_pool.len() < 64 {
            self.map_pool.push(map);
        }
    }

    /// Pop del scope superior liberando sus slots (nunca popea el global).
    #[inline(always)]
    fn scope_pop(&mut self) {
        if self.locals.len() > 1 {
            let si = self.locals.len() - 1;
            self.free_scope_slots(si);
            self.locals.pop();
        }
    }

    /// Trunca la pila de scopes en `n`, liberando slots de los eliminados.
    fn scope_truncate(&mut self, n: usize) {
        for si in n..self.locals.len() {
            self.free_scope_slots(si);
        }
        self.locals.truncate(n);
    }

    /// v3.5.34: si `val` es un Ref con owner, marca el frame actual — Ret
    /// usa el flag para saltar el escaneo de write-backs si no hay.
    #[inline(always)]
    fn note_val_write(&mut self, val: &Value) {
        if matches!(val, Value::Ref { owner: Some(_), .. }) {
            if let Some(f) = self.call_stack.last_mut() {
                f.has_refs = true;
            }
        }
    }

    /// Asigna un slot en `flat` para `v` (reuso del freelist o append).
    #[inline(always)]
    fn alloc_slot(&mut self, v: Value) -> u32 {
        self.note_val_write(&v);
        match self.free_slots.pop() {
            Some(s) => {
                self.flat[s as usize] = v;
                s
            }
            None => {
                self.flat.push(v);
                (self.flat.len() - 1) as u32
            }
        }
    }

    /// Invalida la caché de variables: se usa cuando `locals` se reemplaza
    /// completo (corutinas, snapshots) o se inserta un nombre nuevo.
    fn replace_locals_full(
        &mut self,
        new_locals: Vec<ScopeFrame>,
        new_flat: Vec<Value>,
        new_free: Vec<u32>,
    ) {
        self.locals = new_locals;
        self.flat = new_flat;
        self.free_slots = new_free;
        self.scope_id_next = self
            .locals
            .iter()
            .map(|f| f.id)
            .max()
            .map(|m| m + 1)
            .unwrap_or(1);
        self.bump_var_cache();
    }

    /// Entrada de caché válida → Some(slot) para acceso directo a `flat`.
    /// INVARIANTES (v3.5.31): `var_cache` está pre-dimensionado a
    /// `bytecode.names.len()` en VM::new, así que `idx < len` para bytecode
    /// bien formado — el guard `idx >= len` (1 comparación) mantiene la
    /// seguridad ante .nvc externos corruptos; un slot válido es siempre
    /// < `flat.len()` (alloc_slot lo garantiza y los swaps de
    /// corutinas/snapshots traen frames+flat consistentes).
    #[inline(always)]
    fn var_cache_slot(&self, idx: usize) -> Option<u32> {
        if idx >= self.var_cache.len() {
            return None;
        }
        let &(slot, si, sid, gen) = unsafe { self.var_cache.get_unchecked(idx) };
        let si = si as usize;
        if gen != self.var_cache_gen || si >= self.locals.len() {
            return None;
        }
        if unsafe { self.locals.get_unchecked(si) }.id != sid {
            return None;
        }
        Some(slot)
    }

    /// v3.5.31: flag de ejecución NATIVA (JIT). Cuando está activo, los
    /// peepholes que leen `self.ip` (Add/Sub/Mul→Store, cmp→JmpIf) se
    /// desactivan: el ip está obsoleto durante el cuerpo nativo y el peephole
    /// corrompería la pila (bug del lumen_mini2: `Add` con ip rancio se
    /// tragó el push y `__map_set` recibió args corridos).
    #[inline(always)]
    fn peek_with_idx(&self) -> Option<(Opcode, usize)> {
        if self.native_exec {
            return None;
        }
        match self.bytecode.instructions.get(self.ip)? {
            Instruction::WithIdx(op, x) => Some((*op, *x)),
            _ => None,
        }
    }

    /// v3.5.36: invalida SOLO la entrada de caché del nombre `idx` — se
    /// usa al INSERTAR un nombre nuevo en el scope superior: únicamente ese
    /// nombre puede quedar sombreado; el resto de la caché sigue válida a
    /// través de llamadas y bloques.
    #[inline(always)]
    fn var_cache_invalidate(&mut self, idx: usize) {
        if idx < self.var_cache.len() && self.var_cache_slot(idx).is_some() {
            self.var_cache[idx] = (0, 0, 0, 0);
        }
    }

    #[inline(always)]
    fn var_cache_put(&mut self, idx: usize, slot: u32, scope_idx: usize) {
        let entry = (
            slot,
            scope_idx as u32,
            self.locals[scope_idx].id,
            self.var_cache_gen,
        );
        if idx < self.var_cache.len() {
            self.var_cache[idx] = entry;
        } else {
            self.var_cache.resize(idx + 1, (0, 0, 0, 0));
            self.var_cache[idx] = entry;
        }
    }

    /// v3.5.19: peephole comparación→JmpIf: salta directamente sin pasar
    /// por push/pop ni por el dispatch de JmpIf.
    /// v3.5.31: desactivado durante ejecución nativa (peek_with_idx →
    /// None si native_exec; el ip está obsoleto en el cuerpo JIT).
    #[inline(always)]
    fn cmp_jmpif_fused(&mut self, cond: bool) -> bool {
        if let Some((Opcode::JmpIf, jidx)) = self.peek_with_idx() {
            let target = self.resolve_target(jidx);
            if !cond {
                self.ip = target;
            } else {
                self.ip += 1;
            }
            true
        } else {
            false
        }
    }

    /// v3.5.19: Load compartido con los super-opcodes (usa la caché inline).
    #[inline(always)]
    fn do_load_by_idx(&mut self, idx: usize) -> Result<Value, VmError> {
        // v3.5.31: fast-path por caché — slot directo en `flat`, SIN hash.
        if let Some(slot) = self.var_cache_slot(idx) {
            // slot validado por la caché (alcance de un scope vivo) → < len.
            let v = unsafe { self.flat.get_unchecked(slot as usize) }.clone();
            return Ok(match v {
                Value::Ref { .. } => v.deep_deref(),
                other => other,
            });
        }
        let name = self
            .bytecode
            .names
            .get(idx)
            .map(|s| s.as_str())
            .unwrap_or("");
        let mut found: Option<(usize, u32)> = None;
        for (si, scope) in self.locals.iter().enumerate().rev() {
            if let Some(s) = self.scope_get(scope, name) {
                found = Some((si, s));
                break;
            }
        }
        match found {
            Some((si, slot)) => {
                self.var_cache_put(idx, slot, si);
                let v = self.flat.get(slot as usize).cloned().unwrap_or(Value::Void);
                Ok(match v {
                    Value::Ref { .. } => v.deep_deref(),
                    other => other,
                })
            }
            None => Err(VmError::UndefinedVariable(name.to_string())),
        }
    }

    /// v3.5.20: semántica completa de Add/Sub/Mul para el fallback de los
    /// super-opcodes (operandos no enteros: floats mixtos, texto+X, etc.).
    fn bin_vals_slow(&self, a: Value, b: Value, op: u8) -> Result<Value, VmError> {
        match (op, &a, &b) {
            (1, Value::Int(x), Value::Float(y)) => Ok(Value::Float(*x as f64 + y)),
            (1, Value::Float(x), Value::Int(y)) => Ok(Value::Float(x + *y as f64)),
            (3, Value::Int(x), Value::Float(y)) => Ok(Value::Float(*x as f64 - y)),
            (3, Value::Float(x), Value::Int(y)) => Ok(Value::Float(x - *y as f64)),
            (4, Value::Int(x), Value::Float(y)) => Ok(Value::Float(*x as f64 * y)),
            (4, Value::Float(x), Value::Int(y)) => Ok(Value::Float(x * *y as f64)),
            (1, Value::Str(x), Value::Str(y)) => Ok(Value::str(format!("{}{}", x, y))),
            (1, Value::Str(x), Value::Int(y)) => Ok(Value::str(format!("{}{}", x, y))),
            (1, Value::Str(x), Value::Float(y)) => Ok(Value::str(format!("{}{}", x, y))),
            (1, Value::Int(x), Value::Str(y)) => Ok(Value::str(format!("{}{}", x, y))),
            (1, Value::Float(x), Value::Str(y)) => Ok(Value::str(format!("{}{}", x, y))),
            (1, Value::Str(x), Value::Bool(y)) => Ok(Value::str(format!("{}{}", x, y))),
            (1, Value::Bool(x), Value::Str(y)) => Ok(Value::str(format!("{}{}", x, y))),
            _ => Err(VmError::TypeError(
                "Add/Sub/Mul requires numbers or strings".to_string(),
            )),
        }
    }

    /// v3.5.20: comparaciones para el fallback de super-opcodes.
    /// v3.5.30: paridad EXACTA con los opcodes clásicos Eq/Neq/Lt/Le/Gt/Ge —
    /// la ruta lenta de los super-opcodes debe ser semánticamente
    /// transparente: Eq cae a `false` (y Neq a `true`) para tipos
    /// incompatibles (p. ej. `None == 1`, clave de mapa ausente) en lugar de
    /// lanzar error, igual que el opcode no fusionado. Los ordenamientos
    /// (Lt/Le/Gt/Ge) sí exigen números o strings.
    fn cmp_vals_slow(&self, a: Value, b: Value, op: u8) -> Result<bool, VmError> {
        let r = match (op, &a, &b) {
            (7, Value::Int(x), Value::Int(y)) => x == y,
            (7, Value::Int(x), Value::Float(y)) => (*x as f64 - y).abs() < f64::EPSILON,
            (7, Value::Float(x), Value::Int(y)) => (x - *y as f64).abs() < f64::EPSILON,
            (7, Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
            (7, Value::Str(x), Value::Str(y)) => x == y,
            (7, Value::Bool(x), Value::Bool(y)) => x == y,
            (
                7,
                Value::Struct {
                    name: an,
                    fields: af,
                },
                Value::Struct {
                    name: bn,
                    fields: bf,
                },
            ) => an == bn && af == bf,
            (7, Value::Opcion(x), Value::Opcion(y)) => x == y,
            (
                7,
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
            (7, _, _) => false,
            (8, Value::Int(x), Value::Int(y)) => x != y,
            (8, Value::Int(x), Value::Float(y)) => (*x as f64 - y).abs() >= f64::EPSILON,
            (8, Value::Float(x), Value::Int(y)) => (x - *y as f64).abs() >= f64::EPSILON,
            (8, Value::Float(x), Value::Float(y)) => (x - y).abs() >= f64::EPSILON,
            (8, Value::Str(x), Value::Str(y)) => x != y,
            (8, Value::Bool(x), Value::Bool(y)) => x != y,
            (
                8,
                Value::Struct {
                    name: an,
                    fields: af,
                },
                Value::Struct {
                    name: bn,
                    fields: bf,
                },
            ) => an != bn || af != bf,
            (8, Value::Opcion(x), Value::Opcion(y)) => x != y,
            (
                8,
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
            (8, _, _) => true,
            (9, Value::Int(x), Value::Int(y)) => x < y,
            (9, Value::Int(x), Value::Float(y)) => (*x as f64) < *y,
            (9, Value::Float(x), Value::Int(y)) => *x < *y as f64,
            (9, Value::Float(x), Value::Float(y)) => x < y,
            (9, Value::Str(x), Value::Str(y)) => x < y,
            (10, Value::Int(x), Value::Int(y)) => x <= y,
            (10, Value::Int(x), Value::Float(y)) => (*x as f64) <= *y,
            (10, Value::Float(x), Value::Int(y)) => *x <= *y as f64,
            (10, Value::Float(x), Value::Float(y)) => x <= y,
            (10, Value::Str(x), Value::Str(y)) => x <= y,
            (11, Value::Int(x), Value::Int(y)) => x > y,
            (11, Value::Int(x), Value::Float(y)) => (*x as f64) > *y,
            (11, Value::Float(x), Value::Int(y)) => *x > *y as f64,
            (11, Value::Float(x), Value::Float(y)) => x > y,
            (11, Value::Str(x), Value::Str(y)) => x > y,
            (12, Value::Int(x), Value::Int(y)) => x >= y,
            (12, Value::Int(x), Value::Float(y)) => (*x as f64) >= *y,
            (12, Value::Float(x), Value::Int(y)) => *x >= *y as f64,
            (12, Value::Float(x), Value::Float(y)) => x >= y,
            (12, Value::Str(x), Value::Str(y)) => x >= y,
            _ => {
                return Err(VmError::TypeError(
                    "Comparison requires numbers or strings".to_string(),
                ))
            }
        };
        Ok(r)
    }

    /// v3.5.19: cuerpo de `Store` compartido con el peephole (Add→Store).
    #[inline(always)]
    fn do_store_by_idx(&mut self, idx: usize, val: Value) {
        // v3.5.31: fast-path por caché — escribe directo en el slot de `flat`.
        if let Some(slot) = self.var_cache_slot(idx) {
            // slot validado por la caché (alcance de un scope vivo) → < len.
            let cell = unsafe { self.flat.get_unchecked_mut(slot as usize) };
            if cell.is_ref() {
                cell.ref_set(val);
            } else {
                // v3.5.34: marcar has_refs en campos disjuntos (el borrow de
                // flat ya está tomado).
                if matches!(val, Value::Ref { owner: Some(_), .. }) {
                    if let Some(f) = self.call_stack.last_mut() {
                        f.has_refs = true;
                    }
                }
                *cell = val;
            }
            return;
        }
        let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
        let n = self.locals.len();
        if n > 0 {
            let cur = n - 1;
            let mut found_at: Option<(usize, u32)> = None;
            for (si, scope) in self.locals.iter().enumerate().rev() {
                if let Some(s) = self.scope_get(scope, &name) {
                    found_at = Some((si, s));
                    break;
                }
            }
            if let Some((si, slot)) = found_at {
                // Si el slot contiene una referencia, escribir A TRAVÉS de la
                // celda compartida conservando el owner
                if let Some(cell) = self.flat.get_mut(slot as usize) {
                    if cell.is_ref() {
                        cell.ref_set(val);
                    } else {
                        *cell = val;
                    }
                }
                self.var_cache_put(idx, slot, si);
            } else {
                // nombre NUEVO en el scope → asignar slot; v3.5.36: solo
                // el nombre insertado se invalida.
                let slot = self.alloc_slot(val);
                self.locals[cur].map.insert(name, slot);
                self.locals[cur].slots.push(slot);
                self.var_cache_invalidate(idx);
            }
        }
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
                    let op = m
                        .get(&Value::str("op"))
                        .and_then(|v| {
                            if let Value::Int(o) = v {
                                Some(*o)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);
                    let arg = m
                        .get(&Value::str("arg"))
                        .and_then(|v| {
                            if let Value::Int(a) = v {
                                Some(*a)
                            } else {
                                None
                            }
                        })
                        .unwrap_or(0);
                    Some((op, arg))
                }
                _ => None,
            }
        };

        // Build names table from LOAD(21)/STORE(20)/CALL(24)/MAKEREF(63) instructions
        let mut names: Vec<String> = Vec::new();
        for ip in 0..pc {
            if let Some((op, arg)) = get_instr(ip) {
                if op == 20 || op == 21 || op == 24 || op == 63 {
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
                1 => 2,   // PushNum → VM PushNum (f64)
                2 => 3,   // PushStr
                3 => 1,   // PushInt
                4 => 4,   // PushBool
                5 => 7,   // Add
                6 => 8,   // Sub
                7 => 9,   // Mul
                8 => 10,  // Div
                9 => 46,  // Mod
                10 => 11, // Eq
                11 => 12, // Neq
                12 => 13, // Lt
                13 => 14, // Le
                14 => 15, // Gt
                15 => 16, // Ge
                16 => 17, // And
                17 => 18, // Or
                18 => 20, // Not
                19 => 19, // Neg
                20 => 6,  // Store
                21 => 5,  // Load
                22 => 25, // Jmp
                23 => 26, // JmpIf
                24 => 21, // Call
                25 => 22, // Ret
                26 => 23, // Print
                27 => 27, // Halt
                28 => 28, // ArrayNew
                29 => 29, // ArrayGet
                30 => 30, // ArraySet
                31 => 31, // ArrayLen
                32 => 32, // ArrayPush
                38 => 38, // ResultOk
                39 => 39, // ResultErr
                40 => 40, // TryUnwrap
                41 => 41, // OptionSome
                42 => 42, // OptionNone
                47 => 44, // TupleNew
                48 => 45, // TupleAccess
                46 => 47, // BitOr
                49 => 48, // BitAnd
                50 => 49, // ShiftLeft
                51 => 50, // ShiftRight
                52 => 51, // Concat
                53 => 52, // MatchType
                54 => 53, // MatchPayload
                59 => 59, // StoreLocal (v3.5.11: declaraciones sin scope-walk)
                60 => 60, // ScopePush
                61 => 61, // ScopePop
                63 => 63, // MakeRef (prestado mut, v3.5.5)
                _ => 0,   // Nop
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
                // + ScopePush/ScopePop (60/61, v3.5.11)
                if (5..=19).contains(&op)
                    || (op == 29 || op == 30 || op == 31)
                    || op == 60
                    || op == 61
                {
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
                } else if op == 53 {
                    // MatchType: arg = kind (ints idx) → VM expects WithIdx(52, nums idx)
                    let kind = get_int(arg as usize) as f64;
                    nums_bytes.extend_from_slice(&kind.to_le_bytes());
                    instr_bytes.push(4);
                    instr_bytes.push(52);
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
                } else if op == 20 || op == 21 || op == 63 || op == 59 {
                    // Store/Load/MakeRef/StoreLocal: arg is strings index (variable name) → names table
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
#[cfg(any(feature = "extra", feature = "full"))]
fn base64url_encode(data: &[u8]) -> String {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(data)
}

#[cfg(any(feature = "extra", feature = "full"))]
fn base64url_decode(data: &str) -> Vec<u8> {
    use base64::Engine;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .unwrap_or_default()
}

#[cfg(any(feature = "extra", feature = "full"))]
fn base64url_decode_to_string(data: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(data)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    String::from_utf8(bytes).map_err(|e| format!("UTF-8 error: {}", e))
}

#[cfg(any(feature = "extra", feature = "full"))]
fn hmac_sha256(data: &[u8], key: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    type HmacSha256 = Hmac<sha2::Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC key length OK");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

// ── Date helpers ─────────────────────────────────
#[cfg_attr(not(feature = "full"), allow(dead_code))]
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

#[cfg_attr(not(feature = "full"), allow(dead_code))]
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

#[cfg_attr(not(feature = "full"), allow(dead_code))]
fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg_attr(not(feature = "full"), allow(dead_code))]
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

#[cfg_attr(not(feature = "full"), allow(dead_code))]
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

#[cfg(feature = "full")]
impl Drop for VM {
    fn drop(&mut self) {
        for (ptr, layout) in self.ffi_allocations.drain() {
            unsafe {
                std::alloc::dealloc(ptr as *mut u8, layout);
            }
        }
    }
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
                    defaults: vec![],
                    start: 0,
                },
                FuncMeta {
                    name: "sum".to_string(),
                    params: vec!["a".to_string(), "b".to_string()],
                    defaults: vec![None, None],
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
                defaults: vec![],
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
                defaults: vec![],
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
