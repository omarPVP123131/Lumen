use std::env;
use std::fs;
use std::path::Path;
use std::process;

use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;
use lumen_ir::IRBuilder;
use lumen_codegen::{Codegen, disassemble, Bytecode};
use lumen_vm::VM;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("LÚMEN v{}", env!("CARGO_PKG_VERSION"));
        eprintln!("Uso: lumen <comando> [archivo]");
        eprintln!("Comandos:");
        eprintln!("  run <archivo>      Ejecuta un programa fuente o bytecode");
        eprintln!("  build <archivo>    Compila a bytecode (.nvc)");
        eprintln!("  check <archivo>    Verifica sintaxis y semántica");
        eprintln!("  disasm <archivo>   Desensambla bytecode .nvc");
        process::exit(1);
    }

    let command = &args[1];
    match command.as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            let path = &args[2];
            if path.ends_with(".nvc") {
                run_bytecode(path);
            } else {
                run_source(path);
            }
        }
        "build" => {
            if args.len() < 3 {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            build_bytecode(&args[2]);
        }
        "check" => {
            if args.len() < 3 {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            check_source(&args[2]);
        }
        "disasm" => {
            if args.len() < 3 {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            disasm_file(&args[2]);
        }
        "--version" | "-v" => {
            println!("LÚMEN v{}", env!("CARGO_PKG_VERSION"));
        }
        _ => {
            eprintln!("Comando desconocido: '{}'", command);
            eprintln!("Usa 'lumen run', 'lumen build', 'lumen check', o 'lumen disasm'");
            process::exit(1);
        }
    }
}

fn run_source(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();

    if !lex_errors.is_empty() {
        for err in &lex_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.pos.line, err.pos.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let parser = Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();

    if !parse_errors.is_empty() {
        for err in &parse_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.span.start.line, err.span.start.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);

    if !sem_errors.is_empty() {
        for err in &sem_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.span.start.line, err.span.start.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let builder = IRBuilder::new();
    let ir_program = builder.build(&program);

    let codegen = Codegen::new();
    let (bytecode, _warnings) = codegen.generate(&ir_program);

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

fn build_bytecode(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        for err in &lex_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.pos.line, err.pos.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let parser = Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        for err in &parse_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.span.start.line, err.span.start.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        for err in &sem_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.span.start.line, err.span.start.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let builder = IRBuilder::new();
    let ir_program = builder.build(&program);

    let codegen = Codegen::new();
    let (bytecode, _warnings) = codegen.generate(&ir_program);

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

fn check_source(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    let lexer = Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        for err in &lex_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.pos.line, err.pos.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let parser = Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        for err in &parse_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.span.start.line, err.span.start.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        for err in &sem_errors {
            eprintln!("Error {} [{}:{}]: {}", err.code, err.span.start.line, err.span.start.col, err.message);
            eprintln!("  Sugerencia: {}", err.suggestion);
        }
        process::exit(1);
    }

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
