#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum OpCode {
    PushNum = 0x01,

    Add = 0x10,
    Sub = 0x11,
    Mul = 0x12,
    Div = 0x13,

    Eq = 0x20,
    Lt = 0x21,
    Gt = 0x22,

    Store = 0x30,
    Load  = 0x31,

    Jmp = 0x40,
    JmpIfFalse = 0x41,

    Print = 0x50,
    DebugStack = 0x51,

    Halt = 0xFF,
}

impl OpCode {
    #[inline(always)]
    pub fn from(byte: u8) -> Option<Self> {
        use OpCode::*;
        Some(match byte {
            0x01 => PushNum,

            0x10 => Add,
            0x11 => Sub,
            0x12 => Mul,
            0x13 => Div,

            0x20 => Eq,
            0x21 => Lt,
            0x22 => Gt,

            0x30 => Store,
            0x31 => Load,

            0x40 => Jmp,
            0x41 => JmpIfFalse,

            0x50 => Print,
            0x51 => DebugStack,

            0xFF => Halt,
            _ => return None,
        })
    }
}
