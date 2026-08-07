use lumen_lexer::token::Span;
use serde::{Deserialize, Serialize};

pub type Program = Vec<DeclOrStmt>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeclOrStmt {
    Decl(Decl),
    Stmt(Stmt),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestructureTarget {
    pub var_type: Option<Type>,
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Decl {
    Variable {
        var_type: Type,
        name: String,
        init: Option<Box<Expr>>,
        span: Span,
    },
    Destructure {
        targets: Vec<DestructureTarget>,
        init: Box<Expr>,
        span: Span,
    },
    Function {
        return_type: Type,
        name: String,
        params: Vec<Param>,
        body: Vec<DeclOrStmt>,
        type_params: Vec<String>,
        type_param_bounds: Vec<(String, String)>,
        is_async: bool,
        span: Span,
    },
    Struct {
        name: String,
        fields: Vec<StructField>,
        type_params: Vec<String>,
        type_param_bounds: Vec<(String, String)>,
        span: Span,
    },
    Enum {
        name: String,
        variants: Vec<EnumVariant>,
        span: Span,
    },
    Const {
        var_type: Type,
        name: String,
        value: Box<Expr>,
        span: Span,
    },
    Rasgo {
        name: String,
        methods: Vec<TraitMethod>,
        associated_types: Vec<AssociatedType>,
        span: Span,
    },
    ImplRasgo {
        trait_name: String,
        target_type: Type,
        associated_types: Vec<ImplAssociatedType>,
        methods: Vec<Decl>,
        span: Span,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub field_type: Type,
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumVariant {
    pub name: String,
    pub types: Vec<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraitMethod {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Type,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssociatedType {
    pub name: String,
    pub default: Option<Type>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplAssociatedType {
    pub name: String,
    pub target_type: Type,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub param_type: Type,
    pub name: String,
    pub default: Option<Box<Expr>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchArm {
    pub value: Expr,
    pub guard: Option<Box<Expr>>,
    pub body: Vec<DeclOrStmt>,
    pub alt_values: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Stmt {
    Assignment {
        name: String,
        value: Box<Expr>,
        span: Span,
    },
    If {
        condition: Box<Expr>,
        then_body: Vec<DeclOrStmt>,
        else_body: Option<Vec<DeclOrStmt>>,
        span: Span,
    },
    IfLet {
        pattern: Expr,
        value: Box<Expr>,
        then_body: Vec<DeclOrStmt>,
        else_body: Option<Vec<DeclOrStmt>>,
        span: Span,
    },
    GuardLet {
        pattern: Expr,
        value: Box<Expr>,
        else_body: Vec<DeclOrStmt>,
        span: Span,
    },
    While {
        condition: Box<Expr>,
        body: Vec<DeclOrStmt>,
        span: Span,
    },
    For {
        init: Box<Decl>,
        condition: Box<Expr>,
        update: Box<Stmt>,
        body: Vec<DeclOrStmt>,
        span: Span,
    },
    Return {
        value: Option<Box<Expr>>,
        span: Span,
    },
    Break {
        label: Option<String>,
        span: Span,
    },
    Continue {
        label: Option<String>,
        span: Span,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        default: Option<Vec<DeclOrStmt>>,
        span: Span,
    },
    Expr {
        expr: Box<Expr>,
        span: Span,
    },
    FieldAssign {
        expr: Box<Expr>,
        field: String,
        value: Box<Expr>,
        span: Span,
    },
    ArraySet {
        arr: Box<Expr>,
        index: Box<Expr>,
        value: Box<Expr>,
        span: Span,
    },
    Block {
        stmts: Vec<DeclOrStmt>,
        span: Span,
    },
    Import {
        path: String,
        alias: Option<String>,
        span: Span,
    },
    ForEach {
        var_name: String,
        expr: Box<Expr>,
        body: Vec<DeclOrStmt>,
        span: Span,
    },
    Destructure {
        targets: Vec<DestructureTarget>,
        value: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Int {
        value: i64,
        span: Span,
    },
    Float {
        value: f64,
        span: Span,
    },
    Str {
        value: String,
        span: Span,
    },
    Bool {
        value: bool,
        span: Span,
    },
    Ident {
        name: String,
        span: Span,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        resolved_method: Option<String>,
        span: Span,
    },
    Unary {
        op: UnOp,
        operand: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        type_args: Vec<Type>,
        span: Span,
    },
    Grouping {
        expr: Box<Expr>,
        span: Span,
    },
    Cast {
        expr: Box<Expr>,
        cast_type: Type,
        span: Span,
    },
    List {
        items: Vec<Expr>,
        span: Span,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    MethodCall {
        expr: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        resolved_func: Option<String>,
        span: Span,
    },
    Lambda {
        params: Vec<Param>,
        body: Vec<DeclOrStmt>,
        span: Span,
    },
    StructInit {
        struct_name: String,
        fields: Vec<(String, Expr)>,
        type_args: Vec<Type>,
        span: Span,
    },
    FieldAccess {
        expr: Box<Expr>,
        field: String,
        span: Span,
    },
    Exito {
        expr: Box<Expr>,
        span: Span,
    },
    Error {
        expr: Box<Expr>,
        span: Span,
    },
    Intentar {
        expr: Box<Expr>,
        span: Span,
    },
    Algun {
        expr: Box<Expr>,
        span: Span,
    },
    Ninguno {
        span: Span,
    },
    EnumCtor {
        enum_name: String,
        variant: String,
        args: Vec<Expr>,
        span: Span,
    },
    Tuple {
        items: Vec<Expr>,
        span: Span,
    },
    TupleAccess {
        expr: Box<Expr>,
        index: usize,
        span: Span,
    },
    Ternary {
        condition: Box<Expr>,
        true_branch: Box<Expr>,
        false_branch: Box<Expr>,
        span: Span,
    },
    Esperar {
        expr: Box<Expr>,
        span: Span,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum BinOp {
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
    ShiftLeft,
    ShiftRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UnOp {
    Negate,
    Not,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Numero,
    Entero,
    Decimal,
    Texto,
    Booleano,
    Lista(Box<Type>),
    Func {
        param_types: Vec<Type>,
        return_type: Box<Type>,
    },
    Struct(String),
    GenericStruct {
        name: String,
        args: Vec<Type>,
    },
    Resultado {
        ok: Box<Type>,
        err: Box<Type>,
    },
    Opcion(Box<Type>),
    Tuple(Vec<Type>),
    ImplTrait(String),
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Int { span, .. }
            | Expr::Float { span, .. }
            | Expr::Str { span, .. }
            | Expr::Bool { span, .. }
            | Expr::Ident { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Call { span, .. }
            | Expr::Grouping { span, .. }
            | Expr::Cast { span, .. }
            | Expr::List { span, .. }
            | Expr::Index { span, .. }
            | Expr::MethodCall { span, .. }
            | Expr::Lambda { span, .. }
            | Expr::StructInit { span, .. }
            | Expr::FieldAccess { span, .. }
            | Expr::Exito { span, .. }
            | Expr::Error { span, .. }
            | Expr::Intentar { span, .. }
            | Expr::Algun { span, .. }
            | Expr::Ninguno { span, .. }
            | Expr::EnumCtor { span, .. }
            | Expr::Tuple { span, .. }
            | Expr::TupleAccess { span, .. }
            | Expr::Ternary { span, .. } => *span,
            Expr::Esperar { span, .. } => *span,
        }
    }
}
