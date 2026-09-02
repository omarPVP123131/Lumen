use lumen_lexer::token::Span;

#[derive(Debug, Clone)]
pub struct SemError {
    pub code: String,
    pub message: String,
    pub span: Span,
    pub suggestion: String,
}
