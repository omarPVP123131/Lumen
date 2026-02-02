// src/compiler/mod.rs
pub mod lexer;
pub mod ast;
pub mod parser;
pub mod codegen;

use lexer::Lexer;
use parser::Parser;
use codegen::CodeGenerator;

pub fn compile(source: &str) -> Result<Vec<u8>, String> {
    // Fase 1: Lexer (texto -> tokens)
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize();
    
    // Fase 2: Parser (tokens -> AST)
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    
    // Fase 3: Code Generation (AST -> bytecode)
    let mut codegen = CodeGenerator::new();
    let bytecode = codegen.generate(&program);
    
    Ok(bytecode)
}