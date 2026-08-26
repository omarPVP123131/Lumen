pub mod builder;
pub mod comptime;
pub mod ir;

pub use builder::IRBuilder;
pub use ir::{Func, Instr, Op, Program, Value};
