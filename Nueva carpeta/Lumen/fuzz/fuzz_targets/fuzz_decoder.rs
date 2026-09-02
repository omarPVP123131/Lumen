#![no_main]

use libfuzzer_sys::fuzz_target;
use lumen_codegen::bytecode::Bytecode;

fuzz_target!(|data: &[u8]| {
    let _ = Bytecode::decode(data);
});
