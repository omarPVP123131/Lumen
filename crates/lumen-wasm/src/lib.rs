use wasm_bindgen::prelude::*;
use lumen_parser::ast::Program;

#[wasm_bindgen]
pub struct LumenRuntime {
    output: String,
}

#[wasm_bindgen]
impl LumenRuntime {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        LumenRuntime { output: String::new() }
    }

    pub fn run(&mut self, source: &str) -> String {
        self.output.clear();

        // Lexer
        let lexer = lumen_lexer::Lexer::new(source);
        let (tokens, lex_errors) = lexer.tokenize();
        if !lex_errors.is_empty() {
            return format!("Error léxico [{}:{}]: {}", 
                lex_errors[0].pos.line, lex_errors[0].pos.col, lex_errors[0].message);
        }

        // Parser  
        let mut parser = lumen_parser::Parser::new(tokens);
        let (mut program, parse_errors) = parser.parse();
        if !parse_errors.is_empty() {
            return format!("Error sintáctico [{}]: {}", parse_errors[0].span.start.line, parse_errors[0].message);
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
            Ok(()) => {
                vm.output().join("\n")
            }
            Err(e) => {
                format!("Error runtime: {}", e)
            }
        }
    }

    pub fn version() -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }
}
