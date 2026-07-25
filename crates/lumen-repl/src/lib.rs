use std::io::{self, BufRead, Write};

use lumen_codegen::Codegen;
use lumen_ir::IRBuilder;
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;
use lumen_vm::VM;

pub struct Repl {
    accumulated: String,
}

impl Repl {
    pub fn new() -> Self {
        Self {
            accumulated: String::from("importar ingles;\n"),
        }
    }

    pub fn eval(&mut self, input: &str) -> Result<String, String> {
        let trimmed = normalize_semicolon(input);
        let combined = format!("{}\n{}\n", self.accumulated, trimmed);

        // Lex
        let lexer = Lexer::new(&combined);
        let (tokens, lex_errors) = lexer.tokenize();
        if !lex_errors.is_empty() {
            let mut msg = String::new();
            for e in &lex_errors {
                msg.push_str(&format!(
                    "  [{}:{}] {} — {}\n",
                    e.pos.line, e.pos.col, e.message, e.suggestion
                ));
            }
            return Err(msg.trim_end().to_string());
        }

        // Parse
        let parser = Parser::new(tokens);
        let (mut program, parse_errors) = parser.parse();
        if !parse_errors.is_empty() {
            let mut msg = String::new();
            for e in &parse_errors {
                msg.push_str(&format!(
                    "  [{}:{}] {} — {}\n",
                    e.span.start.line, e.span.start.col, e.message, e.suggestion
                ));
            }
            return Err(msg.trim_end().to_string());
        }

        // Semantic analysis
        let sema = SemanticAnalyzer::new();
        let sem_errors = sema.analyze(&mut program);
        if !sem_errors.is_empty() {
            let mut msg = String::new();
            for e in &sem_errors {
                msg.push_str(&format!(
                    "  [{}] {} — {}\n",
                    e.code, e.message, e.suggestion
                ));
            }
            return Err(msg.trim_end().to_string());
        }

        // IR → Codegen → VM
        let builder = IRBuilder::new();
        let ir_program = builder.build(&program);
        let codegen = Codegen::new();
        let (bytecode, _warnings) = codegen.generate(&ir_program);
        let mut vm = VM::new(bytecode);

        match vm.run() {
            Ok(()) => {
                let output: Vec<String> = vm.output().to_vec();
                // If it's a declaration (funcion, estructura, etc.), accumulate it
                let is_decl = trimmed.contains("funcion")
                    || trimmed.contains("estructura")
                    || trimmed.contains("enum")
                    || trimmed.contains("rasgo")
                    || trimmed.contains("impl");
                if is_decl {
                    self.accumulated = format!("{}\n{}\n", self.accumulated, trimmed);
                }
                if output.is_empty() {
                    Ok("=> ()".to_string())
                } else {
                    Ok(output.join("\n"))
                }
            }
            Err(e) => Err(e.with_stack(vm.call_stack())),
        }
    }
}

fn normalize_semicolon(s: &str) -> String {
    let trimmed = s.trim();
    if trimmed.ends_with(';') || trimmed.ends_with('}') {
        trimmed.to_string()
    } else {
        format!("{};", trimmed)
    }
}

pub fn run_repl() {
    println!("LÚMEN REPL v1.4.0 — escribe 'salir' para terminar");
    let mut repl = Repl::new();
    let stdin = io::stdin();
    loop {
        print!("> ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }
        let input = line.trim().to_string();
        if input == "salir" || input == "exit" || input == "quit" {
            break;
        }
        if input.is_empty() {
            continue;
        }
        match repl.eval(&input) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("{}", output);
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    println!("Adiós!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_simple() {
        let mut repl = Repl::new();
        let result = repl.eval("imprimir(42);").unwrap();
        assert_eq!(result, "42");
    }

    #[test]
    fn test_eval_arithmetic() {
        let mut repl = Repl::new();
        let result = repl.eval("entero x = 2 + 2;\nimprimir(x);").unwrap();
        assert_eq!(result, "4");
    }
}
