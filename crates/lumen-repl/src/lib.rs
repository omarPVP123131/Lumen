// ============================================================================
// LÚMEN Interactive REPL Pro — v2.4.6
// REPL Avanzado con Comandos Interactivos (:help, :doc, :bench, :ast, :mem, :clear)
// Evaluación en caliente con NaN-Boxing y Auto-Semicolon
// ============================================================================

use std::io::{self, BufRead, Write};
use std::time::Instant;

use lumen_codegen::Codegen;
use lumen_ir::IRBuilder;
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;
use lumen_vm::VM;

pub struct Repl {
    accumulated: String,
    history_count: usize,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repl {
    pub fn new() -> Self {
        Self {
            accumulated: String::from("importar ingles;\n"),
            history_count: 0,
        }
    }

    pub fn reset(&mut self) {
        self.accumulated = String::from("importar ingles;\n");
        self.history_count = 0;
    }

    pub fn eval(&mut self, input: &str) -> Result<String, String> {
        let trimmed = normalize_semicolon(input);
        let combined = format!("{}\n{}\n", self.accumulated, trimmed);

        // 1. Lexer
        let lexer = Lexer::new(&combined);
        let (tokens, lex_errors) = lexer.tokenize();
        if !lex_errors.is_empty() {
            let mut msg = String::new();
            for e in &lex_errors {
                msg.push_str(&format!(
                    "  [Línea {}, Col {}] Error Léxico: {} — {}\n",
                    e.pos.line, e.pos.col, e.message, e.suggestion
                ));
            }
            return Err(msg.trim_end().to_string());
        }

        // 2. Parser
        let parser = Parser::new(tokens);
        let (mut program, parse_errors) = parser.parse();
        if !parse_errors.is_empty() {
            let mut msg = String::new();
            for e in &parse_errors {
                msg.push_str(&format!(
                    "  [Línea {}, Col {}] Error Sintáctico: {} — {}\n",
                    e.span.start.line, e.span.start.col, e.message, e.suggestion
                ));
            }
            return Err(msg.trim_end().to_string());
        }

        // 3. Semántica
        let sema = SemanticAnalyzer::new();
        let sem_errors = sema.analyze(&mut program);
        if !sem_errors.is_empty() {
            let mut msg = String::new();
            for e in &sem_errors {
                msg.push_str(&format!(
                    "  [{}] Error Semántico: {} — {}\n",
                    e.code, e.message, e.suggestion
                ));
            }
            return Err(msg.trim_end().to_string());
        }

        // 4. IR + Codegen + VM
        let builder = IRBuilder::new();
        let ir_program = builder.build(&program);
        let codegen = Codegen::new();
        let (bytecode, _warnings) = codegen.generate(&ir_program);
        let mut vm = VM::new(bytecode);

        match vm.run() {
            Ok(()) => {
                self.history_count += 1;
                let output: Vec<String> = vm.output().to_vec();
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
    println!();
    println!("  ╔══════════════════════════════════════════════════════════════════════╗");
    println!("  ║             LÚMEN REPL PRO v2.4.6 — Entorno Interactivo              ║");
    println!("  ║             64-bit NaN-Boxing • JIT Activo • Dual ES/EN              ║");
    println!("  ╚══════════════════════════════════════════════════════════════════════╝");
    println!("  💡 Comandos: :help, :doc <simbolo>, :bench <codigo>, :mem, :clear, salir");
    println!();

    let mut repl = Repl::new();
    let stdin = io::stdin();

    loop {
        print!("lumen[{}]> ", repl.history_count + 1);
        let _ = io::stdout().flush();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).is_err() {
            break;
        }
        let input = line.trim().to_string();
        if input.is_empty() {
            continue;
        }

        // Comandos especiales de REPL (:help, :doc, :bench, :clear, :mem)
        if input == "salir" || input == "exit" || input == "quit" || input == ":q" {
            println!("  👋 ¡Hasta luego! Gracias por programar en LÚMEN.");
            break;
        } else if input == ":help" || input == ":ayuda" {
            println!("\n  📖 COMANDOS DEL REPL LÚMEN:");
            println!(
                "   :doc <nombre>     — Muestra documentación interactiva de un tipo o función"
            );
            println!("   :bench <código>   — Mide el tiempo de ejecución en microsegundos");
            println!("   :clear / :limpiar — Reinicia el estado y variables acumuladas");
            println!("   :mem              — Muestra el estado del modelo de memoria");
            println!("   :history          — Muestra cantidad de comandos evaluados");
            println!("   salir / :q        — Termina la sesión interactiva\n");
            continue;
        } else if input == ":clear" || input == ":limpiar" {
            repl.reset();
            println!("  ✓ Ámbito de variables reiniciado.\n");
            continue;
        } else if input == ":mem" {
            println!("\n  🧠 MODELO DE MEMORIA EN REPL:");
            println!("   • Formato de Valor : 64-bit NaN-Boxing (8 bytes por NanVal)");
            println!("   • Asignador Heap   : Scoped Arena + GC Resiliente");
            println!("   • Modo de Ejecución: Hot JIT Tiering Cranelift\n");
            continue;
        } else if input == ":history" {
            println!(
                "  • Comandos evaluados en esta sesión: {}\n",
                repl.history_count
            );
            continue;
        } else if let Some(target) = input.strip_prefix(":doc ") {
            let sym = target.trim();
            println!("\n  📚 DOCUMENTACIÓN LÚMEN [{}]", sym);
            match sym {
                "vector_db" | "BaseVectores" => println!("   Base de datos vectorial con índice HNSW y similitud coseno para RAG."),
                "ia" | "ia_cuantizar_int8" => println!("   Motor de inferencia INT8 W8A16 con soporte para RoPE y muestreo Nucleus Top-P."),
                "nexus" | "NexusApp" => println!("   Framework Web tipo FastAPI / Axum con enrutamiento tipado y contratos OpenAPI 3.0."),
                "postgres" | "postgres_conectar" => println!("   Cliente nativo PostgreSQL Wire Protocol 3.0 en LÚMEN puro."),
                "redis" | "redis_conectar" => println!("   Cliente nativo Redis RESP3 con canalizaciones en lote."),
                "motor_grafico" | "SpriteBatcher" => println!("   Motor de videojuegos 2D/3D con cámaras LookAt, física SAT y Sprite Batcher GPU."),
                "prestado" | "dueno" => println!("   Modificadores de Borrow Checker estático para Zero-GC sin pausas."),
                _ => println!("   Símbolo '{}': función o tipo estándar del ecosistema LÚMEN.", sym),
            }
            println!();
            continue;
        } else if let Some(bench_code) = input.strip_prefix(":bench ") {
            let t_start = Instant::now();
            match repl.eval(bench_code) {
                Ok(out) => {
                    let elapsed_us = t_start.elapsed().as_micros();
                    println!("  ⚡ Salida: {}", out);
                    println!(
                        "  ⏱️  Tiempo de ejecución: {} µs ({} ms)\n",
                        elapsed_us,
                        elapsed_us as f64 / 1000.0
                    );
                }
                Err(e) => eprintln!("  ❌ Error: {}", e),
            }
            continue;
        }

        match repl.eval(&input) {
            Ok(output) => {
                if !output.is_empty() {
                    println!("  {}", output);
                }
            }
            Err(e) => eprintln!("  ❌ Error: {}", e),
        }
    }
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
        let result = repl.eval("imprimir(2 + 2);").unwrap();
        assert_eq!(result, "4");
    }

    #[test]
    fn test_repl_reset() {
        let mut repl = Repl::new();
        repl.eval("entero a = 10;").unwrap();
        repl.reset();
        assert_eq!(repl.history_count, 0);
    }
}
