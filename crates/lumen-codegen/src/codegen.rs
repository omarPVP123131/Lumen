use std::collections::HashMap;
use lumen_ir::ir::*;
use crate::bytecode::*;

pub struct Codegen {
    bytecode: Bytecode,
    label_map: HashMap<usize, usize>,
    func_starts: HashMap<String, usize>,
}

impl Codegen {
    pub fn new() -> Self {
        Self {
            bytecode: Bytecode::new(),
            label_map: HashMap::new(),
            func_starts: HashMap::new(),
        }
    }

    pub fn generate(mut self, program: &Program) -> (Bytecode, Vec<(usize, String)>) {
        let warnings = Vec::new();

        // First pass: compute label positions (instruction indices)
        for (func_name, func) in &program.funcs {
            let mut offset = self.bytecode.instructions.len();
            self.func_starts.insert(func_name.clone(), offset);
            for instr in &func.instrs {
                if let Instr::Label(l) = instr {
                    self.label_map.entry(*l).or_insert(offset);
                }
                offset += instr_count(instr);
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
        let mut func_list: Vec<(String, usize)> = self.func_starts.iter()
            .map(|(k, v)| (k.clone(), *v))
            .collect();
        func_list.sort_by(|a, b| a.1.cmp(&b.1));
        for (name, start) in &func_list {
            let params = program.funcs.get(name)
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
                self.bytecode.ints.push(*n);
                let idx = self.bytecode.ints.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::PushInt, idx));
            }
            Instr::ConstFloat(n) => {
                self.bytecode.nums.push(*n);
                let idx = self.bytecode.nums.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::PushNum, idx));
            }
            Instr::ConstStr(s) => {
                self.bytecode.strings.push(s.clone());
                let idx = self.bytecode.strings.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::PushStr, idx));
            }
            Instr::ConstBool(b) => {
                self.bytecode.instructions.push(Instruction::WithBool(Opcode::PushBool, *b));
            }
            Instr::Load(name) => {
                self.bytecode.names.push(name.clone());
                let idx = self.bytecode.names.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::Load, idx));
            }
            Instr::Store(name) => {
                self.bytecode.names.push(name.clone());
                let idx = self.bytecode.names.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::Store, idx));
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
                self.bytecode.names.push(name.clone());
                let idx = self.bytecode.names.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::Call, idx));
                self.bytecode.nums.push(*argc as f64);
                let num_idx = self.bytecode.nums.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::Nop, num_idx));
            }
            Instr::Return => {
                self.bytecode.instructions.push(Instruction::Simple(Opcode::Ret));
            }
            Instr::Print => {
                self.bytecode.instructions.push(Instruction::Simple(Opcode::Print));
            }
            Instr::Jmp(label) => {
                let offset = self.label_map.get(label).copied().unwrap_or(0);
                self.bytecode.nums.push(offset as f64);
                let idx = self.bytecode.nums.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::Jmp, idx));
            }
            Instr::JmpIf(label) => {
                let offset = self.label_map.get(label).copied().unwrap_or(0);
                self.bytecode.nums.push(offset as f64);
                let idx = self.bytecode.nums.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::JmpIf, idx));
            }
            Instr::Halt => {
                self.bytecode.instructions.push(Instruction::Simple(Opcode::Halt));
            }
            Instr::Label(_) => {}
            Instr::Phi(_, _) => {}
            Instr::Read => {}
            Instr::ArrayNew(n) => {
                self.bytecode.nums.push(*n as f64);
                let idx = self.bytecode.nums.len() - 1;
                self.bytecode.instructions.push(Instruction::WithIdx(Opcode::ArrayNew, idx));
            }
            Instr::ArrayGet => {
                self.bytecode.instructions.push(Instruction::Simple(Opcode::ArrayGet));
            }
            Instr::ArraySet => {
                self.bytecode.instructions.push(Instruction::Simple(Opcode::ArraySet));
            }
            Instr::ArrayLen => {
                self.bytecode.instructions.push(Instruction::Simple(Opcode::ArrayLen));
            }
            Instr::ArrayPush => {
                self.bytecode.instructions.push(Instruction::Simple(Opcode::ArrayPush));
            }
        }
    }

}

fn instr_count(instr: &Instr) -> usize {
    match instr {
        Instr::Label(_) | Instr::Phi(_, _) | Instr::Read => 0,
        Instr::Call(_, _) => 2,
        Instr::ArrayNew(_) => 1,
        _ => 1,
    }
}
