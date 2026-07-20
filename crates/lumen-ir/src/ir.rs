use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Ident(String),
    Temp(usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Op {
    Add,
    Sub,
    Mul,
    Div,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    ConstInt(i64),
    ConstFloat(f64),
    ConstStr(String),
    ConstBool(bool),
    Load(String),
    Store(String),
    Binary(Op),
    Unary(Op),
    Call(String, usize),
    FuncRef(String),
    CallValue(usize),
    Return,
    Print,
    Read,
    ArrayNew(usize),
    ArrayGet,
    ArraySet,
    ArrayLen,
    ArrayPush,
    StructNew(String, usize),
    StructGet,
    StructSet,
    ResultOk,
    ResultErr,
    TryUnwrap,
    OptionSome,
    OptionNone,
    Jmp(usize),
    JmpIf(usize),
    Label(usize),
    Phi(usize, usize),
    Nop,
    Halt,
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub params: Vec<String>,
    pub entry: usize,
    pub instrs: Vec<Instr>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub funcs: BTreeMap<String, Func>,
    pub entry: String,
}

impl Program {
    pub fn new() -> Self {
        Self {
            funcs: BTreeMap::new(),
            entry: String::new(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
