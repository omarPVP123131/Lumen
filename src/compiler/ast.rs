// src/compiler/ast.rs
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(i32),
    String(String),
    Variable(String),
    BinOp {
        left: Box<Expr>,
        op: BinOperator,
        right: Box<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinOperator {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Lt,
    Gt,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    VarDecl {
        name: String,
        value: Expr,
    },
    Assignment {
        target: String,
        value: Expr,
    },
    Print {
        expr: Expr,
    },
    If {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    Import {
        module: String,
    },
}

#[derive(Debug)]
pub struct Program {
    pub statements: Vec<Statement>,
}

impl Program {
    pub fn new() -> Self {
        Self { statements: Vec::new() }
    }
}
