#[cfg(any(feature = "extra", feature = "full"))]
pub mod coro_ffi;
#[cfg(feature = "full")]
pub mod crypto_ffi;
#[cfg(feature = "full")]
pub mod gui_ffi;
#[cfg(feature = "aot")]
pub mod jit;
pub mod min_regex;
pub mod native_lex;
pub mod value;
pub mod vm;

pub use value::Value;
pub use vm::{CallFrame, VmError, VM};

// Puente para builtins: motor propio (v3.4.5)

pub fn lumen_min_regex_new(pat: &str) -> Result<crate::min_regex::Regex, String> {
    crate::min_regex::Regex::new(pat)
}
