// src/compiler/lexer.rs
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literales
    Number(i32),
    String(String),
    Identifier(String),

    // Palabras clave (abarcando ES/EN)
    Numero,      // numero | number
    Imprimir,    // imprimir | print
    Si,          // si | if
    Sino,        // sino | else
    Mientras,    // mientras | while
    Import,      // import

    // Operadores
    Plus,        // +
    Minus,       // -
    Star,        // *
    Slash,       // /

    // Comparadores
    Equal,       // ==
    Less,        // <
    Greater,     // >

    // Asignación
    Assign,      // =

    // Delimitadores
    LParen,      // (
    RParen,      // )
    LBrace,      // {
    RBrace,      // }

    // Control
    Newline,
    Eof,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "Number({})", n),
            Token::String(s) => write!(f, "String(\"{}\")", s),
            Token::Identifier(s) => write!(f, "Identifier({})", s),
            Token::Numero => write!(f, "numero"),
            Token::Imprimir => write!(f, "imprimir"),
            Token::Si => write!(f, "si"),
            Token::Sino => write!(f, "sino"),
            Token::Mientras => write!(f, "mientras"),
            Token::Import => write!(f, "import"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Equal => write!(f, "=="),
            Token::Less => write!(f, "<"),
            Token::Greater => write!(f, ">"),
            Token::Assign => write!(f, "="),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Newline => write!(f, "\\n"),
            Token::Eof => write!(f, "EOF"),
        }
    }
}

/// Token con info de posición (línea/columna)
#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub token: Token,
    pub line: usize,
    pub col: usize,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    current_char: Option<char>,
    pub line: usize,
    pub col: usize,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        let chars: Vec<char> = input.chars().collect();
        let current_char = chars.get(0).copied();
        Lexer {
            input: chars,
            position: 0,
            current_char,
            line: 1,
            col: 1,
        }
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char {
            if ch == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
        self.position += 1;
        self.current_char = self.input.get(self.position).copied();
    }

    fn peek(&self, offset: usize) -> Option<char> {
        self.input.get(self.position + offset).copied()
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn read_number(&mut self) -> i32 {
        let mut num_str = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_ascii_digit() {
                num_str.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        num_str.parse().unwrap_or(0)
    }

    fn read_identifier(&mut self) -> String {
        let mut id = String::new();
        while let Some(ch) = self.current_char {
            if ch.is_alphanumeric() || ch == '_' {
                id.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        id
    }

    fn read_string(&mut self) -> Result<String, String> {
        // assume current_char == '"' or '\''
        let quote = self.current_char.ok_or("Unexpected EOF in string")?;
        self.advance(); // consume opening quote
        let mut s = String::new();
        while let Some(ch) = self.current_char {
            if ch == quote {
                self.advance(); // consume closing quote
                return Ok(s);
            }
            // simple escape support for \"
            if ch == '\\' {
                if let Some(next) = self.peek(1) {
                    match next {
                        'n' => {
                            s.push('\n');
                            self.advance();
                            self.advance();
                            continue;
                        }
                        't' => {
                            s.push('\t');
                            self.advance();
                            self.advance();
                            continue;
                        }
                        '\\' => {
                            s.push('\\');
                            self.advance();
                            self.advance();
                            continue;
                        }
                        '"' => {
                            s.push('"');
                            self.advance();
                            self.advance();
                            continue;
                        }
                        '\'' => {
                            s.push('\'');
                            self.advance();
                            self.advance();
                            continue;
                        }
                        _ => {
                            // unknown escape: push next as-is
                            self.advance();
                            if let Some(ch2) = self.current_char {
                                s.push(ch2);
                                self.advance();
                                continue;
                            } else {
                                return Err("Unterminated escape".into());
                            }
                        }
                    }
                } else {
                    return Err("Unterminated escape".into());
                }
            }

            s.push(ch);
            self.advance();
        }
        Err("Unterminated string literal".into())
    }

    pub fn next_token(&mut self) -> TokenInfo {
        let start_line = self.line;
        let start_col = self.col;

        self.skip_whitespace();

        let tok = match self.current_char {
            None => Token::Eof,

            Some('\n') => {
                self.advance();
                Token::Newline
            }

            Some('+') => { self.advance(); Token::Plus }
            Some('-') => { self.advance(); Token::Minus }
            Some('*') => { self.advance(); Token::Star }
            Some('/') => { self.advance(); Token::Slash }
            Some('<') => { self.advance(); Token::Less }
            Some('>') => { self.advance(); Token::Greater }

            Some('=') => {
                self.advance();
                if self.current_char == Some('=') {
                    self.advance();
                    Token::Equal
                } else {
                    Token::Assign
                }
            }

            Some('(') => { self.advance(); Token::LParen }
            Some(')') => { self.advance(); Token::RParen }
            Some('{') => { self.advance(); Token::LBrace }
            Some('}') => { self.advance(); Token::RBrace }

            Some('"') | Some('\'') => {
                match self.read_string() {
                    Ok(s) => Token::String(s),
                    Err(e) => panic!("Lexer string error: {}", e),
                }
            }

            Some(ch) if ch.is_ascii_digit() => {
                Token::Number(self.read_number())
            }

            Some(ch) if ch.is_alphabetic() || ch == '_' => {
                let id = self.read_identifier();
                // Map both Spanish and English keywords to a single token set.
                match id.as_str() {
                    // number / numero
                    "numero" | "number" => Token::Numero,

                    // print / imprimir
                    "imprimir" | "print" => Token::Imprimir,

                    // if / si
                    "si" | "if" => Token::Si,

                    // else / sino
                    "sino" | "else" => Token::Sino,

                    // while / mientras
                    "mientras" | "while" => Token::Mientras,

                    // import
                    "import" => Token::Import,

                    _ => Token::Identifier(id),
                }
            }

            Some(ch) => {
                panic!("Carácter inesperado: {}", ch);
            }
        };

        TokenInfo { token: tok, line: start_line, col: start_col }
    }

    pub fn tokenize(&mut self) -> Vec<TokenInfo> {
        let mut tokens = Vec::new();
        loop {
            let ti = self.next_token();
            tokens.push(ti.clone());
            if ti.token == Token::Eof {
                break;
            }
        }
        tokens
    }
}
