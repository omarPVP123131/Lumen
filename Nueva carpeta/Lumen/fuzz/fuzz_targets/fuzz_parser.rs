#![no_main]

use libfuzzer_sys::fuzz_target;
use lumen_lexer::Lexer;
use lumen_parser::Parser;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let lexer = Lexer::new(s);
        let (tokens, _lex_errors) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (_program, _parse_errors) = parser.parse();
    }
});
