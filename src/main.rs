// src/main.rs
mod vm;
mod instructions;
mod compiler;
mod cli;

fn main() {
    if let Err(e) = cli::run() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}