#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

// ── JS eval bridge (WASM target) ──────────────────────────────────────────
#[cfg(feature = "wasm")]
fn js_eval(js: &str) -> String {
    js_sys::eval(js)
        .ok()
        .and_then(|v| v.as_string())
        .unwrap_or_default()
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
        LumenRuntime {
            output: String::new(),
        }
    }

    /// Ejecuta código fuente LÚMEN y devuelve la salida.
    pub fn run(&mut self, source: &str) -> String {
        self.output.clear();
        run_lumen(source)
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
