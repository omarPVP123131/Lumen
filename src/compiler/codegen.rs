use crate::compiler::ast::*;
use crate::instructions::OpCode;
use crate::vm::constant_pool::ConstantPool;
use std::collections::HashMap;

pub struct CodeGenerator {
    bytecode: Vec<u8>,
    variables: HashMap<String, u32>,
    next_var_addr: u32,
    pool: ConstantPool,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            variables: HashMap::new(),
            next_var_addr: 0,
            pool: ConstantPool::new(),
        }
    }

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.bytecode.push(opcode as u8);
    }

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn current_position(&self) -> u32 {
        self.bytecode.len() as u32
    }

    fn patch_jump(&mut self, position: u32, target: u32) {
        let pos = position as usize;
        self.bytecode[pos..pos + 4].copy_from_slice(&target.to_le_bytes());
    }

    fn get_or_create_var(&mut self, name: &str) -> u32 {
        *self.variables.entry(name.to_string()).or_insert_with(|| {
            let addr = self.next_var_addr;
            self.next_var_addr += 1;
            addr
        })
    }

    pub fn generate(mut self, program: &Program) -> (Vec<u8>, ConstantPool) {
        for stmt in &program.statements {
            self.generate_statement(stmt);
        }
        self.emit_opcode(OpCode::Halt);
        (self.bytecode, self.pool)
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDecl { name, value }
            | Statement::Assignment { target: name, value } => {
                self.generate_expr(value);
                let addr = self.get_or_create_var(name);
                self.emit_opcode(OpCode::Store);
                self.emit_u32(addr);
            }

            Statement::Print { expr } => {
                self.generate_expr(expr);
                self.emit_opcode(OpCode::Print);
            }

            Statement::If { condition, then_body, else_body } => {
                self.generate_expr(condition);
                self.emit_opcode(OpCode::JmpIfFalse);
                let jump_else = self.current_position();
                self.emit_u32(0);

                for stmt in then_body {
                    self.generate_statement(stmt);
                }

                if let Some(else_body) = else_body {
                    self.emit_opcode(OpCode::Jmp);
                    let jump_end = self.current_position();
                    self.emit_u32(0);

                    let else_start = self.current_position();
                    self.patch_jump(jump_else, else_start);

                    for stmt in else_body {
                        self.generate_statement(stmt);
                    }

                    self.patch_jump(jump_end, self.current_position());
                } else {
                    self.patch_jump(jump_else, self.current_position());
                }
            }

            Statement::While { condition, body } => {
                let loop_start = self.current_position();
                self.generate_expr(condition);

                self.emit_opcode(OpCode::JmpIfFalse);
                let jump_end = self.current_position();
                self.emit_u32(0);

                for stmt in body {
                    self.generate_statement(stmt);
                }

                self.emit_opcode(OpCode::Jmp);
                self.emit_u32(loop_start);

                self.patch_jump(jump_end, self.current_position());
            }

            Statement::Import { .. } => {}
        }
    }

    fn generate_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                let id = self.pool.add_int(*n as i64);
                self.emit_opcode(OpCode::PushNum);
                self.emit_u32(id);
            }

            Expr::Variable(name) => {
                let addr = self.get_or_create_var(name);
                self.emit_opcode(OpCode::Load);
                self.emit_u32(addr);
            }

            Expr::BinOp { left, op, right } => {
                self.generate_expr(left);
                self.generate_expr(right);

                self.emit_opcode(match op {
                    BinOperator::Add => OpCode::Add,
                    BinOperator::Sub => OpCode::Sub,
                    BinOperator::Mul => OpCode::Mul,
                    BinOperator::Div => OpCode::Div,
                    BinOperator::Eq  => OpCode::Eq,
                    BinOperator::Lt  => OpCode::Lt,
                    BinOperator::Gt  => OpCode::Gt,
                });
            }

            Expr::String(_) => {
                panic!("Strings ya no existen en bytecode")
            }
        }
    }
}
