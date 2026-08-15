use lumen_ir::builder::IRBuilder;
use lumen_sema::loader::ModuleLoader;
use lumen_sema::sema::SemanticAnalyzer;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let path = args.get(1).expect("uso: cgen <file.nv> [out.c]");
    let source = std::fs::read_to_string(path).unwrap();
    let mut lib_dirs: Vec<std::path::PathBuf> = vec![];
    for d in ["stdlib", "../stdlib"] {
        if Path::new(d).is_dir() {
            lib_dirs.push(d.into());
        }
    }
    let mut loader = ModuleLoader::new(lib_dirs);
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
    let c = lumen_aot::compile_to_c(&ir);
    let out_path = args
        .get(2)
        .cloned()
        .unwrap_or_else(|| format!("{}.c", path));
    std::fs::write(&out_path, &c).unwrap();
    println!("C escrito en {}", out_path);
}
