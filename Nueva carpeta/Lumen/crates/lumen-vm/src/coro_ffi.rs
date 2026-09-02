#![cfg(any(feature = "extra", feature = "full"))]
// Coroutine system — lightweight cooperative multitasking
// No ASM needed — VM-level context switching (save/restore stack + locals)

pub struct Coroutine {
    pub ip: usize,
    pub stack: Vec<crate::value::Value>,
    pub locals: Vec<crate::vm::ScopeFrame>,
    /// v3.5.31: la arena de valores y el freelist van con los scopes — los
    /// slots de un coroutine NO son los de otro (identidades distintas).
    pub flat: Vec<crate::value::Value>,
    pub free_slots: Vec<u32>,
    pub fn_name: String,
    pub is_done: bool,
}

impl Coroutine {
    pub fn new(fn_name: &str, ip: usize) -> Self {
        Coroutine {
            ip,
            stack: Vec::new(),
            locals: Vec::new(),
            flat: Vec::new(),
            free_slots: Vec::new(),
            fn_name: fn_name.to_string(),
            is_done: false,
        }
    }
}
