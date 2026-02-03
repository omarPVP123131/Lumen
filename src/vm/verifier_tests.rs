use crate::vm::verifier::verify;
use crate::instructions::OpCode;

#[test]
fn invalid_bytecode_table() {
    let cases: &[(&str, Vec<u8>)] = &[
        ("opcode invalido", vec![255]),
        ("push truncado", vec![OpCode::PushNum as u8, 1, 2]),
        ("add sin stack", vec![OpCode::Add as u8]),
        ("store sin valor", vec![OpCode::Store as u8, 0, 0, 0, 0]),
        ("jmp truncado", vec![OpCode::Jmp as u8, 1, 2]),
        ("jmpiffalse sin stack", vec![OpCode::JmpIfFalse as u8, 0, 0, 0, 0]),
    ];

    for (name, code) in cases {
        assert!(
            verify(code).is_err(),
            "el caso '{}' debería fallar",
            name
        );
    }
}

#[test]
fn valid_bytecode() {
    let code = vec![
        OpCode::PushNum as u8, 0, 0, 0, 0,
        OpCode::PushNum as u8, 0, 0, 0, 1,
        OpCode::Add as u8,
        OpCode::Halt as u8,
    ];

    assert!(verify(&code).is_ok());
}
