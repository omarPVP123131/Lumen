// src/cli/commands.rs
use crate::compiler::{compile, CompiledProgram};
use crate::vm::vm::VM;
use std::fs;

pub fn run(filename: &str) -> Result<(), String> {
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file: {}", e))?;

    // compile ahora devuelve CompiledProgram { bytecode, pool }
    let CompiledProgram { bytecode, pool } = compile(&source)?;

    // pasamos el bytecode (como slice) y la pool (moved)
    let mut vm = VM::new(&bytecode, pool).map_err(|e| format!("VM init error: {}", e))?;
    vm.run();

    Ok(())
}

pub fn build(filename: &str) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file: {}", e))?;

    // Compile
    println!("Compiling {}...", filename);
    let CompiledProgram { bytecode, pool: _ } = compile(&source)?;

    // Write bytecode file (.nvc) — writing only the bytecode for now
    let output_filename = filename.replace(".lumen", ".nvc");
    fs::write(&output_filename, &bytecode)
        .map_err(|e| format!("Error writing bytecode: {}", e))?;

    println!("Bytecode written to {}", output_filename);
    Ok(())
}

pub fn check(filename: &str) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file: {}", e))?;

    // Try to compile (syntax + codegen)
    println!("Checking {}...", filename);
    let _compiled = compile(&source)?; // we ignore the artifact; compile() will error if invalid

    println!("Syntax OK");
    Ok(())
}
