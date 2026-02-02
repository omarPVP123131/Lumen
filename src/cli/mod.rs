// src/cli/mod.rs
pub mod commands;

use std::env;

pub fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_help();
        return Ok(());
    }
    
    match args[1].as_str() {
        "run" => {
            if args.len() < 3 {
                return Err("Usage: lumen run <file.lumen>".to_string());
            }
            commands::run(&args[2])
        }
        "build" => {
            if args.len() < 3 {
                return Err("Usage: lumen build <file.lumen>".to_string());
            }
            commands::build(&args[2])
        }
        "check" => {
            if args.len() < 3 {
                return Err("Usage: lumen check <file.lumen>".to_string());
            }
            commands::check(&args[2])
        }
        "--version" | "-v" => {
            println!("LUMEN v0.1.0");
            Ok(())
        }
        "--help" | "-h" => {
            print_help();
            Ok(())
        }
        _ => {
            Err(format!("Unknown command: {}", args[1]))
        }
    }
}

fn print_help() {
    println!("LUMEN v0.1.0 - Educational Programming Language");
    println!();
    println!("USAGE:");
    println!("    lumen <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    run <file>      Compile and execute a .lumen file");
    println!("    build <file>    Compile to bytecode (.nvc)");
    println!("    check <file>    Check syntax without executing");
    println!();
    println!("OPTIONS:");
    println!("    -h, --help      Print help information");
    println!("    -v, --version   Print version");
}