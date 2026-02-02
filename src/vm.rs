use crate::instructions::OpCode;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Value {
    Int(i32),
    Str(String),
}

#[derive(Debug)]
#[allow(dead_code)] // los campos se usan para diagnóstico
pub enum VMError {
    UnknownOpcode(u8),
    StackUnderflow,
    InvalidJump(usize),
    DivisionByZero,
    InvalidMemoryAccess(usize),
    CodeExhausted,
    TypeError,
    InvalidStringIndex(usize),
}

pub struct VM {
    code: Vec<u8>,
    ip: usize,
    running: bool,

    // Stack tipado interno del VM
    stack: Vec<Value>,

    // Memoria global
    memory: HashMap<usize, Value>,

    // Pool de strings (inyectado por el loader más adelante)
    string_pool: Vec<String>,
}

impl VM {
    /// Firma compatible con el CLI actual
    pub fn new(code: Vec<u8>) -> Self {
        VM {
            code,
            ip: 0,
            running: true,
            stack: Vec::new(),
            memory: HashMap::new(),
            string_pool: Vec::new(),
        }
    }

    /* ========= STACK HELPERS ========= */

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, VMError> {
        self.stack.pop().ok_or(VMError::StackUnderflow)
    }

    fn pop_int(&mut self) -> Result<i32, VMError> {
        match self.pop()? {
            Value::Int(v) => Ok(v),
            _ => Err(VMError::TypeError),
        }
    }

    /* ========= READERS ========= */

    fn read_i32(&mut self) -> Result<i32, VMError> {
        if self.ip + 4 > self.code.len() {
            return Err(VMError::CodeExhausted);
        }
        let bytes = &self.code[self.ip..self.ip + 4];
        self.ip += 4;
        Ok(i32::from_le_bytes(bytes.try_into().unwrap()))
    }

    fn read_usize(&mut self) -> Result<usize, VMError> {
        if self.ip + 4 > self.code.len() {
            return Err(VMError::CodeExhausted);
        }
        let bytes = &self.code[self.ip..self.ip + 4];
        self.ip += 4;
        Ok(u32::from_le_bytes(bytes.try_into().unwrap()) as usize)
    }

    /* ========= EXECUTION ========= */

    pub fn run(&mut self) -> Result<(), VMError> {
        while self.running {
            if self.ip >= self.code.len() {
                return Err(VMError::CodeExhausted);
            }

            let opcode_byte = self.code[self.ip];
            self.ip += 1;

            let opcode = OpCode::from(opcode_byte)
                .ok_or(VMError::UnknownOpcode(opcode_byte))?;

            match opcode {
                /* ===== CAPA 1 ===== */

                OpCode::PushNum => {
                    let v = self.read_i32()?;
                    self.push(Value::Int(v));
                }

                OpCode::Print => {
                    match self.pop()? {
                        Value::Int(v) => println!("{}", v),
                        Value::Str(s) => println!("{}", s),
                    }
                }

                OpCode::Halt => {
                    self.running = false;
                }

                /* ===== CAPA 2: ARITMÉTICA ===== */

                OpCode::Add => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(a + b));
                }

                OpCode::Sub => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(a - b));
                }

                OpCode::Mul => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(a * b));
                }

                OpCode::Div => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    if b == 0 {
                        return Err(VMError::DivisionByZero);
                    }
                    self.push(Value::Int(a / b));
                }

                /* ===== VARIABLES ===== */

                OpCode::Store => {
                    let addr = self.read_usize()?;
                    let value = self.pop()?;
                    self.memory.insert(addr, value);
                }

                OpCode::Load => {
                    let addr = self.read_usize()?;
                    let value = self.memory
                        .get(&addr)
                        .cloned()
                        .ok_or(VMError::InvalidMemoryAccess(addr))?;
                    self.push(value);
                }

                /* ===== COMPARACIONES ===== */

                OpCode::Eq => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(if a == b { 1 } else { 0 }));
                }

                OpCode::Lt => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(if a < b { 1 } else { 0 }));
                }

                OpCode::Gt => {
                    let b = self.pop_int()?;
                    let a = self.pop_int()?;
                    self.push(Value::Int(if a > b { 1 } else { 0 }));
                }

                /* ===== CONTROL DE FLUJO ===== */

                OpCode::Jmp => {
                    let addr = self.read_usize()?;
                    if addr >= self.code.len() {
                        return Err(VMError::InvalidJump(addr));
                    }
                    self.ip = addr;
                }

                OpCode::JmpIfFalse => {
                    let addr = self.read_usize()?;
                    let cond = self.pop_int()?;
                    if cond == 0 {
                        if addr >= self.code.len() {
                            return Err(VMError::InvalidJump(addr));
                        }
                        self.ip = addr;
                    }
                }

                /* ===== CAPA 3: STRINGS ===== */

                OpCode::PushStr => {
                    let idx = self.read_usize()?;
                    let s = self.string_pool
                        .get(idx)
                        .cloned()
                        .ok_or(VMError::InvalidStringIndex(idx))?;
                    self.push(Value::Str(s));
                }

                /* ===== DEBUG ===== */

                OpCode::DebugStack => {
                    println!("[DEBUG] Stack: {:?}", self.stack);
                }
            }
        }

        Ok(())
    }
}
