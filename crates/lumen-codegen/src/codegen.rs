use crate::bytecode::*;
use lumen_ir::ir::*;
use std::collections::HashMap;

pub struct Codegen {
    bytecode: Bytecode,
    label_map: HashMap<usize, usize>,
    func_starts: HashMap<String, usize>,
    string_cache: HashMap<String, usize>,
    int_cache: HashMap<i64, usize>,
    num_cache: HashMap<usize, usize>,
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
        let key = n.to_bits() as usize;
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
        let mut running_offset = 0;
        for (func_name, func) in &program.funcs {
            self.func_starts.insert(func_name.clone(), running_offset);
            for instr in &func.instrs {
                if let Instr::Label(l) = instr {
                    self.label_map.entry(*l).or_insert(running_offset);
                }
                running_offset += instr_count(instr);
            }
        }

        // Second pass: emit instructions
        for (func_name, func) in &program.funcs {
            let offset = self.bytecode.instructions.len();
            self.func_starts.insert(func_name.clone(), offset);
            for instr in &func.instrs {
                self.emit_ir(instr);
            }
        }

        // Populate bytecode.funcs sorted by start position
        let mut func_list: Vec<(String, usize)> = self
            .func_starts
            .iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        func_list.sort_by(|a, b| a.1.cmp(&b.1));
        for (name, start) in &func_list {
            let params = program
                .funcs
                .get(name)
                .map(|f| f.params.clone())
                .unwrap_or_default();
            self.bytecode.funcs.push(FuncMeta {
                name: name.clone(),
                params,
                start: *start,
            });
        }

        (self.bytecode, warnings)
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
            Instr::Binary(op) => {
                let opcode = match op {
                    Op::Add => Opcode::Add,
                    Op::Sub => Opcode::Sub,
                    Op::Mul => Opcode::Mul,
                    Op::Div => Opcode::Div,
                    Op::Equal => Opcode::Eq,
                    Op::NotEqual => Opcode::Neq,
                    Op::Less => Opcode::Lt,
                    Op::LessEqual => Opcode::Le,
                    Op::Greater => Opcode::Gt,
                    Op::GreaterEqual => Opcode::Ge,
                    Op::And => Opcode::And,
                    Op::Or => Opcode::Or,
                    _ => Opcode::Nop,
                };
                self.bytecode.instructions.push(Instruction::Simple(opcode));
            }
            Instr::Unary(op) => {
                let opcode = match op {
                    Op::Negate => Opcode::Neg,
                    Op::Not => Opcode::Not,
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
            Instr::Label(_) => {}
            Instr::Phi(_, _) => {}
            Instr::Read => {}
            Instr::Nop => {
                self.bytecode
                    .instructions
                    .push(Instruction::Simple(Opcode::Nop));
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
        }
    }
}

fn instr_count(instr: &Instr) -> usize {
    match instr {
        Instr::Label(_) | Instr::Phi(_, _) | Instr::Read | Instr::Nop => 0,
        Instr::Call(_, _) => 2,
        Instr::ArrayNew(_) => 1,
        Instr::StructNew(_, _) => 2,
        _ => 1,
    }
}
