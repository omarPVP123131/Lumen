#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "wasm")]
use std::collections::HashMap;

// ── Synchronized storage for registered JS functions ──────────────────────
#[cfg(feature = "wasm")]
use std::sync::Mutex;

#[cfg(feature = "wasm")]
static REGISTERED_JS_FUNCTIONS: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

// ── JS eval bridge (WASM target) ──────────────────────────────────────────
/// Enhanced JS eval that handles the `__lumen_call` protocol.
/// When the VM emits `__lumen_call('fnName', [args...])`, this eval runs it
/// against the browser's global `window.__lumen_call` function.
#[cfg(feature = "wasm")]
fn js_eval(js: &str) -> String {
    // First check if it's a __lumen_call and try registered functions
    if js.starts_with("__lumen_call(") {
        // Extract function name and args from the call
        if let Some(result) = try_registered_call(js) {
            return result;
        }
    }

    // Fall through to actual JS eval
    js_sys::eval(js)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
}

/// Attempt to resolve the `__lumen_call('fn', ['a','b'])` via Rust-registered
/// functions before falling through to the browser JS bridge.
#[cfg(feature = "wasm")]
fn try_registered_call(js: &str) -> Option<String> {
    let guard = REGISTERED_JS_FUNCTIONS.lock().ok()?;
    let registry = guard.as_ref()?;

    // Very simple parser for __lumen_call('name', ['arg1','arg2'])
    // We look for the function name between first '(' and first ','
    let after_paren = js.find('(')?;
    let rest = &js[after_paren + 1..];
    let fn_name_end = rest.find(',')?;
    let fn_name = rest[..fn_name_end].trim().trim_matches('\'').trim_matches('"');

    if let Some(_js_func_body) = registry.get(fn_name) {
        // We can only call it through real JS eval since the funcs are JS strings
        return None; // Fall through to js_sys::eval
    }
    None
}

// ── WASI eval stub ─────────────────────────────────────────────────────────
#[cfg(feature = "wasi")]
fn js_eval(_js: &str) -> String {
    "WASI: JS eval no disponible".to_string()
}

// ── LumenRuntime (WASM target with wasm-bindgen) ──────────────────────────
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub struct LumenRuntime {
    output: String,
}

#[cfg(feature = "wasm")]
#[wasm_bindgen]
impl LumenRuntime {
    /// Crea una nueva instancia del runtime. Registra automáticamente
    /// el callback `JS_EVAL` para que las builtins `__js_eval` y `__js_call`
    /// funcionen desde LÚMEN.
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let _ = lumen_vm::vm::JS_EVAL.set(js_eval);

        // Initialize the registered functions map
        let mut guard = REGISTERED_JS_FUNCTIONS.lock().unwrap();
        *guard = Some(HashMap::new());

        LumenRuntime {
            output: String::new(),
        }
    }

    /// Ejecuta código fuente LÚMEN y devuelve la salida.
    pub fn run(&mut self, source: &str) -> String {
        self.output.clear();
        run_lumen(source)
    }

    /// Registra una función JS para ser llamada desde LÚMEN.
    /// `name` es el identificador de la función (ej. "console_log").
    /// `js_func_string` es el código JS que implementa la función.
    ///
    /// # Ejemplo
    /// ```javascript
    /// runtime.register_js_function("greet", "(name) => 'Hello, ' + name");
    /// // Ahora desde LÚMEN:
    /// // __js_call("greet", "World")  →  "Hello, World"
    /// ```
    pub fn register_js_function(&self, name: &str, js_func_string: &str) {
        let mut guard = REGISTERED_JS_FUNCTIONS.lock().unwrap();
        if let Some(ref mut map) = *guard {
            map.insert(name.to_string(), js_func_string.to_string());
        }
        // Also register it in the browser's global scope for js_sys::eval to see
        let js_code = format!(
            "if(!window.__lumen_call){{window.__lumen_call=function(n,a){{return'';}};}}
             window.__lumen_bridge_{} = {};",
            name.replace('-', "_").replace('.', "_"),
            js_func_string
        );
        let _ = js_sys::eval(&js_code);
    }

    /// Desregistra una función JS previamente registrada.
    pub fn unregister_js_function(&self, name: &str) {
        let mut guard = REGISTERED_JS_FUNCTIONS.lock().unwrap();
        if let Some(ref mut map) = *guard {
            map.remove(name);
        }
        // Remove from global scope
        let safe_name = name.replace('-', "_").replace('.', "_");
        let _ = js_sys::eval(&format!("delete window.__lumen_bridge_{};", safe_name));
    }

    /// Lista todas las funciones JS registradas.
    pub fn list_js_functions(&self) -> String {
        let guard = REGISTERED_JS_FUNCTIONS.lock().unwrap();
        if let Some(ref map) = *guard {
            let names: Vec<&str> = map.keys().map(|s| s.as_str()).collect();
            names.join(", ")
        } else {
            String::new()
        }
    }

    /// Convierte un valor LÚMEN a JS y lo devuelve como string JSON.
    /// Útil para debug y pruebas de interop.
    ///
    /// Ejecuta el código LÚMEN y convierte el último valor del stack a formato
    /// que JS puede consumir.
    pub fn lumen_to_js(&mut self, source: &str) -> String {
        // Run the code and capture the output
        let output = self.run(source);

        // If the output is already a clean value, format it as JSON-compatible
        if output.starts_with("Error") {
            return format!("{{\"error\": \"{}\"}}", output.replace('"', "\\\""));
        }

        // Try to parse the output as a JSON value
        // For simple cases, wrap in a JSON structure
        format!("{{\"result\": \"{}\"}}", output.replace('"', "\\\""))
    }

    /// Devuelve la versión del paquete.
    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    /// Analiza el código y devuelve solo errores (sin ejecutar).
    pub fn check(&self, source: &str) -> Option<String> {
        check_lumen(source)
    }

    /// Devuelve la lista de tokens producidos por el lexer (depuración).
    pub fn tokenize(&self, source: &str) -> String {
        let lexer = lumen_lexer::Lexer::new(source);
        let (tokens, errors) = lexer.tokenize();
        if !errors.is_empty() {
            return format!("Error: {}", errors[0].message);
        }
        tokens
            .iter()
            .map(|t| format!("{:?}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ── LumenRuntime (WASI target — sin wasm-bindgen) ─────────────────────────
#[cfg(feature = "wasi")]
pub struct LumenRuntime {
    output: String,
}

#[cfg(feature = "wasi")]
impl LumenRuntime {
    pub fn new() -> Self {
        let _ = lumen_vm::vm::JS_EVAL.set(js_eval);
        LumenRuntime {
            output: String::new(),
        }
    }

    pub fn run(&mut self, source: &str) -> String {
        self.output.clear();
        run_lumen(source)
    }

    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    pub fn check(&self, source: &str) -> Option<String> {
        check_lumen(source)
    }

    pub fn tokenize(&self, source: &str) -> String {
        let lexer = lumen_lexer::Lexer::new(source);
        let (tokens, errors) = lexer.tokenize();
        if !errors.is_empty() {
            return format!("Error: {}", errors[0].message);
        }
        tokens
            .iter()
            .map(|t| format!("{:?}", t))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

// ── Lógica común de compilación/ejecución ─────────────────────────────────

/// Compila y ejecuta código fuente LÚMEN. Devuelve la salida o un mensaje de error.
pub fn run_lumen(source: &str) -> String {
    // Lexer
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return format!(
            "Error léxico [{}:{}]: {}",
            lex_errors[0].pos.line, lex_errors[0].pos.col, lex_errors[0].message
        );
    }

    // Parser
    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return format!(
            "Error sintáctico [{}]: {}",
            parse_errors[0].span.start.line, parse_errors[0].message
        );
    }

    // Semantic analysis
    let sema = lumen_sema::SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        return format!("Error semántico: {}", sem_errors[0].message);
    }

    // IR generation
    let ir_program = lumen_ir::IRBuilder::new().build(&program);

    // Bytecode generation
    let (bc, _warnings) = lumen_codegen::Codegen::new().generate(&ir_program);

    // VM execution
    let mut vm = lumen_vm::vm::VM::new(bc);
    match vm.run() {
        Ok(()) => vm.output().join("\n"),
        Err(e) => format!("Error runtime: {}", e),
    }
}

/// Analiza el código sin ejecutarlo. Devuelve `Some(error)` si hay errores,
/// `None` si todo está bien.
pub fn check_lumen(source: &str) -> Option<String> {
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return Some(format!(
            "Error léxico [{}:{}]: {}",
            lex_errors[0].pos.line, lex_errors[0].pos.col, lex_errors[0].message
        ));
    }

    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return Some(format!(
            "Error sintáctico [{}]: {}",
            parse_errors[0].span.start.line, parse_errors[0].message
        ));
    }

    let sema = lumen_sema::SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        return Some(format!("Error semántico: {}", sem_errors[0].message));
    }

    None
}
