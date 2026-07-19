pub mod sema;
pub mod error;
pub mod loader;

pub use sema::{SemanticAnalyzer, TypeInfo};
pub use error::SemError;
pub use loader::{ModuleLoader, ModuleError};
