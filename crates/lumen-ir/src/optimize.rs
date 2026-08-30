//! v3.5.19 — Optimizador IR: constant folding de aritmética sobre literales.
//!
//! Reduce secuencias `Const a; Const b; Binary op` a un solo `Const r`
//! (y análogo para `Unary`). Beneficia a TODOS los backends: menos
//! instrucciones en la VM, menos ops de stack en C, menos box/unbox en
//! Cranelift. Conservador: solo pliega cuando el resultado es idéntico en
//! VM/C/Cranelift (wrapping exacto, sin división por cero, sin MIN/-1).

use crate::ir::{Func, Instr, Op, Program};

/// Valor abstracto de la pila de operandos durante el plegado.
#[derive(Clone, Copy)]
enum Abs {
    Int(i64),
    Float(f64),
    Bool(bool),
    /// Texto conocido solo como TIPO (no se pliega: identidad/asignación).
    Txt,
    Unknown,
}

/// Intenta plegar `a op b` con operandos constantes.
fn fold_bin(op: &Op, a: Abs, b: Abs) -> Option<Abs> {
    use Abs::*;
    match (a, b) {
        (Int(x), Int(y)) => match op {
            Op::Add => Some(Int(x.wrapping_add(y))),
            Op::Sub => Some(Int(x.wrapping_sub(y))),
            Op::Mul => Some(Int(x.wrapping_mul(y))),
            Op::Div => {
                if y == 0 || (x == i64::MIN && y == -1) {
                    None // lanza en runtime; no plegar
                } else {
                    Some(Int(x / y))
                }
            }
            Op::Mod => {
                if y == 0 || (x == i64::MIN && y == -1) {
                    None
                } else {
                    Some(Int(x % y))
                }
            }
            Op::BitAnd => Some(Int(x & y)),
            Op::BitOr => Some(Int(x | y)),
            Op::BitXor => Some(Int(x ^ y)),
            Op::Equal => Some(Bool(x == y)),
            Op::NotEqual => Some(Bool(x != y)),
            Op::Less => Some(Bool(x < y)),
            Op::LessEqual => Some(Bool(x <= y)),
            Op::Greater => Some(Bool(x > y)),
            Op::GreaterEqual => Some(Bool(x >= y)),
            _ => None,
        },
        (Float(x), Float(y)) => match op {
            Op::Add => Some(Float(x + y)),
            Op::Sub => Some(Float(x - y)),
            Op::Mul => Some(Float(x * y)),
            Op::Div => Some(Float(x / y)),
            Op::Equal => Some(Bool(x == y)),
            Op::NotEqual => Some(Bool(x != y)),
            Op::Less => Some(Bool(x < y)),
            Op::LessEqual => Some(Bool(x <= y)),
            Op::Greater => Some(Bool(x > y)),
            Op::GreaterEqual => Some(Bool(x >= y)),
            _ => None,
        },
        (Int(x), Float(y)) => fold_bin(op, Float(x as f64), Float(y)),
        (Float(x), Int(y)) => fold_bin(op, Float(x), Float(y as f64)),
        (Bool(x), Bool(y)) => match op {
            Op::Equal => Some(Bool(x == y)),
            Op::NotEqual => Some(Bool(x != y)),
            _ => None,
        },
        _ => None,
    }
}

fn fold_un(op: &Op, a: Abs) -> Option<Abs> {
    use Abs::*;
    match (op, a) {
        (Op::Negate, Int(x)) => Some(Int(x.wrapping_neg())),
        (Op::Negate, Float(x)) => Some(Float(-x)),
        (Op::BitNot, Int(x)) => Some(Int(!x)),
        (Op::Not, Bool(x)) => Some(Bool(!x)),
        _ => None,
    }
}

/// Tipo del resultado sin plegar (para plegados en cascada).
fn result_kind(op: &Op, a: Abs, b: Abs) -> Abs {
    use Abs::*;
    match op {
        Op::Equal | Op::NotEqual | Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual => {
            Bool(false)
        }
        Op::Concat => match (a, b) {
            (_, Unknown) | (Unknown, _) => Unknown,
            _ => Txt,
        },
        _ => match (a, b) {
            (Float(_), _) | (_, Float(_)) => Float(0.0),
            (Int(_), Int(_)) => Int(0),
            _ => Unknown,
        },
    }
}

fn const_instr(r: Abs) -> Instr {
    match r {
        Abs::Int(v) => Instr::ConstInt(v),
        Abs::Float(v) => Instr::ConstFloat(v),
        Abs::Bool(v) => Instr::ConstBool(v),
        _ => unreachable!(),
    }
}

pub fn optimize(program: &mut Program) {
    for func in program.funcs.values_mut() {
        optimize_func(func);
    }
}

fn optimize_func(func: &mut Func) {
    // Pila abstracta: (valor, índice en `out` si lo produjo una constante).
    let mut out: Vec<Instr> = Vec::with_capacity(func.instrs.len());
    let mut st: Vec<(Abs, Option<usize>)> = Vec::new();

    for ins in &func.instrs {
        match ins {
            Instr::ConstInt(v) => {
                out.push(ins.clone());
                st.push((Abs::Int(*v), Some(out.len() - 1)));
            }
            Instr::ConstFloat(v) => {
                out.push(ins.clone());
                st.push((Abs::Float(*v), Some(out.len() - 1)));
            }
            Instr::ConstBool(v) => {
                out.push(ins.clone());
                st.push((Abs::Bool(*v), Some(out.len() - 1)));
            }
            Instr::ConstStr(_) => {
                out.push(ins.clone());
                st.push((Abs::Txt, Some(out.len() - 1)));
            }
            Instr::Unary(op) => {
                let (a, ia) = st.pop().unwrap_or((Abs::Unknown, None));
                if ia.is_some() {
                    if let Some(r) = fold_un(op, a) {
                        out.push(const_instr(r));
                        st.push((r, Some(out.len() - 1)));
                    } else {
                        out.push(ins.clone());
                        st.push((Abs::Unknown, None));
                    }
                } else {
                    out.push(ins.clone());
                    st.push((Abs::Unknown, None));
                }
            }
            Instr::Binary(op) => {
                let (b, ib) = st.pop().unwrap_or((Abs::Unknown, None));
                let (a, ia) = st.pop().unwrap_or((Abs::Unknown, None));
                // Solo se pliega con AMBAS constantes reales (índice Some):
                // los Abs de tipo-solo (resultado de ops no plegados) no
                // llevan valor fiable.
                let folded = if ia.is_some() && ib.is_some() {
                    fold_bin(op, a, b)
                } else {
                    None
                };
                if let Some(r) = folded {
                    // Quitar de `out` las constantes plegadas (índices
                    // decrecientes para no invalidar). Solo se quitan si el
                    // slot sigue siendo una Const (seguridad extra).
                    let mut idxs: Vec<usize> = ia.into_iter().chain(ib).collect();
                    idxs.sort_unstable_by(|x, y| y.cmp(x));
                    for idx in idxs {
                        if idx < out.len()
                            && matches!(
                                out[idx],
                                Instr::ConstInt(_) | Instr::ConstFloat(_) | Instr::ConstBool(_)
                            )
                        {
                            out.remove(idx);
                        }
                    }
                    out.push(const_instr(r));
                    st.push((r, Some(out.len() - 1)));
                } else {
                    out.push(ins.clone());
                    st.push((result_kind(op, a, b), None));
                }
            }
            Instr::Label(_) | Instr::Jmp(_) | Instr::JmpIf(_) | Instr::Phi(..) => {
                out.push(ins.clone());
                st.clear(); // punto de control: estado desconocido
            }
            Instr::Store(_) | Instr::StoreLocal(_) => {
                st.pop();
                out.push(ins.clone());
            }
            // v3.5.34 (bug real del folder): las instrucciones que CONSUMEN
            // operandos y producen un valor (neto 0 o mixto) deben
            // desapilar explícitamente lo consumido: el modelo por delta
            // neto NO lo hacía y pliegues posteriores BORRABAN del `out`
            // constantes que eran argumentos (p.ej. `f(3) + 1` perdía el
            // `3` y el `Add` → "Stack underflow" en runtime).
            Instr::Call(_, argc) | Instr::EnumCtor { argc, .. } => {
                for _ in 0..*argc {
                    st.pop();
                }
                out.push(ins.clone());
                st.push((Abs::Unknown, None));
            }
            Instr::CallValue(argc) => {
                // argc args + el valor-función debajo.
                for _ in 0..(argc + 1) {
                    st.pop();
                }
                out.push(ins.clone());
                st.push((Abs::Unknown, None));
            }
            Instr::ArrayNew(n) => {
                for _ in 0..*n {
                    st.pop();
                }
                out.push(ins.clone());
                st.push((Abs::Unknown, None));
            }
            Instr::StructNew(_, n) => {
                for _ in 0..(2 * *n) {
                    st.pop();
                }
                out.push(ins.clone());
                st.push((Abs::Unknown, None));
            }
            Instr::TupleNew(n) => {
                for _ in 0..*n {
                    st.pop();
                }
                out.push(ins.clone());
                st.push((Abs::Unknown, None));
            }
            Instr::OptionSome
            | Instr::ResultOk
            | Instr::ResultErr
            | Instr::MatchType(_)
            | Instr::MatchPayload
            | Instr::MatchVariant(_)
            | Instr::TupleAccess(_)
            | Instr::TryUnwrap => {
                st.pop();
                out.push(ins.clone());
                st.push((Abs::Unknown, None));
            }
            other => {
                // Regla general por delta de profundidad de pila.
                let delta: i32 = match other {
                    Instr::Load(_) | Instr::Read | Instr::FuncRef(_) | Instr::OptionNone => 1,
                    Instr::ArrayPush => -1,
                    Instr::ArrayGet => -1,
                    Instr::ArraySet => -2,
                    Instr::ArrayLen => 0,
                    Instr::ArrayPushVar(_) => 0,
                    Instr::StructGet => -1,
                    Instr::StructSet => -2,
                    Instr::MakeRef(_) => 1,
                    Instr::Return | Instr::Halt => -1,
                    Instr::Print => -1,
                    Instr::PushHandler(_)
                    | Instr::PopHandler
                    | Instr::ScopePush
                    | Instr::ScopePop
                    | Instr::Nop => 0,
                    Instr::ConstInt(_)
                    | Instr::ConstFloat(_)
                    | Instr::ConstBool(_)
                    | Instr::ConstStr(_)
                    | Instr::Unary(_)
                    | Instr::Binary(_)
                    | Instr::Label(_)
                    | Instr::Jmp(_)
                    | Instr::JmpIf(_)
                    | Instr::Phi(..)
                    | Instr::Store(_)
                    | Instr::StoreLocal(_)
                    | Instr::Call(_, _)
                    | Instr::CallValue(_)
                    | Instr::ArrayNew(_)
                    | Instr::StructNew(_, _)
                    | Instr::TupleNew(_)
                    | Instr::OptionSome
                    | Instr::ResultOk
                    | Instr::ResultErr
                    | Instr::MatchType(_)
                    | Instr::MatchPayload
                    | Instr::MatchVariant(_)
                    | Instr::TupleAccess(_)
                    | Instr::TryUnwrap
                    | Instr::EnumCtor { .. } => unreachable!(),
                };
                let mut d = delta;
                while d < 0 {
                    st.pop();
                    d += 1;
                }
                out.push(other.clone());
                for _ in 0..delta.max(0) {
                    st.push((Abs::Unknown, None));
                }
            }
        }
    }
    func.instrs = out;
}
