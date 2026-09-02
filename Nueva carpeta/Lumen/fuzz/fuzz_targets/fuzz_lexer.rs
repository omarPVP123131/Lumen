#![no_main]

use libfuzzer_sys::fuzz_target;
use lumen_lexer::Lexer;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let lexer = Lexer::new(s);
        let (_tokens, _errors) = lexer.tokenize();
    }
});
