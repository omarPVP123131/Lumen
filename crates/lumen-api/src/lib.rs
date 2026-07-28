// LÚMEN Compiler API — Embedded scripting engine for Rust
// Provides compile, check, and run functionality for LÚMEN source code.

use lumen_codegen::Codegen;
use lumen_ir::IRBuilder;
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;
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
#[allow(dead_code)]
pub struct LumenEngine {
    lib_dirs: Vec<PathBuf>,
}

impl LumenEngine {
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
        let (mut program, parse_errors) = parser.parse();

        if !parse_errors.is_empty() {
            return Err(parse_errors.iter().map(|e| format!("{:?}", e)).collect());
        }

        let sema = SemanticAnalyzer::new();
        let sem_errors = sema.analyze(&mut program);

        if !sem_errors.is_empty() {
            return Err(sem_errors.iter().map(|e| format!("{:?}", e)).collect());
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
        let (mut program, parse_errors) = parser.parse();

        if !parse_errors.is_empty() {
            return Err(format!("{:?}", parse_errors[0]));
        }

        let sema = SemanticAnalyzer::new();
        let sem_errors = sema.analyze(&mut program);

        if !sem_errors.is_empty() {
            return Err(format!("{:?}", sem_errors[0]));
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

        let (bytecode, _) =
            lumen_codegen::Bytecode::decode(&bytecode_data)
                .map_err(|e| format!("Decode error: {}", e))?;

        let mut vm = VM::new(bytecode);
        vm.run().map_err(|e| format!("Runtime error: {:?}", e))?;

        let output = vm.stack_top().map(|v| format!("{}", v)).unwrap_or_default();
        Ok(output)
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
}
