// src/instructions.rs

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OpCode {
    // === CAPA 1: Básicas ===
    PushNum     = 0x01,
    Add         = 0x02,
    Print       = 0x03,
    Halt        = 0xFF,

    // === CAPA 2: Aritmética ===
    Sub         = 0x04,
    Mul         = 0x05,
    Div         = 0x06,

    // === CAPA 2: Variables ===
    Store       = 0x10,
    Load        = 0x11,

    // === CAPA 2: Comparaciones ===
    Eq          = 0x20,
    Lt          = 0x21,
    Gt          = 0x22,

    // === CAPA 2: Control de flujo ===
    Jmp         = 0x30,
    JmpIfFalse  = 0x31,

    // === CAPA 3: Tipos de alto nivel ===
    PushStr     = 0x40,

    // === DEBUG ===
    DebugStack  = 0xFE,
}

impl OpCode {
    pub fn from(byte: u8) -> Option<Self> {
        match byte {
            // Capa 1
            0x01 => Some(OpCode::PushNum),
            0x02 => Some(OpCode::Add),
            0x03 => Some(OpCode::Print),
            0xFF => Some(OpCode::Halt),

            // Capa 2 - Aritmética
            0x04 => Some(OpCode::Sub),
            0x05 => Some(OpCode::Mul),
            0x06 => Some(OpCode::Div),

            // Capa 2 - Variables
            0x10 => Some(OpCode::Store),
            0x11 => Some(OpCode::Load),

            // Capa 2 - Comparaciones
            0x20 => Some(OpCode::Eq),
            0x21 => Some(OpCode::Lt),
            0x22 => Some(OpCode::Gt),

            // Capa 2 - Control de flujo
            0x30 => Some(OpCode::Jmp),
            0x31 => Some(OpCode::JmpIfFalse),

            // Capa 3 - Strings
            0x40 => Some(OpCode::PushStr),

            // Debug
            0xFE => Some(OpCode::DebugStack),

            _ => None,
        }
    }

    #[allow(dead_code)]
    pub fn name(&self) -> &'static str {
        match self {
            OpCode::PushNum => "PUSH_NUM",
            OpCode::Add => "ADD",
            OpCode::Print => "PRINT",
            OpCode::Halt => "HALT",
            OpCode::Sub => "SUB",
            OpCode::Mul => "MUL",
            OpCode::Div => "DIV",
            OpCode::Store => "STORE",
            OpCode::Load => "LOAD",
            OpCode::Eq => "EQ",
            OpCode::Lt => "LT",
            OpCode::Gt => "GT",
            OpCode::Jmp => "JMP",
            OpCode::JmpIfFalse => "JMP_IF_FALSE",
            OpCode::PushStr => "PUSH_STR",
            OpCode::DebugStack => "DEBUG_STACK",
        }
    }
}
