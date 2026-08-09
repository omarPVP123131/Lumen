#![cfg(feature = "full")]
// Coroutine system — lightweight cooperative multitasking
// No ASM needed — VM-level context switching (save/restore stack + locals)

pub struct Coroutine {
    pub ip: usize,
    pub stack: Vec<crate::value::Value>,
    pub locals: Vec<std::collections::HashMap<String, crate::value::Value, crate::value::FixHasher>>,
    pub fn_name: String,
    pub is_done: bool,
}

impl Coroutine {
    pub fn new(fn_name: &str, ip: usize) -> Self {
        Coroutine {
            ip,
            stack: Vec::new(),
            locals: Vec::new(),
            fn_name: fn_name.to_string(),
            is_done: false,
        }
    }
}
