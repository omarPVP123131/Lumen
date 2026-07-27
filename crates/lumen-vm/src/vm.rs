use crate::value::Value;
use lumen_codegen::bytecode::{Bytecode, FuncMeta, Instruction, Opcode};
use std::collections::HashMap;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone)]
pub struct CallFrame {
    pub func_name: String,
    pub return_ip: usize,
}

#[derive(Debug)]
pub enum VmError {
    Runtime(String),
    StackUnderflow,
    UndefinedVariable(String),
    UndefinedFunction(String),
    DivisionByZero,
    TypeError(String),
}

impl VmError {
    pub fn with_stack(self, stack: &[CallFrame]) -> String {
        let msg = match &self {
            VmError::Runtime(s) => format!("Error: {}", s),
            VmError::StackUnderflow => "Error: Stack underflow".to_string(),
            VmError::UndefinedVariable(s) => format!("Error: Variable '{}' no definida", s),
            VmError::UndefinedFunction(s) => format!("Error: Función '{}' no definida", s),
            VmError::DivisionByZero => "Error: División por cero".to_string(),
            VmError::TypeError(s) => format!("Error de tipo: {}", s),
        };
        if stack.is_empty() {
            msg
        } else {
            let trace: Vec<String> = stack
                .iter()
                .map(|f| format!("  · {}", f.func_name))
                .collect();
            format!("{}\n\nPila de llamadas:\n{}", msg, trace.join("\n"))
        }
    }
}

pub struct VM {
    stack: Vec<Value>,
    locals: Vec<HashMap<String, Value>>,
    ip: usize,
    bytecode: Bytecode,
    output: Vec<String>,
    call_stack: Vec<CallFrame>,
    func_index_cache: HashMap<String, usize>,
    pub debug: bool,
    pub breakpoints: Vec<usize>,
    step_mode: bool,
    last_instr: Option<Instruction>,
    pub instr_count: usize,
    tcp_listener: Option<std::net::TcpListener>,
}

impl VM {
    pub fn new(bytecode: Bytecode) -> Self {
        let ip = bytecode
            .funcs
            .iter()
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
            debug: false,
            breakpoints: Vec::new(),
            step_mode: false,
            last_instr: None,
            instr_count: 0,
            tcp_listener: None,
        }
    }

    fn find_func(&self, name: &str) -> Option<&FuncMeta> {
        self.func_index_cache
            .get(name)
            .and_then(|&idx| self.bytecode.funcs.get(idx))
    }

    pub fn run(&mut self) -> Result<(), VmError> {
        loop {
            if self.ip >= self.bytecode.instructions.len() {
                break;
            }
            let instr = self.bytecode.instructions[self.ip].clone();
            self.ip += 1;
            self.instr_count += 1;
            self.last_instr = Some(instr.clone());
            if self.debug && (self.breakpoints.contains(&self.ip) || self.step_mode) {
                println!(
                    "\n[DEBUG] ip={} instr={:?} stack={} vars={}",
                    self.ip,
                    instr,
                    self.stack.len(),
                    self.locals.last().map_or(0, |l| l.len())
                );
                if self.step_mode {
                    let mut input = String::new();
                    std::io::stdin().read_line(&mut input).ok();
                    match input.trim() {
                        "c" | "continue" => self.step_mode = false,
                        "q" | "quit" => return Ok(()),
                        _ => {}
                    }
                }
            }
            self.execute(&instr)?;
        }
        Ok(())
    }

    pub fn set_breakpoint(&mut self, ip: usize) {
        self.breakpoints.push(ip);
    }
    pub fn step(&mut self) -> Result<(), VmError> {
        self.step_mode = true;
        self.debug = true;
        if self.ip >= self.bytecode.instructions.len() {
            return Ok(());
        }
        let instr = self.bytecode.instructions[self.ip].clone();
        self.ip += 1;
        self.instr_count += 1;
        self.last_instr = Some(instr.clone());
        self.execute(&instr)
    }
    pub fn stack_top(&self) -> Option<&Value> {
        self.stack.last()
    }
    pub fn current_locals(&self) -> Option<&HashMap<String, Value>> {
        self.locals.last()
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn call_stack(&self) -> &[CallFrame] {
        &self.call_stack
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
                    (Value::Str(a), Value::Int(b)) => self.push(Value::Str(format!("{}{}", a, b))),
                    (Value::Str(a), Value::Float(b)) => {
                        self.push(Value::Str(format!("{}{}", a, b)))
                    }
                    (Value::Int(a), Value::Str(b)) => self.push(Value::Str(format!("{}{}", a, b))),
                    (Value::Float(a), Value::Str(b)) => {
                        self.push(Value::Str(format!("{}{}", a, b)))
                    }
                    (Value::Str(a), Value::Bool(b)) => self.push(Value::Str(format!("{}{}", a, b))),
                    (Value::Bool(a), Value::Str(b)) => self.push(Value::Str(format!("{}{}", a, b))),
                    _ => {
                        return Err(VmError::TypeError(
                            "Add requires numbers or strings".to_string(),
                        ))
                    }
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
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a / b)),
                    (Value::Int(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(*a as f64 / b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if *b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a / *b as f64))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a / b))
                    }
                    _ => return Err(VmError::TypeError("Div requires numbers".to_string())),
                }
            }
            Opcode::Mod => {
                let b = self.pop()?;
                let a = self.pop()?;
                match (&a, &b) {
                    (Value::Int(_), Value::Int(0)) => return Err(VmError::DivisionByZero),
                    (Value::Int(a), Value::Int(b)) => self.push(Value::Int(a.rem_euclid(*b))),
                    (Value::Int(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(*a as f64 % b))
                    }
                    (Value::Float(a), Value::Int(b)) => {
                        if *b == 0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a % *b as f64))
                    }
                    (Value::Float(a), Value::Float(b)) => {
                        if *b == 0.0 {
                            return Err(VmError::DivisionByZero);
                        }
                        self.push(Value::Float(a % b))
                    }
                    _ => return Err(VmError::TypeError("Mod requires numbers".to_string())),
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
                    (
                        Value::Struct {
                            name: an,
                            fields: af,
                        },
                        Value::Struct {
                            name: bn,
                            fields: bf,
                        },
                    ) => an == bn && af == bf,
                    (Value::Opcion(a), Value::Opcion(b)) => a == b,
                    (
                        Value::Enum {
                            name: an,
                            variant: av,
                            fields: af,
                        },
                        Value::Enum {
                            name: bn,
                            variant: bv,
                            fields: bf,
                        },
                    ) => an == bn && av == bv && af == bf,
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
                    (
                        Value::Struct {
                            name: an,
                            fields: af,
                        },
                        Value::Struct {
                            name: bn,
                            fields: bf,
                        },
                    ) => an != bn || af != bf,
                    (Value::Opcion(a), Value::Opcion(b)) => a != b,
                    (
                        Value::Enum {
                            name: an,
                            variant: av,
                            fields: af,
                        },
                        Value::Enum {
                            name: bn,
                            variant: bv,
                            fields: bf,
                        },
                    ) => an != bn || av != bv || af != bf,
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
                if let Some(frame) = self.call_stack.pop() {
                    self.locals.pop();
                    self.ip = frame.return_ip;
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
                    _ => {
                        return Err(VmError::TypeError(
                            "StructGet requires string field name".to_string(),
                        ))
                    }
                };
                match struct_val {
                    Value::Struct { fields, .. } => {
                        let val = fields.iter().find(|(name, _)| name == &field);
                        match val {
                            Some((_, v)) => self.push(v.clone()),
                            None => {
                                return Err(VmError::Runtime(format!(
                                    "Campo '{}' no encontrado en struct",
                                    field
                                )))
                            }
                        }
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "StructGet requires struct value".to_string(),
                        ))
                    }
                }
            }
            Opcode::StructSet => {
                let new_val = self.pop()?;
                let field_name = self.pop()?;
                let struct_val = self.pop()?;
                let field = match &field_name {
                    Value::Str(s) => s.clone(),
                    _ => {
                        return Err(VmError::TypeError(
                            "StructSet requires string field name".to_string(),
                        ))
                    }
                };
                match struct_val {
                    Value::Struct { name, mut fields } => {
                        let pos = fields.iter().position(|(n, _)| n == &field);
                        match pos {
                            Some(i) => {
                                fields[i] = (field, new_val);
                                self.push(Value::Struct { name, fields });
                            }
                            None => {
                                return Err(VmError::Runtime(format!(
                                    "Campo '{}' no encontrado en struct",
                                    field
                                )))
                            }
                        }
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "StructSet requires struct value".to_string(),
                        ))
                    }
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
                let container = self.pop()?;
                match container {
                    Value::Array(arr) => {
                        let idx = match &index {
                            Value::Int(i) => *i,
                            _ => {
                                return Err(VmError::TypeError(
                                    "ArrayGet requires integer index for arrays".to_string(),
                                ))
                            }
                        };
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango (largo: {})",
                                idx,
                                arr.len()
                            )));
                        }
                        self.push(arr[idx as usize].clone());
                    }
                    Value::Str(s) => {
                        let idx = match &index {
                            Value::Int(i) => *i,
                            _ => {
                                return Err(VmError::TypeError(
                                    "ArrayGet requires integer index for strings".to_string(),
                                ))
                            }
                        };
                        let chars: Vec<char> = s.chars().collect();
                        if idx < 0 || idx as usize >= chars.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango (largo: {})",
                                idx,
                                chars.len()
                            )));
                        }
                        self.push(Value::Str(chars[idx as usize].to_string()));
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "ArrayGet requires array or string".to_string(),
                        ))
                    }
                }
            }
            Opcode::ArraySet => {
                let value = self.pop()?;
                let index = self.pop()?;
                let array = self.pop()?;
                match (array, index, value) {
                    (Value::Array(mut arr), Value::Int(idx), val) => {
                        if idx < 0 || idx as usize >= arr.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango (largo: {})",
                                idx,
                                arr.len()
                            )));
                        }
                        arr[idx as usize] = val;
                        self.push(Value::Array(arr));
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "ArraySet requires array, integer index, and value".to_string(),
                        ))
                    }
                }
            }
            Opcode::ArrayLen => {
                let array = self.pop()?;
                match array {
                    Value::Array(arr) => self.push(Value::Int(arr.len() as i64)),
                    Value::Str(s) => self.push(Value::Int(s.chars().count() as i64)),
                    _ => {
                        return Err(VmError::TypeError(
                            "ArrayLen requires array or string".to_string(),
                        ))
                    }
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
                        return Err(VmError::TypeError(
                            "ArrayPush requires array as receiver".to_string(),
                        ));
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
                        if let Some(frame) = self.call_stack.pop() {
                            self.locals.pop();
                            self.ip = frame.return_ip;
                        }
                        self.push(err_wrapper);
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "TryUnwrap requires a result value".to_string(),
                        ))
                    }
                }
            }
            Opcode::OptionSome => {
                let val = self.pop()?;
                self.push(Value::Opcion(Some(Box::new(val))));
            }
            Opcode::OptionNone => {
                self.push(Value::Opcion(None));
            }
            Opcode::TupleNew => {
                // handled in execute_with_idx
            }
            Opcode::TupleAccess => {
                // handled in execute_with_idx
            }
            Opcode::EnumCtor => {
                // handled in execute_with_idx
            }
            _ => {}
        }
        Ok(())
    }

    fn execute_with_num(&mut self, op: Opcode, n: f64) -> Result<(), VmError> {
        match op {
            Opcode::PushNum => {
                self.push(Value::Float(n));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn execute_with_str(&mut self, op: Opcode, s: &str) -> Result<(), VmError> {
        match op {
            Opcode::PushStr => {
                self.push(Value::Str(s.to_string()));
                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn execute_with_bool(&mut self, op: Opcode, b: bool) -> Result<(), VmError> {
        match op {
            Opcode::PushBool => {
                self.push(Value::Bool(b));
                Ok(())
            }
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
                    } else {
                        0
                    }
                } else {
                    0
                };
                let mut args: Vec<Value> = Vec::new();
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
                } else if name == "a_texto" || name == "to_texto" || name == "__str_from" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s));
                } else if name == "largo" || name == "len" {
                    match args.into_iter().next() {
                        Some(Value::Array(v)) => self.push(Value::Int(v.len() as i64)),
                        Some(Value::Str(s)) => self.push(Value::Int(s.chars().count() as i64)),
                        Some(other) => {
                            return Err(VmError::TypeError(format!(
                                "'largo' espera lista o texto, no {:?}",
                                other
                            )))
                        }
                        None => {
                            return Err(VmError::TypeError(
                                "'largo' espera 1 argumento".to_string(),
                            ))
                        }
                    }
                } else if name == "agregar" || name == "push" {
                    let mut iter = args.into_iter();
                    let list = iter.next().unwrap_or(Value::Array(vec![]));
                    let item = iter.next().unwrap_or(Value::Void);
                    match list {
                        Value::Array(mut v) => {
                            v.push(item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "'agregar' espera una lista".to_string(),
                            ))
                        }
                    }
                } else if name == "__str_len" || name == "__str_longitud" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Int(s.len() as i64));
                } else if name == "__str_upper" || name == "__str_mayusculas" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s.to_uppercase()));
                } else if name == "__str_lower" || name == "__str_minusculas" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s.to_lowercase()));
                } else if name == "__str_trim" || name == "__str_recortar" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s.trim().to_string()));
                } else if name == "__str_contains" || name == "__str_contiene" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let sub = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Bool(s.contains(&sub)));
                } else if name == "__str_split" || name == "__str_dividir" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let delim = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let parts: Vec<Value> = if delim.is_empty() {
                        s.chars().map(|c| Value::Str(c.to_string())).collect()
                    } else {
                        s.split(&delim).map(|p| Value::Str(p.to_string())).collect()
                    };
                    self.push(Value::Array(parts));
                } else if name == "__file_read" || name == "__leer_archivo" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::read_to_string(&path) {
                        Ok(content) => self.push(Value::Exito(Box::new(Value::Str(content)))),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__file_write" || name == "__escribir_archivo" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::write(&path, &content) {
                        Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__file_exists" || name == "__existe_archivo" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Bool(std::path::Path::new(&path).exists()));
                } else if name == "__time_now" || name == "__tiempo_ahora" {
                    use std::time::{SystemTime, UNIX_EPOCH};
                    let now = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap_or_default();
                    self.push(Value::Str(format!("{}", now.as_secs())));
                } else if name == "__list_reverse" || name == "__lista_invertir" {
                    let mut arr = match args.into_iter().next() {
                        Some(Value::Array(v)) => v,
                        Some(other) => {
                            return Err(VmError::TypeError(format!(
                                "__list_reverse espera una lista, no {:?}",
                                other
                            )))
                        }
                        None => {
                            return Err(VmError::TypeError(
                                "__list_reverse espera 1 argumento".to_string(),
                            ))
                        }
                    };
                    arr.reverse();
                    self.push(Value::Array(arr));
                } else if name == "__list_sort" || name == "__lista_ordenar" {
                    let mut arr = match args.into_iter().next() {
                        Some(Value::Array(v)) => v,
                        Some(other) => {
                            return Err(VmError::TypeError(format!(
                                "__list_sort espera una lista, no {:?}",
                                other
                            )))
                        }
                        None => {
                            return Err(VmError::TypeError(
                                "__list_sort espera 1 argumento".to_string(),
                            ))
                        }
                    };
                    arr.sort_by(|a, b| {
                        let an = a.as_num().unwrap_or(f64::MAX);
                        let bn = b.as_num().unwrap_or(f64::MAX);
                        an.partial_cmp(&bn).unwrap_or(std::cmp::Ordering::Equal)
                    });
                    self.push(Value::Array(arr));
                } else if name == "__map_new" || name == "__map_nuevo" {
                    self.push(Value::Map(vec![]));
                } else if name == "__map_set" || name == "__map_poner" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(vec![]));
                    let k = it.next().unwrap_or(Value::Void);
                    let v = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(mut p) => {
                            if let Some(pos) = p.iter().position(|(pk, _)| *pk == k) {
                                p[pos] = (k, v);
                            } else {
                                p.push((k, v));
                            }
                            self.push(Value::Map(p));
                        }
                        _ => return Err(VmError::TypeError("__map_set espera diccionario".into())),
                    }
                } else if name == "__map_get" || name == "__map_obtener" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(vec![]));
                    let k = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(p) => {
                            if let Some((_, v)) = p.iter().find(|(pk, _)| *pk == k) {
                                self.push(v.clone());
                            } else {
                                self.push(Value::Void);
                            }
                        }
                        _ => return Err(VmError::TypeError("__map_get espera diccionario".into())),
                    }
                } else if name == "__map_len" || name == "__map_longitud" {
                    let m = args.into_iter().next().unwrap_or(Value::Map(vec![]));
                    match m {
                        Value::Map(p) => self.push(Value::Int(p.len() as i64)),
                        _ => return Err(VmError::TypeError("__map_len espera diccionario".into())),
                    }
                } else if name == "__map_keys" || name == "__map_claves" {
                    let m = args.into_iter().next().unwrap_or(Value::Map(vec![]));
                    match m {
                        Value::Map(p) => {
                            self.push(Value::Array(p.into_iter().map(|(k, _)| k).collect()))
                        }
                        _ => {
                            return Err(VmError::TypeError("__map_keys espera diccionario".into()))
                        }
                    }
                } else if name == "__map_contains" || name == "__map_contiene" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(vec![]));
                    let k = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(p) => self.push(Value::Bool(p.iter().any(|(pk, _)| *pk == k))),
                        _ => {
                            return Err(VmError::TypeError(
                                "__map_contains espera diccionario".into(),
                            ))
                        }
                    }
                } else if name == "__set_new" || name == "__conjunto_nuevo" {
                    self.push(Value::Map(vec![]));
                } else if name == "__set_add" || name == "__conjunto_agregar" {
                    let mut it = args.into_iter();
                    let s = it.next().unwrap_or(Value::Map(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match s {
                        Value::Map(mut p) => {
                            if !p.iter().any(|(k, _)| *k == item) {
                                p.push((item, Value::Bool(true)));
                            }
                            self.push(Value::Map(p));
                        }
                        _ => return Err(VmError::TypeError("__set_add espera conjunto".into())),
                    }
                } else if name == "__set_has" || name == "__conjunto_tiene" {
                    let mut it = args.into_iter();
                    let s = it.next().unwrap_or(Value::Map(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match s {
                        Value::Map(p) => self.push(Value::Bool(p.iter().any(|(k, _)| *k == item))),
                        _ => return Err(VmError::TypeError("__set_has espera conjunto".into())),
                    }
                } else if name == "__set_union" || name == "__conjunto_unir" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(vec![]));
                    let b = it.next().unwrap_or(Value::Map(vec![]));
                    match (a, b) {
                        (Value::Map(p1), Value::Map(p2)) => {
                            let mut m = p1;
                            for (k, v) in p2 {
                                if !m.iter().any(|(mk, _)| *mk == k) {
                                    m.push((k, v));
                                }
                            }
                            self.push(Value::Map(m));
                        }
                        _ => return Err(VmError::TypeError("__set_union espera conjuntos".into())),
                    }
                } else if name == "__set_inter" || name == "__conjunto_interseccion" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(vec![]));
                    let b = it.next().unwrap_or(Value::Map(vec![]));
                    match (a, b) {
                        (Value::Map(p1), Value::Map(p2)) => {
                            let r: Vec<_> = p1
                                .into_iter()
                                .filter(|(k, _)| p2.iter().any(|(k2, _)| *k2 == *k))
                                .collect();
                            self.push(Value::Map(r));
                        }
                        _ => return Err(VmError::TypeError("__set_inter espera conjuntos".into())),
                    }
                } else if name == "__set_diff" || name == "__conjunto_diferencia" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(vec![]));
                    let b = it.next().unwrap_or(Value::Map(vec![]));
                    match (a, b) {
                        (Value::Map(p1), Value::Map(p2)) => {
                            let r: Vec<_> = p1
                                .into_iter()
                                .filter(|(k, _)| !p2.iter().any(|(k2, _)| *k2 == *k))
                                .collect();
                            self.push(Value::Map(r));
                        }
                        _ => return Err(VmError::TypeError("__set_diff espera conjuntos".into())),
                    }
                } else if name == "__deque_new" || name == "__deque_nuevo" {
                    self.push(Value::Array(vec![]));
                } else if name == "__deque_push_front" || name == "__deque_agregar_frente" {
                    let mut it = args.into_iter();
                    let d = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match d {
                        Value::Array(mut v) => {
                            v.insert(0, item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__deque_push_front espera deque".into(),
                            ))
                        }
                    }
                } else if name == "__deque_push_back" || name == "__deque_agregar_final" {
                    let mut it = args.into_iter();
                    let d = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match d {
                        Value::Array(mut v) => {
                            v.push(item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError("__deque_push_back espera deque".into()))
                        }
                    }
                } else if name == "__deque_pop_front" || name == "__deque_quitar_frente" {
                    let d = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match d {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.remove(0)
                        }),
                        _ => {
                            return Err(VmError::TypeError("__deque_pop_front espera deque".into()))
                        }
                    }
                } else if name == "__deque_pop_back" || name == "__deque_quitar_final" {
                    let d = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match d {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.pop().unwrap_or(Value::Void)
                        }),
                        _ => {
                            return Err(VmError::TypeError("__deque_pop_back espera deque".into()))
                        }
                    }
                } else if name == "__deque_len" || name == "__deque_longitud" {
                    let d = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match d {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__deque_len espera deque".into())),
                    }
                } else if name == "__heap_new" || name == "__monticulo_nuevo" {
                    self.push(Value::Array(vec![]));
                } else if name == "__heap_push" || name == "__monticulo_agregar" {
                    let mut it = args.into_iter();
                    let h = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match h {
                        Value::Array(mut v) => {
                            v.push(item);
                            v.sort_by(|a, b| {
                                let an = a.as_num().unwrap_or(f64::MIN);
                                let bn = b.as_num().unwrap_or(f64::MIN);
                                bn.partial_cmp(&an).unwrap_or(std::cmp::Ordering::Equal)
                            });
                            self.push(Value::Array(v));
                        }
                        _ => return Err(VmError::TypeError("__heap_push espera heap".into())),
                    }
                } else if name == "__heap_pop" || name == "__monticulo_quitar" {
                    let h = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match h {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.remove(0)
                        }),
                        _ => return Err(VmError::TypeError("__heap_pop espera heap".into())),
                    }
                } else if name == "__heap_peek" || name == "__monticulo_ver" {
                    let h = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match h {
                        Value::Array(v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v[0].clone()
                        }),
                        _ => return Err(VmError::TypeError("__heap_peek espera heap".into())),
                    }
                } else if name == "__heap_len" || name == "__monticulo_longitud" {
                    let h = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match h {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__heap_len espera heap".into())),
                    }
                } else if name == "__linked_new" || name == "__enlazada_nuevo" {
                    self.push(Value::Array(vec![]));
                } else if name == "__linked_push_front" || name == "__enlazada_agregar_frente" {
                    let mut it = args.into_iter();
                    let l = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match l {
                        Value::Array(mut v) => {
                            v.insert(0, item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_push_front espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_push_back" || name == "__enlazada_agregar_final" {
                    let mut it = args.into_iter();
                    let l = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match l {
                        Value::Array(mut v) => {
                            v.push(item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_push_back espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_pop_front" || name == "__enlazada_quitar_frente" {
                    let l = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match l {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.remove(0)
                        }),
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_pop_front espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_pop_back" || name == "__enlazada_quitar_final" {
                    let l = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match l {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.pop().unwrap_or(Value::Void)
                        }),
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_pop_back espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_len" || name == "__enlazada_longitud" {
                    let l = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match l {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__linked_len espera linked".into())),
                    }
                } else if name == "__regex_new" || name == "__regex_nuevo" {
                    let pat = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&pat) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__regex_is_match" || name == "__regex_coincide" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => self.push(Value::Bool(r.is_match(&text))),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__regex_captures" || name == "__regex_capturar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => {
                            if let Some(caps) = r.captures(&text) {
                                let vs: Vec<Value> = caps
                                    .iter()
                                    .map(|m| {
                                        Value::Str(
                                            m.map(|x| x.as_str().to_string()).unwrap_or_default(),
                                        )
                                    })
                                    .collect();
                                self.push(Value::Array(vs));
                            } else {
                                self.push(Value::Array(vec![]));
                            }
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__regex_replace" || name == "__regex_reemplazar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let rep = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => {
                            self.push(Value::Str(r.replace_all(&text, rep.as_str()).to_string()))
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__unicode_normalize" || name == "__unicode_normalizar" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let form = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let nf: String = match form.as_str() {
                        "NFC" => s.nfc().collect(),
                        "NFD" => s.nfd().collect(),
                        "NFKC" => s.nfkc().collect(),
                        "NFKD" => s.nfkd().collect(),
                        _ => s.nfc().collect(),
                    };
                    self.push(Value::Str(nf));
                } else if name == "__str_pad_start" || name == "__str_padding_inicio" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
                    let ch = args
                        .get(2)
                        .map(|v| format!("{}", v))
                        .unwrap_or_default()
                        .chars()
                        .next()
                        .unwrap_or(' ');
                    self.push(Value::Str(format!(
                        "{}{}",
                        ch.to_string().repeat(len.saturating_sub(s.len())),
                        s
                    )));
                } else if name == "__str_pad_end" || name == "__str_padding_fin" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
                    let ch = args
                        .get(2)
                        .map(|v| format!("{}", v))
                        .unwrap_or_default()
                        .chars()
                        .next()
                        .unwrap_or(' ');
                    self.push(Value::Str(format!(
                        "{}{}",
                        s,
                        ch.to_string().repeat(len.saturating_sub(s.len()))
                    )));
                } else if name == "__encoding_utf8" || name == "__codificacion_utf8" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Array(
                        s.bytes().map(|b| Value::Int(b as i64)).collect(),
                    ));
                } else if name == "__encoding_from_utf8" || name == "__desde_utf8" {
                    let arr = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match arr {
                        Value::Array(v) => {
                            let bytes: Vec<u8> = v
                                .iter()
                                .filter_map(|x| {
                                    if let Value::Int(n) = x {
                                        Some(*n as u8)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            self.push(Value::Str(String::from_utf8_lossy(&bytes).to_string()));
                        }
                        _ => self.push(Value::Str(String::new())),
                    }
                } else if name == "__buf_reader" || name == "__lector_buffer" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::read_to_string(&path) {
                        Ok(c) => {
                            let lines: Vec<Value> =
                                c.lines().map(|l| Value::Str(l.to_string())).collect();
                            self.push(Value::Array(lines));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__buf_writer" || name == "__escritor_buffer" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::write(&path, &content) {
                        Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__stream_chunks" || name == "__stream_trozos" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let size = args.get(1).and_then(|v| v.as_num()).unwrap_or(4096.0) as usize;
                    match std::fs::read(&path) {
                        Ok(data) => {
                            let chunks: Vec<Value> = data
                                .chunks(size)
                                .map(|c| {
                                    Value::Array(c.iter().map(|&b| Value::Int(b as i64)).collect())
                                })
                                .collect();
                            self.push(Value::Array(chunks));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__tcp_connect" || name == "__tcp_conectar" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::net::TcpStream::connect(&addr) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__tcp_listen" || name == "__tcp_escuchar" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::net::TcpListener::bind(&addr) {
                        Ok(l) => {
                            self.tcp_listener = Some(l);
                            self.push(Value::Bool(true));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__tcp_accept" || name == "__tcp_aceptar" {
                    match &self.tcp_listener {
                        Some(l) => match l.accept() {
                            Ok((_stream, _)) => {
                                let addr = _stream
                                    .peer_addr()
                                    .map(|a| a.to_string())
                                    .unwrap_or_default();
                                self.push(Value::Str(addr));
                            }
                            Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                        },
                        None => {
                            self.push(Value::Error(Box::new(Value::Str("Sin listener".into()))))
                        }
                    }
                } else if name == "__http_get" || name == "__http_obtener" {
                    let url = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match reqwest::blocking::get(&url) {
                        Ok(resp) => {
                            let body = resp.text().unwrap_or_default();
                            self.push(Value::Str(body));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__http_post" || name == "__http_enviar" {
                    let url = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let body = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match reqwest::blocking::Client::new()
                        .post(&url)
                        .body(body)
                        .send()
                    {
                        Ok(resp) => {
                            let text = resp.text().unwrap_or_default();
                            self.push(Value::Str(text));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__http_server" || name == "__http_servidor" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(format!("HTTP server on {}", addr)));
                } else if name == "__serial_open" || name == "__serial_abrir" {
                    self.push(Value::Bool(true));
                } else if name == "__json_parse" || name == "__json_parsear" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match serde_json::from_str::<serde_json::Value>(&s) {
                        Ok(v) => self.push(json_value_to_lumen(v)),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__json_stringify" || name == "__json_texto" {
                    let val = args.first().cloned().unwrap_or(Value::Void);
                    let json = lumen_value_to_json(&val);
                    self.push(Value::Str(serde_json::to_string(&json).unwrap_or_default()));
                } else if let Some(func) = self.find_func(&name) {
                    let func_start = func.start;
                    let func_params = func.params.clone();
                    self.call_stack.push(CallFrame {
                        func_name: name.clone(),
                        return_ip: self.ip,
                    });
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
                    _ => {
                        return Err(VmError::TypeError(
                            "Se esperaba una función para llamar".to_string(),
                        ))
                    }
                };
                if name == "imprimir" || name == "print" {
                    for arg in args {
                        let s = format!("{}", arg);
                        self.output.push(s);
                    }
                    self.push(Value::Void);
                } else if name == "leer" || name == "read" {
                    self.push(Value::Str(String::new()));
                } else if name == "a_texto" || name == "to_texto" || name == "__str_from" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s));
                } else if name == "__str_len" || name == "__str_longitud" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Int(s.len() as i64));
                } else if name == "__str_upper" || name == "__str_mayusculas" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s.to_uppercase()));
                } else if name == "__str_lower" || name == "__str_minusculas" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s.to_lowercase()));
                } else if name == "__str_trim" || name == "__str_recortar" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(s.trim().to_string()));
                } else if name == "__str_contains" || name == "__str_contiene" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let sub = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Bool(s.contains(&sub)));
                } else if name == "__str_split" || name == "__str_dividir" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let delim = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let parts: Vec<Value> = if delim.is_empty() {
                        s.chars().map(|c| Value::Str(c.to_string())).collect()
                    } else {
                        s.split(&delim).map(|p| Value::Str(p.to_string())).collect()
                    };
                    self.push(Value::Array(parts));
                } else if name == "__map_new" || name == "__map_nuevo" {
                    self.push(Value::Map(vec![]));
                } else if name == "__map_set" || name == "__map_poner" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(vec![]));
                    let k = it.next().unwrap_or(Value::Void);
                    let v = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(mut p) => {
                            if let Some(pos) = p.iter().position(|(pk, _)| *pk == k) {
                                p[pos] = (k, v);
                            } else {
                                p.push((k, v));
                            }
                            self.push(Value::Map(p));
                        }
                        _ => return Err(VmError::TypeError("__map_set espera diccionario".into())),
                    }
                } else if name == "__map_get" || name == "__map_obtener" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(vec![]));
                    let k = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(p) => {
                            if let Some((_, v)) = p.iter().find(|(pk, _)| *pk == k) {
                                self.push(v.clone());
                            } else {
                                self.push(Value::Void);
                            }
                        }
                        _ => return Err(VmError::TypeError("__map_get espera diccionario".into())),
                    }
                } else if name == "__map_len" || name == "__map_longitud" {
                    let m = args.into_iter().next().unwrap_or(Value::Map(vec![]));
                    match m {
                        Value::Map(p) => self.push(Value::Int(p.len() as i64)),
                        _ => return Err(VmError::TypeError("__map_len espera diccionario".into())),
                    }
                } else if name == "__map_keys" || name == "__map_claves" {
                    let m = args.into_iter().next().unwrap_or(Value::Map(vec![]));
                    match m {
                        Value::Map(p) => {
                            self.push(Value::Array(p.into_iter().map(|(k, _)| k).collect()))
                        }
                        _ => {
                            return Err(VmError::TypeError("__map_keys espera diccionario".into()))
                        }
                    }
                } else if name == "__map_contains" || name == "__map_contiene" {
                    let mut it = args.into_iter();
                    let m = it.next().unwrap_or(Value::Map(vec![]));
                    let k = it.next().unwrap_or(Value::Void);
                    match m {
                        Value::Map(p) => self.push(Value::Bool(p.iter().any(|(pk, _)| *pk == k))),
                        _ => {
                            return Err(VmError::TypeError(
                                "__map_contains espera diccionario".into(),
                            ))
                        }
                    }
                } else if name == "__set_new" || name == "__conjunto_nuevo" {
                    self.push(Value::Map(vec![]));
                } else if name == "__set_add" || name == "__conjunto_agregar" {
                    let mut it = args.into_iter();
                    let s = it.next().unwrap_or(Value::Map(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match s {
                        Value::Map(mut p) => {
                            if !p.iter().any(|(k, _)| *k == item) {
                                p.push((item, Value::Bool(true)));
                            }
                            self.push(Value::Map(p));
                        }
                        _ => return Err(VmError::TypeError("__set_add espera conjunto".into())),
                    }
                } else if name == "__set_has" || name == "__conjunto_tiene" {
                    let mut it = args.into_iter();
                    let s = it.next().unwrap_or(Value::Map(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match s {
                        Value::Map(p) => self.push(Value::Bool(p.iter().any(|(k, _)| *k == item))),
                        _ => return Err(VmError::TypeError("__set_has espera conjunto".into())),
                    }
                } else if name == "__set_union" || name == "__conjunto_unir" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(vec![]));
                    let b = it.next().unwrap_or(Value::Map(vec![]));
                    match (a, b) {
                        (Value::Map(p1), Value::Map(p2)) => {
                            let mut m = p1;
                            for (k, v) in p2 {
                                if !m.iter().any(|(mk, _)| *mk == k) {
                                    m.push((k, v));
                                }
                            }
                            self.push(Value::Map(m));
                        }
                        _ => return Err(VmError::TypeError("__set_union espera conjuntos".into())),
                    }
                } else if name == "__set_inter" || name == "__conjunto_interseccion" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(vec![]));
                    let b = it.next().unwrap_or(Value::Map(vec![]));
                    match (a, b) {
                        (Value::Map(p1), Value::Map(p2)) => {
                            let r: Vec<_> = p1
                                .into_iter()
                                .filter(|(k, _)| p2.iter().any(|(k2, _)| *k2 == *k))
                                .collect();
                            self.push(Value::Map(r));
                        }
                        _ => return Err(VmError::TypeError("__set_inter espera conjuntos".into())),
                    }
                } else if name == "__set_diff" || name == "__conjunto_diferencia" {
                    let mut it = args.into_iter();
                    let a = it.next().unwrap_or(Value::Map(vec![]));
                    let b = it.next().unwrap_or(Value::Map(vec![]));
                    match (a, b) {
                        (Value::Map(p1), Value::Map(p2)) => {
                            let r: Vec<_> = p1
                                .into_iter()
                                .filter(|(k, _)| !p2.iter().any(|(k2, _)| *k2 == *k))
                                .collect();
                            self.push(Value::Map(r));
                        }
                        _ => return Err(VmError::TypeError("__set_diff espera conjuntos".into())),
                    }
                } else if name == "__deque_new" || name == "__deque_nuevo" {
                    self.push(Value::Array(vec![]));
                } else if name == "__deque_push_front" || name == "__deque_agregar_frente" {
                    let mut it = args.into_iter();
                    let d = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match d {
                        Value::Array(mut v) => {
                            v.insert(0, item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__deque_push_front espera deque".into(),
                            ))
                        }
                    }
                } else if name == "__deque_push_back" || name == "__deque_agregar_final" {
                    let mut it = args.into_iter();
                    let d = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match d {
                        Value::Array(mut v) => {
                            v.push(item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError("__deque_push_back espera deque".into()))
                        }
                    }
                } else if name == "__deque_pop_front" || name == "__deque_quitar_frente" {
                    let d = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match d {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.remove(0)
                        }),
                        _ => {
                            return Err(VmError::TypeError("__deque_pop_front espera deque".into()))
                        }
                    }
                } else if name == "__deque_pop_back" || name == "__deque_quitar_final" {
                    let d = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match d {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.pop().unwrap_or(Value::Void)
                        }),
                        _ => {
                            return Err(VmError::TypeError("__deque_pop_back espera deque".into()))
                        }
                    }
                } else if name == "__deque_len" || name == "__deque_longitud" {
                    let d = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match d {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__deque_len espera deque".into())),
                    }
                } else if name == "__heap_new" || name == "__monticulo_nuevo" {
                    self.push(Value::Array(vec![]));
                } else if name == "__heap_push" || name == "__monticulo_agregar" {
                    let mut it = args.into_iter();
                    let h = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match h {
                        Value::Array(mut v) => {
                            v.push(item);
                            v.sort_by(|a, b| {
                                let an = a.as_num().unwrap_or(f64::MIN);
                                let bn = b.as_num().unwrap_or(f64::MIN);
                                bn.partial_cmp(&an).unwrap_or(std::cmp::Ordering::Equal)
                            });
                            self.push(Value::Array(v));
                        }
                        _ => return Err(VmError::TypeError("__heap_push espera heap".into())),
                    }
                } else if name == "__heap_pop" || name == "__monticulo_quitar" {
                    let h = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match h {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.remove(0)
                        }),
                        _ => return Err(VmError::TypeError("__heap_pop espera heap".into())),
                    }
                } else if name == "__heap_peek" || name == "__monticulo_ver" {
                    let h = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match h {
                        Value::Array(v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v[0].clone()
                        }),
                        _ => return Err(VmError::TypeError("__heap_peek espera heap".into())),
                    }
                } else if name == "__heap_len" || name == "__monticulo_longitud" {
                    let h = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match h {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__heap_len espera heap".into())),
                    }
                } else if name == "__linked_new" || name == "__enlazada_nuevo" {
                    self.push(Value::Array(vec![]));
                } else if name == "__linked_push_front" || name == "__enlazada_agregar_frente" {
                    let mut it = args.into_iter();
                    let l = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match l {
                        Value::Array(mut v) => {
                            v.insert(0, item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_push_front espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_push_back" || name == "__enlazada_agregar_final" {
                    let mut it = args.into_iter();
                    let l = it.next().unwrap_or(Value::Array(vec![]));
                    let item = it.next().unwrap_or(Value::Void);
                    match l {
                        Value::Array(mut v) => {
                            v.push(item);
                            self.push(Value::Array(v));
                        }
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_push_back espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_pop_front" || name == "__enlazada_quitar_frente" {
                    let l = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match l {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.remove(0)
                        }),
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_pop_front espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_pop_back" || name == "__enlazada_quitar_final" {
                    let l = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match l {
                        Value::Array(mut v) => self.push(if v.is_empty() {
                            Value::Void
                        } else {
                            v.pop().unwrap_or(Value::Void)
                        }),
                        _ => {
                            return Err(VmError::TypeError(
                                "__linked_pop_back espera linked".into(),
                            ))
                        }
                    }
                } else if name == "__linked_len" || name == "__enlazada_longitud" {
                    let l = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match l {
                        Value::Array(v) => self.push(Value::Int(v.len() as i64)),
                        _ => return Err(VmError::TypeError("__linked_len espera linked".into())),
                    }
                } else if name == "__regex_new" || name == "__regex_nuevo" {
                    let pat = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&pat) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__regex_is_match" || name == "__regex_coincide" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => self.push(Value::Bool(r.is_match(&text))),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__regex_captures" || name == "__regex_capturar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => {
                            if let Some(caps) = r.captures(&text) {
                                let vs: Vec<Value> = caps
                                    .iter()
                                    .map(|m| {
                                        Value::Str(
                                            m.map(|x| x.as_str().to_string()).unwrap_or_default(),
                                        )
                                    })
                                    .collect();
                                self.push(Value::Array(vs));
                            } else {
                                self.push(Value::Array(vec![]));
                            }
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__regex_replace" || name == "__regex_reemplazar" {
                    let re_s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let text = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let rep = args.get(2).map(|v| format!("{}", v)).unwrap_or_default();
                    match regex::Regex::new(&re_s) {
                        Ok(r) => {
                            self.push(Value::Str(r.replace_all(&text, rep.as_str()).to_string()))
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__unicode_normalize" || name == "__unicode_normalizar" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let form = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    let nf: String = match form.as_str() {
                        "NFC" => s.nfc().collect(),
                        "NFD" => s.nfd().collect(),
                        "NFKC" => s.nfkc().collect(),
                        "NFKD" => s.nfkd().collect(),
                        _ => s.nfc().collect(),
                    };
                    self.push(Value::Str(nf));
                } else if name == "__str_pad_start" || name == "__str_padding_inicio" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
                    let ch = args
                        .get(2)
                        .map(|v| format!("{}", v))
                        .unwrap_or_default()
                        .chars()
                        .next()
                        .unwrap_or(' ');
                    self.push(Value::Str(format!(
                        "{}{}",
                        ch.to_string().repeat(len.saturating_sub(s.len())),
                        s
                    )));
                } else if name == "__str_pad_end" || name == "__str_padding_fin" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let len = args.get(1).and_then(|v| v.as_num()).unwrap_or(0.0) as usize;
                    let ch = args
                        .get(2)
                        .map(|v| format!("{}", v))
                        .unwrap_or_default()
                        .chars()
                        .next()
                        .unwrap_or(' ');
                    self.push(Value::Str(format!(
                        "{}{}",
                        s,
                        ch.to_string().repeat(len.saturating_sub(s.len()))
                    )));
                } else if name == "__encoding_utf8" || name == "__codificacion_utf8" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Array(
                        s.bytes().map(|b| Value::Int(b as i64)).collect(),
                    ));
                } else if name == "__encoding_from_utf8" || name == "__desde_utf8" {
                    let arr = args.into_iter().next().unwrap_or(Value::Array(vec![]));
                    match arr {
                        Value::Array(v) => {
                            let bytes: Vec<u8> = v
                                .iter()
                                .filter_map(|x| {
                                    if let Value::Int(n) = x {
                                        Some(*n as u8)
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            self.push(Value::Str(String::from_utf8_lossy(&bytes).to_string()));
                        }
                        _ => self.push(Value::Str(String::new())),
                    }
                } else if name == "__buf_reader" || name == "__lector_buffer" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::read_to_string(&path) {
                        Ok(c) => {
                            let lines: Vec<Value> =
                                c.lines().map(|l| Value::Str(l.to_string())).collect();
                            self.push(Value::Array(lines));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__buf_writer" || name == "__escritor_buffer" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let content = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match std::fs::write(&path, &content) {
                        Ok(_) => self.push(Value::Exito(Box::new(Value::Bool(true)))),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__stream_chunks" || name == "__stream_trozos" {
                    let path = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let size = args.get(1).and_then(|v| v.as_num()).unwrap_or(4096.0) as usize;
                    match std::fs::read(&path) {
                        Ok(data) => {
                            let chunks: Vec<Value> = data
                                .chunks(size)
                                .map(|c| {
                                    Value::Array(c.iter().map(|&b| Value::Int(b as i64)).collect())
                                })
                                .collect();
                            self.push(Value::Array(chunks));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__tcp_connect" || name == "__tcp_conectar" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::net::TcpStream::connect(&addr) {
                        Ok(_) => self.push(Value::Bool(true)),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__tcp_listen" || name == "__tcp_escuchar" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match std::net::TcpListener::bind(&addr) {
                        Ok(l) => {
                            self.tcp_listener = Some(l);
                            self.push(Value::Bool(true));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__tcp_accept" || name == "__tcp_aceptar" {
                    match &self.tcp_listener {
                        Some(l) => match l.accept() {
                            Ok((_stream, _)) => {
                                let addr = _stream
                                    .peer_addr()
                                    .map(|a| a.to_string())
                                    .unwrap_or_default();
                                self.push(Value::Str(addr));
                            }
                            Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                        },
                        None => {
                            self.push(Value::Error(Box::new(Value::Str("Sin listener".into()))))
                        }
                    }
                } else if name == "__http_get" || name == "__http_obtener" {
                    let url = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match reqwest::blocking::get(&url) {
                        Ok(resp) => {
                            let body = resp.text().unwrap_or_default();
                            self.push(Value::Str(body));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__http_post" || name == "__http_enviar" {
                    let url = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    let body = args.get(1).map(|v| format!("{}", v)).unwrap_or_default();
                    match reqwest::blocking::Client::new()
                        .post(&url)
                        .body(body)
                        .send()
                    {
                        Ok(resp) => {
                            let text = resp.text().unwrap_or_default();
                            self.push(Value::Str(text));
                        }
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__http_server" || name == "__http_servidor" {
                    let addr = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    self.push(Value::Str(format!("HTTP server on {}", addr)));
                } else if name == "__serial_open" || name == "__serial_abrir" {
                    self.push(Value::Bool(true));
                } else if name == "__json_parse" || name == "__json_parsear" {
                    let s = args.first().map(|v| format!("{}", v)).unwrap_or_default();
                    match serde_json::from_str::<serde_json::Value>(&s) {
                        Ok(v) => self.push(json_value_to_lumen(v)),
                        Err(e) => self.push(Value::Error(Box::new(Value::Str(e.to_string())))),
                    }
                } else if name == "__json_stringify" || name == "__json_texto" {
                    let val = args.first().cloned().unwrap_or(Value::Void);
                    let json = lumen_value_to_json(&val);
                    self.push(Value::Str(serde_json::to_string(&json).unwrap_or_default()));
                } else if let Some(func) = self.find_func(&name) {
                    let func_start = func.start;
                    let func_params = func.params.clone();
                    self.call_stack.push(CallFrame {
                        func_name: name.clone(),
                        return_ip: self.ip,
                    });
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
                    } else {
                        0
                    }
                } else {
                    0
                };
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
                let fields: Vec<(String, Value)> = field_names
                    .into_iter()
                    .zip(field_values)
                    .map(|(name, val)| {
                        let n = match name {
                            Value::Str(s) => s,
                            _ => "?".to_string(),
                        };
                        (n, val)
                    })
                    .collect();
                self.push(Value::Struct {
                    name: struct_name,
                    fields,
                });
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
            Opcode::TupleNew => {
                let n = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let mut items = Vec::with_capacity(n);
                for _ in 0..n {
                    items.push(self.pop()?);
                }
                items.reverse();
                self.push(Value::Tuple(items));
            }
            Opcode::TupleAccess => {
                let index = self.bytecode.nums.get(idx).copied().unwrap_or(0.0) as usize;
                let tuple_val = self.pop()?;
                match tuple_val {
                    Value::Tuple(items) => {
                        if index >= items.len() {
                            return Err(VmError::Runtime(format!(
                                "Índice {} fuera de rango para tupla de {} elementos",
                                index,
                                items.len()
                            )));
                        }
                        self.push(items[index].clone());
                    }
                    _ => {
                        return Err(VmError::TypeError(
                            "TupleAccess requires a tuple value".to_string(),
                        ));
                    }
                }
            }
            Opcode::EnumCtor => {
                let enum_name = self.bytecode.strings.get(idx).cloned().unwrap_or_default();
                let var_idx = self.ip;
                self.ip += 1;
                let variant = if var_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, vidx) = &self.bytecode.instructions[var_idx] {
                        self.bytecode
                            .strings
                            .get(*vidx)
                            .cloned()
                            .unwrap_or_default()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let argc_idx = self.ip;
                self.ip += 1;
                let argc = if argc_idx < self.bytecode.instructions.len() {
                    if let Instruction::WithIdx(_, nidx) = &self.bytecode.instructions[argc_idx] {
                        self.bytecode.nums.get(*nidx).copied().unwrap_or(0.0) as usize
                    } else {
                        0
                    }
                } else {
                    0
                };
                let mut fields = Vec::with_capacity(argc);
                for _ in 0..argc {
                    fields.push(self.pop()?);
                }
                fields.reverse();
                self.push(Value::Enum {
                    name: enum_name,
                    variant,
                    fields,
                });
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

fn json_value_to_lumen(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::Void,
        serde_json::Value::Bool(b) => Value::Bool(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::Int(i)
            } else {
                Value::Float(n.as_f64().unwrap_or(0.0))
            }
        }
        serde_json::Value::String(s) => Value::Str(s),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.into_iter().map(json_value_to_lumen).collect())
        }
        serde_json::Value::Object(map) => {
            let pairs: Vec<(Value, Value)> = map
                .into_iter()
                .map(|(k, v)| (Value::Str(k), json_value_to_lumen(v)))
                .collect();
            Value::Map(pairs)
        }
    }
}

fn lumen_value_to_json(v: &Value) -> serde_json::Value {
    match v {
        Value::Int(n) => serde_json::json!(*n),
        Value::Float(n) => serde_json::json!(*n),
        Value::Str(s) => serde_json::json!(s),
        Value::Bool(b) => serde_json::json!(*b),
        Value::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(lumen_value_to_json).collect())
        }
        Value::Map(pairs) => {
            let mut map = serde_json::Map::new();
            for (k, v) in pairs {
                if let Value::Str(key) = k {
                    map.insert(key.clone(), lumen_value_to_json(v));
                }
            }
            serde_json::Value::Object(map)
        }
        _ => serde_json::Value::Null,
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
                FuncMeta {
                    name: "__main__".to_string(),
                    params: vec![],
                    start: 0,
                },
                FuncMeta {
                    name: "sum".to_string(),
                    params: vec!["a".to_string(), "b".to_string()],
                    start: 6,
                },
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
            funcs: vec![FuncMeta {
                name: "__main__".to_string(),
                params: vec![],
                start: 0,
            }],
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
            funcs: vec![FuncMeta {
                name: "__main__".to_string(),
                params: vec![],
                start: 0,
            }],
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
    fn test_add_str_num_concatenates() {
        let bc = Bytecode {
            instructions: vec![
                Instruction::WithIdx(Opcode::PushStr, 0),
                Instruction::WithIdx(Opcode::PushNum, 0),
                Instruction::Simple(Opcode::Add),
                Instruction::Simple(Opcode::Print),
                Instruction::Simple(Opcode::Halt),
            ],
            strings: vec!["x".to_string()],
            ints: vec![],
            nums: vec![1.0],
            names: vec![],
            funcs: vec![],
        };
        let mut vm = VM::new(bc);
        assert!(vm.run().is_ok());
        assert_eq!(vm.output(), &["x1"]);
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
