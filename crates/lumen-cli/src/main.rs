use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use lumen_codegen::{disassemble, Bytecode, Codegen};
use lumen_ir::IRBuilder;
use lumen_lexer::token::Span;
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

    // Add stdlib/ as default search path if it exists
    let stdlib_path = PathBuf::from("stdlib");
    if stdlib_path.is_dir() && !lib_dirs.iter().any(|p| p == &stdlib_path) {
        lib_dirs.push(stdlib_path);
    }
    let stdlib_alt = PathBuf::from("../stdlib");
    if stdlib_alt.is_dir() && !lib_dirs.iter().any(|p| p == &stdlib_alt) {
        lib_dirs.push(stdlib_alt);
    }

    Config {
        command,
        file,
        lib_dirs,
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2
        || matches!(
            args.get(1).map(|s| s.as_str()),
            Some("--help" | "-h" | "help")
        )
    {
        eprintln!(
            "LÚMEN v{} — Lenguaje de programación educativo bilingüe",
            env!("CARGO_PKG_VERSION")
        );
        eprintln!();
        eprintln!("USO:");
        eprintln!("  lumen [opciones] <comando> <archivo>");
        eprintln!();
        eprintln!("COMANDOS:");
        eprintln!("  run <archivo>       Ejecuta un programa .nv o bytecode .nvc");
        eprintln!("  build <archivo>     Compila a bytecode (.nvc)");
        eprintln!("  check <archivo>     Verifica sintaxis y semántica");
        eprintln!("  disasm <archivo>    Desensambla bytecode .nvc");
        eprintln!();
        eprintln!("OPCIONES:");
        eprintln!("  -L, --lib-dir <dir>  Directorio de búsqueda para módulos (stdlib)");
        eprintln!("  -v, --version        Muestra la versión");
        eprintln!("  -h, --help           Muestra esta ayuda");
        eprintln!();
        eprintln!("EJEMPLOS:");
        eprintln!("  lumen run hello.nv          Ejecutar un programa");
        eprintln!("  lumen build programa.nv     Compilar a .nvc");
        eprintln!("  lumen run -L stdlib/ test.nv  Ejecutar con stdlib");
        eprintln!();
        eprintln!("SINTAXIS BÁSICA:");
        eprintln!("  imprimir(\"Hola\");           // Mostrar en pantalla");
        eprintln!("  entero x = 42;              // Declarar variable");
        eprintln!("  si (x > 0) {{ ... }}          // Condicional");
        eprintln!("  mientras (x < 10) {{ ... }}   // Bucle");
        eprintln!("  funcion entero f(a) {{ ... }} // Función");
        eprintln!("  lista<entero> v = [1,2,3];  // Lista/Array");
        eprintln!("  largo(v)                    // Longitud de lista");
        eprintln!("  agregar(v, 4)               // Agregar elemento");
        process::exit(if args.len() == 1 { 1 } else { 0 });
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
                    eprintln!();
                    eprintln!("  \x1b[1;31mE063\x1b[0m \x1b[1mImport circular detectado\x1b[0m");
                    eprintln!(
                        "  \x1b[1;34m-->\x1b[0m {}:{}:{}",
                        path.display(),
                        span.start.line,
                        span.start.col
                    );
                    eprintln!("   \x1b[1;33mAyuda:\x1b[0m Revisa las dependencias entre módulos");
                    eprintln!();
                }
                lumen_sema::ModuleError::Io { path, message } => {
                    eprintln!(
                        "  \x1b[1;31mError\x1b[0m al cargar '{}': {}",
                        path.display(),
                        message
                    );
                }
                lumen_sema::ModuleError::Lex { path, details } => {
                    for d in details {
                        eprintln!(
                            "  \x1b[1;31mError léxico\x1b[0m en '{}': {}",
                            path.display(),
                            d
                        );
                    }
                }
                lumen_sema::ModuleError::Parse { path, details } => {
                    for d in details {
                        eprintln!(
                            "  \x1b[1;31mError sintáctico\x1b[0m en '{}': {}",
                            path.display(),
                            d
                        );
                    }
                }
            }
            process::exit(1);
        }
    }
}

fn show_error(source: &str, path: &str, code: &str, message: &str, span: &Span, suggestion: &str) {
    let line = span.start.line;
    let col = span.start.col;
    let line_str = source.lines().nth(line - 1).unwrap_or("");
    eprintln!();
    eprintln!("  \x1b[1;31m{}\x1b[0m \x1b[1m{}\x1b[0m", code, message);
    eprintln!("  \x1b[1;34m-->\x1b[0m {}:{}:{}", path, line, col);
    eprintln!("   \x1b[1;34m|\x1b[0m");
    eprintln!("  \x1b[1;34m{}\x1b[0m \x1b[1m|\x1b[0m {}", line, line_str);
    let underline = format!(
        "{}{}",
        " ".repeat(line.to_string().len() + 2 + col),
        "^".repeat(span.end.col.saturating_sub(col).max(1))
    );
    eprintln!(
        "  {} \x1b[1;32m{}\x1b[0m",
        " ".repeat(line.to_string().len() + 1),
        underline
    );
    eprintln!("   \x1b[1;34m|\x1b[0m");
    eprintln!("   \x1b[1;33mAyuda:\x1b[0m {}", suggestion);
    eprintln!();
}

fn show_sema_errors(errors: &[lumen_sema::SemError], source: &str, path: &str) -> bool {
    if errors.is_empty() {
        return false;
    }
    for err in errors {
        show_error(
            source,
            path,
            &err.code,
            &err.message,
            &err.span,
            &err.suggestion,
        );
    }
    true
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
        show_sema_errors(&sem_errors, &source, path);
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
            eprintln!("{}", e.with_stack(vm.call_stack()));
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
                    eprintln!("{}", e.with_stack(vm.call_stack()));
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
