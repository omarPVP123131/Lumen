use crate::token::Pos;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LexError {
    pub code: String,
    pub message: String,
    pub pos: Pos,
    pub suggestion: String,
}

pub type LexResult<T> = Result<T, Vec<LexError>>;
