// src/compiler/codegen.rs
use crate::compiler::ast::*;
use crate::instructions::OpCode;
use std::collections::HashMap;

pub struct CodeGenerator {
    bytecode: Vec<u8>,

    // Variables numéricas
    variables: HashMap<String, usize>,
    next_var_addr: usize,

    // Pool de strings (índice estable)
    string_pool: Vec<String>,
}

impl CodeGenerator {
    pub fn new() -> Self {
        Self {
            bytecode: Vec::new(),
            variables: HashMap::new(),
            next_var_addr: 0,
            string_pool: Vec::new(),
        }
    }

    /* ========= EMIT ========= */

    fn emit_opcode(&mut self, opcode: OpCode) {
        self.bytecode.push(opcode as u8);
    }

    fn emit_i32(&mut self, value: i32) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_u32(&mut self, value: u32) {
        self.bytecode.extend_from_slice(&value.to_le_bytes());
    }

    fn emit_usize(&mut self, value: usize) {
        self.emit_u32(value as u32);
    }

    fn current_position(&self) -> usize {
        self.bytecode.len()
    }

    fn patch_jump(&mut self, position: usize, target: usize) {
        let bytes = (target as u32).to_le_bytes();
        self.bytecode[position..position + 4].copy_from_slice(&bytes);
    }

    /* ========= VARIABLES ========= */

    fn get_or_create_var(&mut self, name: &str) -> usize {
        if let Some(&addr) = self.variables.get(name) {
            addr
        } else {
            let addr = self.next_var_addr;
            self.variables.insert(name.to_string(), addr);
            self.next_var_addr += 1;
            addr
        }
    }

    /* ========= STRINGS ========= */

    fn intern_string(&mut self, value: &str) -> usize {
        if let Some((idx, _)) = self
            .string_pool
            .iter()
            .enumerate()
            .find(|(_, s)| s == &value)
        {
            idx
        } else {
            let idx = self.string_pool.len();
            self.string_pool.push(value.to_string());
            idx
        }
    }

    /* ========= ENTRY ========= */

    pub fn generate(&mut self, program: &Program) -> Vec<u8> {
        for stmt in &program.statements {
            self.generate_statement(stmt);
        }

        self.emit_opcode(OpCode::Halt);
        self.bytecode.clone()
    }

    /* ========= STATEMENTS ========= */

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Import { .. } => {
                // Los imports se resuelven en una fase anterior (loader / linker)
                // El codegen no genera bytecode aquí
            }

            Statement::VarDecl { name, value } => {
                self.generate_expr(value);
                let addr = self.get_or_create_var(name);
                self.emit_opcode(OpCode::Store);
                self.emit_usize(addr);
            }

            Statement::Assignment { target, value } => {
                self.generate_expr(value);
                let addr = self.get_or_create_var(target);
                self.emit_opcode(OpCode::Store);
                self.emit_usize(addr);
            }

            Statement::Print { expr } => {
                self.generate_expr(expr);
                self.emit_opcode(OpCode::Print);
            }

            Statement::If {
                condition,
                then_body,
                else_body,
            } => {
                self.generate_expr(condition);

                self.emit_opcode(OpCode::JmpIfFalse);
                let jump_to_else = self.current_position();
                self.emit_usize(0);

                for stmt in then_body {
                    self.generate_statement(stmt);
                }

                if let Some(else_stmts) = else_body {
                    self.emit_opcode(OpCode::Jmp);
                    let jump_to_end = self.current_position();
                    self.emit_usize(0);

                    let else_start = self.current_position();
                    self.patch_jump(jump_to_else, else_start);

                    for stmt in else_stmts {
                        self.generate_statement(stmt);
                    }

                    let end_pos = self.current_position();
                    self.patch_jump(jump_to_end, end_pos);
                } else {
                    let end_pos = self.current_position();
                    self.patch_jump(jump_to_else, end_pos);
                }
            }

            Statement::While { condition, body } => {
                let loop_start = self.current_position();

                self.generate_expr(condition);

                self.emit_opcode(OpCode::JmpIfFalse);
                let jump_to_end = self.current_position();
                self.emit_usize(0);

                for stmt in body {
                    self.generate_statement(stmt);
                }

                self.emit_opcode(OpCode::Jmp);
                self.emit_usize(loop_start);

                let end_pos = self.current_position();
                self.patch_jump(jump_to_end, end_pos);
            }
        }
    }

    /* ========= EXPRESSIONS ========= */

    fn generate_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Number(n) => {
                self.emit_opcode(OpCode::PushNum);
                self.emit_i32(*n);
            }

            Expr::String(value) => {
                let idx = self.intern_string(value);
                self.emit_opcode(OpCode::PushStr);
                self.emit_usize(idx);
            }

            Expr::Variable(name) => {
                let addr = self.get_or_create_var(name);
                self.emit_opcode(OpCode::Load);
                self.emit_usize(addr);
            }

            Expr::BinOp { left, op, right } => {
                self.generate_expr(left);
                self.generate_expr(right);

                match op {
                    BinOperator::Add => self.emit_opcode(OpCode::Add),
                    BinOperator::Sub => self.emit_opcode(OpCode::Sub),
                    BinOperator::Mul => self.emit_opcode(OpCode::Mul),
                    BinOperator::Div => self.emit_opcode(OpCode::Div),
                    BinOperator::Eq => self.emit_opcode(OpCode::Eq),
                    BinOperator::Lt => self.emit_opcode(OpCode::Lt),
                    BinOperator::Gt => self.emit_opcode(OpCode::Gt),
                }
            }
        }
    }
}
