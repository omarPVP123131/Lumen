// src/cli/commands.rs
use std::fs;
use crate::compiler::compile;
use crate::vm::VM;

pub fn run(filename: &str) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file: {}", e))?;
    
    // Compile
    println!("Compiling {}...", filename);
    let bytecode = compile(&source)?;
    
    // Execute
    println!("Executing...");
    let mut vm = VM::new(bytecode);
    vm.run().map_err(|e| format!("Runtime error: {:?}", e))?;
    
    Ok(())
}

pub fn build(filename: &str) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file: {}", e))?;
    
    // Compile
    println!("Compiling {}...", filename);
    let bytecode = compile(&source)?;
    
    // Write bytecode file
    let output_filename = filename.replace(".lumen", ".nvc");
    fs::write(&output_filename, bytecode)
        .map_err(|e| format!("Error writing bytecode: {}", e))?;
    
    println!("Bytecode written to {}", output_filename);
    Ok(())
}

pub fn check(filename: &str) -> Result<(), String> {
    // Read source file
    let source = fs::read_to_string(filename)
        .map_err(|e| format!("Error reading file: {}", e))?;
    
    // Try to compile (syntax check)
    println!("Checking {}...", filename);
    compile(&source)?;
    
    println!("Syntax OK");
    Ok(())
}
