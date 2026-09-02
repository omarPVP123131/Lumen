use lumen_lexer::token::Span;

#[derive(Debug, Clone)]
pub struct ParseError {
    pub code: String,
    pub message: String,
    pub span: Span,
    pub suggestion: String,
}
