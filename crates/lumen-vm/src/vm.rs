use std::collections::HashMap;
use lumen_codegen::bytecode::{Bytecode, Instruction, Opcode, FuncMeta};
use crate::value::Value;

#[derive(Debug)]
pub enum VmError {
    Runtime(String),
    StackUnderflow,
    UndefinedVariable(String),
    UndefinedFunction(String),
    DivisionByZero,
    TypeError(String),
}

pub struct VM {
    stack: Vec<Value>,
    locals: Vec<HashMap<String, Value>>,
    ip: usize,
    bytecode: Bytecode,
    output: Vec<String>,
    call_stack: Vec<usize>,
    func_index_cache: HashMap<String, usize>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        let ip = bytecode.funcs.iter()
            .find(|f| f.name == "__main__")
            .or_else(|| bytecode.funcs.iter().find(|f| f.name == "main"))
            .or_else(|| bytecode.funcs.first())
            .map(|f| f.start)
            .unwrap_or(0);
        let mut func_index_cache = HashMap::new();
        for (i, func) in bytecode.funcs.iter().enumerate() {
            func_index_cache.insert(func.name.clone(), i);
        }
        Self {
            stack: Vec::new(),
            locals: vec![HashMap::new()],
            ip,
            bytecode,
            output: Vec::new(),
            call_stack: Vec::new(),
            func_index_cache,
        }
    }

    fn find_func(&self, name: &str) -> Option<&FuncMeta> {
        self.func_index_cache.get(name)
            .and_then(|&idx| self.bytecode.funcs.get(idx))
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        loop {
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }
            let instr = self.bytecode.instructions[self.ip].clone();
            self.ip += 1;
            self.execute(&instr)?;
        }
        Ok(())
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    fn execute(&mut self, instr: &Instruction) -> Result<(), VmError> {
        match instr {
            Instruction::Simple(op) => self.execute_simple(*op),
            Instruction::WithNum(op, n) => self.execute_with_num(*op, *n),
            Instruction::WithStr(op, s) => self.execute_with_str(*op, s),
            Instruction::WithBool(op, b) => self.execute_with_bool(*op, *b),
            Instruction::WithIdx(op, idx) => self.execute_with_idx(*op, *idx),
        }
    }

    fn execute_simple(&mut self, op: Opcode) -> Result<(), VmError> {
        match op {
            Opcode::Add => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a + b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(*a as f64 + b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a + *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a + b)),
                    (Value::Str(a), Value::Str(b)) => self.push(Value::Str(format!("{}{}", a, b))),
                    _ => return Err(VmError::TypeError("Add requires numbers or strings".to_string())),
                }
            }
            Opcode::Sub => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a - b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(*a as f64 - b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a - *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a - b)),
                    _ => return Err(VmError::TypeError("Sub requires numbers".to_string())),
                }
            }
            Opcode::Mul => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a * b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Float(*a as f64 * b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Float(a * *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Float(a * b)),
                    _ => return Err(VmError::TypeError("Mul requires numbers".to_string())),
                }
            }
            Opcode::Div => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(_), Value::Int(0)) => return Err(VmError::DivisionByZero),
                    (Value::Int(a), Value::Int(b)) => {
                        if a % b == 0 {
                            self.push(Value::Int(a / b))
                        } else {
                            self.push(Value::Float(*a as f64 / *b as f64))
                        }
                    }
                    (Value::Int(a), Value::Float(b)) => {
                        if *b == 0.0 { return Err(VmError::DivisionByZero); }
                        self.push(Value::Float(*a as f64 / b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if *b == 0 { return Err(VmError::DivisionByZero); }
                        self.push(Value::Float(a / *b as f64))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if *b == 0.0 { return Err(VmError::DivisionByZero); }
                        self.push(Value::Float(a / b))
                    }
                    _ => return Err(VmError::TypeError("Div requires numbers".to_string())),
                }
            }
            Opcode::Eq => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => a == b,
                    (Value::Int(a), Value::Float(b)) => (*a as f64 - b).abs() < f64::EPSILON,
                    (Value::Float(a), Value::Int(b)) => (a - *b as f64).abs() < f64::EPSILON,
                    (Value::Float(a), Value::Float(b)) => (a - b).abs() < f64::EPSILON,
                    (Value::Str(a), Value::Str(b)) => a == b,
                    (Value::Bool(a), Value::Bool(b)) => a == b,
                    (Value::Struct { name: an, fields: af }, Value::Struct { name: bn, fields: bf }) => {
                        an == bn && af == bf
                    }
                    _ => false,
                };
                self.push(Value::Bool(result));
            }
            Opcode::Neq => {
                let b = self.pop()?;
                let a = self.pop()?;
                let result = match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => a != b,
                    (Value::Int(a), Value::Float(b)) => (*a as f64 - b).abs() >= f64::EPSILON,
                    (Value::Float(a), Value::Int(b)) => (a - *b as f64).abs() >= f64::EPSILON,
                    (Value::Float(a), Value::Float(b)) => (a - b).abs() >= f64::EPSILON,
                    (Value::Str(a), Value::Str(b)) => a != b,
                    (Value::Bool(a), Value::Bool(b)) => a != b,
                    (Value::Struct { name: an, fields: af }, Value::Struct { name: bn, fields: bf }) => {
                        an != bn || af != bf
                    }
                    _ => true,
                };
                self.push(Value::Bool(result));
            }
            Opcode::Lt => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a < b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) < *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a < *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a < b)),
                    _ => return Err(VmError::TypeError("Lt requires numbers".to_string())),
                }
            }
            Opcode::Le => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a <= b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) <= *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a <= *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a <= b)),
                    _ => return Err(VmError::TypeError("Le requires numbers".to_string())),
                }
            }
            Opcode::Gt => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a > b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) > *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a > *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a > b)),
                    _ => return Err(VmError::TypeError("Gt requires numbers".to_string())),
                }
            }
            Opcode::Ge => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Bool(a >= b)),
                    (Value::Int(a), Value::Float(b)) => self.push(Value::Bool((*a as f64) >= *b)),
                    (Value::Float(a), Value::Int(b)) => self.push(Value::Bool(*a >= *b as f64)),
                    (Value::Float(a), Value::Float(b)) => self.push(Value::Bool(a >= b)),
                    _ => return Err(VmError::TypeError("Ge requires numbers".to_string())),
                }
            }
            Opcode::And => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Value::Bool(a.is_truthy() && b.is_truthy()));
            }
            Opcode::Or => {
                let b = self.pop()?;
                let a = self.pop()?;
                self.push(Value::Bool(a.is_truthy() || b.is_truthy()));
            }
            Opcode::Neg => {
                let a = self.pop()?;
                match a {
                    Value::Int(n) => self.push(Value::Int(-n)),
                    Value::Float(n) => self.push(Value::Float(-n)),
                    _ => return Err(VmError::TypeError("Neg requires number".to_string())),
                }
            }
            Opcode::Not => {
                let a = self.pop()?;
                self.push(Value::Bool(!a.is_truthy()));
            }
            Opcode::Ret => {
                let ret_val = self.pop().unwrap_or(Value::Void);
                if let Some(return_ip) = self.call_stack.pop() {
                    self.locals.pop();
                    self.ip = return_ip;
                }
                self.push(ret_val);
            }
            Opcode::Print => {
                let val = self.pop()?;
                let s = format!("{}", val);
                self.output.push(s);
            }
            Opcode::Halt => {
                self.ip = usize::MAX;
            }
            Opcode::StructNew => {
                // handled in execute_with_idx
            }
            Opcode::StructGet => {
                let field_name = self.pop()?;
                let struct_val = self.pop()?;
                let field = match &field_name {
                    Value::Str(s) => s.clone(),
                    _ => return Err(VmError::TypeError("StructGet requires string field name".to_string())),
                };
                match struct_val {
                    Value::Struct { fields, .. } => {
                        let val = fields.iter().find(|(name, _)| name == &field);
                        match val {
                            Some((_, v)) => self.push(v.clone()),
                            None => return Err(VmError::Runtime(format!("Campo '{}' no encontrado en struct", field))),
                        }
                    }
                    _ => return Err(VmError::TypeError("StructGet requires struct value".to_string())),
                }
            }
            Opcode::StructSet => {
                let new_val = self.pop()?;
                let field_name = self.pop()?;
                let struct_val = self.pop()?;
                let field = match &field_name {
                    Value::Str(s) => s.clone(),
                    _ => return Err(VmError::TypeError("StructSet requires string field name".to_string())),
                };
                match struct_val {
                    Value::Struct { name, mut fields } => {
                        let pos = fields.iter().position(|(n, _)| n == &field);
                        match pos {
                            Some(i) => {
                                fields[i] = (field, new_val);
                                self.push(Value::Struct { name, fields });
                            }
                            None => return Err(VmError::Runtime(format!("Campo '{}' no encontrado en struct", field))),
                        }
                    }
                    _ => return Err(VmError::TypeError("StructSet requires struct value".to_string())),
                }
            }
            Opcode::FuncRef => {
                // handled in execute_with_idx
            }
            Opcode::CallValue => {
                // handled in execute_with_idx
            }
            Opcode::ArrayNew => {
                // handled in execute_with_idx
            }
            Opcode::ArrayGet => {
                let index = self.pop()?;
                let array = self.pop()?;
                match (array, index) {
                    (Value::Array(arr), Value::Int(idx)) => {
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(VmError::Runtime(format!("Índice {} fuera de rango (largo: {})", idx, arr.len())));
                        }
                        self.push(arr[idx as usize].clone());
                    }
                    _ => return Err(VmError::TypeError("ArrayGet requires array and integer index".to_string())),
                }
            }
            Opcode::ArraySet => {
                let value = self.pop()?;
                let index = self.pop()?;
                let array = self.pop()?;
                match (array, index, value) {
                    (Value::Array(mut arr), Value::Int(idx), val) => {
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(VmError::Runtime(format!("Índice {} fuera de rango (largo: {})", idx, arr.len())));
                        }
                        arr[idx as usize] = val;
                        self.push(Value::Array(arr));
                    }
                    _ => return Err(VmError::TypeError("ArraySet requires array, integer index, and value".to_string())),
                }
            }
            Opcode::ArrayLen => {
                let array = self.pop()?;
                match array {
                    Value::Array(arr) => self.push(Value::Int(arr.len() as i64)),
                    _ => return Err(VmError::TypeError("ArrayLen requires array".to_string())),
                }
            }
            Opcode::ArrayPush => {
                let value = self.pop()?;
                let array = self.pop()?;
                match array {
                    Value::Array(mut arr) => {
                        arr.push(value);
                        self.push(Value::Array(arr));
                    }
                    other => {
                        self.push(other);
                        self.push(value);
                        return Err(VmError::TypeError("ArrayPush requires array as receiver".to_string()));
                    }
                }
            }
            Opcode::ResultOk => {
                let val = self.pop()?;
                self.push(Value::Exito(Box::new(val)));
            }
            Opcode::ResultErr => {
                let val = self.pop()?;
                self.push(Value::Error(Box::new(val)));
            }
            Opcode::TryUnwrap => {
                let val = self.pop()?;
                match val {
                    Value::Exito(inner) => {
                        self.push(*inner);
                    }
                    Value::Error(inner) => {
                        let err_wrapper = Value::Error(inner);
                        if let Some(return_ip) = self.call_stack.pop() {
                            self.locals.pop();
                            self.ip = return_ip;
                        }
                        self.push(err_wrapper);
                    }
                    _ => return Err(VmError::TypeError("TryUnwrap requires a result value".to_string())),
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_with_num(&mut self, op: Opcode, n: f64) -> Result<(), VmError> {
        match op {
            Opcode::PushNum => { self.push(Value::Float(n)); Ok(()) }
            _ => Ok(()),
        }
    }

    fn execute_with_str(&mut self, op: Opcode, s: &str) -> Result<(), VmError> {
        match op {
            Opcode::PushStr => { self.push(Value::Str(s.to_string())); Ok(()) }
            _ => Ok(()),
        }
    }

    fn execute_with_bool(&mut self, op: Opcode, b: bool) -> Result<(), VmError> {
        match op {
            Opcode::PushBool => { self.push(Value::Bool(b)); Ok(()) }
            _ => Ok(()),
        }
    }

    fn execute_with_idx(&mut self, op: Opcode, idx: usize) -> Result<(), VmError> {
        match op {
            Opcode::PushInt => {
                let n = self.bytecode.ints.get(idx).copied().unwrap_or(0);
                self.push(Value::Int(n));
            }
            Opcode::PushNum => {
                let n = self.bytecode.nums.get(idx).copied().unwrap_or(0.0);
                self.push(Value::Float(n));
            }
            Opcode::PushStr => {
                let s = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                self.push(Value::Str(s));
            }
            Opcode::PushBool => {
                self.push(Value::Bool(idx != 0));
            }
            Opcode::Load => {
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let val = self.lookup(&name)?;
                self.push(val);
            }
            Opcode::Store => {
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let val = self.pop()?;
                self.locals.last_mut().unwrap().insert(name, val);
            }
            Opcode::Call => {
                let name = self.bytecode.names.get(idx).cloned().unwrap_or_default();
                let argc_idx = self.ip;
                self.ip += 1;
                let argc = if argc_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, nidx) = &self.bytecode.instructions[argc_idx] {
                        self.bytecode.nums.get(*nidx).copied().unwrap_or(0.0) as usize
                    } else { 0 }
                } else { 0 };
                let mut args = Vec::new();
                for _ in 0..argc {
                    args.push(self.pop()?);
                }
                args.reverse();
                if name == "imprimir" || name == "print" {
                    for arg in args {
                        let s = format!("{}", arg);
                        self.output.push(s);
                    }
                    self.push(Value::Void);
                } else if name == "leer" || name == "read" {
                    self.push(Value::Str(String::new()));
                } else if let Some(func) = self.find_func(&name) {
                    let func_start = func.start;
                    let func_params = func.params.clone();
                    self.call_stack.push(self.ip);
                    let mut scope = HashMap::new();
                    for (i, param_name) in func_params.iter().enumerate() {
                        if let Some(arg) = args.get(i) {
                            scope.insert(param_name.clone(), arg.clone());
                        }
                    }
                    self.locals.push(scope);
                    self.ip = func_start;
                } else {
                    return Err(VmError::UndefinedFunction(name));
                }
            }
            Opcode::FuncRef => {
                let name = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                self.push(Value::Func(name));
            }
            Opcode::CallValue => {
                let argc = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let mut args = Vec::new();
                for _ in 0..argc {
                    args.push(self.pop()?);
                }
                args.reverse();
                let callee = self.pop()?;
                let name = match &callee {
                    Value::Func(n) => n.clone(),
                    _ => return Err(VmError::TypeError("Se esperaba una función para llamar".to_string())),
                };
                if name == "imprimir" || name == "print" {
                    for arg in args {
                        let s = format!("{}", arg);
                        self.output.push(s);
                    }
                    self.push(Value::Void);
                } else if name == "leer" || name == "read" {
                    self.push(Value::Str(String::new()));
                } else if let Some(func) = self.find_func(&name) {
                    let func_start = func.start;
                    let func_params = func.params.clone();
                    self.call_stack.push(self.ip);
                    let mut scope = HashMap::new();
                    for (i, param_name) in func_params.iter().enumerate() {
                        if let Some(arg) = args.get(i) {
                            scope.insert(param_name.clone(), arg.clone());
                        }
                    }
                    self.locals.push(scope);
                    self.ip = func_start;
                } else {
                    return Err(VmError::UndefinedFunction(name));
                }
            }
            Opcode::ArrayNew => {
                let n = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(Value::Array(items));
            }
            Opcode::StructNew => {
                let struct_name = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                let argc_idx = self.ip;
                self.ip += 1;
                let count = if argc_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, nidx) = &self.bytecode.instructions[argc_idx] {
                        self.bytecode.nums.get(*nidx).copied().unwrap_or(0.0) as usize
                    } else { 0 }
                } else { 0 };
                let mut field_names = Vec::with_capacity(count);
                for _ in 0..count {
                    field_names.push(self.pop()?);
                }
                field_names.reverse();
                let mut field_values = Vec::with_capacity(count);
                for _ in 0..count {
                    field_values.push(self.pop()?);
                }
                field_values.reverse();
                let fields: Vec<(String, Value)> = field_names.into_iter().zip(field_values.into_iter())
                    .map(|(name, val)| {
                        let n = match name {
                            Value::Str(s) => s,
                            _ => "?".to_string(),
                        };
                        (n, val)
                    })
                    .collect();
                self.push(Value::Struct { name: struct_name, fields });
            }
            Opcode::Jmp => {
                let target = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                self.ip = target;
            }
            Opcode::JmpIf => {
                let val = self.pop()?;
                if !val.is_truthy() {
                    let target = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                    self.ip = target;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn push(&mut self, val: Value) {
        self.stack.push(val);
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn lookup(&self, name: &str) -> Result<Value, VmError> {
        for scope in self.locals.iter().rev() {
            if let Some(val) = scope.get(name) {
                return Ok(val.clone());
            }
        }
        Err(VmError::UndefinedVariable(name.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lumen_codegen::bytecode::{Bytecode, FuncMeta, Instruction, Opcode};

    fn make_bc(instrs: Vec<Instruction>) -> Bytecode {
        Bytecode {
            instructions: instrs,
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        }
    }

    #[test]
    fn test_halt() {
        let bc = make_bc(vec![Instruction::Simple(Opcode::Halt)]);
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_push_num() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_add() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![2.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_print() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["hola".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["hola"]);
    }

    #[test]
    fn test_division_by_zero() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Div),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![1.0, 0.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_store_and_load() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::Store, 0),
                Instruction::WithIdx(Opcode::Load, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec!["x".to_string()],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["42"]);
    }

    #[test]
    fn test_comparisons() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Lt),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![1.0, 2.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_conditional_jump() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushBool, 0),
                Instruction::WithIdx(Opcode::JmpIf, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![4.0, 42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_neg() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Neg),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["-42"]);
    }

    #[test]
    fn test_not() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::Simple(Opcode::Not),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["false"]);
    }

    #[test]
    fn test_jmp() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::Jmp, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        // Should skip PushNum and Print, output is empty
        assert!(vm.output().is_empty());
    }

    #[test]
    fn test_jmpif_true_does_not_jump() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::WithIdx(Opcode::JmpIf, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![4.0, 42.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["42"]);
    }

    #[test]
    fn test_call_builtin_print() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 0),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["test".to_string()],
            ints: vec![],
            nums: vec![1.0],
            names: vec!["imprimir".to_string()],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["test"]);
    }

    #[test]
    fn test_call_builtin_read() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 0),
                Instruction::WithIdx(Opcode::Call, 1),
                Instruction::WithIdx(Opcode::Nop, 1),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![0.0, 1.0],
            names: vec!["leer".to_string(), "imprimir".to_string()],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        // leer pushes empty string, then imprimir prints it
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &[""]);
    }

    #[test]
    fn test_call_user_function() {
        let bc = Bytecode {
            instructions: vec![
                // __main__: push 3, push 4, call sum, print result, halt
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 0), // same num
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 1),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
                // sum function at offset 6: load a, load b, add, ret
                Instruction::WithIdx(Opcode::Load, 1),
                Instruction::WithIdx(Opcode::Load, 2),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Ret),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 2.0],
            names: vec!["sum".to_string(), "a".to_string(), "b".to_string()],
            funcs: vec![
                FuncMeta { name: "__main__".to_string(), params: vec![], start: 0 },
                FuncMeta { name: "sum".to_string(), params: vec!["a".to_string(), "b".to_string()], start: 6 },
            ],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["6"]);
    }

    #[test]
    fn test_call_undefined_function() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::Call, 0),
                Instruction::WithIdx(Opcode::Nop, 0),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![0.0],
            names: vec!["nonexistent".to_string()],
            funcs: vec![
                FuncMeta { name: "__main__".to_string(), params: vec![], start: 0 },
            ],
        };
        let mut vm = VM::new(bc);
        let result = vm.run();
        assert!(result.is_err());
        match result.unwrap_err() {
            VmError::UndefinedFunction(_) => {}
            other => panic!("Expected UndefinedFunction, got {:?}", other),
        }
    }

    #[test]
    fn test_ret_without_call() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Ret),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![42.0],
            names: vec![],
            funcs: vec![
                FuncMeta { name: "__main__".to_string(), params: vec![], start: 0 },
            ],
        };
        let mut vm = VM::new(bc);
        // Ret without call should just push the value and continue (no call_stack to pop)
        assert!(vm.run().is_ok());
    }

    #[test]
    fn test_arithmetic_mul() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Mul),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![6.0, 7.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["42"]);
    }

    #[test]
    fn test_arithmetic_sub() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Sub),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![10.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["7"]);
    }

    #[test]
    fn test_arithmetic_div() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Div),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![10.0, 2.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["5"]);
    }

    #[test]
    fn test_comparison_eq() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Eq),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_comparison_neq() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Neq),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_logical_and() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::Simple(Opcode::And),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_logical_or() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithBool(Opcode::PushBool, false),
                Instruction::WithBool(Opcode::PushBool, true),
                Instruction::Simple(Opcode::Or),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_type_error_on_add_str() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["x".to_string()],
            ints: vec![],
            nums: vec![1.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_type_error_on_sub_str() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::Simple(Opcode::Sub),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["x".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_type_error_on_mul_str() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::Simple(Opcode::Mul),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["x".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_err());
    }

    #[test]
    fn test_ge_comparison() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Ge),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_le_comparison() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Le),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 5.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_gt_comparison() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Gt),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![5.0, 3.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["true"]);
    }

    #[test]
    fn test_string_add_in_vm() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushStr, 1),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["hola ".to_string(), "mundo".to_string()],
            ints: vec![],
            nums: vec![],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["hola mundo"]);
    }

    #[test]
    fn test_sub_negative_result() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::WithIdx(Opcode::PushNum, 1),
                Instruction::Simple(Opcode::Sub),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec![],
            ints: vec![],
            nums: vec![3.0, 7.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["-4"]);
    }
}


