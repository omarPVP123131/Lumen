pub mod builder;
pub mod comptime;
pub mod ir;
pub mod optimize;

pub use builder::IRBuilder;
pub use ir::{Func, Instr, Op, Program, Value};
