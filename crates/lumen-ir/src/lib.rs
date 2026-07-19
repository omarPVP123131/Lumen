pub mod ir;
pub mod builder;

pub use ir::{Instr, Value, Func, Program, Op};
pub use builder::IRBuilder;
