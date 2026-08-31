use crate::bytecode::*;
use lumen_ir::ir::*;
use std::collections::HashMap;

pub struct Codegen {
    bytecode: Bytecode,
    label_map: HashMap<usize, usize>,
    func_starts: HashMap<String, usize>,
    string_cache: HashMap<String, usize>,
    int_cache: HashMap<i64, usize>,
    num_cache: HashMap<u64, usize>,
    name_cache: HashMap<String, usize>,
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            label_map: HashMap::new(),
            func_starts: HashMap::new(),
            string_cache: HashMap::new(),
            int_cache: HashMap::new(),
            num_cache: HashMap::new(),
            name_cache: HashMap::new(),
        }
    }

    fn intern_string(&mut self, s: &str) -> usize {
        if let Some(&idx) = self.string_cache.get(s) {
            idx
        } else {
            let idx = self.bytecode.strings.len();
            self.bytecode.strings.push(s.to_string());
            self.string_cache.insert(s.to_string(), idx);
            idx
        }
    }

    fn intern_int(&mut self, n: i64) -> usize {
        if let Some(&idx) = self.int_cache.get(&n) {
            idx
        } else {
            let idx = self.bytecode.ints.len();
            self.bytecode.ints.push(n);
            self.int_cache.insert(n, idx);
            idx
        }
    }

    fn intern_num(&mut self, n: f64) -> usize {
        let key = n.to_bits();
        if let Some(&idx) = self.num_cache.get(&key) {
            idx
        } else {
            let idx = self.bytecode.nums.len();
            self.bytecode.nums.push(n);
            self.num_cache.insert(key, idx);
            idx
        }
    }

    fn intern_name(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.name_cache.get(name) {
            idx
        } else {
            let idx = self.bytecode.names.len();
            self.bytecode.names.push(name.to_string());
            self.name_cache.insert(name.to_string(), idx);
            idx
        }
    }

    pub fn generate(mut self, program: &Program) -> (Bytecode, Vec<(usize, String)>) {
        let warnings = Vec::new();

        // First pass: compute label positions (instruction indices)
        // v3.5.20: el conteo considera la FUSIÓN de super-opcodes (4 IR → 1).
        let mut running_offset = 0;
        for (func_name, func) in &program.funcs {
            self.func_starts.insert(func_name.clone(), running_offset);
            let live = live_label_indices(&func.instrs);
            let mut i = 0;
            while i < func.instrs.len() {
                if let Instr::Label(l) = &func.instrs[i] {
                    self.label_map.insert(*l, running_offset);
                }
                if let Some((_, consumed)) = try_fuse(&func.instrs, i, &live) {
                    running_offset += 1;
                    i += consumed;
                } else {
                    running_offset += instr_count(&func.instrs[i]);
                    i += 1;
                }
            }
        }

        // Second pass: emit instructions
        // v3.5.20: con super-opcodes fusionados.
        for (func_name, func) in &program.funcs {
            let offset = self.bytecode.instructions.len();
            self.func_starts.insert(func_name.clone(), offset);
            let live = live_label_indices(&func.instrs);
            let mut i = 0;
            while i < func.instrs.len() {
                if let Some((fused, consumed)) = try_fuse(&func.instrs, i, &live) {
                    self.emit_fused(fused);
                    i += consumed;
                    continue;
                }
                self.emit_ir(&func.instrs[i]);
                i += 1;
            }
        }

        // Populate bytecode.funcs sorted by start position
        let mut func_list: Vec<(String, usize)> = self
            .func_starts
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        func_list.sort_by_key(|a| a.1);
        for (name, start) in &func_list {
            let (params, defaults) = program
                .funcs
                .get(name)
                .map(|f| {
                    (
                        f.params.clone(),
                        f.defaults
                            .iter()
                            .map(|d| match d {
                                None => None,
                                Some(v) => match v {
                                    lumen_ir::ir::Value::Int(i) => Some(DefaultValue::Int(*i)),
                                    lumen_ir::ir::Value::Float(v) => Some(DefaultValue::Float(*v)),
                                    lumen_ir::ir::Value::Str(s) => {
                                        Some(DefaultValue::Str(s.clone()))
                                    }
                                    lumen_ir::ir::Value::Bool(b) => Some(DefaultValue::Bool(*b)),
                                    _ => None,
                                },
                            })
                            .collect(),
                    )
                })
                .unwrap_or((Vec::new(), Vec::new()));
            self.bytecode.funcs.push(FuncMeta {
                name: name.clone(),
                params,
                defaults,
                start: *start,
            });
        }

        (self.bytecode, warnings)
    }

    /// v3.5.20: emite un super-opcode fusionado (interna nombres/labels).
    fn emit_fused(&mut self, fused: Fused) {
        match fused {
            Fused::BinK { op, a, k, d } => {
                let a_idx = self.intern_name(&a);
                let d_idx = self.intern_name(&d);
                self.bytecode.instructions.push(Instruction::FusedBinK {
                    op,
                    a: a_idx,
                    k,
                    d: d_idx,
                });
            }
            Fused::Bin { op, a, b, d } => {
                let a_idx = self.intern_name(&a);
                let b_idx = self.intern_name(&b);
                let d_idx = self.intern_name(&d);
                self.bytecode.instructions.push(Instruction::FusedBin {
                    op,
                    a: a_idx,
                    b: b_idx,
                    d: d_idx,
                });
            }
            Fused::BinKLocal { op, a, k, d } => {
                let a_idx = self.intern_name(&a);
                let d_idx = self.intern_name(&d);
                self.bytecode
                    .instructions
                    .push(Instruction::FusedBinKLocal {
                        op,
                        a: a_idx,
                        k,
                        d: d_idx,
                    });
            }
            Fused::BinLocal { op, a, b, d } => {
                let a_idx = self.intern_name(&a);
                let b_idx = self.intern_name(&b);
                let d_idx = self.intern_name(&d);
                self.bytecode.instructions.push(Instruction::FusedBinLocal {
                    op,
                    a: a_idx,
                    b: b_idx,
                    d: d_idx,
                });
            }
            Fused::CmpKJmp { op, a, k, label } => {
                let a_idx = self.intern_name(&a);
                let offset = self.label_map.get(&label).copied().unwrap_or(0);
                let t_idx = self.intern_num(offset as f64);
                self.bytecode.instructions.push(Instruction::FusedCmpKJmp {
                    op,
                    a: a_idx,
                    k,
                    target: t_idx,
                });
            }
            Fused::CmpJmp { op, a, b, label } => {
                let a_idx = self.intern_name(&a);
                let b_idx = self.intern_name(&b);
                let offset = self.label_map.get(&label).copied().unwrap_or(0);
                let t_idx = self.intern_num(offset as f64);
                self.bytecode.instructions.push(Instruction::FusedCmpJmp {
                    op,
                    a: a_idx,
                    b: b_idx,
                    target: t_idx,
                });
            }
            Fused::BinCmpJmp {
                op1,
                op2,
                a,
                b,
                c,
                label,
            } => {
                let a_idx = self.intern_name(&a);
                let b_idx = self.intern_name(&b);
                let c_idx = self.intern_name(&c);
                let offset = self.label_map.get(&label).copied().unwrap_or(0);
                let t_idx = self.intern_num(offset as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::FusedBinCmpJmp {
                        op1,
                        op2,
                        a: a_idx,
                        b: b_idx,
                        c: c_idx,
                        target: t_idx,
                    });
            }
            Fused::BinKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                label,
            } => {
                let a_idx = self.intern_name(&a);
                let b_idx = self.intern_name(&b);
                let offset = self.label_map.get(&label).copied().unwrap_or(0);
                let t_idx = self.intern_num(offset as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::FusedBinKCmpJmp {
                        op1,
                        op2,
                        a: a_idx,
                        b: b_idx,
                        k,
                        target: t_idx,
                    });
            }
            Fused::BinKKCmpJmp {
                op1,
                op2,
                a,
                b,
                k,
                label,
            } => {
                let a_idx = self.intern_name(&a);
                let offset = self.label_map.get(&label).copied().unwrap_or(0);
                let t_idx = self.intern_num(offset as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::FusedBinKKCmpJmp {
                        op1,
                        op2,
                        a: a_idx,
                        b,
                        k,
                        target: t_idx,
                    });
            }
        }
    }

    fn emit_ir(&mut self, instr: &Instr) {
        match instr {
            Instr::ConstInt(n) => {
                let idx = self.intern_int(*n);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::PushInt, idx));
            }
            Instr::ConstFloat(n) => {
                let idx = self.intern_num(*n);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::PushNum, idx));
            }
            Instr::ConstStr(s) => {
                let idx = self.intern_string(s);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::PushStr, idx));
            }
            Instr::ConstBool(b) => {
                self.bytecode
                    .instructions
                    .push(Instruction::WithBool(Opcode::PushBool, *b));
            }
            Instr::Load(name) => {
                let idx = self.intern_name(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Load, idx));
            }
            Instr::Store(name) => {
                let idx = self.intern_name(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Store, idx));
            }
            Instr::StoreLocal(name) => {
                let idx = self.intern_name(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::StoreLocal, idx));
            }
            Instr::Binary(op) => {
                let opcode = match op {
                    Op::Add => Opcode::Add,
                    Op::Concat => Opcode::Concat,
                    Op::Sub => Opcode::Sub,
                    Op::Mul => Opcode::Mul,
                    Op::Div => Opcode::Div,
                    Op::Mod => Opcode::Mod,
                    Op::Equal => Opcode::Eq,
                    Op::NotEqual => Opcode::Neq,
                    Op::Less => Opcode::Lt,
                    Op::LessEqual => Opcode::Le,
                    Op::Greater => Opcode::Gt,
                    Op::GreaterEqual => Opcode::Ge,
                    Op::And => Opcode::And,
                    Op::Or => Opcode::Or,
                    Op::BitOr => Opcode::BitOr,
                    Op::BitAnd => Opcode::BitAnd,
                    Op::BitXor => Opcode::BitXor,
                    Op::ShiftLeft => Opcode::ShiftLeft,
                    Op::ShiftRight => Opcode::ShiftRight,
                    _ => Opcode::Nop,
                };
                self.bytecode.instructions.push(Instruction::Simple(opcode));
            }
            Instr::Unary(op) => {
                let opcode = match op {
                    Op::Negate => Opcode::Neg,
                    Op::Not => Opcode::Not,
                    Op::BitNot => Opcode::BitNot,
                    _ => Opcode::Nop,
                };
                self.bytecode.instructions.push(Instruction::Simple(opcode));
            }
            Instr::Call(name, argc) => {
                let idx = self.intern_name(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Call, idx));
                let num_idx = self.intern_num(*argc as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Nop, num_idx));
            }
            Instr::FuncRef(name) => {
                let idx = self.intern_string(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::FuncRef, idx));
            }
            Instr::CallValue(argc) => {
                let idx = self.intern_num(*argc as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::CallValue, idx));
            }
            Instr::Return => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::Ret));
            }
            Instr::Print => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::Print));
            }
            Instr::Jmp(label) => {
                let offset = self.label_map.get(label).copied().unwrap_or(0);
                let idx = self.intern_num(offset as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Jmp, idx));
            }
            Instr::JmpIf(label) => {
                let offset = self.label_map.get(label).copied().unwrap_or(0);
                let idx = self.intern_num(offset as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::JmpIf, idx));
            }
            Instr::Halt => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::Halt));
            }
            Instr::PushHandler(label) => {
                let offset = self.label_map.get(label).copied().unwrap_or(0);
                let idx = self.intern_num(offset as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::PushHandler, idx));
            }
            Instr::PopHandler => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::PopHandler));
            }
            Instr::ScopePush => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ScopePush));
            }
            Instr::ScopePop => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ScopePop));
            }
            Instr::MatchVariant(variant_name) => {
                self.bytecode.instructions.push(Instruction::WithStr(
                    Opcode::MatchVariant,
                    variant_name.clone(),
                ));
            }
            Instr::MakeRef(name) => {
                let idx = self.intern_name(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::MakeRef, idx));
            }
            Instr::Label(_) => {}
            Instr::Phi(_, _) => {}
            Instr::Read => {}
            Instr::Nop => {
                // v3.5.31: los Nop (scopes vacíos eliminados) NO se emiten —
                // instr_count ya los cuenta como 0 en la primera pasada.
            }
            Instr::ArrayNew(n) => {
                let idx = self.intern_num(*n as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::ArrayNew, idx));
            }
            Instr::ArrayGet => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ArrayGet));
            }
            Instr::ArraySet => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ArraySet));
            }
            Instr::ArrayLen => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ArrayLen));
            }
            Instr::ArrayPush => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ArrayPush));
            }
            Instr::ArrayPushVar(name) => {
                let idx = self.intern_name(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::ArrayPushVar, idx));
            }
            Instr::ArraySetVar(name) => {
                let idx = self.intern_name(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::ArraySetVar, idx));
            }
            Instr::StructNew(name, count) => {
                let idx = self.intern_string(name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::StructNew, idx));
                let num_idx = self.intern_num(*count as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Nop, num_idx));
            }
            Instr::StructGet => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::StructGet));
            }
            Instr::StructSet => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::StructSet));
            }
            Instr::ResultOk => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ResultOk));
            }
            Instr::ResultErr => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::ResultErr));
            }
            Instr::TryUnwrap => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::TryUnwrap));
            }
            Instr::OptionSome => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::OptionSome));
            }
            Instr::OptionNone => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::OptionNone));
            }
            Instr::MatchType(kind) => {
                let idx = self.intern_num(*kind as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::MatchType, idx));
            }
            Instr::MatchPayload => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::MatchPayload));
            }
            Instr::TupleNew(count) => {
                let idx = self.intern_num(*count as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::TupleNew, idx));
            }
            Instr::TupleAccess(index) => {
                let idx = self.intern_num(*index as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::TupleAccess, idx));
            }
            Instr::EnumCtor {
                enum_name,
                variant,
                argc,
            } => {
                let name_idx = self.intern_string(enum_name);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::EnumCtor, name_idx));
                let var_idx = self.intern_string(variant);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Nop, var_idx));
                let num_idx = self.intern_num(*argc as f64);
                self.bytecode
                    .instructions
                    .push(Instruction::WithIdx(Opcode::Nop, num_idx));
            }
        }
    }
}

fn instr_count(instr: &Instr) -> usize {
    match instr {
        Instr::Label(_) | Instr::Phi(_, _) | Instr::Read | Instr::Nop => 0,
        Instr::Call(_, _) => 2,
        Instr::ArrayNew(_) => 1,
        Instr::TupleNew(_) => 1,
        Instr::TupleAccess(_) => 1,
        Instr::StructNew(_, _) => 2,
        Instr::EnumCtor { .. } => 3,
        _ => 1,
    }
}

/// v3.5.20: sub-op numérico de los super-opcodes (numeración backend C).
fn fused_op_code(op: &lumen_ir::ir::Op) -> Option<u8> {
    use lumen_ir::ir::Op;
    Some(match op {
        Op::Add => 1,
        Op::Sub => 3,
        Op::Mul => 4,
        Op::Div => 5,
        Op::Mod => 6,
        Op::Equal => 7,
        Op::NotEqual => 8,
        Op::Less => 9,
        Op::LessEqual => 10,
        Op::Greater => 11,
        Op::GreaterEqual => 12,
        _ => return None,
    })
}

fn is_fusable_arith(op: &lumen_ir::ir::Op) -> bool {
    matches!(
        op,
        lumen_ir::ir::Op::Add | lumen_ir::ir::Op::Sub | lumen_ir::ir::Op::Mul
    )
}

fn is_fusable_cmp(op: &lumen_ir::ir::Op) -> bool {
    use lumen_ir::ir::Op;
    matches!(
        op,
        Op::Equal | Op::NotEqual | Op::Less | Op::LessEqual | Op::Greater | Op::GreaterEqual
    )
}

/// v3.5.31: aritmética fusable en los patrones de 6 instrucciones
/// (aritmética + comparación + salto). Div/Mod incluidos: los handlers
/// reproducen la semántica EXACTA de los opcodes clásicos.
fn is_fusable_arith6(op: &lumen_ir::ir::Op) -> bool {
    use lumen_ir::ir::Op;
    matches!(op, Op::Add | Op::Sub | Op::Mul | Op::Div | Op::Mod)
}

/// Resultado del peephole: qué super-opcode emitir (con nombres/constes;
/// el llamador interna los índices).
enum Fused {
    BinK {
        op: u8,
        a: String,
        k: i64,
        d: String,
    },
    Bin {
        op: u8,
        a: String,
        b: String,
        d: String,
    },
    /// v3.5.41 (bug #10): fusión de una DECLARACIÓN (StoreLocal) — el
    /// destino se resuelve en el scope actual del frame, no en scopes de
    /// frames ancestros (la variante Bin/BinK conserva la semántica de
    /// ASIGNACIÓN, que sí busca el binding más cercano).
    BinKLocal {
        op: u8,
        a: String,
        k: i64,
        d: String,
    },
    BinLocal {
        op: u8,
        a: String,
        b: String,
        d: String,
    },
    CmpKJmp {
        op: u8,
        a: String,
        k: i64,
        label: usize,
    },
    CmpJmp {
        op: u8,
        a: String,
        b: String,
        label: usize,
    },
    // v3.5.31: aritmética + comparación + salto (6 IR → 1 super-opcode).
    // Patrones de bucle: `si (i*i <= n) ...`, `si (n%i == 0) ...`.
    // op1 ∈ {1 Add, 3 Sub, 4 Mul, 5 Div, 6 Mod}; op2 ∈ {7..12 cmp}.
    BinCmpJmp {
        op1: u8,
        op2: u8,
        a: String,
        b: String,
        c: String,
        label: usize,
    },
    BinKCmpJmp {
        op1: u8,
        op2: u8,
        a: String,
        b: String,
        k: i64,
        label: usize,
    },
    BinKKCmpJmp {
        op1: u8,
        op2: u8,
        a: String,
        b: i64,
        k: i64,
        label: usize,
    },
}

/// v3.5.20: PEEPHOLE de super-opcodes. Detecta los patrones canónicos de
/// bucle (4 instrucciones IR → 1 super-opcode). `live_label_idx`: índices
/// que son destino de algún salto — ninguna posición INTERIOR del patrón
/// puede serlo (un salto que caiga a mitad del patrón lo invalida).
fn try_fuse(
    instrs: &[lumen_ir::ir::Instr],
    i: usize,
    live_label_idx: &std::collections::HashSet<usize>,
) -> Option<(Fused, usize)> {
    use lumen_ir::ir::Instr;
    // v3.5.31: primero los patrones de 6 (aritmética+cmp+salto), luego los
    // clásicos de 4. Ninguna posición interior puede ser destino de salto.
    if i + 6 <= instrs.len() && (1..6).all(|off| !live_label_idx.contains(&(i + off))) {
        let f6 = match (
            &instrs[i],
            &instrs[i + 1],
            &instrs[i + 2],
            &instrs[i + 3],
            &instrs[i + 4],
            &instrs[i + 5],
        ) {
            (
                Instr::Load(a),
                Instr::Load(b),
                Instr::Binary(op1),
                Instr::Load(c),
                Instr::Binary(op2),
                Instr::JmpIf(label),
            ) if is_fusable_arith6(op1) && is_fusable_cmp(op2) => Some((
                Fused::BinCmpJmp {
                    op1: fused_op_code(op1)?,
                    op2: fused_op_code(op2)?,
                    a: a.clone(),
                    b: b.clone(),
                    c: c.clone(),
                    label: *label,
                },
                6,
            )),
            (
                Instr::Load(a),
                Instr::Load(b),
                Instr::Binary(op1),
                Instr::ConstInt(k),
                Instr::Binary(op2),
                Instr::JmpIf(label),
            ) if is_fusable_arith6(op1) && is_fusable_cmp(op2) => Some((
                Fused::BinKCmpJmp {
                    op1: fused_op_code(op1)?,
                    op2: fused_op_code(op2)?,
                    a: a.clone(),
                    b: b.clone(),
                    k: *k,
                    label: *label,
                },
                6,
            )),
            (
                Instr::Load(a),
                Instr::ConstInt(b),
                Instr::Binary(op1),
                Instr::ConstInt(k),
                Instr::Binary(op2),
                Instr::JmpIf(label),
            ) if is_fusable_arith6(op1) && is_fusable_cmp(op2) => Some((
                Fused::BinKKCmpJmp {
                    op1: fused_op_code(op1)?,
                    op2: fused_op_code(op2)?,
                    a: a.clone(),
                    b: *b,
                    k: *k,
                    label: *label,
                },
                6,
            )),
            _ => None,
        };
        if f6.is_some() {
            return f6;
        }
    }
    if i + 4 > instrs.len() {
        return None;
    }
    for off in 1..4 {
        if live_label_idx.contains(&(i + off)) {
            return None;
        }
    }
    match (&instrs[i], &instrs[i + 1], &instrs[i + 2], &instrs[i + 3]) {
        (Instr::Load(a), Instr::ConstInt(k), Instr::Binary(op), Instr::Store(d))
            if is_fusable_arith(op) =>
        {
            Some((
                Fused::BinK {
                    op: fused_op_code(op)?,
                    a: a.clone(),
                    k: *k,
                    d: d.clone(),
                },
                4,
            ))
        }
        // v3.5.41 (bug #10): la DECLARACIÓN conserva la semántica de
        // StoreLocal (scope actual) — antes se fusionaba igual que la
        // ASIGNACIÓN y la recursión corrompía los locales del llamador.
        (Instr::Load(a), Instr::ConstInt(k), Instr::Binary(op), Instr::StoreLocal(d))
            if is_fusable_arith(op) =>
        {
            Some((
                Fused::BinKLocal {
                    op: fused_op_code(op)?,
                    a: a.clone(),
                    k: *k,
                    d: d.clone(),
                },
                4,
            ))
        }
        (Instr::Load(a), Instr::Load(b), Instr::Binary(op), Instr::Store(d))
            if is_fusable_arith(op) =>
        {
            Some((
                Fused::Bin {
                    op: fused_op_code(op)?,
                    a: a.clone(),
                    b: b.clone(),
                    d: d.clone(),
                },
                4,
            ))
        }
        (Instr::Load(a), Instr::Load(b), Instr::Binary(op), Instr::StoreLocal(d))
            if is_fusable_arith(op) =>
        {
            Some((
                Fused::BinLocal {
                    op: fused_op_code(op)?,
                    a: a.clone(),
                    b: b.clone(),
                    d: d.clone(),
                },
                4,
            ))
        }
        (Instr::Load(a), Instr::ConstInt(k), Instr::Binary(op), Instr::JmpIf(label))
            if is_fusable_cmp(op) =>
        {
            Some((
                Fused::CmpKJmp {
                    op: fused_op_code(op)?,
                    a: a.clone(),
                    k: *k,
                    label: *label,
                },
                4,
            ))
        }
        (Instr::Load(a), Instr::Load(b), Instr::Binary(op), Instr::JmpIf(label))
            if is_fusable_cmp(op) =>
        {
            Some((
                Fused::CmpJmp {
                    op: fused_op_code(op)?,
                    a: a.clone(),
                    b: b.clone(),
                    label: *label,
                },
                4,
            ))
        }
        _ => None,
    }
}

/// Índices de instrucción (dentro de `instrs`) que son destino de algún
/// salto/manejador: esas posiciones no pueden quedar dentro de un patrón
/// fusionado.
fn live_label_indices(instrs: &[lumen_ir::ir::Instr]) -> std::collections::HashSet<usize> {
    use lumen_ir::ir::Instr;
    let mut targets: std::collections::HashSet<usize> = std::collections::HashSet::new();
    for ins in instrs {
        match ins {
            Instr::Jmp(l) | Instr::JmpIf(l) | Instr::PushHandler(l) => {
                targets.insert(*l);
            }
            _ => {}
        }
    }
    let mut idxs = std::collections::HashSet::new();
    for (i, ins) in instrs.iter().enumerate() {
        if let Instr::Label(l) = ins {
            if targets.contains(l) {
                idxs.insert(i);
            }
        }
    }
    idxs
}
