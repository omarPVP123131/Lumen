use std::sync::LazyLock;

use crate::instructions::OpCode;
use crate::vm::{
    decoder::Decoder,
    verifier::verify,
    value::*,
    constant_pool::ConstantPool,
};

type Handler = fn(&mut VM);

pub struct VM<'a> {
    dec: Decoder<'a>,
    stack: Vec<Value>,
    memory: Vec<Value>,
    pool: ConstantPool,
    running: bool,
}

/* ================= DISPATCH ================= */

static DISPATCH: LazyLock<[Handler; 256]> = LazyLock::new(build_dispatch);

fn build_dispatch() -> [Handler; 256] {
    let mut t = [unhandled as Handler; 256];

    t[OpCode::PushNum as usize] = push_num;

    t[OpCode::Add as usize] = add;
    t[OpCode::Sub as usize] = sub;
    t[OpCode::Mul as usize] = mul;
    t[OpCode::Div as usize] = div;

    t[OpCode::Eq as usize] = eq;
    t[OpCode::Lt as usize] = lt;
    t[OpCode::Gt as usize] = gt;

    t[OpCode::Load as usize]  = load;
    t[OpCode::Store as usize] = store;

    t[OpCode::Jmp as usize]        = jmp;
    t[OpCode::JmpIfFalse as usize] = jmp_if_false;

    t[OpCode::Print as usize] = print;
    t[OpCode::DebugStack as usize] = debug_stack;

    t[OpCode::Halt as usize] = halt;

    t
}

/* ================= VM ================= */

impl<'a> VM<'a> {
    pub fn new(code: &'a [u8], pool: ConstantPool) -> Result<Self, String> {
        verify(code)?;

        Ok(Self {
            dec: Decoder::new(code),
            stack: Vec::with_capacity(256),
            memory: vec![int(0); 256],
            pool,
            running: true,
        })
    }

    pub fn run(&mut self) {
        while self.running {
            let op = self.dec.read_opcode() as u8;
            DISPATCH[op as usize](self);
        }
    }
}

/* ================= OPCODES ================= */

fn unhandled(_: &mut VM) {
    panic!("opcode no manejado");
}

/* ---------- stack ---------- */

fn push_num(vm: &mut VM) {
    let id = vm.dec.read_u32();
    let v = vm.pool.get_int(id);
    vm.stack.push(int(v));
}

/* ---------- aritmética ---------- */

fn add(vm: &mut VM) {
    let b = as_int(vm.stack.pop().unwrap());
    let a = as_int(vm.stack.pop().unwrap());
    vm.stack.push(int(a + b));
}

fn sub(vm: &mut VM) {
    let b = as_int(vm.stack.pop().unwrap());
    let a = as_int(vm.stack.pop().unwrap());
    vm.stack.push(int(a - b));
}

fn mul(vm: &mut VM) {
    let b = as_int(vm.stack.pop().unwrap());
    let a = as_int(vm.stack.pop().unwrap());
    vm.stack.push(int(a * b));
}

fn div(vm: &mut VM) {
    let b = as_int(vm.stack.pop().unwrap());
    let a = as_int(vm.stack.pop().unwrap());
    vm.stack.push(int(a / b));
}

/* ---------- comparaciones ---------- */

fn eq(vm: &mut VM) {
    let b = as_int(vm.stack.pop().unwrap());
    let a = as_int(vm.stack.pop().unwrap());
    vm.stack.push(boolv(a == b));
}

fn lt(vm: &mut VM) {
    let b = as_int(vm.stack.pop().unwrap());
    let a = as_int(vm.stack.pop().unwrap());
    vm.stack.push(boolv(a < b));
}

fn gt(vm: &mut VM) {
    let b = as_int(vm.stack.pop().unwrap());
    let a = as_int(vm.stack.pop().unwrap());
    vm.stack.push(boolv(a > b));
}

/* ---------- memoria ---------- */

fn store(vm: &mut VM) {
    let addr = vm.dec.read_u32() as usize;
    let v = vm.stack.pop().unwrap();
    vm.memory[addr] = v;
}

fn load(vm: &mut VM) {
    let addr = vm.dec.read_u32() as usize;
    vm.stack.push(vm.memory[addr]);
}

/* ---------- saltos ---------- */

fn jmp(vm: &mut VM) {
    let target = vm.dec.read_u32() as usize;
    vm.dec.jump(target);
}

fn jmp_if_false(vm: &mut VM) {
    let target = vm.dec.read_u32() as usize;
    let cond = as_bool(vm.stack.pop().unwrap());
    if !cond {
        vm.dec.jump(target);
    }
}

/* ---------- IO / debug ---------- */

fn print(vm: &mut VM) {
    let v = vm.stack.pop().unwrap();

    if is_bool(v) {
        println!("{}", as_bool(v));
    } else {
        println!("{}", as_int(v));
    }
}

fn debug_stack(vm: &mut VM) {
    println!("STACK:");
    for v in vm.stack.iter().rev() {
        if is_bool(*v) {
            println!("  bool {}", as_bool(*v));
        } else {
            println!("  int {}", as_int(*v));
        }
    }
}

/* ---------- control ---------- */

fn halt(vm: &mut VM) {
    vm.running = false;
}
