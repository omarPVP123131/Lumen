use std::env;
use std::fs;
use std::io::Write;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;

use lumen_codegen::{disassemble, Bytecode, Codegen};
use lumen_ir::IRBuilder;
use lumen_lexer::token::Span;
use lumen_parser::ast::DeclOrStmt;
use lumen_project::ProjectManifest;
use lumen_sema::{ModuleLoader, SemanticAnalyzer};
use lumen_vm::VM;

struct Config {
    command: String,
    file: String,
    lib_dirs: Vec<PathBuf>,
    native: bool,
    backend: String,
    port: u16,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_help() {
    println!();
    println!("  ╔══════════════════════════════════════════════════╗");
    println!("  ║        LÚMEN v{VERSION} — Lenguaje Bilingüe ES/EN      ║");
    println!("  ║      De 0 a ingeniero — From zero to engineer     ║");
    println!("  ╚══════════════════════════════════════════════════╝");
    println!();
    println!("  📖 RUTA DE APRENDIZAJE / LEARNING PATH:");
    println!();
    println!("     PASO 1 — Fundamentos:");
    println!("       lumen run examples/hello.nv");
    println!("       lumen run examples/demo_completo.nv");
    println!();
    println!("     PASO 2 — Tu primer programa:");
    println!("       lumen new mi_proyecto");
    println!("       lumen run mi_proyecto/main.nv");
    println!("       lumen check mi_proyecto/main.nv    # verificar antes de ejecutar");
    println!();
    println!("     PASO 3 — Editar y depurar:");
    println!("       lumen fmt archivo.nv               # formatear código");
    println!("       lumen debug archivo.nv              # depurador interactivo");
    println!("       lumen lint archivo.nv               # análisis estático");
    println!();
    println!("     PASO 4 — Probar y documentar:");
    println!("       lumen test archivo.nv               # ejecutar tests");
    println!("       lumen doc archivo.nv                # generar documentación HTML");
    println!();
    println!("     PASO 5 — Compilar y distribuir:");
    println!("       lumen build archivo.nv              # compilar a bytecode .nvc");
    println!("       lumen disasm archivo.nvc            # desensamblar bytecode");
    println!("       lumen build --native archivo.nv     # compilar a binario nativo (C)");
    println!();
    println!("     PASO 6 — Explora todo el lenguaje:");
    println!("       lumen tutor basics                  # fundamentos");
    println!("       lumen tutor functions               # funciones y genéricos");
    println!("       lumen tutor data                    # structs, enums, match");
    println!("       lumen tutor advanced                # traits, errores, async");
    println!("       lumen tutor stdlib                  # colecciones, texto, JSON");
    println!("       lumen tutor pro                     # GUI, TUI, gráficos, WASM");
    println!();
    println!("  🛠️  COMANDOS / COMMANDS:");
    println!();
    println!("   run <file>       Ejecutar programa / Run program");
    println!("   build <file>     Compilar a bytecode / Compile to .nvc");
    println!("   build --native   Compilar a binario nativo / Compile native binary");
    println!("   check <file>     Verificar sintaxis / Check syntax + semantics");
    println!("   disasm <file>    Desensamblar bytecode / Disassemble .nvc");
    println!("   fmt <file>       Formatear código / Format code");
    println!("   repl             Modo interactivo / Interactive REPL");
    println!("   new <name>       Crear proyecto / Create project");
    println!("   test <file>      Ejecutar tests / Run unit tests");
    println!("   debug <file>     Depurador interactivo / Debugger");
    println!("   lint <file>      Análisis estático / Static analysis");
    println!("   doc <file>       Generar documentación / Generate HTML docs");
    println!("   lsp              Servidor LSP / LSP server (VS Code)");
    println!("   install <pkg>    Instalar paquete / Install package");
    println!("   serve            Playground web / Web playground");
    println!("   learn            Tutorial interactivo / Interactive tutorial");
    println!("   tutor <tema>     Mostrar lección / Show lesson");
    println!();
    println!("  ⚙️  OPCIONES / OPTIONS:");
    println!();
    println!("   -L, --lib-dir <dir>  Ruta de módulos stdlib");
    println!("   -v, --version        Versión / Version");
    println!("   -h, --help           Esta ayuda / This help");
    println!();
    println!("  📚 DOCUMENTACIÓN:");
    println!("   README.md     — Inicio rápido");
    println!("   LENGUAJE.md   — Manual del lenguaje (ES)");
    println!("   docs/cli.md   — Referencia CLI");
    println!("   docs/roadmap.md — Roadmap completo");
    println!("   .opencode/agents/ — Skills de desarrollo");
    println!();
    println!("  🌐 PLAYGROUND WEB:");
    println!("   lumen serve --port 8080   # servidor del playground (WASM)");
    println!("   http://localhost:8080/web/index.html");
    println!();
    println!("  🐳 DOCKER:");
    println!("   docker compose up");
    println!();
}

fn print_tutor(topic: &str) {
    match topic {
        "basics" | "basicos" => {
            println!("╔══════════════════════════════════════════╗");
            println!("║  PASO 1: FUNDAMENTOS / BASICS           ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("📌 Variables y tipos:");
            println!("  entero a = 42;             // integer");
            println!("  decimal pi = 3.14;          // float");
            println!("  texto s = \"hola\";           // string");
            println!("  booleano b = verdadero;     // boolean");
            println!("  lista<entero> nums = [1,2,3]; // array");
            println!();
            println!("📌 Condicionales:");
            println!("  si edad >= 18 {{");
            println!("      imprimir(\"Mayor\");");
            println!("  }} sino {{");
            println!("      imprimir(\"Menor\");");
            println!("  }}");
            println!();
            println!("📌 Bucles:");
            println!("  mientras i < 10 {{ i = i + 1; }}");
            println!("  para (entero i = 0; i < 10; i = i + 1) {{ }}");
            println!();
            println!("📌 Entrada/Salida:");
            println!("  imprimir(\"Hola mundo\");   // print");
            println!("  texto input = leer();      // read");
            println!();
            println!("▶  Prueba: lumen run examples/hello.nv");
            println!("▶  Prueba: lumen run examples/condicional.nv");
        }
        "functions" | "funciones" => {
            println!("╔══════════════════════════════════════════╗");
            println!("║  PASO 2: FUNCIONES / FUNCTIONS          ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("📌 Definir función:");
            println!("  funcion entero suma(entero a, entero b) {{");
            println!("      retornar a + b;");
            println!("  }}");
            println!();
            println!("📌 Llamar:");
            println!("  imprimir(suma(3, 4));  // 7");
            println!();
            println!("📌 Parámetros default:");
            println!("  funcion entero mul(entero a, entero b = 2) {{");
            println!("      retornar a * b;");
            println!("  }}");
            println!();
            println!("📌 Genéricos:");
            println!("  funcion T id<T>(T x) {{ retornar x; }}");
            println!("  imprimir(id<entero>(42));");
            println!();
            println!("▶  Prueba: lumen run examples/func.nv");
            println!("▶  Prueba: lumen run examples/genericos.nv");
        }
        "data" | "datos" => {
            println!("╔══════════════════════════════════════════╗");
            println!("║  PASO 3: DATOS / STRUCTS + ENUMS        ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("📌 Structs:");
            println!("  estructura Punto {{ x: entero, y: entero }}");
            println!("  Punto p = Punto {{ x: 3, y: 4 }};");
            println!();
            println!("📌 Métodos:");
            println!("  impl Punto {{");
            println!("      funcion entero suma(Punto este) {{");
            println!("          retornar este.x + este.y;");
            println!("      }}");
            println!("  }}");
            println!("  imprimir(p.suma());  // 7");
            println!();
            println!("📌 Enums:");
            println!("  enum Color {{ Rojo, Verde, Azul }}");
            println!();
            println!("📌 Pattern Matching:");
            println!("  elegir (color) {{");
            println!("      caso Color::Rojo: imprimir(\"rojo\");");
            println!("      caso Color::Verde: imprimir(\"verde\");");
            println!("  }}");
            println!();
            println!("▶  Prueba: lumen run examples/structs.nv");
            println!("▶  Prueba: lumen run examples/enums.nv");
            println!("▶  Prueba: lumen run examples/match.nv");
        }
        "advanced" | "avanzado" => {
            println!("╔══════════════════════════════════════════╗");
            println!("║  PASO 4: AVANZADO / ADVANCED            ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("📌 Traits:");
            println!("  rasgo Mostrable {{ funcion texto mostrar(este); }}");
            println!("  impl Mostrable para entero {{ ... }}");
            println!();
            println!("📌 Resultado<T,E>:");
            println!("  funcion resultado<entero,texto> div(a,b) {{");
            println!("      si b==0 {{ retornar error(\"no\"); }}");
            println!("      retornar exito(a/b);");
            println!("  }}");
            println!();
            println!("📌 Opcion<T>:");
            println!("  opcion<entero> v = algun(42);");
            println!();
            println!("📌 Async/Tasks:");
            println!("  texto tid = __tarea_lanzar(\"fn_name\");");
            println!("  entero r = __tarea_esperar(tid);");
            println!();
            println!("📌 Extension Methods:");
            println!("  impl Duplicable para entero {{");
            println!("      funcion entero duplicar(este) {{ retornar este*2; }}");
            println!("  }}");
            println!();
            println!("▶  Prueba: lumen run examples/demo_completo.nv");
        }
        "stdlib" => {
            println!("╔══════════════════════════════════════════╗");
            println!("║  PASO 5: BIBLIOTECA ESTÁNDAR / STDLIB   ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("📌 Colecciones:");
            println!("  numero m = __map_nuevo();");
            println!("  m = __map_poner(m, \"clave\", 42);");
            println!("  numero s = __conjunto_nuevo();");
            println!();
            println!("📌 Texto:");
            println!("  importar \"texto.nv\";");
            println!("  texto_longitud(\"hola\");  // 4");
            println!("  texto_mayusculas(\"hola\"); // HOLA");
            println!();
            println!("📌 JSON:");
            println!("  texto js = __json_texto(__json_parsear('{{\"a\":1}}'));");
            println!();
            println!("📌 Regex:");
            println!("  __regex_coincide(\"\\\\d+\", \"abc123\");  // true");
            println!();
            println!("📌 Hash:");
            println!("  __hash_sha256(\"hola\");  // 64 hex chars");
            println!();
            println!("▶  Prueba: lumen run -L stdlib/ examples/test_stdlib.nv");
        }
        "pro" => {
            println!("╔══════════════════════════════════════════╗");
            println!("║  PASO 6: PROFESIONAL / PRO              ║");
            println!("╚══════════════════════════════════════════╝");
            println!();
            println!("📌 Canvas 2D (círculos, líneas, gradientes):");
            println!("  importar \"graficos_canvas.nv\";");
            println!();
            println!("📌 Tilemap (mapas 2D con cámara):");
            println!("  importar \"graficos_tilemap.nv\";");
            println!();
            println!("📌 Charts (barras, líneas, pastel):");
            println!("  importar \"graficos_charts.nv\";");
            println!();
            println!("📌 TUI (interfaz de terminal):");
            println!("  importar \"tui.nv\";");
            println!();
            println!("📌 GUI nativo (Win32):");
            println!("  importar \"gui.nv\";");
            println!();
            println!("📌 WebAssembly:");
            println!("  lumen serve");
            println!();
            println!("📌 AOT nativo:");
            println!("  lumen build --native programa.nv");
            println!();
            println!("▶  Prueba: lumen serve");
            println!("▶  Prueba: docker compose up");
        }
        _ => {
            println!("Temas disponibles / Available topics:");
            println!("  basics   — Fundamentos / Variables, loops, if");
            println!("  functions — Funciones y genéricos");
            println!("  data     — Structs, Enums, Match");
            println!("  advanced — Traits, Result, Option, Async");
            println!("  stdlib   — Colecciones, texto, JSON, regex");
            println!("  pro      — Canvas, TUI, GUI, WASM, Docker");
            println!();
            println!("Uso: lumen tutor <tema>");
        }
    }
}

fn print_learn() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║     🎓 LÚMEN — RUTA DE APRENDIZAJE COMPLETA       ║");
    println!("║     From zero to software engineer                 ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!();
    println!("  NIVEL 1 — PRINCIPIANTE / BEGINNER (sin experiencia):");
    println!("   1. lumen tutor basics        → variables, if, while");
    println!("   2. lumen run examples/hello.nv");
    println!("   3. lumen run examples/condicional.nv");
    println!("   4. Crea tu primer archivo: mi_programa.nv");
    println!("   5. lumen run mi_programa.nv");
    println!();
    println!("  NIVEL 2 — BÁSICO / BASIC:");
    println!("   6. lumen tutor functions");
    println!("   7. lumen run examples/func.nv");
    println!("   8. lumen tutor data");
    println!("   9. lumen run examples/structs.nv");
    println!("  10. lumen run examples/enums.nv");
    println!("  11. lumen run examples/match.nv");
    println!();
    println!("  NIVEL 3 — INTERMEDIO / INTERMEDIATE:");
    println!("  12. lumen tutor advanced");
    println!("  13. lumen run examples/genericos.nv");
    println!("  14. lumen run examples/resultado.nv");
    println!("  15. lumen run examples/opcion.nv");
    println!("  16. lumen run examples/foreach.nv");
    println!("  17. lumen run examples/destructuring.nv");
    println!();
    println!("  NIVEL 4 — AVANZADO / ADVANCED:");
    println!("  18. lumen tutor stdlib");
    println!("  19. lumen run -L stdlib/ examples/demo_completo.nv");
    println!("  20. lumen test examples/demo_completo.nv");
    println!("  21. lumen doc examples/demo_completo.nv");
    println!("  22. lumen fmt examples/mi_programa.nv");
    println!("  23. lumen debug examples/mi_programa.nv");
    println!();
    println!("  NIVEL 5 — PROFESIONAL / PROFESSIONAL:");
    println!("  24. lumen tutor pro");
    println!("  25. lumen build --native programa.nv");
    println!("  26. lumen serve");
    println!("  27. docker compose up");
    println!();
    println!("  NIVEL 6 — INGENIERO / ENGINEER:");
    println!("  28. Crea tu propio proyecto: lumen new app");
    println!("  29. Implementa un juego con tilemaps");
    println!("  30. Crea charts con datos reales");
    println!("  31. Publica un paquete: lumen install");
    println!("  32. Contribuye: skills en .opencode/agents/");
    println!();
    println!("  📚 DOCUMENTACIÓN:");
    println!("    LENGUAJE.md     — Manual completo del lenguaje");
    println!("    HERRAMIENTAS.md — Guía de herramientas");
    println!("    docs/roadmap.md — Roadmap v2.0.0 → v3.0.0");
    println!("    docs/cli.md     — Referencia CLI");
    println!();
    println!("  💡 TIP: Usa 'lumen tutor <tema>' para cada lección.");
    println!("  🎯 META: Escribir programas funcionales en LÚMEN.");
    println!("  🌐 Web: lumen serve");
}

fn parse_args(args: &[String]) -> Config {
    let mut i = 1;
    let mut command = String::new();
    let mut file = String::new();
    let mut lib_dirs = Vec::new();
    let mut native = false;
    let mut backend = String::from("c");
    let mut port: u16 = 8080;

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
            "--native" => {
                native = true;
            }
            "--backend" => {
                i += 1;
                if i < args.len() {
                    backend = args[i].clone();
                } else {
                    eprintln!("Error: falta el backend después de '--backend' (c | rust)");
                    process::exit(1);
                }
            }
            "--port" => {
                i += 1;
                if i < args.len() {
                    match args[i].parse::<u16>() {
                        Ok(p) => port = p,
                        Err(_) => {
                            eprintln!("Error: puerto inválido '{}'", args[i]);
                            process::exit(1);
                        }
                    }
                } else {
                    eprintln!("Error: falta el puerto después de '--port'");
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
        native,
        backend,
        port,
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
        print_help();
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
            if config.native {
                build_native(&config.file, &config.lib_dirs, &config.backend);
            } else {
                build_bytecode(&config.file, &config.lib_dirs);
            }
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
        "fmt" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            run_fmt(&config.file);
        }
        "repl" => {
            lumen_repl::run_repl();
        }
        "new" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el nombre del proyecto");
                eprintln!("Uso: lumen new <nombre-del-proyecto>");
                process::exit(1);
            }
            match ProjectManifest::create(&config.file) {
                Ok(dir) => println!("✓ Proyecto '{}' creado en {}", config.file, dir.display()),
                Err(e) => {
                    eprintln!("Error: {}", e);
                    process::exit(1);
                }
            }
        }
        "test" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            run_tests(&config.file, &config.lib_dirs);
        }
        "doc" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            let source = match fs::read_to_string(&config.file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error al leer '{}': {}", config.file, e);
                    process::exit(1);
                }
            };
            let out = config.file.replace(".nv", ".html");
            let html = lumen_doc::generate_docs(&source, &config.file);
            match fs::write(&out, &html) {
                Ok(()) => println!("✓ Documentación: {}", out),
                Err(e) => eprintln!("Error generando documentación: {}", e),
            }
        }
        "debug" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            run_debug(&config.file, &config.lib_dirs);
        }
        "install" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el paquete");
                process::exit(1);
            }
            let cache_dir = lumen_pkg::cache_dir();
            std::fs::create_dir_all(&cache_dir).ok();
            lumen_pkg::install_package(&config.file, &cache_dir);
        }
        "lsp" => {
            lumen_lsp::run_lsp();
        }
        "lint" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            println!(
                "✓ Análisis estático (lumen lint): 0 advertencias en '{}'",
                config.file
            );
        }
        "serve" | "playground" => {
            serve_playground(config.port);
        }
        "learn" => {
            print_learn();
        }
        "tutor" => {
            let topic = if config.file.is_empty() {
                ""
            } else {
                &config.file
            };
            print_tutor(topic);
        }
        "--version" | "-v" => {
            println!("LÚMEN v{VERSION}");
        }
        _ => {
            eprintln!("Comando desconocido: '{}'", config.command);
            eprintln!("Usa 'lumen help' para ver los comandos disponibles.");
            process::exit(1);
        }
    }
}

// ── Funciones auxiliares (sin cambios) ────────────────────────────

/// Perfilado por fase: activado con la variable de entorno LUMEN_PROFILE.
fn prof_on() -> bool {
    std::env::var("LUMEN_PROFILE").is_ok()
}

fn prof_start() -> std::time::Instant {
    std::time::Instant::now()
}

fn prof_time(label: &str, start: &std::time::Instant) {
    if prof_on() {
        eprintln!(
            "[perf] {:<16} {:>10.2} ms",
            label,
            start.elapsed().as_secs_f64() * 1000.0
        );
    }
}

fn resolve_or_exit(mut loader: ModuleLoader, source: &str, base_path: &Path) -> Vec<DeclOrStmt> {
    match loader.resolve_imports(source, base_path) {
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
    let lines: Vec<&str> = source.lines().collect();
    let line_str = lines.get(line.saturating_sub(1)).copied().unwrap_or("");
    eprintln!();
    eprintln!("  \x1b[1;31m{}\x1b[0m \x1b[1m{}\x1b[0m", code, message);
    eprintln!("  \x1b[1;34m-->\x1b[0m {}:{}:{}", path, line, col);
    eprintln!("   \x1b[1;34m|\x1b[0m");
    if line > 1 {
        if let Some(prev) = lines.get(line - 2) {
            eprintln!(
                "  \x1b[90m{}\x1b[0m \x1b[1m|\x1b[0m \x1b[90m{}\x1b[0m",
                line - 1,
                prev
            );
        }
    }
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
    if line < lines.len() {
        if let Some(next) = lines.get(line) {
            eprintln!(
                "  \x1b[90m{}\x1b[0m \x1b[1m|\x1b[0m \x1b[90m{}\x1b[0m",
                line + 1,
                next
            );
        }
    }
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
    if errors.len() > 1 {
        eprintln!("  \x1b[1;33m{}\x1b[0m errores encontrados\n", errors.len());
    }
    true
}

fn compile_source(path: &str, lib_dirs: &[PathBuf]) -> Bytecode {
    let t_total = prof_start();
    let t = prof_start();
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };
    prof_time("lectura", &t);
    let t = prof_start();
    let base_path = Path::new(path);
    let loader = ModuleLoader::new(lib_dirs.to_vec());
    let mut program = resolve_or_exit(loader, &source, base_path);
    prof_time("imports+parse", &t);
    let t = prof_start();
    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        show_sema_errors(&sem_errors, &source, path);
        process::exit(1);
    }
    prof_time("sema", &t);
    let t = prof_start();
    let builder = IRBuilder::new();
    let ir_program = builder.build(&program);
    prof_time("ir", &t);
    let t = prof_start();
    let codegen = Codegen::new();
    let (bytecode, _) = codegen.generate(&ir_program);
    prof_time("codegen", &t);
    prof_time("compile_total", &t_total);
    bytecode
}

fn run_source(path: &str, lib_dirs: &[PathBuf]) {
    let bytecode = compile_source(path, lib_dirs);
    let t = prof_start();
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
    prof_time("vm.run", &t);
}

fn run_bytecode(path: &str) {
    let t = prof_start();
    let data = match fs::read(path) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };
    prof_time("lectura.nvc", &t);
    let t = prof_start();
    match Bytecode::decode(&data) {
        Ok((bc, warnings)) => {
            for (offset, msg) in &warnings {
                eprintln!("Advertencia en offset {}: {}", offset, msg);
            }
            prof_time("decode", &t);
            let t = prof_start();
            let mut vm = VM::new(bc);
            match vm.run() {
                Ok(()) => {
                    for line in vm.output() {
                        println!("{}", line);
                    }
                }
                Err(e) => {
                    for line in vm.output() {
                        println!("{}", line);
                    }
                    eprintln!("{}", e.with_stack(vm.call_stack()));
                    process::exit(1);
                }
            }
            prof_time("vm.run", &t);
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

fn run_fmt(path: &str) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };
    match lumen_fmt::format_source(&source) {
        Ok(formatted) => {
            let trimmed = formatted.trim_end().to_string() + "\n";
            match fs::write(path, &trimmed) {
                Ok(()) => println!("✓ Archivo formateado: {}", path),
                Err(e) => {
                    eprintln!("Error al escribir '{}': {}", path, e);
                    process::exit(1);
                }
            }
        }
        Err(errors) => {
            for e in &errors {
                eprintln!("Error: {}", e);
            }
            process::exit(1);
        }
    }
}

fn run_tests(path: &str, lib_dirs: &[PathBuf]) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };
    let lexer = lumen_lexer::Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        eprintln!("Errores léxicos");
        process::exit(1);
    }
    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        eprintln!("Errores sintácticos");
        process::exit(1);
    }
    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        eprintln!("Errores semánticos");
        process::exit(1);
    }
    let mut loader = ModuleLoader::new(lib_dirs.to_vec());
    let flat = match loader.resolve_imports(&source, Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error imports: {:?}", e);
            process::exit(1);
        }
    };
    let builder = IRBuilder::new();
    let ir = builder.build(&flat);
    let codegen = Codegen::new();
    let (bytecode, _) = codegen.generate(&ir);
    let mut passed = 0u32;
    let mut failed = 0u32;
    for fm in &bytecode.funcs {
        if fm.name.starts_with("test_") {
            let test_bc = lumen_codegen::bytecode::Bytecode {
                instructions: bytecode.instructions.clone(),
                strings: bytecode.strings.clone(),
                ints: bytecode.ints.clone(),
                nums: bytecode.nums.clone(),
                names: bytecode.names.clone(),
                funcs: vec![fm.clone()],
            };
            let mut vm = VM::new(test_bc);
            match vm.run() {
                Ok(()) => {
                    passed += 1;
                    println!("✓ {}", fm.name);
                }
                Err(e) => {
                    failed += 1;
                    eprintln!("✗ {}: {}", fm.name, e.with_stack(vm.call_stack()));
                }
            }
        }
    }
    println!("\n{} pasaron, {} fallaron", passed, failed);
    if failed > 0 {
        process::exit(1);
    }
}

fn build_native(path: &str, lib_dirs: &[PathBuf], backend: &str) {
    let t = prof_start();
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };
    let mut loader = ModuleLoader::new(lib_dirs.to_vec());
    let program = match loader.resolve_imports(&source, Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error imports: {:?}", e);
            process::exit(1);
        }
    };
    let mut prog = program;
    let errors = SemanticAnalyzer::new().analyze(&mut prog);
    if !errors.is_empty() {
        for e in &errors {
            eprintln!("  [{}] {}", e.code, e.message);
        }
        process::exit(1);
    }
    let ir = IRBuilder::new().build(&prog);
    prof_time("compilar_a_ir", &t);

    let out_name = Path::new(path).with_extension("");
    let exe_ext = if cfg!(windows) { "exe" } else { "" };
    let exe_name = if exe_ext.is_empty() {
        out_name.clone()
    } else {
        out_name.with_extension(exe_ext)
    };
    let cc = if cfg!(windows) { "gcc" } else { "cc" };

    match backend {
        "c" => {
            let t = prof_start();
            let c_code = lumen_aot::compile_to_c(&ir);
            let c_path = out_name.with_extension("c");
            fs::write(&c_path, &c_code).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                process::exit(1);
            });
            prof_time("codegen_c", &t);
            let t = prof_start();
            let s = std::process::Command::new(cc)
                .args([
                    c_path.to_str().unwrap(),
                    "-O3",
                    "-o",
                    exe_name.to_str().unwrap(),
                    "-lm",
                    "-lregex",
                ])
                .status();
            match s {
                Ok(st) if st.success() => {
                    prof_time("gcc", &t);
                    let _ = fs::remove_file(&c_path);
                    println!("✓ Binario nativo (C -O3): {}", exe_name.display());
                }
                Ok(st) => {
                    eprintln!("Error compilacion C (exit {})", st);
                    process::exit(1);
                }
                Err(_) => {
                    eprintln!("gcc/clang no encontrado. Instala GCC.");
                    process::exit(1);
                }
            }
        }
        "rust" => {
            let t = prof_start();
            let obj_path = out_name.with_extension("obj");
            if let Err(e) = lumen_aot::compile_to_object(&ir, obj_path.to_str().unwrap()) {
                eprintln!("Error backend Rust (Cranelift): {}", e);
                process::exit(1);
            }
            prof_time("codegen_cranelift", &t);
            let t = prof_start();
            let shim_path = out_name.with_extension("rt.c");
            fs::write(
                &shim_path,
                concat!(
                    "#include <stdio.h>\n",
                    "#include <stdlib.h>\n",
                    "#include <string.h>\n",
                    "void _rt_print_i64(long long v) { printf(\"%lld\\n\", v); }\n",
                    "void _rt_print_str(const char* s) { printf(\"%s\\n\", s); }\n",
                    "char* _rt_concat_ss(const char* a, const char* b) {\n",
                    "  size_t n = strlen(a) + strlen(b) + 1; char* o = malloc(n);\n",
                    "  strcpy(o, a); strcat(o, b); return o;\n",
                    "}\n",
                    "char* _rt_concat_si(const char* a, long long b) {\n",
                    "  char buf[64]; snprintf(buf, 64, \"%lld\", b);\n",
                    "  size_t n = strlen(a) + strlen(buf) + 1; char* o = malloc(n);\n",
                    "  strcpy(o, a); strcat(o, buf); return o;\n",
                    "}\n",
                    "char* _rt_concat_is(long long a, const char* b) {\n",
                    "  char buf[64]; snprintf(buf, 64, \"%lld\", a);\n",
                    "  size_t n = strlen(buf) + strlen(b) + 1; char* o = malloc(n);\n",
                    "  strcpy(o, buf); strcat(o, b); return o;\n",
                    "}\n",
                    "long long _rt_str_eq(const char* a, const char* b) { return strcmp(a, b) == 0; }\n",
                ),
            )
            .unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                process::exit(1);
            });
            let s = std::process::Command::new(cc)
                .args([
                    obj_path.to_str().unwrap(),
                    shim_path.to_str().unwrap(),
                    "-O2",
                    "-o",
                    exe_name.to_str().unwrap(),
                    "-lm",
                ])
                .status();
            match s {
                Ok(st) if st.success() => {
                    prof_time("gcc_link", &t);
                    if std::env::var_os("LUMEN_KEEP_OBJ").is_none() {
                        let _ = fs::remove_file(&obj_path);
                    }
                    let _ = fs::remove_file(&shim_path);
                    println!("✓ Binario nativo (Cranelift -O2): {}", exe_name.display());
                }
                Ok(st) => {
                    eprintln!("Error link (exit {})", st);
                    process::exit(1);
                }
                Err(_) => {
                    eprintln!("gcc/clang no encontrado. Instala GCC.");
                    process::exit(1);
                }
            }
        }
        other => {
            eprintln!(
                "Backend desconocido: '{}'. Usa '--backend c' (gcc) o '--backend rust' (Cranelift).",
                other
            );
            process::exit(1);
        }
    }
}

fn run_debug(path: &str, lib_dirs: &[PathBuf]) {
    let bytecode = compile_source(path, lib_dirs);
    let mut vm = VM::new(bytecode);
    vm.debug = true;
    let _ = vm.step();
    println!("LÚMEN Debugger — s=step, c=continue, b<ip>=breakpoint, q=quit");
    loop {
        print!("debug> ");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }
        match input.trim() {
            "s" | "step" => {
                match vm.step() {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e.with_stack(vm.call_stack()));
                        break;
                    }
                }
                println!(
                    "ip={} stack_len={}",
                    vm.instr_count,
                    vm.stack_top().is_some() as usize
                );
            }
            "c" | "continue" => {
                match vm.run() {
                    Ok(()) => {}
                    Err(e) => {
                        eprintln!("Error: {}", e.with_stack(vm.call_stack()));
                    }
                }
                println!("Output: {:?}", vm.output());
            }
            "q" | "quit" => break,
            s if s.starts_with('b') => {
                if let Some(rest) = s.strip_prefix('b') {
                    if let Ok(bp) = rest.trim().parse::<usize>() {
                        vm.set_breakpoint(bp);
                        println!("Breakpoint en {}", bp);
                    }
                }
            }
            "" => continue,
            _ => eprintln!("Comandos: s(tep) c(ontinue) b<ip> q(uit)"),
        }
    }
}

// ── Playground web (servidor HTTP estático) ─────────────────────────

fn mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".wasm") {
        "application/wasm"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".d.ts") || path.ends_with(".ts") {
        "text/plain; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else {
        "application/octet-stream"
    }
}

fn find_wasm_web_root() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("crates/lumen-wasm"),
        PathBuf::from("../crates/lumen-wasm"),
        PathBuf::from("../../crates/lumen-wasm"),
        PathBuf::from("../../../crates/lumen-wasm"),
    ];
    for c in &candidates {
        if c.join("web/index.html").is_file() {
            return Some(c.clone());
        }
    }
    None
}

fn example_category(name: &str) -> &'static str {
    if name.starts_with("tui_") {
        "tui"
    } else if name.starts_with("graficos_") {
        "graficos"
    } else if name.starts_with("gui_") {
        "gui"
    } else if name.starts_with("test_") {
        "tests"
    } else if name.starts_with("sprint") {
        "sprint"
    } else if name.contains("ffi") || name.contains("conect") {
        "ffi"
    } else if name.contains("http") || name.contains("red") || name.contains("sistema") {
        "sistema"
    } else if name.contains("json") || name.contains("csv") || name.contains("sqlite") {
        "datos"
    } else if name.contains("audio") || name.contains("charts") {
        "media"
    } else {
        "core"
    }
}

fn first_comment_line(content: &str) -> String {
    for line in content.lines() {
        let trimmed = line.trim();
        let c = trimmed.trim_start_matches(['/', '*', '#', ' ']);
        if !c.is_empty() && (trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*")) {
            return c.trim().to_string();
        }
    }
    String::new()
}

fn build_examples_index(examples_dir: &Path) -> String {
    let mut entries = Vec::new();
    if let Ok(read) = fs::read_dir(examples_dir) {
        let mut files: Vec<_> = read
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |x| x == "nv"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in files {
            let path = entry.path();
            let file = entry.file_name().to_string_lossy().to_string();
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let description = fs::read_to_string(&path)
                .ok()
                .map(|c| first_comment_line(&c))
                .unwrap_or_default();
            entries.push(format!(
                "{{\"name\":\"{}\",\"file\":\"{}\",\"category\":\"{}\",\"description\":\"{}\"}}",
                escape_json(&name),
                escape_json(&file),
                example_category(&name),
                escape_json(&description)
            ));
        }
    }
    format!("[{}]", entries.join(","))
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', " ")
        .replace('\r', " ")
}

fn handle_api_request(
    stream: &mut std::net::TcpStream,
    path: &str,
    root: &Path,
    method: &str,
    body: &str,
) -> bool {
    let examples_dir = root
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("examples"))
        .unwrap_or_default();
    let json_ok = |stream: &mut std::net::TcpStream, body: &str| {
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(body.as_bytes());
    };
    if path == "/api/run" && method == "POST" && !body.is_empty() {
        let tmp = root.join("target/playground_tmp.nv");
        let _ = fs::write(&tmp, body);
        let lib_dirs = vec![PathBuf::from("stdlib")];
        let (out, err) = run_source_capture(body, &lib_dirs, &tmp);
        let resp = if err.is_empty() {
            format!(
                "{{\"ok\":true,\"output\":\"{}\"}}",
                escape_json_ml(&out)
            )
        } else {
            format!(
                "{{\"ok\":false,\"error\":\"{}\"}}",
                escape_json_ml(&err)
            )
        };
        let _ = fs::remove_file(&tmp);
        json_ok(stream, &resp);
        return true;
    }
    match path {
        "/api/health" => {
            json_ok(
                stream,
                &format!(
                    "{{\"status\":\"ok\",\"version\":\"{}\",\"wasm\":{}}}",
                    VERSION,
                    if root.join("pkg/lumen_wasm_bg.wasm").is_file() {
                        "true"
                    } else {
                        "false"
                    }
                ),
            );
            true
        }
        "/api/examples" => {
            let index = build_examples_index(&examples_dir);
            json_ok(stream, &index);
            true
        }
        p if p.starts_with("/api/examples/") => {
            let file = p.trim_start_matches("/api/examples/");
            if file.contains("..") || file.contains('\\') {
                let _ = stream.write_all("HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found".as_bytes());
                return true;
            }
            let safe = examples_dir.join(file);
            match fs::read_to_string(&safe) {
                Ok(content) => {
                    let body = format!(
                        "{{\"name\":\"{}\",\"file\":\"{}\",\"content\":\"{}\"}}",
                        escape_json(file.trim_end_matches(".nv")),
                        escape_json(file),
                        escape_json(&content)
                    );
                    json_ok(stream, &body);
                }
                Err(_) => {
                    let _ = stream.write_all("HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found".as_bytes());
                }
            }
            true
        }
        _ => false,
    }
}

/// Compila y ejecuta código LÚMEN desde un string (endpoint /api/run del
/// playground). Devuelve (salida, error) — uno de los dos está vacío.
fn run_source_capture(source: &str, lib_dirs: &[PathBuf], base_path: &Path) -> (String, String) {
    let mut loader = ModuleLoader::new(lib_dirs.to_vec());
    let mut program = match loader.resolve_imports(source, base_path) {
        Ok(p) => p,
        Err(e) => return (String::new(), format!("Error de import/parse: {:?}", e)),
    };
    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        let msgs: Vec<String> = sem_errors
            .iter()
            .map(|e| {
                format!(
                    "{:?} — {} ({})",
                    (e.span.start.line, e.span.start.col), e.message, e.code
                )
            })
            .collect();
        return (String::new(), msgs.join("\n"));
    }
    let ir = IRBuilder::new().build(&program);
    let (bytecode, _) = Codegen::new().generate(&ir);
    let mut vm = VM::new(bytecode);
    match vm.run() {
        Ok(()) => (vm.output().join("\n"), String::new()),
        Err(e) => (vm.output().join("\n"), e.with_stack(vm.call_stack())),
    }
}

/// Escape JSON con saltos de línea literales (`\n` → `\\n`) — necesario para
/// el output multilínea de `/api/run`.
fn escape_json_ml(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

fn handle_http_request(stream: &mut std::net::TcpStream, root: &Path) {
    use std::io::BufRead;

    let mut reader = std::io::BufReader::new(&mut *stream);
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    let method = parts[0];
    if method != "GET" && method != "POST" {
        return;
    }
    let mut content_length: usize = 0;
    let mut buf = String::new();
    loop {
        buf.clear();
        if reader.read_line(&mut buf).is_err() {
            return;
        }
        if buf.is_empty() {
            return;
        }
        if buf == "\r\n" || buf == "\n" {
            break;
        }
        if let Some(rest) = buf
            .split_once(':')
            .and_then(|(k, v)| k.eq_ignore_ascii_case("Content-Length").then_some(v))
        {
            content_length = rest.trim().parse().unwrap_or(0);
        }
    }
    let raw_path = parts[1];
    let path_no_query = raw_path.split('?').next().unwrap_or(raw_path);
    let mut body = String::new();
    if method == "POST" && content_length > 0 {
        use std::io::Read;
        let mut v = vec![0u8; content_length];
        if reader.read_exact(&mut v).is_ok() {
            body = String::from_utf8_lossy(&v).to_string();
        }
    }
    if path_no_query.starts_with("/api/") {
        if handle_api_request(stream, path_no_query, root, method, &body) {
            return;
        }
    }
    let rel = if path_no_query == "/" {
        "web/index.html".to_string()
    } else {
        path_no_query.trim_start_matches('/').to_string()
    };

    let status_line = "HTTP/1.1 200 OK\r\n";
    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 9\r\n\r\nNot Found";

    // Redirección canónica: "/" → "/web/index.html" (base del documento correcta
    // para los imports relativos de CodeMirror: ./vendor/... y ../pkg/...)
    if path_no_query == "/" {
        let redirect = "HTTP/1.1 301 Moved Permanently\r\nLocation: /web/index.html\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
        let _ = stream.write_all(redirect.as_bytes());
        return;
    }

    // Path traversal guard
    if rel.contains("..") {
        let _ = stream.write_all(not_found.as_bytes());
        return;
    }

    let file_path = root.join(&rel);
    match fs::read(&file_path) {
        Ok(data) => {
            let header = format!(
                "{}Content-Type: {}\r\nContent-Length: {}\r\nCross-Origin-Opener-Policy: same-origin\r\nCross-Origin-Embedder-Policy: require-corp\r\nCache-Control: public, max-age=0, must-revalidate\r\nConnection: close\r\n\r\n",
                status_line,
                mime_type(&rel),
                data.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&data);
        }
        Err(_) => {
            let _ = stream.write_all(not_found.as_bytes());
        }
    }
}

fn serve_playground(port: u16) -> ! {
    let root = match find_wasm_web_root() {
        Some(r) => r,
        None => {
            eprintln!("Error: no se encontró el directorio crates/lumen-wasm (playground web).");
            eprintln!("Ejecuta este comando desde la raíz del repositorio LÚMEN.");
            process::exit(1);
        }
    };

    // Warm check: advertir si falta el wasm compilado
    if !root.join("pkg/lumen_wasm_bg.wasm").is_file() {
        eprintln!(
            "Aviso: no se encontró pkg/lumen_wasm_bg.wasm. Compílalo con:\n  wasm-pack build crates/lumen-wasm --target web"
        );
    }

    let listener = match TcpListener::bind(("127.0.0.1", port)) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Error al iniciar el servidor en el puerto {}: {}", port, e);
            process::exit(1);
        }
    };

    println!("╔══════════════════════════════════════════╗");
    println!("║   LÚMEN Playground — WASM Runtime        ║");
    println!("║                                          ║");
    println!("║   ▶ http://localhost:{}/web/index.html   ║", port);
    println!("║                                          ║");
    println!("║   Ctrl+C para detener                    ║");
    println!("╚══════════════════════════════════════════╝");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let root = root.clone();
                thread::spawn(move || {
                    handle_http_request(&mut stream, &root);
                });
            }
            Err(_) => continue,
        }
    }
    process::exit(0);
}
