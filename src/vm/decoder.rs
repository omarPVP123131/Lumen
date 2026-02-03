use crate::instructions::OpCode;

pub struct Decoder<'a> {
    code: &'a [u8],
    pub ip: usize,
}

impl<'a> Decoder<'a> {
    pub fn new(code: &'a [u8]) -> Self {
        Self { code, ip: 0 }
    }

    #[inline(always)]
    pub fn read_u8(&mut self) -> u8 {
        let v = self.code[self.ip];
        self.ip += 1;
        v
    }

    #[inline(always)]
    pub fn read_u32(&mut self) -> u32 {
        let mut buf = [0u8; 4];
        buf.copy_from_slice(&self.code[self.ip..self.ip + 4]);
        self.ip += 4;
        u32::from_le_bytes(buf)
    }

    #[inline(always)]
    pub fn read_opcode(&mut self) -> OpCode {
        OpCode::from(self.read_u8())
            .expect("opcode inválido (verifier falló)")
    }

    #[inline(always)]
    pub fn jump(&mut self, target: usize) {
        self.ip = target;
    }
}
