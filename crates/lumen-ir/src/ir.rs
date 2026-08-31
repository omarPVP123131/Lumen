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
    Concat,
    Sub,
    Mul,
    Div,
    Mod,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    BitOr,
    BitAnd,
    BitXor,
    BitNot,
    ShiftLeft,
    ShiftRight,
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
    StoreLocal(String),
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
    ArrayPushVar(String),
    /// v3.5.40: `a[i] = v` in-place sobre el slot de una variable simple
    /// (espejo de ArrayPushVar; la VM hace pop del receptor obsoleto antes
    /// de mutar para que Arc::make_mut no clone el Vec entero por escritura).
    ArraySetVar(String),
    StructNew(String, usize),
    StructGet,
    StructSet,
    ResultOk,
    ResultErr,
    TryUnwrap,
    OptionSome,
    OptionNone,
    MatchType(u8),
    MatchPayload,
    TupleNew(usize),
    TupleAccess(usize),
    EnumCtor {
        enum_name: String,
        variant: String,
        argc: usize,
    },
    Jmp(usize),
    JmpIf(usize),
    Label(usize),
    Phi(usize, usize),
    Nop,
    Halt,
    PushHandler(usize),
    PopHandler,
    ScopePush,
    ScopePop,
    MatchVariant(String),
    /// Crea una referencia mutable al slot de la variable nombrada (bug #6).
    /// El argumento debe ser un lvalue simple (Ident). El VM apila Value::Ref
    /// con owner (scope_idx, nombre) para hacer write-back en Ret.
    MakeRef(String),
}

#[derive(Debug, Clone)]
pub struct Func {
    pub name: String,
    pub params: Vec<String>,
    pub defaults: Vec<Option<Value>>,
    pub entry: usize,
    pub instrs: Vec<Instr>,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub funcs: BTreeMap<String, Func>,
    pub entry: String,
    /// v3.5.15: función anidada -> función contenedora (para resolución de
    /// capturas en los backends nativos). Vacío para funciones top-level.
    pub parents: BTreeMap<String, String>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            funcs: BTreeMap::new(),
            entry: String::new(),
            parents: BTreeMap::new(),
        }
    }
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}
