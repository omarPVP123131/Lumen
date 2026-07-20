pub const CHUNK_MAGIC: &[u8; 4] = b"LUMN";
pub const CHUNK_VERSION: u32 = 6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Opcode {
    Nop = 0,
    PushInt = 1,
    PushNum = 2,
    PushStr = 3,
    PushBool = 4,
    Load = 5,
    Store = 6,
    Add = 7,
    Sub = 8,
    Mul = 9,
    Div = 10,
    Eq = 11,
    Neq = 12,
    Lt = 13,
    Le = 14,
    Gt = 15,
    Ge = 16,
    And = 17,
    Or = 18,
    Neg = 19,
    Not = 20,
    Call = 21,
    Ret = 22,
    Print = 23,
    Read = 24,
    Jmp = 25,
    JmpIf = 26,
    Halt = 27,
    ArrayNew = 28,
    ArrayGet = 29,
    ArraySet = 30,
    ArrayLen = 31,
    ArrayPush = 32,
    FuncRef = 33,
    CallValue = 34,
    StructNew = 35,
    StructGet = 36,
    StructSet = 37,
    ResultOk = 38,
    ResultErr = 39,
    TryUnwrap = 40,
}

impl Opcode {
    pub fn from_u8(b: u8) -> Option<Opcode> {
        match b {
            0 => Some(Opcode::Nop),
            1 => Some(Opcode::PushInt),
            2 => Some(Opcode::PushNum),
            3 => Some(Opcode::PushStr),
            4 => Some(Opcode::PushBool),
            5 => Some(Opcode::Load),
            6 => Some(Opcode::Store),
            7 => Some(Opcode::Add),
            8 => Some(Opcode::Sub),
            9 => Some(Opcode::Mul),
            10 => Some(Opcode::Div),
            11 => Some(Opcode::Eq),
            12 => Some(Opcode::Neq),
            13 => Some(Opcode::Lt),
            14 => Some(Opcode::Le),
            15 => Some(Opcode::Gt),
            16 => Some(Opcode::Ge),
            17 => Some(Opcode::And),
            18 => Some(Opcode::Or),
            19 => Some(Opcode::Neg),
            20 => Some(Opcode::Not),
            21 => Some(Opcode::Call),
            22 => Some(Opcode::Ret),
            23 => Some(Opcode::Print),
            24 => Some(Opcode::Read),
            25 => Some(Opcode::Jmp),
            26 => Some(Opcode::JmpIf),
            27 => Some(Opcode::Halt),
            28 => Some(Opcode::ArrayNew),
            29 => Some(Opcode::ArrayGet),
            30 => Some(Opcode::ArraySet),
            31 => Some(Opcode::ArrayLen),
             32 => Some(Opcode::ArrayPush),
             33 => Some(Opcode::FuncRef),
             34 => Some(Opcode::CallValue),
              35 => Some(Opcode::StructNew),
              36 => Some(Opcode::StructGet),
              37 => Some(Opcode::StructSet),
              38 => Some(Opcode::ResultOk),
              39 => Some(Opcode::ResultErr),
              40 => Some(Opcode::TryUnwrap),
            _ => None,
        }
    }

    pub fn to_u8(self) -> u8 {
        self as u8
    }
}

#[derive(Debug, Clone)]
pub enum Instruction {
    Simple(Opcode),
    WithNum(Opcode, f64),
    WithStr(Opcode, String),
    WithBool(Opcode, bool),
    WithIdx(Opcode, usize),
}

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub name: String,
    pub params: Vec<String>,
    pub start: usize,
}

#[derive(Debug, Clone)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub strings: Vec<String>,
    pub ints: Vec<i64>,
    pub nums: Vec<f64>,
    pub names: Vec<String>,
    pub funcs: Vec<FuncMeta>,
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            strings: Vec::new(),
            ints: Vec::new(),
            nums: Vec::new(),
            names: Vec::new(),
            funcs: Vec::new(),
        }
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        buf.extend_from_slice(CHUNK_MAGIC);
        buf.extend_from_slice(&CHUNK_VERSION.to_le_bytes());
        buf.extend_from_slice(&(self.strings.len() as u32).to_le_bytes());
        for s in &self.strings {
            buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
            buf.extend_from_slice(s.as_bytes());
        }
        buf.extend_from_slice(&(self.ints.len() as u32).to_le_bytes());
        for n in &self.ints {
            buf.extend_from_slice(&n.to_le_bytes());
        }
        buf.extend_from_slice(&(self.nums.len() as u32).to_le_bytes());
        for n in &self.nums {
            buf.extend_from_slice(&n.to_le_bytes());
        }
        buf.extend_from_slice(&(self.names.len() as u32).to_le_bytes());
        for n in &self.names {
            buf.extend_from_slice(&(n.len() as u32).to_le_bytes());
            buf.extend_from_slice(n.as_bytes());
        }
        buf.extend_from_slice(&(self.funcs.len() as u32).to_le_bytes());
        for func in &self.funcs {
            buf.extend_from_slice(&(func.name.len() as u32).to_le_bytes());
            buf.extend_from_slice(func.name.as_bytes());
            buf.extend_from_slice(&(func.params.len() as u32).to_le_bytes());
            for p in &func.params {
                buf.extend_from_slice(&(p.len() as u32).to_le_bytes());
                buf.extend_from_slice(p.as_bytes());
            }
            buf.extend_from_slice(&func.start.to_le_bytes() as &[u8]);
        }
        buf.extend_from_slice(&(self.instructions.len() as u32).to_le_bytes());
        for instr in &self.instructions {
            match instr {
                Instruction::Simple(op) => {
                    buf.push(0);
                    buf.push(op.to_u8());
                }
                Instruction::WithNum(op, n) => {
                    buf.push(1);
                    buf.push(op.to_u8());
                    let idx = self.nums.iter().position(|x| (x - n).abs() < f64::EPSILON).unwrap_or(0);
                    buf.extend_from_slice(&(idx as u32).to_le_bytes());
                }
                Instruction::WithStr(op, s) => {
                    buf.push(2);
                    buf.push(op.to_u8());
                    let idx = self.strings.iter().position(|x| x == s).unwrap_or(0);
                    buf.extend_from_slice(&(idx as u32).to_le_bytes());
                }
                Instruction::WithBool(op, b) => {
                    buf.push(3);
                    buf.push(op.to_u8());
                    buf.push(if *b { 1 } else { 0 });
                }
                Instruction::WithIdx(op, idx) => {
                    buf.push(4);
                    buf.push(op.to_u8());
                    buf.extend_from_slice(&(idx.clone() as u32).to_le_bytes());
                }
            }
        }
        buf
    }

    pub fn decode(data: &[u8]) -> Result<(Self, Vec<(usize, String)>), String> {
        if data.len() < 8 {
            return Err("Archivo muy corto".to_string());
        }
        if &data[0..4] != CHUNK_MAGIC {
            return Err("Magic number inválido".to_string());
        }
        let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
        if version != CHUNK_VERSION {
            return Err(format!("Versión {} de bytecode no soportada (esperada {})", version, CHUNK_VERSION));
        }

        let mut pos = 8;
        let mut warnings = Vec::new();

        let num_strings = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        pos += 4;
        let mut strings = Vec::with_capacity(num_strings);
        for _ in 0..num_strings {
            if pos + 4 > data.len() {
                return Err("Datos corruptos: se esperaban más strings".to_string());
            }
            let len = u32::from_le_bytes([
                data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
            ]) as usize;
            pos += 4;
            if pos + len > data.len() {
                warnings.push((pos, format!("String de longitud {} excede el buffer", len)));
                break;
            }
            let s = String::from_utf8_lossy(&data[pos..pos + len]).to_string();
            strings.push(s);
            pos += len;
        }

        let num_ints = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        pos += 4;
        let mut ints = Vec::with_capacity(num_ints);
        for _ in 0..num_ints {
            if pos + 8 > data.len() {
                return Err("Datos corruptos: se esperaban más enteros".to_string());
            }
            let n = i64::from_le_bytes([
                data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
                data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7],
            ]);
            ints.push(n);
            pos += 8;
        }

        let num_nums = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        pos += 4;
        let mut nums = Vec::with_capacity(num_nums);
        for _ in 0..num_nums {
            if pos + 8 > data.len() {
                return Err("Datos corruptos: se esperaban más números".to_string());
            }
            let n = f64::from_le_bytes([
                data[pos], data[pos + 1], data[pos + 2], data[pos + 3],
                data[pos + 4], data[pos + 5], data[pos + 6], data[pos + 7],
            ]);
            nums.push(n);
            pos += 8;
        }

        let num_names = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        pos += 4;
        let mut names = Vec::with_capacity(num_names);
        for _ in 0..num_names {
            if pos + 4 > data.len() {
                return Err("Datos corruptos: se esperaban más nombres".to_string());
            }
            let len = u32::from_le_bytes([
                data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
            ]) as usize;
            pos += 4;
            if pos + len > data.len() {
                break;
            }
            let s = String::from_utf8_lossy(&data[pos..pos + len]).to_string();
            names.push(s);
            pos += len;
        }

        let num_funcs = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        pos += 4;
        let mut funcs = Vec::with_capacity(num_funcs);
        for _ in 0..num_funcs {
            if pos + 4 > data.len() { break; }
            let name_len = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
            pos += 4;
            if pos + name_len > data.len() { break; }
            let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
            pos += name_len;
            if pos + 4 > data.len() { break; }
            let num_params = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
            pos += 4;
            let mut params = Vec::with_capacity(num_params);
            for _ in 0..num_params {
                if pos + 4 > data.len() { break; }
                let plen = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                pos += 4;
                if pos + plen > data.len() { break; }
                let p = String::from_utf8_lossy(&data[pos..pos + plen]).to_string();
                params.push(p);
                pos += plen;
            }
            if pos + 8 > data.len() { break; }
            let start = u64::from_le_bytes([
                data[pos], data[pos+1], data[pos+2], data[pos+3],
                data[pos+4], data[pos+5], data[pos+6], data[pos+7],
            ]) as usize;
            pos += 8;
            funcs.push(FuncMeta { name, params, start });
        }

        let num_instrs = u32::from_le_bytes([
            data[pos], data[pos + 1], data[pos + 2], data[pos + 3]
        ]) as usize;
        pos += 4;
        let mut instructions = Vec::with_capacity(num_instrs);
        for _ in 0..num_instrs {
            if pos >= data.len() {
                break;
            }
            let tag = data[pos];
            pos += 1;
            if pos >= data.len() {
                break;
            }
            let op_byte = data[pos];
            pos += 1;
            let op = Opcode::from_u8(op_byte).unwrap_or(Opcode::Nop);
            match tag {
                0 => instructions.push(Instruction::Simple(op)),
                1 => {
                    if pos + 4 > data.len() { break; }
                    let idx = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                    pos += 4;
                    let n = nums.get(idx).copied().unwrap_or(0.0);
                    instructions.push(Instruction::WithNum(op, n));
                }
                2 => {
                    if pos + 4 > data.len() { break; }
                    let idx = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                    pos += 4;
                    let s = strings.get(idx).cloned().unwrap_or_default();
                    instructions.push(Instruction::WithStr(op, s));
                }
                3 => {
                    if pos >= data.len() { break; }
                    let b = data[pos] != 0;
                    pos += 1;
                    instructions.push(Instruction::WithBool(op, b));
                }
                4 => {
                    if pos + 4 > data.len() { break; }
                    let idx = u32::from_le_bytes([data[pos], data[pos+1], data[pos+2], data[pos+3]]) as usize;
                    pos += 4;
                    instructions.push(Instruction::WithIdx(op, idx));
                }
                _ => {
                    warnings.push((pos, format!("Tag de instrucción desconocido: {}", tag)));
                }
            }
        }
        Ok((Bytecode { instructions, strings, ints, nums, names, funcs }, warnings))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_empty() {
        let bc = Bytecode::new();
        let encoded = bc.encode();
        let (decoded, _) = Bytecode::decode(&encoded).unwrap();
        assert_eq!(decoded.instructions.len(), 0);
    }

    #[test]
    fn test_roundtrip_simple() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::Simple(Opcode::Halt),
                Instruction::WithNum(Opcode::PushNum, 42.0),
                Instruction::WithStr(Opcode::PushStr, "hola".to_string()),
                Instruction::WithBool(Opcode::PushBool, true),
            ],
            strings: vec!["hola".to_string()],
            ints: vec![],
            nums: vec![42.0],
            names: vec![],
            funcs: vec![],
        };
        let encoded = bc.encode();
        let (decoded, _) = Bytecode::decode(&encoded).unwrap();
        assert_eq!(decoded.instructions.len(), 4);
        assert_eq!(decoded.strings, vec!["hola"]);
        assert_eq!(decoded.nums, vec![42.0]);
    }

    #[test]
    fn test_invalid_magic() {
        let result = Bytecode::decode(b"XXXX\x01\x00\x00\x00");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Magic"));
    }

    #[test]
    fn test_invalid_version() {
        let mut data = CHUNK_MAGIC.to_vec();
        data.extend_from_slice(&999u32.to_le_bytes());
        let result = Bytecode::decode(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Versión"));
    }

    #[test]
    fn test_truncated_data() {
        let result = Bytecode::decode(&[0x4c, 0x55, 0x4d, 0x4e]);
        assert!(result.is_err());
    }
}
