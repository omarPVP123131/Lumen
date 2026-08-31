use crate::bytecode::{Bytecode, Instruction};

pub fn disassemble(bc: &Bytecode) -> String {
    let mut output = String::new();
    output.push_str("; LÚMEN Bytecode Disassembly\n");
    output.push_str(&format!("; Strings: {} entries\n", bc.strings.len()));
    output.push_str(&format!("; Numbers: {} entries\n", bc.nums.len()));
    output.push_str(&format!("; Names: {} entries\n", bc.names.len()));
    for (i, n) in bc.names.iter().enumerate() {
        output.push_str(&format!(";   name[{}]=\"{}\"\n", i, n));
    }
    output.push_str(&format!("; Funcs: {} entries\n", bc.funcs.len()));
    for (i, f) in bc.funcs.iter().enumerate() {
        output.push_str(&format!(
            ";   func[{}]: name={}, params=[{}], start={}\n",
            i,
            f.name,
            f.params.join(","),
            f.start
        ));
    }
    output.push_str("; ---\n");

    for (i, instr) in bc.instructions.iter().enumerate() {
        output.push_str(&format!("{:04}: ", i));
        match instr {
            Instruction::Simple(op) => {
                output.push_str(&format!("{:?}\n", op));
            }
            Instruction::WithNum(op, n) => {
                output.push_str(&format!("{:?} {}\n", op, n));
            }
            Instruction::WithStr(op, s) => {
                output.push_str(&format!("{:?} \"{}\"\n", op, s));
            }
            Instruction::WithBool(op, b) => {
                output.push_str(&format!("{:?} {}\n", op, b));
            }
            Instruction::WithIdx(op, idx) => {
                output.push_str(&format!("{:?} @{}\n", op, idx));
            }
            // v3.5.20 super-opcodes
            Instruction::FusedBinK { op, a, k, d } => {
                output.push_str(&format!("FUSED_BIN_K op={} @{} k={} -> @{}\n", op, a, k, d));
            }
            Instruction::FusedBin { op, a, b, d } => {
                output.push_str(&format!("FUSED_BIN op={} @{} @{} -> @{}\n", op, a, b, d));
            }
            Instruction::FusedBinKLocal { op, a, k, d } => {
                output.push_str(&format!(
                    "FUSED_BIN_K_LOCAL op={} @{} k={} -> @{}\n",
                    op, a, k, d
                ));
            }
            Instruction::FusedBinLocal { op, a, b, d } => {
                output.push_str(&format!(
                    "FUSED_BIN_LOCAL op={} @{} @{} -> @{}\n",
                    op, a, b, d
                ));
            }
            Instruction::FusedCmpKJmp { op, a, k, target } => {
                output.push_str(&format!(
                    "FUSED_CMP_K_JMP op={} @{} k={} -> @{}\n",
                    op, a, k, target
                ));
            }
            Instruction::FusedCmpJmp { op, a, b, target } => {
                output.push_str(&format!(
                    "FUSED_CMP_JMP op={} @{} @{} -> @{}\n",
                    op, a, b, target
                ));
            }
            Instruction::FusedBinCmpJmp {
                op1,
                op2,
                a,
                b,
                c,
                target,
            } => {
                output.push_str(&format!(
                    "FUSED_BIN_CMP_JMP op1={} op2={} @{} @{} @{} -> @{}\n",
                    op1, op2, a, b, c, target
                ));
            }
            Instruction::FusedBinKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => {
                output.push_str(&format!(
                    "FUSED_BIN_K_CMP_JMP op1={} op2={} @{} @{} k={} -> @{}\n",
                    op1, op2, a, b, k, target
                ));
            }
            Instruction::FusedBinKKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                target,
            } => {
                output.push_str(&format!(
                    "FUSED_BIN_KK_CMP_JMP op1={} op2={} @{} b={} k={} -> @{}\n",
                    op1, op2, a, b, k, target
                ));
            }
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::*;

    #[test]
    fn test_disassemble_empty() {
        let bc = Bytecode::new();
        let output = disassemble(&bc);
        assert!(output.contains("Bytecode"));
    }

    #[test]
    fn test_disassemble_halt() {
        let mut bc = Bytecode::new();
        bc.instructions
            .push(Instruction::Simple(crate::bytecode::Opcode::Halt));
        let output = disassemble(&bc);
        assert!(output.contains("Halt"));
    }

    #[test]
    fn test_disassemble_with_num() {
        let mut bc = Bytecode::new();
        bc.instructions
            .push(Instruction::WithNum(crate::bytecode::Opcode::PushNum, 42.0));
        let output = disassemble(&bc);
        assert!(output.contains("PushNum 42"));
    }

    #[test]
    fn test_disassemble_with_str() {
        let mut bc = Bytecode::new();
        bc.instructions.push(Instruction::WithStr(
            crate::bytecode::Opcode::PushStr,
            "hola".to_string(),
        ));
        let output = disassemble(&bc);
        assert!(output.contains("PushStr"));
        assert!(output.contains("hola"));
    }

    #[test]
    fn test_disassemble_with_bool() {
        let mut bc = Bytecode::new();
        bc.instructions.push(Instruction::WithBool(
            crate::bytecode::Opcode::PushBool,
            true,
        ));
        let output = disassemble(&bc);
        assert!(output.contains("PushBool true"));
    }

    #[test]
    fn test_disassemble_with_idx() {
        let mut bc = Bytecode::new();
        bc.instructions
            .push(Instruction::WithIdx(crate::bytecode::Opcode::Call, 3));
        let output = disassemble(&bc);
        assert!(output.contains("Call @3"));
    }

    #[test]
    fn test_disassemble_multiple_instructions() {
        let mut bc = Bytecode::new();
        bc.instructions
            .push(Instruction::Simple(crate::bytecode::Opcode::Halt));
        bc.instructions
            .push(Instruction::WithIdx(crate::bytecode::Opcode::PushNum, 0));
        bc.instructions.push(Instruction::WithBool(
            crate::bytecode::Opcode::PushBool,
            false,
        ));
        let output = disassemble(&bc);
        assert!(output.contains("0000: Halt"));
        assert!(output.contains("0001: PushNum @0"));
        assert!(output.contains("0002: PushBool false"));
    }

    #[test]
    fn test_disassemble_header_shows_metadata() {
        let mut bc = Bytecode::new();
        bc.strings.push("foo".to_string());
        bc.nums.push(12.34);
        bc.names.push("bar".to_string());
        let output = disassemble(&bc);
        assert!(output.contains("Strings: 1 entries"));
        assert!(output.contains("Numbers: 1 entries"));
        assert!(output.contains("Names: 1 entries"));
    }
}
