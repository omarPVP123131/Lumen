pub mod lexer;
pub mod ast;
pub mod parser;
pub mod codegen;

use lexer::Lexer;
use parser::Parser;
use codegen::CodeGenerator;
use crate::vm::constant_pool::ConstantPool;

/// Resultado completo de compilación
pub struct CompiledProgram {
    pub bytecode: Vec<u8>,
    pub pool: ConstantPool,
}

pub fn compile(source: &str) -> Result<CompiledProgram, String> {
    // Fase 1: Lexer
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();

    // Fase 2: Parser
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;

    // Fase 3: Codegen
    let codegen = CodeGenerator::new();
    let (bytecode, pool) = codegen.generate(&program);

    Ok(CompiledProgram { bytecode, pool })
}
