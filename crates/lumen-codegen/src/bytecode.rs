pub const CHUNK_MAGIC: &[u8; 4] = b"LUMN";
pub const CHUNK_VERSION: u32 = 7;

#[derive(Debug, Clone)]
pub enum DefaultValue {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}

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
    OptionSome = 41,
    OptionNone = 42,
    EnumCtor = 43,
    TupleNew = 44,
    TupleAccess = 45,
    Mod = 46,
    BitOr = 47,
    BitAnd = 48,
    ShiftLeft = 49,
    ShiftRight = 50,
    Concat = 51,
    MatchType = 52,
    MatchPayload = 53,
    BitXor = 54,
    BitNot = 55,
    ArrayPushVar = 56,
    PushHandler = 57,
    PopHandler = 58,
    StoreLocal = 59,
    ScopePush = 60,
    ScopePop = 61,
    MatchVariant = 62,
    MakeRef = 63,
    /// v3.5.40: `a[i] = v` con `a` variable simple — muta el slot in-place
    /// (espejo de ArrayPushVar). Evita que Arc::make_mut clone el Vec entero
    /// por escritura (O(n²) → O(n) en cribas/bucles de marcado).
    ArraySetVar = 64,
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
            41 => Some(Opcode::OptionSome),
            42 => Some(Opcode::OptionNone),
            43 => Some(Opcode::EnumCtor),
            44 => Some(Opcode::TupleNew),
            45 => Some(Opcode::TupleAccess),
            46 => Some(Opcode::Mod),
            47 => Some(Opcode::BitOr),
            48 => Some(Opcode::BitAnd),
            49 => Some(Opcode::ShiftLeft),
            50 => Some(Opcode::ShiftRight),
            51 => Some(Opcode::Concat),
            52 => Some(Opcode::MatchType),
            53 => Some(Opcode::MatchPayload),
            54 => Some(Opcode::BitXor),
            55 => Some(Opcode::BitNot),
            56 => Some(Opcode::ArrayPushVar),
            57 => Some(Opcode::PushHandler),
            58 => Some(Opcode::PopHandler),
            59 => Some(Opcode::StoreLocal),
            60 => Some(Opcode::ScopePush),
            61 => Some(Opcode::ScopePop),
            62 => Some(Opcode::MatchVariant),
            63 => Some(Opcode::MakeRef),
            64 => Some(Opcode::ArraySetVar),
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
    /// v3.5.20 super-opcodes (solo el pipeline Rust los emite; el compilador
    /// self-hosted nunca los genera → fixpoint intacto). `op` usa la
    /// numeración del backend C: 1=Add 3=Sub 4=Mul 7=Eq 8=Ne 9=Lt 10=Le
    /// 11=Gt 12=Ge.
    FusedBinK {
        op: u8,
        a: usize,
        k: i64,
        d: usize,
    },
    FusedBin {
        op: u8,
        a: usize,
        b: usize,
        d: usize,
    },
    /// v3.5.41 (bug #10): igual que FusedBinK/FusedBin pero con semántica
    /// de DECLARACIÓN (StoreLocal): el destino se escribe en el scope
    /// ACTUAL del frame (se crea el binding si no existe) — NUNCA en scopes
    /// de frames ancestros. Solo el pipeline Rust los emite (el compilador
    /// self-hosted nunca los genera → fixpoint intacto).
    FusedBinKLocal {
        op: u8,
        a: usize,
        k: i64,
        d: usize,
    },
    FusedBinLocal {
        op: u8,
        a: usize,
        b: usize,
        d: usize,
    },
    FusedCmpKJmp {
        op: u8,
        a: usize,
        k: i64,
        target: usize,
    },
    FusedCmpJmp {
        op: u8,
        a: usize,
        b: usize,
        target: usize,
    },
    /// v3.5.31: aritmética + comparación + salto (6 IR → 1). op1 ∈
    /// {1 Add, 3 Sub, 4 Mul, 5 Div, 6 Mod}; op2 ∈ {7 Eq..12 Ge}.
    /// Semántica: t = a op1 b; si (t op2 c) es FALSO → salta a target.
    FusedBinCmpJmp {
        op1: u8,
        op2: u8,
        a: usize,
        b: usize,
        c: usize,
        target: usize,
    },
    FusedBinKCmpJmp {
        op1: u8,
        op2: u8,
        a: usize,
        b: usize,
        k: i64,
        target: usize,
    },
    FusedBinKKCmpJmp {
        op1: u8,
        op2: u8,
        a: usize,
        b: i64,
        k: i64,
        target: usize,
    },
}

#[derive(Debug, Clone)]
pub struct FuncMeta {
    pub name: String,
    pub params: Vec<String>,
    pub defaults: Vec<Option<DefaultValue>>,
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
            buf.extend_from_slice(&(func.defaults.len() as u32).to_le_bytes());
            for d in &func.defaults {
                match d {
                    None => buf.push(0),
                    Some(DefaultValue::Int(v)) => {
                        buf.push(1);
                        buf.extend_from_slice(&v.to_le_bytes());
                    }
                    Some(DefaultValue::Float(v)) => {
                        buf.push(2);
                        buf.extend_from_slice(&v.to_le_bytes());
                    }
                    Some(DefaultValue::Str(s)) => {
                        buf.push(3);
                        buf.extend_from_slice(&(s.len() as u32).to_le_bytes());
                        buf.extend_from_slice(s.as_bytes());
                    }
                    Some(DefaultValue::Bool(b)) => {
                        buf.push(4);
                        buf.push(if *b { 1 } else { 0 });
                    }
                }
            }
            buf.extend_from_slice(&(func.start as u64).to_le_bytes());
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
                    let idx = self
                        .nums
                        .iter()
                        .position(|x| (x - n).abs() < f64::EPSILON)
                        .unwrap_or(0);
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
                    buf.extend_from_slice(&(*idx as u32).to_le_bytes());
                }
                Instruction::FusedBinK { op, a, k, d } => {
                    buf.push(5);
                    buf.push(*op);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&k.to_le_bytes());
                    buf.extend_from_slice(&(*d as u32).to_le_bytes());
                }
                Instruction::FusedBin { op, a, b, d } => {
                    buf.push(6);
                    buf.push(*op);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&(*b as u32).to_le_bytes());
                    buf.extend_from_slice(&(*d as u32).to_le_bytes());
                }
                Instruction::FusedBinKLocal { op, a, k, d } => {
                    buf.push(12);
                    buf.push(*op);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&k.to_le_bytes());
                    buf.extend_from_slice(&(*d as u32).to_le_bytes());
                }
                Instruction::FusedBinLocal { op, a, b, d } => {
                    buf.push(13);
                    buf.push(*op);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&(*b as u32).to_le_bytes());
                    buf.extend_from_slice(&(*d as u32).to_le_bytes());
                }
                Instruction::FusedCmpKJmp { op, a, k, target } => {
                    buf.push(7);
                    buf.push(*op);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&k.to_le_bytes());
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                Instruction::FusedCmpJmp { op, a, b, target } => {
                    buf.push(8);
                    buf.push(*op);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&(*b as u32).to_le_bytes());
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                Instruction::FusedBinCmpJmp {
                    op1,
                    op2,
                    a,
                    b,
                    c,
                    target,
                } => {
                    buf.push(9);
                    buf.push(*op1);
                    buf.push(*op2);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&(*b as u32).to_le_bytes());
                    buf.extend_from_slice(&(*c as u32).to_le_bytes());
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                Instruction::FusedBinKCmpJmp {
                    op1,
                    op2,
                    a,
                    b,
                    k,
                    target,
                } => {
                    buf.push(10);
                    buf.push(*op1);
                    buf.push(*op2);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&(*b as u32).to_le_bytes());
                    buf.extend_from_slice(&k.to_le_bytes());
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
                }
                Instruction::FusedBinKKCmpJmp {
                    op1,
                    op2,
                    a,
                    b,
                    k,
                    target,
                } => {
                    buf.push(11);
                    buf.push(*op1);
                    buf.push(*op2);
                    buf.extend_from_slice(&(*a as u32).to_le_bytes());
                    buf.extend_from_slice(&b.to_le_bytes());
                    buf.extend_from_slice(&k.to_le_bytes());
                    buf.extend_from_slice(&(*target as u32).to_le_bytes());
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
        if version != 6 && version != CHUNK_VERSION {
            return Err(format!(
                "Versión {} de bytecode no soportada (esperada {} o 6)",
                version, CHUNK_VERSION
            ));
        }

        let mut pos = 8;
        let mut warnings = Vec::new();

        let num_strings =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let mut strings = Vec::with_capacity(num_strings);
        for _ in 0..num_strings {
            if pos + 4 > data.len() {
                return Err("Datos corruptos: se esperaban más strings".to_string());
            }
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
            if pos + len > data.len() {
                warnings.push((pos, format!("String de longitud {} excede el buffer", len)));
                break;
            }
            let s = String::from_utf8_lossy(&data[pos..pos + len]).to_string();
            strings.push(s);
            pos += len;
        }

        let num_ints =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let mut ints = Vec::with_capacity(num_ints);
        for _ in 0..num_ints {
            if pos + 8 > data.len() {
                return Err("Datos corruptos: se esperaban más enteros".to_string());
            }
            let n = i64::from_le_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]);
            ints.push(n);
            pos += 8;
        }

        let num_nums =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let mut nums = Vec::with_capacity(num_nums);
        for _ in 0..num_nums {
            if pos + 8 > data.len() {
                return Err("Datos corruptos: se esperaban más números".to_string());
            }
            let n = f64::from_le_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]);
            nums.push(n);
            pos += 8;
        }

        let num_names =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let mut names = Vec::with_capacity(num_names);
        for _ in 0..num_names {
            if pos + 4 > data.len() {
                return Err("Datos corruptos: se esperaban más nombres".to_string());
            }
            let len = u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                as usize;
            pos += 4;
            if pos + len > data.len() {
                break;
            }
            let s = String::from_utf8_lossy(&data[pos..pos + len]).to_string();
            names.push(s);
            pos += len;
        }

        let num_funcs =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
        pos += 4;
        let mut funcs = Vec::with_capacity(num_funcs);
        for _ in 0..num_funcs {
            if pos + 4 > data.len() {
                break;
            }
            let name_len =
                u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
            pos += 4;
            if pos + name_len > data.len() {
                break;
            }
            let name = String::from_utf8_lossy(&data[pos..pos + name_len]).to_string();
            pos += name_len;
            if pos + 4 > data.len() {
                break;
            }
            let num_params =
                u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                    as usize;
            pos += 4;
            let mut params = Vec::with_capacity(num_params);
            for _ in 0..num_params {
                if pos + 4 > data.len() {
                    break;
                }
                let plen =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                pos += 4;
                if pos + plen > data.len() {
                    break;
                }
                let p = String::from_utf8_lossy(&data[pos..pos + plen]).to_string();
                params.push(p);
                pos += plen;
            }
            let defaults = if version == 7 {
                if pos + 4 > data.len() {
                    break;
                }
                let num_defaults =
                    u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]])
                        as usize;
                pos += 4;
                let mut defs = Vec::with_capacity(num_defaults);
                for _ in 0..num_defaults {
                    if pos >= data.len() {
                        break;
                    }
                    let tag = data[pos];
                    pos += 1;
                    match tag {
                        0 => defs.push(None),
                        1 => {
                            if pos + 8 > data.len() {
                                break;
                            }
                            let v = i64::from_le_bytes([
                                data[pos],
                                data[pos + 1],
                                data[pos + 2],
                                data[pos + 3],
                                data[pos + 4],
                                data[pos + 5],
                                data[pos + 6],
                                data[pos + 7],
                            ]);
                            pos += 8;
                            defs.push(Some(DefaultValue::Int(v)));
                        }
                        2 => {
                            if pos + 8 > data.len() {
                                break;
                            }
                            let v = f64::from_le_bytes([
                                data[pos],
                                data[pos + 1],
                                data[pos + 2],
                                data[pos + 3],
                                data[pos + 4],
                                data[pos + 5],
                                data[pos + 6],
                                data[pos + 7],
                            ]);
                            pos += 8;
                            defs.push(Some(DefaultValue::Float(v)));
                        }
                        3 => {
                            if pos + 4 > data.len() {
                                break;
                            }
                            let slen = u32::from_le_bytes([
                                data[pos],
                                data[pos + 1],
                                data[pos + 2],
                                data[pos + 3],
                            ]) as usize;
                            pos += 4;
                            if pos + slen > data.len() {
                                break;
                            }
                            let s = String::from_utf8_lossy(&data[pos..pos + slen]).to_string();
                            pos += slen;
                            defs.push(Some(DefaultValue::Str(s)));
                        }
                        4 => {
                            if pos >= data.len() {
                                break;
                            }
                            let b = data[pos] != 0;
                            pos += 1;
                            defs.push(Some(DefaultValue::Bool(b)));
                        }
                        _ => defs.push(None),
                    }
                }
                defs
            } else {
                vec![None; params.len()]
            };
            if pos + 8 > data.len() {
                break;
            }
            let start = u64::from_le_bytes([
                data[pos],
                data[pos + 1],
                data[pos + 2],
                data[pos + 3],
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]) as usize;
            pos += 8;
            funcs.push(FuncMeta {
                name,
                params,
                defaults,
                start,
            });
        }

        let num_instrs =
            u32::from_le_bytes([data[pos], data[pos + 1], data[pos + 2], data[pos + 3]]) as usize;
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
                    if pos + 4 > data.len() {
                        break;
                    }
                    let idx = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    let n = nums.get(idx).copied().unwrap_or(0.0);
                    instructions.push(Instruction::WithNum(op, n));
                }
                2 => {
                    if pos + 4 > data.len() {
                        break;
                    }
                    let idx = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    let s = strings.get(idx).cloned().unwrap_or_default();
                    instructions.push(Instruction::WithStr(op, s));
                }
                3 => {
                    if pos >= data.len() {
                        break;
                    }
                    let b = data[pos] != 0;
                    pos += 1;
                    instructions.push(Instruction::WithBool(op, b));
                }
                4 => {
                    if pos + 4 > data.len() {
                        break;
                    }
                    let idx = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    pos += 4;
                    instructions.push(Instruction::WithIdx(op, idx));
                }
                // v3.5.20 super-opcodes: el byte tras el tag es el sub-op
                // (numeración del backend C), no un Opcode.
                5 => {
                    if pos + 16 > data.len() {
                        break;
                    }
                    let a = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    let k = i64::from_le_bytes([
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ]);
                    let d = u32::from_le_bytes([
                        data[pos + 12],
                        data[pos + 13],
                        data[pos + 14],
                        data[pos + 15],
                    ]) as usize;
                    pos += 16;
                    instructions.push(Instruction::FusedBinK {
                        op: op_byte,
                        a,
                        k,
                        d,
                    });
                }
                6 => {
                    if pos + 12 > data.len() {
                        break;
                    }
                    let a = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    let b = u32::from_le_bytes([
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                    ]) as usize;
                    let d = u32::from_le_bytes([
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ]) as usize;
                    pos += 12;
                    instructions.push(Instruction::FusedBin {
                        op: op_byte,
                        a,
                        b,
                        d,
                    });
                }
                12 => {
                    if pos + 16 > data.len() {
                        break;
                    }
                    let a = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    let k = i64::from_le_bytes([
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ]);
                    let d = u32::from_le_bytes([
                        data[pos + 12],
                        data[pos + 13],
                        data[pos + 14],
                        data[pos + 15],
                    ]) as usize;
                    pos += 16;
                    instructions.push(Instruction::FusedBinKLocal {
                        op: op_byte,
                        a,
                        k,
                        d,
                    });
                }
                13 => {
                    if pos + 12 > data.len() {
                        break;
                    }
                    let a = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    let b = u32::from_le_bytes([
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                    ]) as usize;
                    let d = u32::from_le_bytes([
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ]) as usize;
                    pos += 12;
                    instructions.push(Instruction::FusedBinLocal {
                        op: op_byte,
                        a,
                        b,
                        d,
                    });
                }
                7 => {
                    if pos + 16 > data.len() {
                        break;
                    }
                    let a = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    let k = i64::from_le_bytes([
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ]);
                    let target = u32::from_le_bytes([
                        data[pos + 12],
                        data[pos + 13],
                        data[pos + 14],
                        data[pos + 15],
                    ]) as usize;
                    pos += 16;
                    instructions.push(Instruction::FusedCmpKJmp {
                        op: op_byte,
                        a,
                        k,
                        target,
                    });
                }
                8 => {
                    if pos + 12 > data.len() {
                        break;
                    }
                    let a = u32::from_le_bytes([
                        data[pos],
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                    ]) as usize;
                    let b = u32::from_le_bytes([
                        data[pos + 4],
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                    ]) as usize;
                    let target = u32::from_le_bytes([
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                    ]) as usize;
                    pos += 12;
                    instructions.push(Instruction::FusedCmpJmp {
                        op: op_byte,
                        a,
                        b,
                        target,
                    });
                }
                9 => {
                    // FusedBinCmpJmp: tag op1 op2 a b c target
                    // (pos ya apunta tras op1: op2 en data[pos]).
                    if pos + 17 > data.len() {
                        break;
                    }
                    let op2 = data[pos];
                    let a = u32::from_le_bytes([
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                    ]) as usize;
                    let b = u32::from_le_bytes([
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                        data[pos + 8],
                    ]) as usize;
                    let c = u32::from_le_bytes([
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                        data[pos + 12],
                    ]) as usize;
                    let target = u32::from_le_bytes([
                        data[pos + 13],
                        data[pos + 14],
                        data[pos + 15],
                        data[pos + 16],
                    ]) as usize;
                    pos += 17;
                    instructions.push(Instruction::FusedBinCmpJmp {
                        op1: op_byte,
                        op2,
                        a,
                        b,
                        c,
                        target,
                    });
                }
                10 => {
                    // FusedBinKCmpJmp: tag op1 op2 a b k target
                    // (pos ya apunta tras op1: op2 en data[pos]).
                    if pos + 21 > data.len() {
                        break;
                    }
                    let op2 = data[pos];
                    let a = u32::from_le_bytes([
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                    ]) as usize;
                    let b = u32::from_le_bytes([
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                        data[pos + 8],
                    ]) as usize;
                    let k = i64::from_le_bytes([
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                        data[pos + 12],
                        data[pos + 13],
                        data[pos + 14],
                        data[pos + 15],
                        data[pos + 16],
                    ]);
                    let target = u32::from_le_bytes([
                        data[pos + 17],
                        data[pos + 18],
                        data[pos + 19],
                        data[pos + 20],
                    ]) as usize;
                    pos += 21;
                    instructions.push(Instruction::FusedBinKCmpJmp {
                        op1: op_byte,
                        op2,
                        a,
                        b,
                        k,
                        target,
                    });
                }
                11 => {
                    // FusedBinKKCmpJmp: tag op1 op2 a b k target
                    // (pos ya apunta tras op1: op2 en data[pos]).
                    if pos + 25 > data.len() {
                        break;
                    }
                    let op2 = data[pos];
                    let a = u32::from_le_bytes([
                        data[pos + 1],
                        data[pos + 2],
                        data[pos + 3],
                        data[pos + 4],
                    ]) as usize;
                    let b = i64::from_le_bytes([
                        data[pos + 5],
                        data[pos + 6],
                        data[pos + 7],
                        data[pos + 8],
                        data[pos + 9],
                        data[pos + 10],
                        data[pos + 11],
                        data[pos + 12],
                    ]);
                    let k = i64::from_le_bytes([
                        data[pos + 13],
                        data[pos + 14],
                        data[pos + 15],
                        data[pos + 16],
                        data[pos + 17],
                        data[pos + 18],
                        data[pos + 19],
                        data[pos + 20],
                    ]);
                    let target = u32::from_le_bytes([
                        data[pos + 21],
                        data[pos + 22],
                        data[pos + 23],
                        data[pos + 24],
                    ]) as usize;
                    pos += 25;
                    instructions.push(Instruction::FusedBinKKCmpJmp {
                        op1: op_byte,
                        op2,
                        a,
                        b,
                        k,
                        target,
                    });
                }
                _ => {
                    warnings.push((pos, format!("Tag de instrucción desconocido: {}", tag)));
                }
            }
        }
        Ok((
            Bytecode {
                instructions,
                strings,
                ints,
                nums,
                names,
                funcs,
            },
            warnings,
        ))
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
