use crate::error::LexError;
use crate::token::{Pos, Span, Token, TokenKind};

pub struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    col: usize,
    errors: Vec<LexError>,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            chars: source.chars().collect(),
            pos: 0,
            line: 1,
            col: 1,
            errors: Vec::new(),
        }
    }

    pub fn tokenize(mut self) -> (Vec<Token>, Vec<LexError>) {
        let mut tokens = Vec::new();
        loop {
            self.skip_whitespace();
            if self.is_eof() {
                tokens.push(Token::new(
                    TokenKind::Eof,
                    Span::new(self.current_pos(), self.current_pos()),
                ));
                return (tokens, self.errors);
            }

            match self.advance() {
                // Single-line comment
                Some('/') if self.peek() == Some('/') => {
                    while let Some(ch) = self.peek() {
                        if ch == '\n' || ch == '\r' {
                            break;
                        }
                        self.advance();
                    }
                }
                // Block comment
                Some('/') if self.peek() == Some('*') => {
                    self.advance();
                    loop {
                        match self.advance() {
                            Some('*') if self.peek() == Some('/') => {
                                self.advance();
                                break;
                            }
                            Some(_) => {}
                            None => {
                                self.errors.push(LexError {
                                    code: "E003".to_string(),
                                    message: "Comentario de bloque sin cerrar".to_string(),
                                    pos: self.current_pos(),
                                    suggestion: "Agrega '*/' para cerrar el comentario".to_string(),
                                });
                                break;
                            }
                        }
                    }
                }
                // String literal
                Some('"') => {
                    let start_pos = self.prev_pos();
                    let mut s = String::new();
                    loop {
                        match self.advance() {
                            Some('"') => break,
                            Some('\\') => {
                                match self.advance() {
                                    Some('n') => s.push('\n'),
                                    Some('t') => s.push('\t'),
                                    Some('"') => s.push('"'),
                                    Some('\\') => s.push('\\'),
                                    Some(ch) => s.push(ch),
                                    None => {
                                        self.errors.push(LexError {
                                            code: "E004".to_string(),
                                            message: "Secuencia de escape incompleta".to_string(),
                                            pos: self.current_pos(),
                                            suggestion: "Usa \\n, \\t, \\\" o \\\\".to_string(),
                                        });
                                        break;
                                    }
                                }
                            }
                            Some(ch) => s.push(ch),
                            None => {
                                self.errors.push(LexError {
                                    code: "E002".to_string(),
                                    message: "String literal sin cerrar".to_string(),
                                    pos: start_pos,
                                    suggestion:
                                        "Agrega una comilla doble '\"' al final del texto".to_string(),
                                });
                                break;
                            }
                        }
                    }
                    tokens.push(Token::new(
                        TokenKind::StrLiteral(s),
                        Span::new(start_pos, self.prev_pos()),
                    ));
                }
                // Operators and delimiters
                Some('+') => tokens.push(self.single_token(TokenKind::Plus)),
                Some('-') => tokens.push(self.single_token(TokenKind::Minus)),
                Some('*') => tokens.push(self.single_token(TokenKind::Star)),
                Some('/') => tokens.push(self.single_token(TokenKind::Slash)),
                Some(';') => tokens.push(self.single_token(TokenKind::Semicolon)),
                Some(',') => tokens.push(self.single_token(TokenKind::Comma)),
                Some('(') => tokens.push(self.single_token(TokenKind::LeftParen)),
                Some(')') => tokens.push(self.single_token(TokenKind::RightParen)),
                Some('{') => tokens.push(self.single_token(TokenKind::LeftBrace)),
                Some('}') => tokens.push(self.single_token(TokenKind::RightBrace)),
                Some('[') => tokens.push(self.single_token(TokenKind::LeftBracket)),
                Some(']') => tokens.push(self.single_token(TokenKind::RightBracket)),
                Some('.') => tokens.push(self.single_token(TokenKind::Dot)),
                Some(':') => tokens.push(self.single_token(TokenKind::Colon)),
                Some('=') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(self.double_token(TokenKind::EqualEqual));
                    } else {
                        tokens.push(self.single_token(TokenKind::Equal));
                    }
                }
                Some('!') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(self.double_token(TokenKind::BangEqual));
                    } else {
                        tokens.push(self.single_token(TokenKind::Bang));
                    }
                }
                Some('<') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(self.double_token(TokenKind::LessEqual));
                    } else {
                        tokens.push(self.single_token(TokenKind::Less));
                    }
                }
                Some('>') => {
                    if self.peek() == Some('=') {
                        self.advance();
                        tokens.push(self.double_token(TokenKind::GreaterEqual));
                    } else {
                        tokens.push(self.single_token(TokenKind::Greater));
                    }
                }
                Some('&') => {
                    if self.peek() == Some('&') {
                        self.advance();
                        tokens.push(self.double_token(TokenKind::AndAnd));
                    } else {
                        self.errors.push(LexError {
                            code: "E005".to_string(),
                            message: "Se esperaba '&&'".to_string(),
                            pos: self.prev_pos(),
                            suggestion: "Usa '&&' para el operador lógico Y".to_string(),
                        });
                    }
                }
                Some('|') => {
                    if self.peek() == Some('|') {
                        self.advance();
                        tokens.push(self.double_token(TokenKind::OrOr));
                    } else {
                        self.errors.push(LexError {
                            code: "E005".to_string(),
                            message: "Se esperaba '||'".to_string(),
                            pos: self.prev_pos(),
                            suggestion: "Usa '||' para el operador lógico O".to_string(),
                        });
                    }
                }
                Some(ch) if ch.is_ascii_digit() => {
                    let start_pos = self.prev_pos();
                    let mut num = String::new();
                    num.push(ch);
                    let mut is_float = false;
                    while let Some(next) = self.peek() {
                        if next.is_ascii_digit() {
                            num.push(self.advance().unwrap());
                        } else if next == '.' && !is_float {
                            is_float = true;
                            num.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    tokens.push(Token::new(
                        TokenKind::NumLiteral(num),
                        Span::new(start_pos, self.prev_pos()),
                    ));
                }
                Some(ch) if is_ident_start(ch) => {
                    let start_pos = self.prev_pos();
                    let mut ident = String::new();
                    ident.push(ch);
                    while let Some(next) = self.peek() {
                        if is_ident_continue(next) {
                            ident.push(self.advance().unwrap());
                        } else {
                            break;
                        }
                    }
                    let kind = TokenKind::is_keyword(&ident).unwrap_or(TokenKind::Ident(ident));
                    tokens.push(Token::new(kind, Span::new(start_pos, self.prev_pos())));
                }
                Some(ch) => {
                    self.errors.push(LexError {
                        code: "E001".to_string(),
                        message: format!("Caracter inesperado: '{}'", ch),
                        pos: self.prev_pos(),
                        suggestion: "Revisa la ortografía del código".to_string(),
                    });
                }
                None => return (tokens, self.errors),
            }
        }
    }

    fn single_token(&self, kind: TokenKind) -> Token {
        let pos = self.prev_pos();
        Token::new(kind, Span::new(pos, pos))
    }

    fn double_token(&self, kind: TokenKind) -> Token {
        let start = self.prev_pos();
        let end = self.current_pos();
        Token::new(kind, Span::new(start, end))
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        if let Some(c) = ch {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        ch
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.chars.len()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch == ' ' || ch == '\t' || ch == '\n' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn current_pos(&self) -> Pos {
        Pos::new(self.line, self.col)
    }

    fn prev_pos(&self) -> Pos {
        let line = self.line;
        let col = if self.col > 1 { self.col - 1 } else { 1 };
        Pos::new(line, col)
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize(source: &str) -> Vec<TokenKind> {
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.tokenize();
        assert!(errors.is_empty(), "Lexer errors: {:?}", errors);
        tokens.into_iter().map(|t| t.kind).collect()
    }

    fn tokenize_with_errors(source: &str) -> (Vec<TokenKind>, Vec<LexError>) {
        let lexer = Lexer::new(source);
        let (tokens, errors) = lexer.tokenize();
        (tokens.into_iter().map(|t| t.kind).collect(), errors)
    }

    #[test]
    fn test_empty_source() {
        let kinds = tokenize("");
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_whitespace_only() {
        let kinds = tokenize("   \n  \t  ");
        assert_eq!(kinds, vec![TokenKind::Eof]);
    }

    #[test]
    fn test_keywords_spanish() {
        let kinds = tokenize("si sino mientras para funcion retornar verdadero falso numero texto booleano imprimir leer");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Si,
                TokenKind::Sino,
                TokenKind::Mientras,
                TokenKind::Para,
                TokenKind::Funcion,
                TokenKind::Retornar,
                TokenKind::Verdadero,
                TokenKind::Falso,
                TokenKind::Numero,
                TokenKind::Texto,
                TokenKind::Booleano,
                TokenKind::Imprimir,
                TokenKind::Leer,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_keywords_english() {
        let kinds = tokenize("if else while for function return true false number string boolean print read");
        assert_eq!(
            kinds,
            vec![
                TokenKind::If,
                TokenKind::Else,
                TokenKind::While,
                TokenKind::For,
                TokenKind::Function,
                TokenKind::Return,
                TokenKind::True,
                TokenKind::False,
                TokenKind::Number,
                TokenKind::String,
                TokenKind::Boolean,
                TokenKind::Print,
                TokenKind::Read,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_identifiers() {
        let kinds = tokenize("x _miVar variable123 _");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Ident("x".to_string()),
                TokenKind::Ident("_miVar".to_string()),
                TokenKind::Ident("variable123".to_string()),
                TokenKind::Ident("_".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_numeric_literals() {
        let kinds = tokenize("42 3.14 0 100.0");
        assert_eq!(
            kinds,
            vec![
                TokenKind::NumLiteral("42".to_string()),
                TokenKind::NumLiteral("3.14".to_string()),
                TokenKind::NumLiteral("0".to_string()),
                TokenKind::NumLiteral("100.0".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_string_literals() {
        let kinds = tokenize(r#""hola" "mundo" "" "#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::StrLiteral("hola".to_string()),
                TokenKind::StrLiteral("mundo".to_string()),
                TokenKind::StrLiteral("".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_string_escapes() {
        let kinds = tokenize(r#""line1\nline2" "tab\there" "quote\"" "back\\slash""#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::StrLiteral("line1\nline2".to_string()),
                TokenKind::StrLiteral("tab\there".to_string()),
                TokenKind::StrLiteral("quote\"".to_string()),
                TokenKind::StrLiteral("back\\slash".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_operators() {
        let kinds = tokenize("+ - * / = == != < <= > >= && || !");
        assert_eq!(
            kinds,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Star,
                TokenKind::Slash,
                TokenKind::Equal,
                TokenKind::EqualEqual,
                TokenKind::BangEqual,
                TokenKind::Less,
                TokenKind::LessEqual,
                TokenKind::Greater,
                TokenKind::GreaterEqual,
                TokenKind::AndAnd,
                TokenKind::OrOr,
                TokenKind::Bang,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_delimiters() {
        let kinds = tokenize("( ) { } ; ,");
        assert_eq!(
            kinds,
            vec![
                TokenKind::LeftParen,
                TokenKind::RightParen,
                TokenKind::LeftBrace,
                TokenKind::RightBrace,
                TokenKind::Semicolon,
                TokenKind::Comma,
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_line_comment() {
        let kinds = tokenize("42 // esto es un comentario\n 100");
        assert_eq!(
            kinds,
            vec![
                TokenKind::NumLiteral("42".to_string()),
                TokenKind::NumLiteral("100".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_block_comment() {
        let kinds = tokenize("42 /* bloque */ 100");
        assert_eq!(
            kinds,
            vec![
                TokenKind::NumLiteral("42".to_string()),
                TokenKind::NumLiteral("100".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_block_comment_multiline() {
        let kinds = tokenize("1 /* linea1\n   linea2 */ 2");
        assert_eq!(
            kinds,
            vec![
                TokenKind::NumLiteral("1".to_string()),
                TokenKind::NumLiteral("2".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_unterminated_string() {
        let (_, errors) = tokenize_with_errors(r#" "hola"#);
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E002");
    }

    #[test]
    fn test_unterminated_block_comment() {
        let (_, errors) = tokenize_with_errors("/* comentario sin cerrar");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E003");
    }

    #[test]
    fn test_invalid_character() {
        let (_, errors) = tokenize_with_errors("@");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E001");
    }

    #[test]
    fn test_invalid_and() {
        let (_, errors) = tokenize_with_errors("&");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E005");
    }

    #[test]
    fn test_invalid_or() {
        let (_, errors) = tokenize_with_errors("|");
        assert!(!errors.is_empty());
        assert_eq!(errors[0].code, "E005");
    }

    #[test]
    fn test_positions() {
        let lexer = Lexer::new("numero x = 42\nmientras (x > 0) {\n    x = x - 1\n}");
        let (tokens, errors) = lexer.tokenize();
        assert!(errors.is_empty());
        assert_eq!(tokens[0].pos().line, 1);
        assert_eq!(tokens[0].pos().col, 1);
        assert_eq!(tokens[3].pos().line, 1);
        let mientras_token = &tokens[4];
        assert_eq!(mientras_token.pos().line, 2);
        assert_eq!(mientras_token.pos().col, 1);
    }

    #[test]
    fn test_complex_program() {
        let source = r#"funcion numero suma(numero a, numero b) {
    retornar a + b
}
numero resultado = suma(3, 7)
imprimir(resultado)"#;
        let kinds = tokenize(source);
        assert_eq!(kinds[0], TokenKind::Funcion);
        assert_eq!(kinds[1], TokenKind::Numero);
        assert_eq!(kinds[2], TokenKind::Ident("suma".to_string()));
        assert_eq!(kinds[3], TokenKind::LeftParen);
        assert_eq!(kinds[4], TokenKind::Numero);
        assert_eq!(kinds[5], TokenKind::Ident("a".to_string()));
        assert_eq!(kinds[6], TokenKind::Comma);
        assert_eq!(kinds[7], TokenKind::Numero);
        assert_eq!(kinds[8], TokenKind::Ident("b".to_string()));
        assert_eq!(kinds[9], TokenKind::RightParen);
        assert_eq!(kinds[10], TokenKind::LeftBrace);
        assert_eq!(kinds[11], TokenKind::Retornar);
        assert_eq!(kinds[12], TokenKind::Ident("a".to_string()));
        assert_eq!(kinds[13], TokenKind::Plus);
        assert_eq!(kinds[14], TokenKind::Ident("b".to_string()));
        assert_eq!(kinds[15], TokenKind::RightBrace);
        assert_eq!(kinds[16], TokenKind::Numero);
        assert_eq!(kinds[17], TokenKind::Ident("resultado".to_string()));
        assert_eq!(kinds[18], TokenKind::Equal);
        assert_eq!(kinds[19], TokenKind::Ident("suma".to_string()));
        assert_eq!(kinds[20], TokenKind::LeftParen);
        assert_eq!(kinds[21], TokenKind::NumLiteral("3".to_string()));
        assert_eq!(kinds[22], TokenKind::Comma);
        assert_eq!(kinds[23], TokenKind::NumLiteral("7".to_string()));
        assert_eq!(kinds[24], TokenKind::RightParen);
        assert_eq!(kinds[25], TokenKind::Imprimir);
        assert_eq!(kinds[26], TokenKind::LeftParen);
        assert_eq!(kinds[27], TokenKind::Ident("resultado".to_string()));
        assert_eq!(kinds[28], TokenKind::RightParen);
        assert_eq!(kinds[29], TokenKind::Eof);
    }

    #[test]
    fn test_mixed_indentation() {
        let source = "  \t  numero x\n\t\tx = 5  ";
        let kinds = tokenize(source);
        assert_eq!(
            kinds,
            vec![
                TokenKind::Numero,
                TokenKind::Ident("x".to_string()),
                TokenKind::Ident("x".to_string()),
                TokenKind::Equal,
                TokenKind::NumLiteral("5".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_multiple_errors() {
        let (_, errors) = tokenize_with_errors("@ # $");
        assert_eq!(errors.len(), 3);
        assert!(errors.iter().all(|e| e.code == "E001"));
    }

    #[test]
    fn test_string_with_unicode() {
        let kinds = tokenize(r#""¡Hola, LÚMEN!""#);
        assert_eq!(
            kinds,
            vec![
                TokenKind::StrLiteral("¡Hola, LÚMEN!".to_string()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn test_incomplete_escape() {
        let (_, errors) = tokenize_with_errors(r#""hola\"#);
        assert!(!errors.is_empty());
    }
}
