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
    /// BUG-023: declaración de variable. Liga siempre en el marco actual.
    StoreLocal(String),
    /// BUG-027: descarta el valor en la cima de la pila. Lo emiten las
    /// sentencias-expresión (`imprimir(x);`) cuya evaluación deja un valor que
    /// nadie consume.
    Drop,
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
    /// BUG-022: instala el manejador del `atrapar` cuya etiqueta se indica.
    PushHandler(usize),
    /// BUG-022: desinstala el manejador del `intentar` que acaba de terminar.
    PopHandler,
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
    /// BUG-032: nombres del entorno que una lambda captura por valor. Se
    /// resuelven en el momento de crear la closure (`FuncRef`), de modo que la
    /// closure siga funcionando cuando el marco que la creó ya ha muerto.
    /// Vacío para las funciones normales.
    pub captures: Vec<String>,
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
