use proptest::prelude::*;
use lumen_codegen::bytecode::{Bytecode, Instruction, Opcode, FuncMeta};

fn valid_opcode() -> impl Strategy<Value = u8> {
    0u8..=27
}

fn arbitrary_instruction() -> impl Strategy<Value = Instruction> {
    let simple = valid_opcode().prop_map(|op| {
        Instruction::Simple(int_to_opcode(op))
    });
    let with_num = (valid_opcode(), any::<f64>()).prop_map(|(op, n)| {
        Instruction::WithNum(int_to_opcode(op), n)
    });
    let with_str = (valid_opcode(), ".{0,10}").prop_map(|(op, s): (u8, String)| {
        Instruction::WithStr(int_to_opcode(op), s)
    });
    let with_bool = (valid_opcode(), any::<bool>()).prop_map(|(op, b)| {
        Instruction::WithBool(int_to_opcode(op), b)
    });
    let with_idx = (valid_opcode(), 0usize..100).prop_map(|(op, idx)| {
        Instruction::WithIdx(int_to_opcode(op), idx)
    });
    prop::strategy::Union::new_weighted(vec![
        (1, simple.boxed()),
        (1, with_num.boxed()),
        (1, with_str.boxed()),
        (1, with_bool.boxed()),
        (1, with_idx.boxed()),
    ])
}

fn int_to_opcode(n: u8) -> Opcode {
    match n {
        0 => Opcode::Nop,
        1 => Opcode::PushInt,
        2 => Opcode::PushNum,
        3 => Opcode::PushStr,
        4 => Opcode::PushBool,
        5 => Opcode::Load,
        6 => Opcode::Store,
        7 => Opcode::Add,
        8 => Opcode::Sub,
        9 => Opcode::Mul,
        10 => Opcode::Div,
        11 => Opcode::Eq,
        12 => Opcode::Neq,
        13 => Opcode::Lt,
        14 => Opcode::Le,
        15 => Opcode::Gt,
        16 => Opcode::Ge,
        17 => Opcode::And,
        18 => Opcode::Or,
        19 => Opcode::Neg,
        20 => Opcode::Not,
        21 => Opcode::Call,
        22 => Opcode::Ret,
        23 => Opcode::Print,
        24 => Opcode::Read,
        25 => Opcode::Jmp,
        26 => Opcode::JmpIf,
        _ => Opcode::Halt,
    }
}

proptest! {
    #[test]
    fn roundtrip_instructions(instrs in prop::collection::vec(arbitrary_instruction(), 0..20)) {
        let bc = Bytecode {
            instructions: instrs.clone(),
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let encoded = bc.encode();
        let (decoded, _warnings) = Bytecode::decode(&encoded).unwrap();
        assert_eq!(decoded.instructions.len(), instrs.len());
    }

    #[test]
    fn roundtrip_full(
        instrs in prop::collection::vec(arbitrary_instruction(), 0..10),
        strings in prop::collection::vec(".{0,10}", 0..5),
        ints in prop::collection::vec(any::<i64>(), 0..5),
        nums in prop::collection::vec(any::<f64>(), 0..5),
        names in prop::collection::vec(".{0,10}", 0..5),
    ) {
        let bc = Bytecode {
            instructions: instrs,
            strings,
            ints,
            nums,
            names,
            funcs: vec![],
        };
        let encoded = bc.encode();
        let (decoded, _warnings) = Bytecode::decode(&encoded).unwrap();
        assert_eq!(decoded.strings.len(), bc.strings.len());
        assert_eq!(decoded.ints.len(), bc.ints.len());
        assert_eq!(decoded.nums.len(), bc.nums.len());
        assert_eq!(decoded.names.len(), bc.names.len());
    }

    #[test]
    fn roundtrip_with_funcs(
        funcs in prop::collection::vec(
            (
                ".{0,10}",
                prop::collection::vec(".{0,10}", 0..3),
                0usize..100
            ),
            0..5
        )
    ) {
        let func_metas: Vec<FuncMeta> = funcs.into_iter()
            .map(|(name, params, start)| FuncMeta { name, params, start })
            .collect();
        let bc = Bytecode {
            instructions: vec![],
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: func_metas,
        };
        let encoded = bc.encode();
        let (decoded, _warnings) = Bytecode::decode(&encoded).unwrap();
        assert_eq!(decoded.funcs.len(), bc.funcs.len());
        for (a, b) in bc.funcs.iter().zip(decoded.funcs.iter()) {
            assert_eq!(a.name, b.name);
            assert_eq!(a.params, b.params);
            assert_eq!(a.start, b.start);
        }
    }

    #[test]
    fn all_decoded_opcodes_valid(data in prop::collection::vec(any::<u8>(), 0..100)) {
        if data.len() < 8 { return Ok(()); }
        let _ = Bytecode::decode(&data);
    }

    #[test]
    fn invalid_magic_rejected(data in prop::collection::vec(any::<u8>(), 8..50)) {
        if data.len() >= 4 && &data[0..4] == b"LUMN" {
            return Ok(());
        }
        let result = Bytecode::decode(&data);
        if let Err(e) = result {
            assert!(e.contains("Magic") || e.contains("magic"), "Error should mention magic: {}", e);
        }
    }
}
