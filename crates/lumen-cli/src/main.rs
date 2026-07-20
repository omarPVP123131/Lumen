use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use lumen_codegen::{disassemble, Bytecode, Codegen};
use lumen_ir::IRBuilder;
use lumen_parser::ast::DeclOrStmt;
use lumen_sema::{ModuleLoader, SemanticAnalyzer};
use lumen_vm::VM;

struct Config {
    command: String,
    file: String,
    lib_dirs: Vec<PathBuf>,
}

fn parse_args(args: &[String]) -> Config {
    let mut i = 1;
    let mut command = String::new();
    let mut file = String::new();
    let mut lib_dirs = Vec::new();

    while i < args.len() {
        match args[i].as_str() {
            "-L" | "--lib-dir" => {
                i += 1;
                if i < args.len() {
                    lib_dirs.push(PathBuf::from(&args[i]));
                } else {
                    eprintln!("Error: falta un directorio después de '-L'");
                    process::exit(1);
                }
            }
            s if command.is_empty() => {
                command = s.to_string();
            }
            s if file.is_empty() => {
                file = s.to_string();
            }
            _ => {
                eprintln!("Argumento desconocido: '{}'", args[i]);
                process::exit(1);
            }
        }
        i += 1;
    }

    Config {
        command,
        file,
        lib_dirs,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("LÚMEN v{}", env!("CARGO_PKG_VERSION"));
        eprintln!("Uso: lumen [opciones] <comando> [archivo]");
        eprintln!("Opciones:");
        eprintln!("  -L, --lib-dir <dir>  Agrega un directorio de búsqueda para módulos");
        eprintln!("Comandos:");
        eprintln!("  run <archivo>      Ejecuta un programa fuente o bytecode");
        eprintln!("  build <archivo>    Compila a bytecode (.nvc)");
        eprintln!("  check <archivo>    Verifica sintaxis y semántica");
        eprintln!("  disasm <archivo>   Desensambla bytecode .nvc");
        process::exit(1);
    }

    let config = parse_args(&args);
    match config.command.as_str() {
        "run" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            if config.file.ends_with(".nvc") {
                run_bytecode(&config.file);
            } else {
                run_source(&config.file, &config.lib_dirs);
            }
        }
        "build" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            build_bytecode(&config.file, &config.lib_dirs);
        }
        "check" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            check_source(&config.file, &config.lib_dirs);
        }
        "disasm" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            disasm_file(&config.file);
        }
        "--version" | "-v" => {
            println!("LÚMEN v{}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Comando desconocido: '{}'", config.command);
            eprintln!("Usa 'lumen run', 'lumen build', 'lumen check', o 'lumen disasm'");
            process::exit(1);
        }
    }
}

fn resolve_or_exit(mut loader: ModuleLoader, source: &str, base_dir: &Path) -> Vec<DeclOrStmt> {
    match loader.resolve_imports(source, base_dir) {
        Ok(p) => p,
        Err(e) => {
            match &e {
                lumen_sema::ModuleError::Circular { path, span } => {
                    eprintln!(
                        "Error E063 [{}:{}]: Import circular detectado: '{}'",
                        span.start.line,
                        span.start.col,
                        path.display()
                    );
                    eprintln!("  Sugerencia: Revisa las dependencias entre módulos para eliminar la circularidad");
                }
                lumen_sema::ModuleError::Io { path, message } => {
                    eprintln!("Error de E/S al cargar '{}': {}", path.display(), message);
                }
                lumen_sema::ModuleError::Lex { path, details } => {
                    eprintln!("Error léxico en '{}':", path.display());
                    for d in details {
                        eprintln!("  {}", d);
                    }
                }
                lumen_sema::ModuleError::Parse { path, details } => {
                    eprintln!("Error sintáctico en '{}':", path.display());
                    for d in details {
                        eprintln!("  {}", d);
                    }
                }
            }
            process::exit(1);
        }
    }
}

fn compile_source(path: &str, lib_dirs: &[PathBuf]) -> Bytecode {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    let base_path = Path::new(path);
    let base_dir = base_path.parent().unwrap_or(Path::new("."));
    let loader = ModuleLoader::new(lib_dirs.to_vec());
    let mut program = resolve_or_exit(loader, &source, base_dir);

    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        for err in &sem_errors {
            eprintln!(
                "Error {} [{}:{}]: {}",
                err.code, err.span.start.line, err.span.start.col, err.message
            );
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let builder = IRBuilder::new();
    let ir_program = builder.build(&program);

    let codegen = Codegen::new();
    let (bytecode, _) = codegen.generate(&ir_program);
    bytecode
}

fn run_source(path: &str, lib_dirs: &[PathBuf]) {
    let bytecode = compile_source(path, lib_dirs);
    let mut vm = VM::new(bytecode);
    match vm.run() {
        Ok(()) => {
            for line in vm.output() {
                println!("{}", line);
            }
        }
        Err(e) => {
            eprintln!("Error de ejecución: {:?}", e);
            process::exit(1);
        }
    }
}

fn run_bytecode(path: &str) {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    match Bytecode::decode(&data) {
        Ok((bc, warnings)) => {
            for (offset, msg) in &warnings {
                eprintln!("Advertencia en offset {}: {}", offset, msg);
            }
            let mut vm = VM::new(bc);
            match vm.run() {
                Ok(()) => {
                    for line in vm.output() {
                        println!("{}", line);
                    }
                }
                Err(e) => {
                    eprintln!("Error de ejecución: {:?}", e);
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            eprintln!("Error al decodificar bytecode: {}", e);
            process::exit(1);
        }
    }
}

fn build_bytecode(path: &str, lib_dirs: &[PathBuf]) {
    let bytecode = compile_source(path, lib_dirs);
    let out_path = Path::new(path).with_extension("nvc");
    let encoded = bytecode.encode();
    match fs::write(&out_path, &encoded) {
        Ok(()) => println!("Bytecode generado: {}", out_path.display()),
        Err(e) => {
            eprintln!("Error al escribir '{}': {}", out_path.display(), e);
            process::exit(1);
        }
    }
}

fn check_source(path: &str, lib_dirs: &[PathBuf]) {
    let _ = compile_source(path, lib_dirs);
    println!("✓ El programa es válido (sintaxis y semántica correctas)");
}

fn disasm_file(path: &str) {
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    match Bytecode::decode(&data) {
        Ok((bc, warnings)) => {
            for (offset, msg) in &warnings {
                eprintln!("Advertencia en offset {}: {}", offset, msg);
            }
            print!("{}", disassemble(&bc));
        }
        Err(e) => {
            eprintln!("Error al decodificar bytecode: {}", e);
            process::exit(1);
        }
    }
}
