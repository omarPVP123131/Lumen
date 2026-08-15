use lumen_ir::builder::IRBuilder;
use lumen_sema::loader::ModuleLoader;
use lumen_sema::sema::SemanticAnalyzer;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("uso: ir_dump <file.nv>");
    let source = std::fs::read_to_string(path).unwrap();
    let mut loader = ModuleLoader::new(vec![]);
    let mut prog = match loader.resolve_imports(&source, Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("imports: {:?}", e);
            std::process::exit(1);
        }
    };
    let errors = SemanticAnalyzer::new().analyze(&mut prog);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("  [{}] {}", e.code, e.message);
        }
        std::process::exit(1);
    }
    let ir = IRBuilder::new().build(&prog);
    println!("entry: {}", ir.entry);
    for (name, func) in &ir.funcs {
        println!(
            "\n=== {} (params: {:?}, entry label {}) ===",
            name, func.params, func.entry
        );
        for (i, ins) in func.instrs.iter().enumerate() {
            println!("  {:4} {:?}", i, ins);
        }
    }
}
