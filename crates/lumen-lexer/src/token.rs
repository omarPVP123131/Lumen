use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pos {
    pub line: usize,
    pub col: usize,
}

impl Pos {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: Pos,
    pub end: Pos,
}

impl Span {
    pub fn new(start: Pos, end: Pos) -> Self {
        Self { start, end }
    }

    pub fn merge(&self, other: &Span) -> Span {
        Span {
            start: self.start,
            end: other.end,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }

    pub fn pos(&self) -> Pos {
        self.span.start
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenKind {
    // Keywords (Spanish)
    Si,
    Sino,
    Mientras,
    Para,
    Funcion,
    Retornar,
    Tipo,
    Verdadero,
    Falso,
    Numero,
    Entero,
    Decimal,
    Texto,
    Booleano,
    Imprimir,
    Leer,
    Lista,
    Romper,
    Continuar,
    Elegir,
    Caso,
    Defecto,
    Estructura,
    Importar,
    Como,
    Resultado,
    Exito,
    ErrKeyword,
    Intentar,
    En,
    Enum,
    Opcion,
    Algun,
    Ninguno,
    Const,
    Rasgo,
    Impl,
    Async,
    Esperar,
    Sea,
    Posponer,
    Consultar,
    Donde,
    OrdenarPor,
    Seleccionar,
    Descendente,
    Ascendente,
    Atrapar,
    Prestado,
    Dueno,
    Mut,
    EnTiempoCompilacion,
    Ensamblador,
    BloqueC,
    BloqueRust,
    Puro,
    GrupoTareas,

    // English equivalents
    If,
    Else,
    While,
    For,
    Function,
    Return,
    True,
    False,
    Number,
    Integer,
    Float,
    String,
    Boolean,
    Print,
    Read,
    Array,
    Break,
    Continue,
    Match,
    Case,
    Default,
    Struct,
    Import,
    As,
    Result,
    Ok,
    Err,
    Try,
    In,
    Option,
    Some,
    None,
    Trait,
    Await,
    Let,
    Defer,
    Query,
    Where,
    OrderBy,
    Select,
    Descending,
    Ascending,
    Catch,
    Borrowed,
    Owner,
    Mutable,
    Comptime,
    Asm,
    CBlock,
    RustBlock,
    Pure,
    TaskGroup,

    // Identifiers & Literals
    Ident(String),
    NumLiteral(String),
    StrLiteral(String),
    FStrLiteral(String),

    // Operators
    Plus,         // +
    PlusPlus,     // ++
    Minus,        // -
    Star,         // *
    Slash,        // /
    Percent,      // %
    Equal,        // =
    EqualEqual,   // ==
    Bang,         // !
    BangEqual,    // !=
    Less,         // <
    LessEqual,    // <=
    Greater,      // >
    GreaterEqual, // >=
    AndAnd,       // &&
    OrOr,         // ||
    Pipe,         // |
    PipeGreater,  // |>
    Ampersand,    // &
    Caret,        // ^
    Tilde,        // ~
    ShiftLeft,    // <<
    ShiftRight,   // >>
    Question,     // ?
    QuestionDot,  // ?.
    QuestionColon,// ?:

    // Range operators
    DotDot,      // ..
    DotDotEqual, // ..=

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBrace,    // {
    RightBrace,   // }
    LeftBracket,  // [
    RightBracket, // ]
    Semicolon,    // ;
    Comma,        // ,
    Dot,          // .
    Colon,        // :
    DoubleColon,  // ::

    // Special
    Comment(String),
    Error(String),
    Eof,
}

impl TokenKind {
    pub fn is_keyword(s: &str) -> Option<TokenKind> {
        match s {
            "si" => Some(TokenKind::Si),
            "sino" => Some(TokenKind::Sino),
            "mientras" => Some(TokenKind::Mientras),
            "para" => Some(TokenKind::Para),
            "funcion" => Some(TokenKind::Funcion),
            "retornar" => Some(TokenKind::Retornar),
            "tipo" => Some(TokenKind::Tipo),
            "verdadero" => Some(TokenKind::Verdadero),
            "falso" => Some(TokenKind::Falso),
            "numero" => Some(TokenKind::Numero),
            "entero" => Some(TokenKind::Entero),
            "decimal" => Some(TokenKind::Decimal),
            "texto" => Some(TokenKind::Texto),
            "booleano" => Some(TokenKind::Booleano),
            "imprimir" => Some(TokenKind::Imprimir),
            "leer" => Some(TokenKind::Leer),
            "lista" => Some(TokenKind::Lista),
            "romper" => Some(TokenKind::Romper),
            "continuar" => Some(TokenKind::Continuar),
            "elegir" => Some(TokenKind::Elegir),
            "caso" => Some(TokenKind::Caso),
            "defecto" => Some(TokenKind::Defecto),
            "estructura" => Some(TokenKind::Estructura),
            "if" => Some(TokenKind::If),
            "else" => Some(TokenKind::Else),
            "while" => Some(TokenKind::While),
            "for" => Some(TokenKind::For),
            "function" => Some(TokenKind::Function),
            "return" => Some(TokenKind::Return),
            "true" => Some(TokenKind::True),
            "false" => Some(TokenKind::False),
            "number" => Some(TokenKind::Number),
            "integer" => Some(TokenKind::Integer),
            "float" => Some(TokenKind::Float),
            "string" => Some(TokenKind::String),
            "boolean" => Some(TokenKind::Boolean),
            "print" => Some(TokenKind::Print),
            "read" => Some(TokenKind::Read),
            "array" => Some(TokenKind::Array),
            "break" => Some(TokenKind::Break),
            "continue" => Some(TokenKind::Continue),
            "match" => Some(TokenKind::Match),
            "case" => Some(TokenKind::Case),
            "default" => Some(TokenKind::Default),
            "struct" => Some(TokenKind::Struct),
            "importar" => Some(TokenKind::Importar),
            "import" => Some(TokenKind::Import),
            "como" => Some(TokenKind::Como),
            "as" => Some(TokenKind::As),
            "resultado" => Some(TokenKind::Resultado),
            "result" => Some(TokenKind::Result),
            "exito" => Some(TokenKind::Exito),
            "error" => Some(TokenKind::ErrKeyword),
            "err" => Some(TokenKind::Err),
            "intentar" => Some(TokenKind::Intentar),
            "try" => Some(TokenKind::Try),
            "en" => Some(TokenKind::En),
            "in" => Some(TokenKind::In),
            "enum" => Some(TokenKind::Enum),
            "opcion" => Some(TokenKind::Opcion),
            "option" => Some(TokenKind::Option),
            "algun" => Some(TokenKind::Algun),
            "some" => Some(TokenKind::Some),
            "ninguno" => Some(TokenKind::Ninguno),
            "none" => Some(TokenKind::None),
            "const" => Some(TokenKind::Const),
            "rasgo" => Some(TokenKind::Rasgo),
            "trait" => Some(TokenKind::Trait),
            "impl" => Some(TokenKind::Impl),
            "async" => Some(TokenKind::Async),
            "await" => Some(TokenKind::Await),
            "esperar" => Some(TokenKind::Esperar),
            "sea" => Some(TokenKind::Sea),
            "let" => Some(TokenKind::Let),
            "posponer" => Some(TokenKind::Posponer),
            "defer" => Some(TokenKind::Defer),
            "consultar" => Some(TokenKind::Consultar),
            "query" => Some(TokenKind::Query),
            "donde" => Some(TokenKind::Donde),
            "where" => Some(TokenKind::Where),
            "ordenar_por" => Some(TokenKind::OrdenarPor),
            "order_by" => Some(TokenKind::OrderBy),
            "seleccionar" => Some(TokenKind::Seleccionar),
            "select" => Some(TokenKind::Select),
            "descendente" => Some(TokenKind::Descendente),
            "descending" => Some(TokenKind::Descending),
            "ascendente" => Some(TokenKind::Ascendente),
            "ascending" => Some(TokenKind::Ascending),
            "atrapar" => Some(TokenKind::Atrapar),
            "catch" => Some(TokenKind::Catch),
            "prestado" => Some(TokenKind::Prestado),
            "borrowed" | "ref" => Some(TokenKind::Borrowed),
            "dueno" => Some(TokenKind::Dueno),
            "owner" => Some(TokenKind::Owner),
            "mut" => Some(TokenKind::Mut),
            "mutable" => Some(TokenKind::Mutable),
            "en_tiempo_compilacion" => Some(TokenKind::EnTiempoCompilacion),
            "comptime" => Some(TokenKind::Comptime),
            "ensamblador" => Some(TokenKind::Ensamblador),
            "asm" => Some(TokenKind::Asm),
            "bloque_c" | "codigo_c" => Some(TokenKind::BloqueC),
            "c_block" | "c_code" => Some(TokenKind::CBlock),
            "bloque_rust" | "codigo_rust" => Some(TokenKind::BloqueRust),
            "rust_block" | "rust_code" => Some(TokenKind::RustBlock),
            "puro" => Some(TokenKind::Puro),
            "pure" => Some(TokenKind::Pure),
            "grupo_tareas" => Some(TokenKind::GrupoTareas),
            "task_group" => Some(TokenKind::TaskGroup),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            TokenKind::Si => "si",
            TokenKind::Sino => "sino",
            TokenKind::Mientras => "mientras",
            TokenKind::Para => "para",
            TokenKind::Funcion => "funcion",
            TokenKind::Retornar => "retornar",
            TokenKind::Tipo => "tipo",
            TokenKind::Verdadero | TokenKind::True => "verdadero",
            TokenKind::Falso | TokenKind::False => "falso",
            TokenKind::Numero | TokenKind::Number => "numero",
            TokenKind::Entero | TokenKind::Integer => "entero",
            TokenKind::Decimal | TokenKind::Float => "decimal",
            TokenKind::Texto | TokenKind::String => "texto",
            TokenKind::Booleano | TokenKind::Boolean => "booleano",
            TokenKind::Imprimir | TokenKind::Print => "imprimir",
            TokenKind::Leer | TokenKind::Read => "leer",
            TokenKind::Lista | TokenKind::Array => "lista",
            TokenKind::Romper | TokenKind::Break => "romper",
            TokenKind::Continuar | TokenKind::Continue => "continuar",
            TokenKind::Elegir | TokenKind::Match => "elegir",
            TokenKind::Caso | TokenKind::Case => "caso",
            TokenKind::Defecto | TokenKind::Default => "defecto",
            TokenKind::Estructura | TokenKind::Struct => "estructura",
            TokenKind::Importar | TokenKind::Import => "importar",
            TokenKind::Como | TokenKind::As => "como",
            TokenKind::Resultado | TokenKind::Result => "resultado",
            TokenKind::Exito | TokenKind::Ok => "exito",
            TokenKind::ErrKeyword | TokenKind::Err => "error",
            TokenKind::Intentar | TokenKind::Try => "intentar",
            TokenKind::En | TokenKind::In => "en",
            TokenKind::Opcion | TokenKind::Option => "opcion",
            TokenKind::Algun | TokenKind::Some => "algun",
            TokenKind::Ninguno | TokenKind::None => "ninguno",
            TokenKind::Rasgo | TokenKind::Trait => "rasgo",
            TokenKind::Impl => "impl",
            TokenKind::Posponer | TokenKind::Defer => "posponer",
            _ => "",
        }
    }
}
