use std::process::Command;

#[test]
fn test_hello_world() {
    let mut lexer = lumen_lexer::Lexer::new(r#"imprimir("¡Hola, LÚMEN!")"#);
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty(), "Lexer errors: {:?}", errors);
    assert!(tokens.len() >= 4);
}

#[test]
fn test_full_pipeline_hello() {
    let source = r#"imprimir("¡Hola, LÚMEN!")"#;
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty());

    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, errors) = parser.parse();
    assert!(errors.is_empty());
    assert!(!program.is_empty());
}

#[test]
fn test_full_pipeline_loop() {
    let source = "numero contador = 0
mientras (contador < 5) {
    imprimir(contador)
    contador = contador + 1
}";
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty());

    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, errors) = parser.parse();
    assert!(errors.is_empty());
    assert!(!program.is_empty());
}

#[test]
fn test_full_pipeline_func() {
    let source = "funcion numero suma(numero a, numero b) {
    retornar a + b
}
numero resultado = suma(3, 7)
imprimir(resultado)";
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty());

    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, errors) = parser.parse();
    assert!(errors.is_empty());
}

#[test]
fn test_semantic_errors_caught() {
    let source = r#"numero x = "hola""#;
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, errors) = lexer.tokenize();
    assert!(errors.is_empty());

    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, errors) = parser.parse();
    assert!(errors.is_empty());

    let sema = lumen_sema::SemanticAnalyzer::new();
    let errors = sema.analyze(&mut program);
    assert!(!errors.is_empty());
}

#[test]
fn test_bytecode_roundtrip() {
    use lumen_codegen::Bytecode;

    let mut bc = lumen_codegen::Bytecode {
        instructions: vec![
            lumen_codegen::Instruction::Simple(lumen_codegen::Opcode::Halt),
        ],
        strings: vec![],
        ints: vec![],
        nums: vec![],
        names: vec![],
    };

    let encoded = bc.encode();
    let (decoded, _) = Bytecode::decode(&encoded).unwrap();
    assert_eq!(decoded.instructions.len(), 1);
}

#[test]
fn test_vm_execution() {
    use lumen_codegen::bytecode::{Bytecode, Instruction, Opcode};
    use lumen_vm::VM;

    let mut bc = Bytecode {
        instructions: vec![
            Instruction::WithIdx(Opcode::PushStr, 0),
            Instruction::Simple(Opcode::Print),
            Instruction::Simple(Opcode::Halt),
        ],
        strings: vec!["test".to_string()],
        ints: vec![],
        nums: vec![],
        names: vec![],
        funcs: vec![],
    };
    let mut vm = VM::new(bc);
    assert!(vm.run().is_ok());
    assert_eq!(vm.output(), &["test"]);
}
