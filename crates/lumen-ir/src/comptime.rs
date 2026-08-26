//! Evaluador en tiempo de compilación (bug #7).
//!
//! `comptime { expr }` se evalúa durante la compilación cuando la expresión es
//! cerrada respecto a literales y llamadas a funciones puras del propio
//! programa. El resultado literal REEMPLAZA al nodo `Expr::Comptime`, de modo
//! que el builder emite una única instrucción constante (sin código runtime).
//!
//! Si algo no se puede evaluar (variables externas, builtins con efectos,
//! recursión profunda, límite de pasos), la expresión se deja intacta y se
//! ejecuta en runtime — degradación silenciosa y segura.

use lumen_parser::ast::{BinOp, Decl, DeclOrStmt, Expr, Stmt, UnOp};
use std::collections::HashMap;
const MAX_DEPTH: usize = 128;
const MAX_STEPS: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq)]
pub enum CVal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
}

#[derive(Debug, Clone)]
struct ComptimeFn {
    params: Vec<String>,
    body: Vec<DeclOrStmt>,
}

pub struct ComptimeEvaluator {
    funcs: HashMap<String, ComptimeFn>,
    steps: usize,
}

impl ComptimeEvaluator {
    pub fn new(program: &[DeclOrStmt]) -> Self {
        let mut funcs = HashMap::new();
        for node in program {
            if let DeclOrStmt::Decl(Decl::Function {
                name, params, body, ..
            }) = node
            {
                if !params.iter().any(|p| p.default.is_some()) {
                    funcs.insert(
                        name.clone(),
                        ComptimeFn {
                            params: params.iter().map(|p| p.name.clone()).collect(),
                            body: body.clone(),
                        },
                    );
                }
            }
        }
        Self { funcs, steps: 0 }
    }

    fn tick(&mut self) -> Result<(), ()> {
        self.steps += 1;
        if self.steps > MAX_STEPS {
            Err(())
        } else {
            Ok(())
        }
    }

    /// Pre-paso sobre todo el programa: reemplaza cada `comptime {...}`
    /// evaluable por su literal. Devuelve cuántos nodos se plegaron.
    pub fn rewrite_program(&mut self, program: &mut [DeclOrStmt]) -> usize {
        let mut folded = 0;
        for node in program.iter_mut() {
            match node {
                DeclOrStmt::Stmt(stmt) => folded += self.rewrite_stmt(stmt),
                DeclOrStmt::Decl(Decl::Function { body, .. }) => {
                    for s in body.iter_mut() {
                        folded += self.rewrite_decl_or_stmt(s);
                    }
                }
                _ => {}
            }
        }
        folded
    }

    fn rewrite_decl_or_stmt(&mut self, node: &mut DeclOrStmt) -> usize {
        match node {
            DeclOrStmt::Stmt(s) => self.rewrite_stmt(s),
            DeclOrStmt::Decl(Decl::Variable {
                init: Some(init), ..
            }) => self.rewrite_expr(init),
            _ => 0,
        }
    }

    fn rewrite_stmt(&mut self, stmt: &mut Stmt) -> usize {
        let mut folded = 0;
        match stmt {
            Stmt::Expr { expr, .. } => folded += self.rewrite_expr(expr),
            Stmt::Assignment { value, .. } => folded += self.rewrite_expr(value),
            _ => {}
        }
        folded
    }

    fn rewrite_expr(&mut self, expr: &mut Expr) -> usize {
        // Primero descender (los comptime anidados se pliegan de adentro hacia afuera)
        let mut folded = match expr {
            Expr::Binary { left, right, .. } => self.rewrite_expr(left) + self.rewrite_expr(right),
            Expr::Unary { operand, .. } => self.rewrite_expr(operand),
            Expr::Grouping { expr: inner, .. } => self.rewrite_expr(inner),
            Expr::List { items, .. } => items.iter_mut().map(|i| self.rewrite_expr(i)).sum(),
            _ => 0,
        };
        if let Expr::Comptime { expr: inner, span } = expr {
            // Clonar para evaluar sin problemas de préstamo del AST
            let candidate = (**inner).clone();
            if let Ok(v) = self.eval(&candidate, &HashMap::new(), 0) {
                let mut lit = cval_to_expr(&v);
                set_span(&mut lit, *span);
                *expr = lit;
                folded += 1;
            }
        }
        folded
    }

    fn eval(&mut self, expr: &Expr, env: &HashMap<String, CVal>, depth: usize) -> Result<CVal, ()> {
        if depth > MAX_DEPTH {
            return Err(());
        }
        self.tick()?;
        match expr {
            Expr::Int { value, .. } => Ok(CVal::Int(*value)),
            Expr::Float { value, .. } => Ok(CVal::Float(*value)),
            Expr::Bool { value, .. } => Ok(CVal::Bool(*value)),
            Expr::Str { value, .. } => Ok(CVal::Str(value.clone())),
            Expr::Grouping { expr: inner, .. } => self.eval(inner, env, depth + 1),
            Expr::Ident { name, .. } => env.get(name).cloned().ok_or(()),
            Expr::Unary { op, operand, .. } => {
                let v = self.eval(operand, env, depth + 1)?;
                match (op, v) {
                    (UnOp::Negate, CVal::Int(i)) => Ok(CVal::Int(i.wrapping_neg())),
                    (UnOp::Negate, CVal::Float(f)) => Ok(CVal::Float(-f)),
                    (UnOp::Not, CVal::Bool(b)) => Ok(CVal::Bool(!b)),
                    (UnOp::BitNot, CVal::Int(i)) => Ok(CVal::Int(!i)),
                    _ => Err(()),
                }
            }
            Expr::Binary {
                op, left, right, ..
            } => {
                let lt = self.eval(left, env, depth + 1)?;
                let rt = self.eval(right, env, depth + 1)?;
                eval_binary(*op, lt, rt)
            }
            Expr::Call { callee, args, .. } => {
                let name = match callee.as_ref() {
                    Expr::Ident { name, .. } => name.clone(),
                    _ => return Err(()),
                };
                let mut vals = Vec::new();
                for a in args {
                    vals.push(self.eval(a, env, depth + 1)?);
                }
                // Builtins puros permitidos en comptime
                if let Some(r) = eval_pure_builtin(&name, &vals) {
                    return Ok(r);
                }
                // Llamada a función propia del programa
                let f = self.funcs.get(&name).cloned().ok_or(())?;
                if vals.len() != f.params.len() {
                    return Err(());
                }
                let mut call_env: HashMap<String, CVal> = HashMap::new();
                for (p, v) in f.params.iter().zip(vals) {
                    call_env.insert(p.clone(), v);
                }
                self.exec_fn_body(&f.body, &mut call_env, depth + 1)
            }
            _ => Err(()),
        }
    }

    fn exec_fn_body(
        &mut self,
        body: &[DeclOrStmt],
        env: &mut HashMap<String, CVal>,
        depth: usize,
    ) -> Result<CVal, ()> {
        for node in body {
            self.tick()?;
            match node {
                DeclOrStmt::Stmt(Stmt::Expr { expr, .. }) => {
                    self.eval(expr, env, depth + 1)?;
                }
                DeclOrStmt::Stmt(Stmt::Return { value: Some(v), .. }) => {
                    return self.eval(v, env, depth + 1);
                }
                DeclOrStmt::Decl(Decl::Variable {
                    name,
                    init: Some(init),
                    ..
                }) => {
                    let v = self.eval(init, env, depth + 1)?;
                    env.insert(name.clone(), v);
                }
                _ => return Err(()),
            }
        }
        Err(()) // sin retorno explícito → no plegable
    }
}

fn eval_binary(op: BinOp, lt: CVal, rt: CVal) -> Result<CVal, ()> {
    use CVal::*;
    match (&lt, &rt) {
        // Enteros
        (Int(a), Int(b)) => match op {
            BinOp::Add => Ok(Int(a.wrapping_add(*b))),
            BinOp::Sub => Ok(Int(a.wrapping_sub(*b))),
            BinOp::Mul => Ok(Int(a.wrapping_mul(*b))),
            BinOp::Div if *b != 0 => Ok(Int(a.wrapping_div(*b))),
            BinOp::Mod if *b != 0 => Ok(Int(a.wrapping_rem(*b))),
            BinOp::Equal => Ok(Bool(a == b)),
            BinOp::NotEqual => Ok(Bool(a != b)),
            BinOp::Less => Ok(Bool(a < b)),
            BinOp::LessEqual => Ok(Bool(a <= b)),
            BinOp::Greater => Ok(Bool(a > b)),
            BinOp::GreaterEqual => Ok(Bool(a >= b)),
            BinOp::BitOr => Ok(Int(a | b)),
            BinOp::BitAnd => Ok(Int(a & b)),
            BinOp::BitXor => Ok(Int(a ^ b)),
            BinOp::ShiftLeft => Ok(Int(a.wrapping_shl(*b as u32))),
            BinOp::ShiftRight => Ok(Int(a.wrapping_shr(*b as u32))),
            BinOp::And => Ok(Bool(*a != 0 && *b != 0)),
            BinOp::Or => Ok(Bool(*a != 0 || *b != 0)),
            _ => Err(()),
        },
        // Decimales (o promoción)
        _ if matches!((&lt, &rt), (Float(_), _) | (_, Float(_))
              if matches!(lt, Float(_) | Int(_)) && matches!(rt, Float(_) | Int(_))) =>
        {
            let a = match lt {
                Int(i) => i as f64,
                Float(f) => f,
                _ => return Err(()),
            };
            let b = match rt {
                Int(i) => i as f64,
                Float(f) => f,
                _ => return Err(()),
            };
            match op {
                BinOp::Add => Ok(Float(a + b)),
                BinOp::Sub => Ok(Float(a - b)),
                BinOp::Mul => Ok(Float(a * b)),
                BinOp::Div => Ok(Float(a / b)),
                BinOp::Mod => Ok(Float(a % b)),
                BinOp::Equal => Ok(Bool(a == b)),
                BinOp::NotEqual => Ok(Bool(a != b)),
                BinOp::Less => Ok(Bool(a < b)),
                BinOp::LessEqual => Ok(Bool(a <= b)),
                BinOp::Greater => Ok(Bool(a > b)),
                BinOp::GreaterEqual => Ok(Bool(a >= b)),
                _ => Err(()),
            }
        }
        // Booleanos
        (Bool(a), Bool(b)) => match op {
            BinOp::And => Ok(Bool(*a && *b)),
            BinOp::Or => Ok(Bool(*a || *b)),
            BinOp::Equal => Ok(Bool(a == b)),
            BinOp::NotEqual => Ok(Bool(a != b)),
            _ => Err(()),
        },
        // Textos
        (Str(a), Str(b)) => match op {
            BinOp::Add | BinOp::Concat => {
                let mut s = String::with_capacity(a.len() + b.len());
                s.push_str(a);
                s.push_str(b);
                Ok(Str(s))
            }
            BinOp::Equal => Ok(Bool(a == b)),
            BinOp::NotEqual => Ok(Bool(a != b)),
            _ => Err(()),
        },
        _ => Err(()),
    }
}

fn eval_pure_builtin(name: &str, args: &[CVal]) -> Option<CVal> {
    let a = args.first()?;
    Some(match name {
        "abs" | "absoluto" => match a {
            CVal::Int(i) => CVal::Int(i.abs()),
            CVal::Float(f) => CVal::Float(f.abs()),
            _ => return None,
        },
        "min" | "minimo" | "max" | "maximo" => {
            let is_max = name == "max" || name == "maximo";
            let b = args.get(1)?;
            match (a, b) {
                (CVal::Int(x), CVal::Int(y)) => {
                    let r = if is_max { (*x).max(*y) } else { (*x).min(*y) };
                    CVal::Int(r)
                }
                _ => return None,
            }
        }
        "piso" | "floor" | "techo" | "ceil" | "redondear" | "round" => match a {
            CVal::Float(f) => {
                let r = match name {
                    "piso" | "floor" => f.floor(),
                    "techo" | "ceil" => f.ceil(),
                    _ => f.round(),
                };
                CVal::Int(r as i64)
            }
            CVal::Int(i) => CVal::Int(*i),
            _ => return None,
        },
        "raiz" | "sqrt" => match a {
            CVal::Int(i) => CVal::Float((*i as f64).sqrt()),
            CVal::Float(f) => CVal::Float(f.sqrt()),
            _ => return None,
        },
        "potencia" | "pow" => {
            let b = args.get(1)?;
            match (a, b) {
                (CVal::Int(x), CVal::Int(y)) if *y >= 0 && *y <= u32::MAX as i64 => {
                    CVal::Int(x.wrapping_pow(*y as u32))
                }
                (CVal::Int(x), CVal::Int(y)) => CVal::Float((*x as f64).powi(*y as i32)),
                (x, y) => match (cvt_f(x), cvt_f(y)) {
                    (Some(fx), Some(fy)) => CVal::Float(fx.powf(fy)),
                    _ => return None,
                },
            }
        }
        _ => return None,
    })
}

fn cvt_f(v: &CVal) -> Option<f64> {
    match v {
        CVal::Int(i) => Some(*i as f64),
        CVal::Float(f) => Some(*f),
        _ => None,
    }
}

fn cval_to_expr(v: &CVal) -> Expr {
    use lumen_lexer::token::{Pos, Span};
    let zero = Pos::new(0, 0);
    let span = Span {
        start: zero,
        end: Pos::new(0, 0),
    };
    let _ = zero;
    match v {
        CVal::Int(i) => Expr::Int { value: *i, span },
        CVal::Float(f) => Expr::Float { value: *f, span },
        CVal::Bool(b) => Expr::Bool { value: *b, span },
        CVal::Str(s) => Expr::Str {
            value: s.clone(),
            span,
        },
    }
}

fn set_span(e: &mut Expr, span: lumen_lexer::token::Span) {
    match e {
        Expr::Int { span: s, .. }
        | Expr::Float { span: s, .. }
        | Expr::Str { span: s, .. }
        | Expr::Bool { span: s, .. } => *s = span,
        _ => {}
    }
}
