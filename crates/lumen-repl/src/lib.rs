// ============================================================================
// LÚMEN Interactive REPL Pro — v3.0.0
// REPL Avanzado con Comandos Interactivos (:help, :doc, :bench, :ast, :mem, :clear)
// Evaluación en caliente con NaN-Boxing y Auto-Semicolon
// ============================================================================

use std::io::{self, BufRead, Write};
use std::time::Instant;

use lumen_codegen::Codegen;
use lumen_ir::IRBuilder;
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::{ModuleLoader, SemanticAnalyzer};
use lumen_vm::VM;

pub struct Repl {
    accumulated: String,
    history_count: usize,
    /// BUG-156: rutas de `-L/--lib-dir`, para que el REPL resuelva imports con
    /// los mismos directorios que `lumen run`.
    lib_dirs: Vec<std::path::PathBuf>,
}

impl Default for Repl {
    fn default() -> Self {
        Self::new()
    }
}

impl Repl {
    pub fn new() -> Self {
        Self::con_lib_dirs(Vec::new())
    }

    /// REPL que resuelve los imports usando estos directorios, ademas de la
    /// stdlib embebida.
    pub fn con_lib_dirs(lib_dirs: Vec<std::path::PathBuf>) -> Self {
        Self {
            accumulated: String::from("importar ingles;\n"),
            history_count: 0,
            lib_dirs,
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
        let (_program_parseado, parse_errors) = parser.parse();
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

        // 2b. Imports. BUG-156: el REPL nunca invocaba al `ModuleLoader`, asi
        // que `importar "texto";` no cargaba nada Y NO DABA ERROR: la linea se
        // aceptaba con `=> ()` y las funciones del modulo seguian sin existir.
        // Peor aun, `importar "no_existe";` tambien se aceptaba en silencio, de
        // modo que el REPL no servia para probar un modulo antes de usarlo, que
        // es justo para lo que se usa un REPL. Se resuelve igual que en `run`;
        // la stdlib embebida (BUG-152) hace que funcione sin ficheros al lado.
        let mut loader = ModuleLoader::new(self.lib_dirs.clone());
        let base = std::path::Path::new("<repl>.nv");
        // `resolve_imports` reparsea el fuente y devuelve el programa ya
        // aplanado, asi que sustituye al que acaba de salir del parser; aquel
        // solo sirve para reportar los errores de sintaxis de mas arriba.
        let mut program = match loader.resolve_imports(&combined, base) {
            Ok(p) => p,
            Err(e) => {
                let msg = match &e {
                    lumen_sema::ModuleError::Circular { path, .. } => {
                        format!("  [E063] Import circular detectado: {}", path.display())
                    }
                    lumen_sema::ModuleError::Io { path, message } => {
                        format!("  Error al cargar '{}': {}", path.display(), message)
                    }
                    otro => format!("  Error de import: {:?}", otro),
                };
                return Err(msg);
            }
        };

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
                // BUG-004: además de las declaraciones de tipo/función, hay que
                // persistir las declaraciones de variable para que la línea
                // siguiente las vea. El REPL reejecuta el historial acumulado,
                // así que sólo se guarda lo que declara algo (no las
                // expresiones con efectos, que se duplicarían al reejecutarse).
                if is_persistent_decl(&trimmed) {
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

/// BUG-004: decide si una línea debe conservarse en el estado acumulado del
/// REPL. Se conservan las declaraciones (tipos, funciones y variables) porque
/// definen nombres que las líneas siguientes necesitan resolver.
///
/// Las llamadas y expresiones sueltas NO se conservan: el REPL reejecuta todo
/// el historial en cada línea, así que persistirlas duplicaría su salida y sus
/// efectos secundarios.
fn is_persistent_decl(line: &str) -> bool {
    let t = line.trim();
    if t.is_empty() {
        return false;
    }

    // Declaraciones de tipo / función / trait, en español e inglés.
    const DECL_KEYWORDS: [&str; 12] = [
        "funcion ",
        "function ",
        "estructura ",
        "struct ",
        "enum ",
        "rasgo ",
        "trait ",
        "impl ",
        "importar ",
        "import ",
        "const ",
        "async ",
    ];
    if DECL_KEYWORDS.iter().any(|k| t.starts_with(k)) {
        return true;
    }

    // `sea x = ...` / `let x = ...` — inferencia de tipo.
    if t.starts_with("sea ") || t.starts_with("let ") {
        return true;
    }

    // Declaración con tipo explícito: `entero x = 0;`, `lista<texto> l = [];`,
    // `resultado<entero, texto> r = exito(1);`, `Punto p = Punto { ... };`.
    is_typed_var_decl(t)
}

/// Reconoce `TIPO nombre = valor;` (incluyendo genéricos `T<...>`), que es la
/// forma de declarar variables con tipo explícito en LÚMEN.
fn is_typed_var_decl(t: &str) -> bool {
    // Debe haber una asignación de nivel superior.
    let Some(eq) = find_top_level_assign(t) else {
        return false;
    };
    let head = t[..eq].trim();
    if head.is_empty() {
        return false;
    }

    // Ignora asignaciones a variables ya existentes (`x = 1`), campos
    // (`p.x = 1`) e índices (`l[0] = 1`): no declaran nombres nuevos.
    if head.contains('.') || head.ends_with(']') {
        return false;
    }

    // Separa el nombre final del tipo que lo precede, respetando `<...>`.
    let mut depth = 0i32;
    let mut split = None;
    for (i, c) in head.char_indices() {
        match c {
            '<' => depth += 1,
            '>' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => split = Some(i),
            _ => {}
        }
    }
    let Some(split) = split else {
        return false; // `x = 1` — un solo token: asignación, no declaración.
    };

    let type_part = head[..split].trim();
    let name_part = head[split..].trim();

    !type_part.is_empty()
        && !name_part.is_empty()
        && name_part
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || !c.is_ascii())
        && name_part.chars().next().is_some_and(|c| !c.is_numeric())
}

/// Encuentra el `=` de asignación de nivel superior, ignorando `==`, `!=`,
/// `<=`, `>=`, los que van dentro de cadenas y los de genéricos.
fn find_top_level_assign(t: &str) -> Option<usize> {
    let b = t.as_bytes();
    let mut in_str = false;
    let mut depth = 0i32;
    let mut i = 0;
    while i < b.len() {
        let c = b[i];
        match c {
            b'"' if !in_str => in_str = true,
            b'"' if in_str => in_str = false,
            b'\\' if in_str => i += 1,
            b'(' | b'[' | b'{' if !in_str => depth += 1,
            b')' | b']' | b'}' if !in_str => depth -= 1,
            b'=' if !in_str && depth == 0 => {
                let prev = if i > 0 { b[i - 1] } else { b' ' };
                let next = if i + 1 < b.len() { b[i + 1] } else { b' ' };
                let is_cmp = next == b'='
                    || matches!(prev, b'=' | b'!' | b'<' | b'>' | b'+' | b'-' | b'*' | b'/');
                if !is_cmp {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
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
    run_repl_con_lib_dirs(Vec::new())
}

/// BUG-156: variante que recibe los `-L` de la linea de comandos.
pub fn run_repl_con_lib_dirs(lib_dirs: Vec<std::path::PathBuf>) {
    println!();
    println!("  ╔══════════════════════════════════════════════════════════════════════╗");
    println!("  ║             LÚMEN REPL PRO v3.0.0 — Entorno Interactivo              ║");
    println!("  ║             64-bit NaN-Boxing • JIT Activo • Dual ES/EN              ║");
    println!("  ╚══════════════════════════════════════════════════════════════════════╝");
    println!("  💡 Comandos: :help, :doc <simbolo>, :bench <codigo>, :mem, :clear, salir");
    println!();

    let mut repl = Repl::con_lib_dirs(lib_dirs);
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
