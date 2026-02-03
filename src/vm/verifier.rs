use crate::instructions::OpCode;

pub fn verify(code: &[u8]) -> Result<(), String> {
    let mut ip = 0usize;
    let mut stack: isize = 0;

    while ip < code.len() {
        let op = OpCode::from(code[ip])
            .ok_or_else(|| format!("opcode inválido en {}", ip))?;
        ip += 1;

        match op {
            OpCode::PushNum => {
                if ip + 4 > code.len() {
                    return Err("PushNum truncado".into());
                }
                ip += 4;
                stack += 1;
            }

            OpCode::Add
            | OpCode::Sub
            | OpCode::Mul
            | OpCode::Div
            | OpCode::Eq
            | OpCode::Lt
            | OpCode::Gt => {
                stack -= 1;
            }

            OpCode::Load => {
                if ip + 4 > code.len() {
                    return Err("Load truncado".into());
                }
                ip += 4;
                stack += 1;
            }

            OpCode::Store => {
                if ip + 4 > code.len() {
                    return Err("Store truncado".into());
                }
                ip += 4;
                stack -= 1;
            }

            OpCode::Jmp | OpCode::JmpIfFalse => {
                if ip + 4 > code.len() {
                    return Err("Jump truncado".into());
                }
                let target =
                    u32::from_le_bytes(code[ip..ip + 4].try_into().unwrap()) as usize;
                if target >= code.len() {
                    return Err("Jump fuera de rango".into());
                }
                ip += 4;

                if op == OpCode::JmpIfFalse {
                    stack -= 1;
                }
            }

            OpCode::Print => {
                stack -= 1;
            }

            OpCode::DebugStack => {}

            OpCode::Halt => break,
        }

        if stack < 0 {
            return Err("stack underflow".into());
        }
    }

    Ok(())
}
