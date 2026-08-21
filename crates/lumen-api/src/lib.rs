// LÚMEN Compiler API — Embedded scripting engine for Rust
// Provides compile, check, and run functionality for LÚMEN source code.

use lumen_codegen::Codegen;
use lumen_ir::IRBuilder;
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::{ModuleLoader, SemanticAnalyzer};
use lumen_vm::VM;
use std::path::PathBuf;

/// Result type for LumenEngine operations.
pub type LumenResult<T> = Result<T, String>;

/// The LÚMEN engine: compiles and runs LÚMEN source code from Rust.
///
/// # Example
///
/// ```ignore
/// use lumen_api::LumenEngine;
///
/// let mut engine = LumenEngine::new();
/// let result = engine.run("imprimir(\"Hola desde Rust!\");").unwrap();
/// println!("{}", result);
/// ```
pub struct LumenEngine {
    lib_dirs: Vec<PathBuf>,
}

impl LumenEngine {
    /// BUG-158: resuelve los `importar` del fuente igual que hacen `lumen run`
    /// y `lumen check`. La API publica analizaba el texto AISLADO, asi que todo
    /// simbolo venido de un modulo se reportaba como «no definida» y
    /// `with_lib_dirs` no servia para nada: el campo estaba ahi, con un
    /// `#[allow(dead_code)]` encima, pero nunca se leia.
    fn resolver(&self, source: &str) -> Result<Vec<lumen_parser::ast::DeclOrStmt>, String> {
        let mut loader = ModuleLoader::new(self.lib_dirs.clone());
        loader
            .resolve_imports(source, std::path::Path::new("<api>.nv"))
            .map_err(|e| match e {
                lumen_sema::ModuleError::Circular { path, .. } => {
                    format!("E063: import circular en '{}'", path.display())
                }
                lumen_sema::ModuleError::Io { path, message } => {
                    format!("No se pudo cargar '{}': {}", path.display(), message)
                }
                otro => format!("Error de import: {:?}", otro),
            })
    }

    /// Creates a new LÚMEN engine instance.
    pub fn new() -> Self {
        Self { lib_dirs: vec![] }
    }

    /// Creates a new LÚMEN engine with custom library search paths.
    pub fn with_lib_dirs(lib_dirs: Vec<PathBuf>) -> Self {
        Self { lib_dirs }
    }

    /// Returns the LÚMEN version string.
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    /// Checks LÚMEN source code for syntax and semantic errors.
    ///
    /// Returns `Ok(())` if the code is valid, or `Err(errors)` with a list of error messages.
    pub fn check(&self, source: &str) -> Result<(), Vec<String>> {
        let lexer = Lexer::new(source);
        let (tokens, lex_errors) = lexer.tokenize();

        if !lex_errors.is_empty() {
            return Err(lex_errors.iter().map(|e| format!("{:?}", e)).collect());
        }

        let parser = Parser::new(tokens);
        let (_program, parse_errors) = parser.parse();

        if !parse_errors.is_empty() {
            // BUG-158: `{:?}` volcaba la estructura interna de Rust
            // (`SemError { code: "E042", span: Span { .. } }`) a quien integra
            // el motor. Un consumidor de la API no deberia ver nuestros tipos.
            return Err(parse_errors
                .iter()
                .map(|e| {
                    format!(
                        "{} [{}:{}]: {} — {}",
                        e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
                    )
                })
                .collect());
        }

        let mut program = self.resolver(source).map_err(|e| vec![e])?;

        let sema = SemanticAnalyzer::new();
        let sem_errors = sema.analyze(&mut program);

        if !sem_errors.is_empty() {
            return Err(sem_errors
                .iter()
                .map(|e| {
                    format!(
                        "{} [{}:{}]: {} — {}",
                        e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
                    )
                })
                .collect());
        }

        Ok(())
    }

    /// Compiles LÚMEN source code to bytecode.
    ///
    /// Returns the compiled bytecode as a `Vec<u8>`.
    pub fn compile(&self, source: &str) -> Result<Vec<u8>, String> {
        let lexer = Lexer::new(source);
        let (tokens, lex_errors) = lexer.tokenize();

        if !lex_errors.is_empty() {
            return Err(format!("{:?}", lex_errors[0]));
        }

        let parser = Parser::new(tokens);
        let (_program, parse_errors) = parser.parse();

        if !parse_errors.is_empty() {
            let e = &parse_errors[0];
            return Err(format!(
                "{} [{}:{}]: {} — {}",
                e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
            ));
        }

        let mut program = self.resolver(source)?;

        let sema = SemanticAnalyzer::new();
        let sem_errors = sema.analyze(&mut program);

        if !sem_errors.is_empty() {
            let e = &sem_errors[0];
            return Err(format!(
                "{} [{}:{}]: {} — {}",
                e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
            ));
        }

        let ir = IRBuilder::new().build(&program);
        let (bytecode, _) = Codegen::new().generate(&ir);

        Ok(bytecode.encode())
    }

    /// Runs LÚMEN source code and captures the output.
    ///
    /// Returns the captured output as a `String`, or an error description.
    pub fn run(&mut self, source: &str) -> Result<String, String> {
        let bytecode_data = self.compile(source)?;

        let (bytecode, _) = lumen_codegen::Bytecode::decode(&bytecode_data)
            .map_err(|e| format!("Decode error: {}", e))?;

        let mut vm = VM::new(bytecode);
        vm.run().map_err(|e| format!("Runtime error: {:?}", e))?;

        // BUG-158: `run` documenta «captures the output» y devolvia
        // `stack_top()`, que es otra cosa: el ejemplo del propio crate,
        // `run("imprimir(\"Hola desde Rust!\");")`, devolvia "void" en vez del
        // texto impreso. Se devuelve lo que el programa escribio.
        Ok(vm.output().join("\n"))
    }
}

impl Default for LumenEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let v = LumenEngine::version();
        assert!(!v.is_empty());
    }

    #[test]
    fn test_check_valid() {
        let engine = LumenEngine::new();
        let result = engine.check("imprimir(\"hola\");");
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }

    #[test]
    fn test_check_invalid() {
        let engine = LumenEngine::new();
        let result = engine.check("imprimir(;");
        assert!(result.is_err());
    }

    #[test]
    fn test_compile() {
        let engine = LumenEngine::new();
        let result = engine.compile("imprimir(\"test\");");
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
        let bytecode = result.unwrap();
        assert!(!bytecode.is_empty());
        // Should start with LUMN magic bytes
        assert_eq!(&bytecode[0..4], b"LUMN");
    }

    #[test]
    fn test_run_simple() {
        let mut engine = LumenEngine::new();
        let result = engine.run("imprimir(\"hello\");");
        assert!(result.is_ok(), "Expected Ok, got: {:?}", result);
    }

    /// BUG-158: `run` documenta que captura la salida, y devolvia la cima de la
    /// pila ("void"). Este test mira el CONTENIDO, no solo `is_ok()`.
    #[test]
    fn bug158_run_devuelve_la_salida_no_la_pila() {
        let mut engine = LumenEngine::new();
        let out = engine
            .run("imprimir(\"Hola desde Rust!\");")
            .expect("debe ejecutar");
        assert_eq!(out, "Hola desde Rust!", "run() debe devolver lo impreso");
    }

    /// BUG-158: la API publica no resolvia `importar`, asi que todo simbolo de
    /// la stdlib era un E042 falso para quien embebe el motor.
    #[test]
    fn bug158_api_resuelve_imports() {
        let mut engine = LumenEngine::new();
        let src = "importar \"texto\";\nimprimir(texto_longitud(\"hola\"));";
        engine
            .check(src)
            .expect("check no debe inventar errores sobre un import valido");
        let out = engine.run(src).expect("run debe resolver el import");
        assert_eq!(out, "4");
    }

    /// BUG-158: `with_lib_dirs` aceptaba rutas que nunca se leian.
    #[test]
    fn bug158_with_lib_dirs_se_usa_de_verdad() {
        let dir = std::env::temp_dir().join("lumen_api_bug158");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("saludos.nv"),
            "funcion texto saludo() { retornar \"hola-desde-lib-dir\"; }\n",
        )
        .unwrap();
        let src = "importar \"saludos\";\nimprimir(saludos_saludo());";

        let mut sin = LumenEngine::new();
        assert!(sin.run(src).is_err(), "sin lib_dirs el modulo no existe");

        let mut con = LumenEngine::with_lib_dirs(vec![dir.clone()]);
        let out = con.run(src).expect("con lib_dirs debe encontrarlo");
        assert_eq!(out, "hola-desde-lib-dir");
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// BUG-158: los errores salian como `Debug` de Rust
    /// (`SemError { code: "E042", span: Span { .. } }`).
    #[test]
    fn bug158_errores_legibles_sin_debug_de_rust() {
        let engine = LumenEngine::new();
        let errs = engine
            .check("funcion vacio main() { no_existe(); }")
            .unwrap_err();
        let e = &errs[0];
        assert!(
            !e.contains("SemError {"),
            "no debe filtrar tipos internos: {}",
            e
        );
        assert!(e.starts_with("E042"), "debe empezar por el codigo: {}", e);
        assert!(e.contains("no_existe"), "debe nombrar el simbolo: {}", e);
    }
}
