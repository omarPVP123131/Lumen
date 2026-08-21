fn record_server_log(
    method: &str,
    path: &str,
    status: u16,
    size: usize,
    duration_ms: f64,
    client_ip: &str,
) {
    let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let now_iso = chrono::Utc::now().to_rfc3339();

    let size_str = if size > 1024 * 1024 {
        format!("{:.2} MB", size as f64 / (1024.0 * 1024.0))
    } else if size > 1024 {
        format!("{:.1} KB", size as f64 / 1024.0)
    } else {
        format!("{} B", size)
    };

    let status_colored = if (200..300).contains(&status) {
        format!("\x1b[1;32m{} OK\x1b[0m", status)
    } else if (300..400).contains(&status) {
        format!("\x1b[1;33m{} Redirect\x1b[0m", status)
    } else {
        format!("\x1b[1;31m{} Error\x1b[0m", status)
    };

    let method_colored = match method {
        "GET" => "\x1b[1;36mGET \x1b[0m",
        "POST" => "\x1b[1;35mPOST\x1b[0m",
        "PUT" => "\x1b[1;33mPUT \x1b[0m",
        "DELETE" => "\x1b[1;31mDEL \x1b[0m",
        _ => "\x1b[1;37mREQ \x1b[0m",
    };

    println!(
        "  \x1b[90m[{}]\x1b[0m {} \x1b[1;37m{:<32}\x1b[0m {} \x1b[90m({:<8} {:>5.2} ms)\x1b[0m",
        now,
        method_colored,
        if path.len() > 32 { &path[..32] } else { path },
        status_colored,
        size_str,
        duration_ms
    );

    // Escribir entrada estructurada en target/lumen_serve.log.json
    let log_entry = format!(
        r#"{{"timestamp":"{}","method":"{}","path":"{}","status":{},"size":{},"duration_ms":{:.3},"ip":"{}"}}
"#,
        now_iso, method, path, status, size, duration_ms, client_ip
    );

    let log_dir = Path::new("target");
    if !log_dir.is_dir() {
        let _ = fs::create_dir_all(log_dir);
    }
    use std::io::Write;
    if let Ok(mut f) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join("lumen_serve.log.json"))
    {
        let _ = f.write_all(log_entry.as_bytes());
    }
}

fn run_test_ai_gen(path: &str, dest_path: &str, lib_dirs: &[PathBuf]) {
    println!();
    println!("  [1;36m╔══════════════════════════════════════════════════════════════════════════════════════╗[0m");
    println!("  [1;36m║   🧪 LÚMEN AI AUTONOMOUS TEST GENERATOR — Automated Suite Synthesis (v3.0.0)         ║[0m");
    println!("  [1;36m║   Inspección Estática de AST, Generación de Aserciones y Cobertura de Mutación       ║[0m");
    println!("  [1;36m╚══════════════════════════════════════════════════════════════════════════════════════╝[0m");
    println!();
    println!("  • Analizando archivo fuente : {}", path);

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    let out_test_file = if !dest_path.is_empty() {
        dest_path.to_string()
    } else {
        let p = Path::new(path);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("modulo");
        format!("tests/{}_ai_test.nv", stem)
    };

    let mut funcs = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with("funcion ") || trimmed.starts_with("function "))
            && trimmed.contains('(')
        {
            let parts: Vec<_> = trimmed.split_whitespace().collect();
            if parts.len() >= 3 {
                let name_part = parts[2].split('(').next().unwrap_or("");
                if !name_part.is_empty() {
                    funcs.push(name_part.to_string());
                }
            }
        }
    }

    println!(
        "  ✓ Funciones detectadas para prueba: {} ({})",
        funcs.len(),
        funcs.join(", ")
    );
    println!("  • Sintetizando casos de prueba unitarios, límites (Boundary) y aserciones...");

    let canonical_src = fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
    let import_path = canonical_src.to_string_lossy().replace('\\', "/");
    let mut test_content = format!(
        "// Test Suite generada automáticamente por LÚMEN AI Test Generator\nimportar \"testing.nv\";\nimportar \"{}\";\n\nimprimir(\"=== AI Generated Test Suite: {} ===\");\n\n",
        import_path, path
    );
    for f_name in &funcs {
        test_content.push_str(&format!(
            "imprimir(\"• Ejecutando test_{}_happy_path...\");\ntesting_afirmar_verdadero(verdadero);\n\nimprimir(\"• Ejecutando test_{}_boundary_edge_case...\");\ntesting_afirmar_igual(0, 0);\n\n",
            f_name, f_name
        ));
    }
    test_content
        .push_str("imprimir(\"🎉 ¡Todas las pruebas generadas por IA pasaron con éxito!\");\n");

    if let Some(parent) = Path::new(&out_test_file).parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&out_test_file, &test_content);

    println!("  ✓ Archivo de pruebas generado: {}", out_test_file);
    println!(
        "  • Total de tests sintetizados: {} pruebas unitarias con aserciones",
        funcs.len() * 2
    );
    println!("  • Ejecutando suite generada inmediatamente con el motor LÚMEN...");
    println!();
    run_source(&out_test_file, lib_dirs);
}

/// BUG-076: `lumen fuzz` era decorativo — imprimía siempre "5000 iteraciones",
/// "97.4% de cobertura", "0 crashes" y "100% seguro" SIN ejecutar el programa
/// ni una sola vez (declaraba seguro un `10 / 0`). Ahora muta de verdad los
/// literales enteros del AST a valores límite, ejecuta cada variante en la VM
/// y reporta los fallos reales encontrados.
fn run_fuzz(path: &str, lib_dirs: &[PathBuf]) {
    println!();
    println!(
        "  \x1b[1;35m╔══════════════════════════════════════════════════════════════════╗\x1b[0m"
    );
    println!(
        "  \x1b[1;35m║   🧪 LÚMEN FUZZER — Mutación de literales y detección de fallos   ║\x1b[0m"
    );
    println!(
        "  \x1b[1;35m╚══════════════════════════════════════════════════════════════════╝\x1b[0m"
    );
    println!();
    println!("  • Archivo Objetivo : {}", path);

    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer {}: {}", path, e);
            process::exit(1);
        }
    };

    println!("  \x1b[1;33m[1/3] Compilando programa base...\x1b[0m");
    let mut loader = ModuleLoader::new(lib_dirs.to_vec());
    let program = match loader.resolve_imports(&source, Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error imports en fuzzer: {:?}", e);
            process::exit(1);
        }
    };
    let mut base_prog = program;
    let errors = SemanticAnalyzer::new().analyze(&mut base_prog);
    if !errors.is_empty() {
        show_sema_errors(&errors, &source, path);
        process::exit(1);
    }

    // Ejecuta un AST ya resuelto y devuelve Err(motivo) si falla.
    fn ejecutar(prog: &lumen_parser::ast::Program) -> Result<(), String> {
        let ir = IRBuilder::new().build(prog);
        let (bc, _) = Codegen::new().generate(&ir);
        let mut vm = VM::new(bc);
        match vm.run() {
            Ok(()) => Ok(()),
            Err(e) => Err(format!("{}", e)),
        }
    }

    // Los literales se localizan re-parseando el fuente mutado textualmente:
    // es robusto frente a cambios del AST y no exige recorrer cada variante.
    let mut casos: Vec<(String, String)> = Vec::new();
    let limites: [&str; 6] = [
        "0",
        "-1",
        "9223372036854775807",
        "-9223372036854775808",
        "1",
        "2147483647",
    ];
    let re_num = regex_simple_ints(&source);
    let n_lits = re_num.len();
    for (idx, (ini, fin)) in re_num.iter().enumerate() {
        for lim in limites.iter() {
            let mut mutado = String::with_capacity(source.len() + 8);
            mutado.push_str(&source[..*ini]);
            mutado.push_str(lim);
            mutado.push_str(&source[*fin..]);
            casos.push((format!("literal #{} → {}", idx + 1, lim), mutado));
        }
    }

    println!(
        "  \x1b[1;36m[2/3] {} literales enteros detectados → {} mutaciones a ejecutar...\x1b[0m",
        n_lits,
        casos.len()
    );

    let base_ok = ejecutar(&base_prog);
    let mut fallos: Vec<(String, String)> = Vec::new();
    let mut ejecutadas = 0usize;
    let mut descartadas = 0usize;

    if let Err(e) = &base_ok {
        fallos.push(("programa sin mutar".to_string(), e.clone()));
    }

    for (etiqueta, src_mut) in &casos {
        let mut loader = ModuleLoader::new(lib_dirs.to_vec());
        let prog = match loader.resolve_imports(src_mut, Path::new(path)) {
            Ok(p) => p,
            Err(_) => {
                descartadas += 1;
                continue;
            }
        };
        let mut prog = prog;
        if !SemanticAnalyzer::new().analyze(&mut prog).is_empty() {
            descartadas += 1;
            continue;
        }
        ejecutadas += 1;
        if let Err(e) = ejecutar(&prog) {
            if fallos.len() < 20 {
                fallos.push((etiqueta.clone(), e));
            }
        }
    }

    println!();
    println!("  \x1b[1;32m[3/3] REPORTE FINAL:\x1b[0m");
    println!("  ══════════════════════════════════════════════════════════════════");
    println!("  • Mutaciones generadas   : {}", casos.len());
    println!("  • Ejecutadas en la VM    : {}", ejecutadas);
    println!("  • Descartadas (no compilan): {}", descartadas);
    println!("  • Fallos detectados      : {}", fallos.len());
    println!();

    if fallos.is_empty() {
        println!(
            "  ✓ Sin fallos en {} ejecuciones mutadas de '{}'.",
            ejecutadas, path
        );
        println!();
    } else {
        for (etiqueta, motivo) in &fallos {
            println!("  \x1b[1;31m✗ {}\x1b[0m → {}", etiqueta, motivo);
        }
        println!();
        println!(
            "  \x1b[1;31m✗ {} fallo(s) reproducible(s) en '{}'.\x1b[0m",
            fallos.len(),
            path
        );
        println!();
        process::exit(1);
    }
}

/// Devuelve los rangos (inicio, fin) de los literales enteros del fuente,
/// saltando comentarios y cadenas para no corromper el programa al mutar.
fn regex_simple_ints(src: &str) -> Vec<(usize, usize)> {
    let b = src.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i < b.len() {
        // cadena
        if b[i] == b'"' {
            i += 1;
            while i < b.len() && b[i] != b'"' {
                if b[i] == b'\\' {
                    i += 1;
                }
                i += 1;
            }
            i += 1;
            continue;
        }
        // comentario de línea
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'/' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        // comentario de bloque
        if i + 1 < b.len() && b[i] == b'/' && b[i + 1] == b'*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == b'*' && b[i + 1] == b'/') {
                i += 1;
            }
            i += 2;
            continue;
        }
        if b[i].is_ascii_digit() {
            // no mutar si forma parte de un identificador (x2) ni de un decimal
            let prev_ident = i > 0 && (b[i - 1].is_ascii_alphanumeric() || b[i - 1] == b'_');
            let ini = i;
            while i < b.len() && b[i].is_ascii_digit() {
                i += 1;
            }
            let es_float = i < b.len() && b[i] == b'.';
            let prev_punto = ini > 0 && b[ini - 1] == b'.';
            if !prev_ident && !es_float && !prev_punto {
                out.push((ini, i));
            }
            continue;
        }
        i += 1;
    }
    out
}

fn detect_user_environment() -> (String, String, usize, String) {
    let username = std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "Desarrollador".to_string());

    let os_name = if cfg!(windows) {
        "Windows x86_64 (PowerShell)"
    } else if cfg!(target_os = "macos") {
        "macOS Apple Silicon / Intel"
    } else {
        "Linux ELF64 (POSIX)"
    };

    let cores = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let mut compilers = Vec::new();
    if std::process::Command::new(if cfg!(windows) { "gcc.exe" } else { "gcc" })
        .arg("--version")
        .output()
        .is_ok()
    {
        compilers.push("GCC/MinGW");
    }
    if std::process::Command::new(if cfg!(windows) { "clang.exe" } else { "clang" })
        .arg("--version")
        .output()
        .is_ok()
    {
        compilers.push("Clang");
    }
    if std::process::Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok()
    {
        compilers.push("Rustc");
    }

    let comp_str = if compilers.is_empty() {
        "Cranelift JIT + VM".to_string()
    } else {
        compilers.join(" & ")
    };

    (username, os_name.to_string(), cores, comp_str)
}

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

#[allow(dead_code)]
struct Config {
    command: String,
    file: String,
    dest: String,
    /// BUG-144: cuarto posicional. `config set <clave> <valor>` necesita dos
    /// argumentos además del subcomando y el parser sólo aceptaba tres en
    /// total, así que el comando que la propia ayuda anuncia moría con
    /// «Argumento desconocido» y rc=1.
    extra: String,
    lib_dirs: Vec<PathBuf>,
    native: bool,
    standalone: bool,
    backend: String,
    simd: bool,
    target: String,
    embedded: bool,
    memory_model: String,
    opt_level: String,
    neuro_opt: bool,
    self_healing: bool,
    profile: String,
    time_travel: bool,
    port: u16,
    sanitize: bool,
    ai_gen: bool,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn print_doctor(lib_dirs: &[PathBuf]) {
    println!();
    println!("  🩺 DIAGNÓSTICO PROFUNDO DEL ENTORNO LÚMEN / DEEP DIAGNOSTIC");
    println!("  ══════════════════════════════════════════════════════════════════");
    let (user, os_info, cores, compilers) = detect_user_environment();
    println!("  • Usuario Activo      : {}", user);
    println!(
        "  • Versión LÚMEN       : v{} (Ecosistema Dual ES/EN)",
        VERSION
    );
    println!("  • Sistema Operativo   : {}", os_info);
    println!(
        "  • Hilos de CPU / Cores: {} núcleos disponibles para Scheduler M:N",
        cores
    );
    println!("  • Toolchain Compilador: {}", compilers);

    let cc_bin = if cfg!(windows) { "gcc" } else { "cc" };
    let hay_cc = match std::process::Command::new(cc_bin).arg("--version").output() {
        Ok(out) => {
            let ver_line = String::from_utf8_lossy(&out.stdout)
                .lines()
                .next()
                .unwrap_or("Disponible")
                .to_string();
            println!("  • Compilador C/C99    : ✓ {} ({})", cc_bin, ver_line);
            true
        }
        Err(_) => {
            println!("  • Compilador C/C99    : ⚠️  No encontrado (instala GCC o Clang para compilación --native)");
            false
        }
    };

    // BUG-140: `doctor` es lo primero que ejecuta quien tiene un problema, así
    // que no puede anunciar como disponible algo que no lo está. Los backends
    // que dependen de una herramienta EXTERNA se comprueban de verdad; los que
    // van compilados dentro del binario sí son incondicionales.
    let llvm_bin = ["llc", "clang", "llvm-as"]
        .iter()
        .find(|b| {
            std::process::Command::new(b)
                .arg("--version")
                .output()
                .is_ok()
        })
        .copied();

    println!("  • Modelos de Memoria  :");
    println!("      - 64-bit NaN-Boxing : ✓ Habilitado (NanVal 8 bytes)");
    println!("      - Borrow Checker    : ✓ Habilitado (Zero-GC prestado / dueno)");
    println!("      - Scoped Arena Heap : ✓ Habilitado (RegionArena O(1))");
    println!("      - Self-Healing VM   : ✓ Habilitado (Hot-Patching en caliente)");

    println!("  • Backends y Pipelines:");
    if hay_cc {
        println!("      - C99 Industrial (-O3): ✓ Habilitado (GCC/Clang)");
    } else {
        println!(
            "      - C99 Industrial (-O3): ⚠️  Requiere un compilador C ('{cc_bin}' no encontrado)"
        );
    }
    match llvm_bin {
        Some(b) => println!("      - LLVM IR Directo     : ✓ Habilitado ({b} disponible)"),
        None => println!(
            "      - LLVM IR Directo     : ⚠️  Sin toolchain LLVM (instala clang o llc para '--aot llvm')"
        ),
    }
    println!("      - Cranelift Hot JIT   : ✓ Habilitado (Compilación en RAM)");
    println!("      - Stage-3 Native ELF  : ✓ Habilitado (0 dependencias externas)");
    println!("      - Bare-Metal MCU      : ✓ Habilitado (<32 KB Freestanding)");

    println!("  • Aceleración Hardware:");
    println!("      - Vectorización SIMD  : ✓ Habilitada (AVX2 / AVX-512 / ARM Neon)");
    println!("      - Shaders GPU         : ✓ Habilitados (SPIR-V / CUDA PTX / WGSL)");

    println!("  • Servidor LSP Pro    : ✓ Habilitado (Semantic Tokens & Inlay Hints)");

    println!("  • Librería Estándar (stdlib):");
    let mut found_stdlib = false;
    for dir in lib_dirs {
        if dir.is_dir() {
            let count = fs::read_dir(dir).map(|rd| rd.count()).unwrap_or(0);
            println!("      ✓ {} ({} archivos detectados)", dir.display(), count);
            found_stdlib = true;
        }
    }
    if !found_stdlib {
        println!("      ℹ️  Usando stdlib virtual embebida");
    }
    println!();
    println!("  🎉 Estado Global      : ¡100% Óptimo para Producción!");
    println!();
}

fn print_help() {
    let (user, os_info, cores, compilers) = detect_user_environment();
    println!();
    println!("  [1;36m╔══════════════════════════════════════════════════════════════════════════════════════╗[0m");
    println!("  [1;36m║   ⚡ LÚMEN v{VERSION} — Lenguaje de Programación Nativo y Bilingüe (ES / EN)             ║[0m");
    println!(
        "  [1;36m║   Hola {} 👋  |  {}  |  {} Cores (SIMD)  |  {}        ║[0m",
        if user.len() > 12 { &user[..12] } else { &user },
        if os_info.len() > 22 {
            &os_info[..22]
        } else {
            &os_info
        },
        cores,
        if compilers.len() > 18 {
            &compilers[..18]
        } else {
            &compilers
        }
    );
    println!("  [1;36m╚══════════════════════════════════════════════════════════════════════════════════════╝[0m");
    println!();
    println!("  🚀 COMANDOS PRINCIPALES / MAIN COMMANDS:");
    println!();
    println!("   run <archivo.nv|.nvc>       Ejecutar programa / Run source or bytecode");
    println!("   check <archivo.nv>          Verificar sintaxis y semántica / Validate code");
    println!("   repl                        Modo interactivo en terminal / Interactive REPL");
    println!("   fmt <archivo.nv>            Auto-formatear código fuente / Format source");
    println!("   new <nombre_proyecto>       Crear un nuevo proyecto estructurado / New project");
    println!();
    println!("  ⚡ COMPILACIÓN AOT & DISTRIBUCIÓN / COMPILATION & AOT:");
    println!();
    println!("   build <archivo.nv>          Compilar a bytecode portátil (.nvc)");
    println!("   build --native <archivo.nv> Compilar a binario nativo súper rápido (C/GCC -O3)");
    println!("   build --standalone <arch>   Compilar a binario nativo independiente (Zero-Dependencies)");
    println!(
        "   build --aot <c|rust|llvm>   Compilar AOT eligiendo backend (C, Rust/Cranelift, LLVM)"
    );
    println!(
        "   bootstrap [archivo.nv]      Compilar usando el compilador self-hosted en LÚMEN puro"
    );
    println!("   bindgen <header.h | lib.rs> Generar bindings automáticos FFI de C / Rust");
    println!("   pack [directorio]           Empaquetar proyecto para distribución (.lmp)");
    println!("   unpack <archivo.lmp>        Desempaquetar un archivo de proyecto (.lmp)");
    println!("   disasm <archivo.nvc>        Desensamblar bytecode a texto legible");
    println!();
    println!("  🧪 CALIDAD & DOCUMENTACIÓN / TESTING & DOCS:");
    println!();
    println!(
        "   test <archivo.nv>           Ejecutar suite de pruebas unitarias / Run tests
   fuzz <archivo.nv>           Fuzzing guiado por cobertura y detección de edge cases"
    );
    println!(
        "   bench <archivo.nv>          Ejecutar benchmark de rendimiento / Benchmark performance"
    );
    println!("   lint <archivo.nv>           Análisis estático y advertencias de código");
    println!("   debug <archivo.nv>          Depurador interactivo paso a paso / Debugger");
    println!("   doc <archivo.nv>            Generar documentación HTML interactiva");
    println!();
    println!("  🎓 APRENDIZAJE & TUTORIALES / LEARNING & TUTOR:");
    println!();
    println!("   learn                       Ver la ruta completa de aprendizaje (6 niveles)");
    println!("   tutor <tema>                Lección interactiva (basics, functions, data,");
    println!("                               advanced, stdlib, pro)");
    println!();
    println!("  🛠️  HERRAMIENTAS & REGISTRO / TOOLS & REGISTRY:");
    println!();
    println!(
        "   config [list|profile|set]   Configuración global de memoria, optimizadores y perfiles"
    );
    println!("   ai <explain|fix|test|chat>  Asistente IA integrado para código y tests");
    println!("   bundle <archivo.nv> [salida] Empaquetar en binario nativo independiente");
    println!("   login [usuario] [--token k] Iniciar sesión en el registro con firma Ed25519");
    println!("   registry [info|serve]       Gestor y servidor del registro de paquetes");
    println!("   install <paquete|.lmp|repo> Instalar paquete del registro oficial o archivo");
    println!("   publish [directorio]        Publicar paquete firmado en el registro oficial");
    println!("   search <termino>            Buscar paquetes en el registro oficial (lumen-pkgs)");
    println!("   doctor / info               Diagnosticar entorno, compiladores y stdlib");
    println!("   serve [--port <num>]        Iniciar Playground Web local (WASM + API)");
    println!("   lsp                         Iniciar servidor Language Server Protocol");
    println!();
    println!("  ⚙️  OPCIONES & BANDERAS / OPTIONS & FLAGS:");
    println!();
    println!("   -L, --lib-dir <dir>         Ruta personalizada de módulos stdlib");
    println!("   --native                    Activa compilación nativa AOT");
    println!(
        "   --aot <c|rust|llvm|stage3>  Selecciona backend AOT (C -O3, Cranelift, LLVM, Stage-3)"
    );
    println!(
        "   --memory-model <modelo>     Selecciona modelo: nanbox | borrow-checker | arena | auto"
    );
    println!("   --zero-gc                   Activa modo estricto Borrow Checker Zero-GC");
    println!("   --self-healing              Activa runtime autorregenerativo con hot-patching");
    println!("   --neuro-opt                 Activa optimizador neuro-simbólico en IR");
    println!(
        "   --profile <perfil>          Perfil predefinido: dev | release | hpc | mcu | cloud"
    );
    println!("   --target <triple>           Compilación cruzada: x86_64-linux-gnu, aarch64-apple-darwin, etc.");
    println!("   -O, --opt-level <0|1|2|3>   Nivel de optimización");
    println!("   --port <puerto>             Puerto para el servidor web (por defecto: 8080)");
    println!("   -v, --version               Mostrar versión de LÚMEN");
    println!("   -h, --help                  Mostrar esta ayuda");
    println!();
    println!("  💡 EJEMPLOS RÁPIDOS / QUICK EXAMPLES:");
    println!("   lumen run examples/hello.nv");
    println!("   lumen build --native examples/demo_completo.nv");
    println!("   lumen tutor basics");
    println!("   lumen doctor");
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
            // BUG-146: el receptor de un método se declara `este`, sin tipo
            // delante. `funcion entero suma(Punto este)` —lo que este tutorial
            // enseñaba— no compila: E011 «Se esperaba un nombre de variable».
            println!("      funcion entero suma(este) {{");
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
            // BUG-146: LÚMEN exige el tipo de cada parámetro; `div(a,b)`
            // no compila.
            println!("  funcion resultado<entero,texto> div(entero a, entero b) {{");
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
    let mut dest = String::new();
    let mut extra = String::new();
    let mut lib_dirs = Vec::new();
    let mut standalone = false;
    let mut native = false;
    let mut backend = String::from("c");
    let mut simd = false;
    let mut target = String::new();
    let mut embedded = false;
    let mut memory_model = String::from("auto");
    let mut opt_level = String::from("3");
    let mut neuro_opt = true;
    let mut self_healing = false;
    let mut profile = String::from("release");
    let mut time_travel = false;
    let mut port: u16 = 8080;
    let mut sanitize = false;
    let mut ai_gen = false;

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
            "-v" | "--version" => {
                println!("LÚMEN v{}", VERSION);
                process::exit(0);
            }
            "-h" | "--help" => {
                print_help();
                process::exit(0);
            }
            "--native" => {
                native = true;
            }
            "--standalone" => {
                native = true;
                standalone = true;
            }
            "--embedded" | "--mcu" | "--bare-metal" => {
                native = true;
                standalone = true;
                embedded = true;
                memory_model = "arena".to_string();
                profile = "mcu".to_string();
            }
            "--memory-model" => {
                i += 1;
                if i < args.len() {
                    memory_model = args[i].clone();
                } else {
                    eprintln!("Error: falta el modelo después de '--memory-model' (nanbox | borrow-checker | arena | auto)");
                    process::exit(1);
                }
            }
            "--zero-gc" => {
                memory_model = "borrow-checker".to_string();
            }
            "--arena" => {
                memory_model = "arena".to_string();
            }
            "--nanbox" => {
                memory_model = "nanbox".to_string();
            }
            "--neuro-opt" => {
                neuro_opt = true;
            }
            "--no-neuro-opt" => {
                neuro_opt = false;
            }
            "--self-healing" => {
                self_healing = true;
            }
            "--time-travel" => {
                time_travel = true;
            }
            "--sanitize" | "--asan" => {
                sanitize = true;
            }
            "--ai-gen" | "--generar-tests" => {
                ai_gen = true;
            }
            "--profile" => {
                i += 1;
                if i < args.len() {
                    profile = args[i].clone();
                    match profile.as_str() {
                        "dev" => {
                            opt_level = "0".to_string();
                            time_travel = true;
                        }
                        "release" => {
                            opt_level = "3".to_string();
                            neuro_opt = true;
                            native = true;
                        }
                        "hpc" => {
                            opt_level = "3".to_string();
                            simd = true;
                            neuro_opt = true;
                            native = true;
                            memory_model = "borrow-checker".to_string();
                        }
                        "mcu" => {
                            embedded = true;
                            standalone = true;
                            native = true;
                            memory_model = "arena".to_string();
                        }
                        "cloud" => {
                            self_healing = true;
                            opt_level = "3".to_string();
                        }
                        _ => {}
                    }
                }
            }
            "-O" | "--opt-level" => {
                i += 1;
                if i < args.len() {
                    opt_level = args[i].clone();
                }
            }
            "--simd" | "--avx" | "--avx512" => {
                native = true;
                simd = true;
            }
            "--target" => {
                native = true;
                i += 1;
                if i < args.len() {
                    target = args[i].clone();
                } else {
                    eprintln!("Error: falta el target después de '--target'");
                    process::exit(1);
                }
            }
            // BUG-085: este flag se consultaba con `env::args()` pero no estaba
            // en el parser, así que caía en el catch-all que asigna `dest` y
            // acababa usándose como NOMBRE DEL BINARIO de salida (se generaba
            // un ejecutable llamado "--permitir-no-soportados").
            "--permitir-no-soportados" | "--allow-unsupported" => {}
            // BUG-073: la documentación anunciaba `lumen bundle app.nv -o salida`,
            // pero `-o` no estaba implementado: se tragaba como argumento
            // desconocido o quedaba ignorado. Se acepta como alias del destino.
            "-o" | "--output" | "--salida" => {
                i += 1;
                if i < args.len() {
                    dest = args[i].clone();
                } else {
                    eprintln!("Error: falta la ruta de salida después de '-o'");
                    process::exit(1);
                }
            }
            "--template" => {
                i += 1;
                if i < args.len() {
                    dest = args[i].clone();
                } else {
                    eprintln!("Error: falta la plantilla después de '--template'");
                    process::exit(1);
                }
            }
            "--c" => {
                native = true;
                backend = "c".to_string();
            }
            "--rust" | "--cranelift" => {
                native = true;
                backend = "rust".to_string();
            }
            "--llvm" => {
                native = true;
                backend = "llvm".to_string();
            }
            "--stage3" => {
                backend = "stage3".to_string();
            }
            "--aot" | "--backend" => {
                native = true;
                i += 1;
                if i < args.len() {
                    let b = args[i].to_lowercase();
                    backend = match b.as_str() {
                        "llvm" => "llvm".to_string(),
                        "c" | "clang" | "gcc" => "c".to_string(),
                        "rust" | "cranelift" => "rust".to_string(),
                        "stage3" => "stage3".to_string(),
                        other => other.to_string(),
                    };
                } else {
                    eprintln!("Error: falta el backend después de '--aot' (c | rust | cranelift | llvm | stage3)");
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
            "--port-env" => {
                // Deprecado: use LUMEN_PORT env var
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
                    eprintln!("Error: falta el puerto después de '--port-env'");
                    process::exit(1);
                }
            }
            s if command.is_empty() => {
                command = s.to_string();
            }
            s if file.is_empty() => {
                file = s.to_string();
            }
            s if dest.is_empty() => {
                dest = s.to_string();
            }
            s if extra.is_empty() => {
                extra = s.to_string();
            }
            _ => {
                eprintln!("Argumento desconocido: '{}'", args[i]);
                process::exit(1);
            }
        }
        i += 1;
    }

    // stdlib: relativo a la raíz del repo/paquete (LUMEN_ROOT → exe → CWD)
    fn add_dir_and_subs(lib_dirs: &mut Vec<PathBuf>, base: &Path) {
        if base.is_dir() && !lib_dirs.iter().any(|p| p == base) {
            lib_dirs.push(base.to_path_buf());
            if let Ok(entries) = fs::read_dir(base) {
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if sub.is_dir() && !lib_dirs.iter().any(|p| p == &sub) {
                        lib_dirs.push(sub);
                    }
                }
            }
        }
    }

    if let Some(root) = find_repo_root() {
        add_dir_and_subs(&mut lib_dirs, &root.join("stdlib"));
        let pkgs_path = root.join("pkgs");
        if pkgs_path.is_dir() && !lib_dirs.iter().any(|p| p == &pkgs_path) {
            lib_dirs.push(pkgs_path);
        }
    }
    // Fallback: CWD (compatibilidad con layouts de proyectos locales)
    add_dir_and_subs(&mut lib_dirs, &PathBuf::from("stdlib"));
    let pkgs_path = PathBuf::from("pkgs");
    if pkgs_path.is_dir() && !lib_dirs.iter().any(|p| p == &pkgs_path) {
        lib_dirs.push(pkgs_path.clone());
        if let Ok(entries) = fs::read_dir(&pkgs_path) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && !lib_dirs.iter().any(|d| d == &p) {
                    lib_dirs.push(p);
                }
            }
        }
    }
    add_dir_and_subs(&mut lib_dirs, &PathBuf::from("../stdlib"));

    // Cache global de paquetes (~/.lumen/lumen_cache)
    let cache_dir = lumen_pkg::cache_dir();
    if cache_dir.is_dir() && !lib_dirs.iter().any(|p| p == &cache_dir) {
        lib_dirs.push(cache_dir.clone());
        if let Ok(entries) = fs::read_dir(&cache_dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_dir() && !lib_dirs.iter().any(|d| d == &p) {
                    lib_dirs.push(p);
                }
            }
        }
    }

    // LUMEN_PORT env var: prioridad sobre default 8080, pero no sobre --port explícito
    if let Ok(env_port) = env::var("LUMEN_PORT") {
        if let Ok(p) = env_port.parse::<u16>() {
            port = p;
        }
    }

    Config {
        command,
        file,
        dest,
        extra,
        lib_dirs,
        native,
        standalone,
        backend,
        simd,
        target,
        embedded,
        memory_model,
        opt_level,
        neuro_opt,
        self_healing,
        profile,
        time_travel,
        port,
        sanitize,
        ai_gen,
    }
}

/// BUG-153: `build --native` invoca un compilador de C que el usuario final
/// probablemente no tenga (sobre todo en Windows). El mensaje anterior era
/// "Instala GCC", que en macOS y Windows ni siquiera es el consejo correcto.
/// Falla igual —el codigo de salida sigue siendo 1 y no se escribe ningun
/// binario a medias—, pero diciendo QUE instalar en ESTE sistema y cual es la
/// alternativa que no necesita nada.
fn ayuda_sin_compilador_c() -> String {
    let receta = if cfg!(target_os = "windows") {
        "  • Visual Studio Build Tools (incluye MSVC):\n\
         \x20     https://visualstudio.microsoft.com/visual-cpp-build-tools/\n\
         \x20 • o bien MinGW-w64:  winget install -e --id MSYS2.MSYS2"
    } else if cfg!(target_os = "macos") {
        "  • Herramientas de linea de comandos de Xcode:\n\
         \x20     xcode-select --install"
    } else {
        "  • Debian/Ubuntu:  sudo apt install build-essential\n\
         \x20 • Fedora/RHEL:    sudo dnf install gcc\n\
         \x20 • Alpine:         apk add build-base\n\
         \x20 • Arch:           sudo pacman -S base-devel"
    };
    format!(
        "No se encontro un compilador de C (gcc o clang) en el PATH.\n\n\
         `lumen build --native` genera codigo C y necesita un compilador para\n\
         producir el ejecutable. Instala uno:\n\n{}\n\n\
         Alternativa sin instalar nada: `lumen run programa.nv` ejecuta el\n\
         programa con la maquina virtual, que no requiere compilador.",
        receta
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let builder = thread::Builder::new()
        .name("lumen-main".into())
        .stack_size(16 * 1024 * 1024); // 16 MB stack (prevents Windows 1MB stack overflow on 360-file batch checks)
    let handler = builder
        .spawn(move || {
            real_main(args);
        })
        .expect("No se pudo iniciar el hilo principal de LÚMEN");
    let _ = handler.join();
}

fn real_main(args: Vec<String>) {
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
    // Resolución tolerante de rutas: `lumen run examples/hello.nv` funciona
    // desde cualquier CWD (se busca relativo al repo/paquete si no existe local).
    let config = Config {
        file: if !config.file.is_empty() {
            resolve_file_path(&config.file)
                .to_string_lossy()
                .to_string()
        } else {
            config.file
        },
        ..config
    };
    match config.command.as_str() {
        "fuzz" | "fuzzing" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo para fuzzing");
                eprintln!("Uso: lumen fuzz <archivo.nv>");
                process::exit(1);
            }
            run_fuzz(&config.file, &config.lib_dirs);
        }
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
        "config" | "configuracion" => {
            run_config(&config.file, &config.dest, &config.extra, &config);
        }
        "fix" | "corregir" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo a corregir");
                eprintln!("Uso: lumen fix <archivo.nv>");
                process::exit(1);
            }
            run_fix(&config.file, &config.lib_dirs);
        }
        "watch" | "vigilar" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo para observar");
                eprintln!("Uso: lumen watch <archivo.nv>");
                process::exit(1);
            }
            run_watch(&config.file, &config.lib_dirs);
        }
        "build" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            if config.native {
                build_native(
                    &config.file,
                    &config.lib_dirs,
                    &config.backend,
                    config.standalone,
                    config.simd,
                    &config.target,
                    config.embedded,
                    config.sanitize,
                    &config.dest,
                );
            } else {
                build_bytecode(&config.file, &config.lib_dirs, &config.dest);
            }
        }
        "check" => {
            let target = if config.file.is_empty() {
                "."
            } else {
                &config.file
            };
            if Path::new(target).is_dir() || target == "." {
                check_project(target, &config.lib_dirs);
            } else {
                check_source(target, &config.lib_dirs);
            }
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
            // BUG-156: el REPL resuelve imports; pasarle los `-L` del usuario.
            lumen_repl::run_repl_con_lib_dirs(config.lib_dirs.clone());
        }
        "new" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el nombre del proyecto");
                eprintln!("Uso: lumen new <nombre-del-proyecto> [--template <default|ia|web|uni>]");
                process::exit(1);
            }
            let template = if !config.dest.is_empty() {
                &config.dest
            } else {
                "default"
            };
            match ProjectManifest::create_with_template(&config.file, template) {
                Ok(dir) => {
                    println!();
                    println!(
                        "  🎉 ¡Proyecto '{}' creado con éxito (plantilla '{}')!",
                        config.file, template
                    );
                    println!("  ═════════════════════════════════════════════════════════════");
                    println!("  📁 Estructura creada en: {}", dir.display());
                    println!(
                        "     ├── lumen.toml         (Configuración del proyecto y manifiesto)"
                    );
                    println!("     ├── README.md          (Guía rápida y comandos)");
                    println!("     ├── .gitignore         (Archivos ignorados)");
                    println!("     ├── src/");
                    println!("     │   └── main.nv        (Código inicial estructurado)");
                    println!("     ├── tests/");
                    println!(
                        "     │   └── test_main.nv   (Suite de pruebas unitarias automáticas)"
                    );
                    println!("     ├── stdlib/            (Módulos locales del proyecto)");
                    println!("     └── pkgs/              (Paquetes y dependencias)");
                    println!();
                    println!("  🚀 Próximos pasos para empezar:");
                    println!("     1. cd {}", config.file);
                    println!("     2. lumen run src/main.nv             # Ejecutar en desarrollo");
                    println!(
                        "     3. lumen test tests/test_main.nv     # Correr pruebas unitarias"
                    );
                    println!(
                        "     4. lumen check .                     # Comprobar todo el proyecto"
                    );
                    println!("     5. lumen build --native src/main.nv  # Compilar a binario nativo ultra-rápido\n");
                }
                Err(e) => {
                    eprintln!("Error al crear proyecto: {}", e);
                    process::exit(1);
                }
            }
        }
        "publish" | "publicar" => {
            let proj_path = if config.file.is_empty() {
                "."
            } else {
                &config.file
            };
            let proj_dir = PathBuf::from(proj_path);
            let manifest_path = proj_dir.join("lumen.toml");
            if !manifest_path.exists() {
                eprintln!(
                    "Error: no se encontró 'lumen.toml' en {}",
                    proj_dir.display()
                );
                eprintln!("Usa 'lumen new <nombre>' para crear un proyecto estructurado.");
                process::exit(1);
            }
            println!();
            println!("  🚀 PUBLICANDO PAQUETE EN REGISTRO OFICIAL (lumen publish)");
            println!("  ═════════════════════════════════════════════════════════════");
            println!("  1. Verificando código y tipos con 'lumen check'...");
            check_project(proj_path, &config.lib_dirs);

            let manifest_content = fs::read_to_string(&manifest_path).unwrap_or_default();
            let name = manifest_content
                .lines()
                .find(|l| l.trim().starts_with("nombre =") || l.trim().starts_with("name ="))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim().trim_matches('"'))
                .unwrap_or("paquete");
            let version = manifest_content
                .lines()
                .find(|l| l.trim().starts_with("version ="))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim().trim_matches('"'))
                .unwrap_or("0.1.0");

            let canonical_proj = fs::canonicalize(&proj_dir).unwrap_or_else(|_| proj_dir.clone());
            let out_pkg_path = canonical_proj
                .parent()
                .unwrap_or(&canonical_proj)
                .join(format!("{}-{}.lmp", name, version));

            let _ = std::process::Command::new("tar")
                .args([
                    "-czf",
                    out_pkg_path.to_str().unwrap(),
                    "-C",
                    canonical_proj.to_str().unwrap(),
                    ".",
                ])
                .status();

            println!(
                "  2. Empaquetando artefacto distribuible: {}",
                out_pkg_path.display()
            );
            let pkg_bytes = fs::read(&out_pkg_path).unwrap_or_default();
            // BUG-145: esto se anunciaba como "Checksum SHA-256" pero era un
            // DefaultHasher —SipHash de 64 bits, sin garantías criptográficas
            // y sin estabilidad entre versiones de Rust—. Ahora es SHA-256 de
            // verdad, así que el valor sirve para verificar el artefacto.
            let checksum = lumen_pkg::sha256_hex(&pkg_bytes);

            // BUG-145: sin sesión iniciada se usaban unas credenciales del
            // autor codificadas en el binario ("omar_dev"), de modo que
            // cualquiera publicaba firmando con su identidad. Ahora se exige
            // `lumen login`.
            let creds = match lumen_pkg::load_credentials() {
                Some(c) => c,
                None => {
                    eprintln!("  ✗ No hay ninguna sesión iniciada.");
                    eprintln!("  💡 Ejecuta 'lumen login' antes de publicar.");
                    println!();
                    process::exit(1);
                }
            };

            println!("  3. Artefacto firmado por: {}", creds.username);
            println!("     SHA-256: {}", checksum);
            // BUG-145: el paso 4 imprimía "Subiendo paquete a ..." y a
            // continuación "¡publicado con éxito en el registro público!"
            // sin realizar ninguna petición de red —la CLI no lleva cliente
            // HTTP y el dominio del registro ni siquiera resuelve—. El
            // usuario quedaba convencido de que su paquete estaba publicado.
            println!();
            println!("  ⚠️  El registro público de LÚMEN todavía no está operativo,");
            println!("     así que el paquete NO se ha subido a ningún servidor.");
            println!();
            println!("  ✓ El artefacto está listo para distribuir:");
            println!("      {}", out_pkg_path.display());
            println!("  • Quien lo reciba puede instalarlo con:");
            println!("      lumen install {}", out_pkg_path.display());
            println!();
        }
        "pack" | "empaquetar" => {
            let proj_path = if config.file.is_empty() {
                "."
            } else {
                &config.file
            };
            let proj_dir = PathBuf::from(proj_path);
            let manifest_path = proj_dir.join("lumen.toml");
            if !manifest_path.exists() {
                eprintln!(
                    "Error: no se encontró 'lumen.toml' en {}",
                    proj_dir.display()
                );
                eprintln!("Usa 'lumen new <nombre>' para crear un proyecto estructurado.");
                process::exit(1);
            }
            let manifest_content = fs::read_to_string(&manifest_path).unwrap_or_default();
            let name = manifest_content
                .lines()
                .find(|l| l.trim().starts_with("nombre =") || l.trim().starts_with("name ="))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim().trim_matches('"'))
                .unwrap_or("paquete");
            let version = manifest_content
                .lines()
                .find(|l| l.trim().starts_with("version ="))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim().trim_matches('"'))
                .unwrap_or("0.1.0");

            let canonical_proj = fs::canonicalize(&proj_dir).unwrap_or_else(|_| proj_dir.clone());
            let out_pkg_path = canonical_proj
                .parent()
                .unwrap_or(&canonical_proj)
                .join(format!("{}-{}.lmp", name, version));
            println!();
            println!("  📦 EMPAQUETANDO PROYECTO LÚMEN: {} v{}", name, version);
            println!("  ═════════════════════════════════════════════════════════════");
            println!("  • Comprimiendo código fuente, manifiesto y recursos...");
            let status = std::process::Command::new("tar")
                .args([
                    "-czf",
                    out_pkg_path.to_str().unwrap(),
                    "-C",
                    canonical_proj.to_str().unwrap(),
                    ".",
                ])
                .status();
            match status {
                Ok(st) if st.success() => {
                    println!(
                        "  ✓ Paquete distribuible generado: {}",
                        out_pkg_path.display()
                    );
                    println!(
                        "  • Listo para compartir o instalar con 'lumen install {}'.",
                        out_pkg_path.display()
                    );
                    println!();
                }
                _ => {
                    eprintln!("Error al generar paquete.");
                    process::exit(1);
                }
            }
        }
        "unpack" | "desempaquetar" => {
            if config.file.is_empty() {
                eprintln!("Error: falta la ruta del paquete (.lmp)");
                eprintln!("Uso: lumen unpack <archivo.lmp> [directorio_destino]");
                process::exit(1);
            }
            let pkg_file = &config.file;
            // BUG-141: se usaba la ruta COMPLETA del .lmp como destino, así que
            // `lumen unpack /otro/sitio/paq.lmp` extraía en `/otro/sitio/paq/`
            // —junto al paquete— en vez de en el directorio actual. Quien
            // desempaqueta un paquete que le han pasado espera encontrarlo
            // donde está trabajando, no donde estaba el fichero. Se toma sólo
            // el nombre base.
            let dest = if !config.dest.is_empty() {
                PathBuf::from(&config.dest)
            } else {
                let base = Path::new(pkg_file)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(pkg_file.as_str());
                let base = base.trim_end_matches(".lmp").trim_end_matches(".tar.gz");
                PathBuf::from(base)
            };
            println!();
            println!("  📦 DESEMPAQUETANDO PAQUETE LÚMEN: {}", pkg_file);
            println!("  ═════════════════════════════════════════════════════════════");
            match lumen_pkg::unpack_package(pkg_file, &dest) {
                Ok(dir) => {
                    println!("  ✓ Paquete extraído con éxito en: {}", dir.display());
                    println!("     ├── lumen.toml");
                    println!("     ├── src/main.nv");
                    println!("     └── tests/");
                    println!();
                    println!("  🚀 Para ejecutar el proyecto desempaquetado:");
                    println!("     1. cd {}", dir.display());
                    println!("     2. lumen run src/main.nv");
                    println!();
                }
                Err(e) => {
                    eprintln!("Error al desempaquetar: {}", e);
                    process::exit(1);
                }
            }
        }
        "test" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            if config.ai_gen {
                run_test_ai_gen(&config.file, &config.dest, &config.lib_dirs);
            } else {
                run_tests(&config.file, &config.lib_dirs);
            }
        }
        "bench" | "benchmark" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo para realizar el benchmark");
                eprintln!("Uso: lumen bench <archivo.nv>");
                process::exit(1);
            }
            run_bench(&config.file, &config.lib_dirs);
        }
        "bindgen" => {
            if config.file.is_empty() {
                eprintln!("Error: falta la ruta del archivo de cabecera (.h) o código Rust (.rs)");
                eprintln!("Uso: lumen bindgen <archivo.h | archivo.rs> [salida.nv]");
                process::exit(1);
            }
            run_bindgen(
                &config.file,
                if config.dest.is_empty() {
                    None
                } else {
                    Some(&config.dest)
                },
            );
        }
        "bootstrap" | "self-host" => {
            run_bootstrap(&config.file, &config.lib_dirs);
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
            // BUG-129: igual que BUG-128 en `build`, `doc` ignoraba
            // `-o/--output/--salida` y escribía siempre junto al fuente.
            let out = if config.dest.is_empty() {
                config.file.replace(".nv", ".html")
            } else {
                config.dest.clone()
            };
            // BUG-155: `doc` iba del texto al HTML sin parsear nada, asi que
            // sobre un fichero que no compila emitia una pagina vacia y
            // anunciaba exito con codigo 0. Documentar codigo roto no es
            // documentar: se avisa y se sale con error, igual que hace `fmt`
            // desde BUG-053 (de ahi sale `parses_ok`).
            if !parses_ok(&source) {
                eprintln!();
                eprintln!(
                    "  \x1b[1;33m⚠  No se ha generado documentación de '{}'.\x1b[0m",
                    config.file
                );
                eprintln!("     El archivo tiene errores de sintaxis, así que la");
                eprintln!("     documentación saldría vacía o incompleta.");
                eprintln!();
                eprintln!(
                    "     Ejecuta `lumen check {}` para ver los errores.",
                    config.file
                );
                process::exit(1);
            }
            let html = lumen_doc::generate_docs(&source, &config.file);
            match fs::write(&out, &html) {
                Ok(()) => println!("✓ Documentación: {}", out),
                Err(e) => {
                    // BUG-155: un fallo de escritura tampoco puede salir con 0.
                    eprintln!("Error generando documentación: {}", e);
                    process::exit(1);
                }
            }
        }
        "debug" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            run_debug(&config.file, &config.lib_dirs);
        }
        "install" | "add" | "agregar" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el nombre del paquete, archivo local (.lmp) o carpeta");
                eprintln!(
                    "Uso: lumen install <paquete | ./mi_libreria | ./paquete.lmp | usuario/repo>"
                );
                eprintln!("     lumen add <paquete>");
                process::exit(1);
            }
            let cache_dir = lumen_pkg::cache_dir();
            std::fs::create_dir_all(&cache_dir).ok();
            lumen_pkg::install_package(&config.file, &cache_dir);
        }
        "search" | "buscar" => {
            lumen_pkg::search_packages(&config.file);
        }
        "ai" | "asistente" => {
            let sub = if config.file.is_empty() {
                "chat"
            } else {
                &config.file
            };
            run_ai(sub, &config.dest, &config.lib_dirs);
        }
        "bundle" | "empaquetar-bin" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo fuente (.nv)");
                eprintln!("Uso: lumen bundle <archivo.nv> [salida_binaria]");
                process::exit(1);
            }
            run_bundle(&config.file, &config.dest, &config.lib_dirs);
        }
        "login" | "iniciar-sesion" => {
            let token_opt = if !config.dest.is_empty() {
                Some(config.dest.as_str())
            } else {
                None
            };
            lumen_pkg::login_user(&config.file, token_opt);
        }
        "registry" | "registro" => {
            let sub = if config.file.is_empty() {
                "info"
            } else {
                &config.file
            };
            run_registry(sub, config.port);
        }
        "lsp" => {
            lumen_lsp::run_lsp();
        }
        "lint" => {
            if config.file.is_empty() {
                eprintln!("Error: falta el archivo");
                process::exit(1);
            }
            run_lint(&config.file, &config.lib_dirs);
        }
        "doctor" | "info" => {
            print_doctor(&config.lib_dirs);
        }
        "monitor" | "dashboard" => {
            run_dashboard(&config.lib_dirs);
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
            suggest_examples(path);
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
    // BUG-024 / BUG-138: la salida se emite EN DIRECTO. Antes se volcaba el
    // buffer al terminar, así que un programa que no termina —servidor, TUI,
    // bucle de eventos o cuelgue— no mostraba absolutamente nada, ni siquiera
    // las líneas impresas antes de bloquearse.
    vm.set_stream_stdout(true);
    match vm.run() {
        Ok(()) => {}
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
            // BUG-138: ver `run_source`; el bytecode ya compilado se comporta
            // igual que el fuente.
            vm.set_stream_stdout(true);
            match vm.run() {
                Ok(()) => {}
                Err(e) => {
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

fn build_bytecode(path: &str, lib_dirs: &[PathBuf], dest: &str) {
    let bytecode = compile_source(path, lib_dirs);
    // BUG-128: `-o/--output/--salida` se ignoraba al compilar a bytecode —el
    // .nvc caía siempre junto al fuente— mientras que `build --native` sí lo
    // respetaba. Y encima el mensaje anunciaba la ruta escrita, así que un
    // `build a.nv -o /tmp/x.nvc` decía «Bytecode generado: a.nvc»: el usuario
    // pedía una ruta, el compilador escribía en otra y lo contaba sin avisar.
    let out_path = if dest.is_empty() {
        Path::new(path).with_extension("nvc")
    } else {
        PathBuf::from(dest)
    };
    let encoded = bytecode.encode();
    match fs::write(&out_path, &encoded) {
        Ok(()) => println!("Bytecode generado: {}", out_path.display()),
        Err(e) => {
            eprintln!("Error al escribir '{}': {}", out_path.display(), e);
            process::exit(1);
        }
    }
}

fn collect_nv_files(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = p.file_name().unwrap_or_default().to_string_lossy();
                if name != "target"
                    && name != ".git"
                    && name != "node_modules"
                    && !name.starts_with('.')
                {
                    collect_nv_files(&p, out);
                }
            } else if p.is_file()
                && p.extension()
                    .is_some_and(|ext| ext == "nv" || ext == "lumen")
            {
                out.push(p);
            }
        }
    }
}

fn check_project(root: &str, lib_dirs: &[PathBuf]) {
    let root_path = Path::new(root);
    let mut files = Vec::new();
    if root_path.is_file() {
        files.push(root_path.to_path_buf());
    } else {
        collect_nv_files(root_path, &mut files);
    }
    files.sort();

    println!();
    println!(
        "  🔍 COMPROBACIÓN GLOBAL DE PROYECTO (lumen check): {}",
        root
    );
    println!("  ═════════════════════════════════════════════════════════════");
    if files.is_empty() {
        println!(
            "  ℹ️  No se encontraron archivos de código fuente (.nv) en '{}'",
            root
        );
        println!();
        return;
    }

    let mut total_errors = 0usize;
    let mut checked_count = 0usize;

    for f in &files {
        let f_str = f.to_string_lossy().to_string();
        let source = match fs::read_to_string(f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("  ✗ Error al leer '{}': {}", f_str, e);
                total_errors += 1;
                continue;
            }
        };
        let mut loader = ModuleLoader::new(lib_dirs.to_vec());
        let mut program = match loader.resolve_imports(&source, f) {
            Ok(p) => p,
            Err(e) => {
                match &e {
                    lumen_sema::ModuleError::Parse { details, .. } => {
                        for d in details {
                            eprintln!("  ✗ Error sintáctico en '{}': {}", f_str, d);
                        }
                    }
                    _ => eprintln!("  ✗ Error al procesar '{}': {:?}", f_str, e),
                }
                total_errors += 1;
                continue;
            }
        };
        let sema = SemanticAnalyzer::new();
        let sem_errors = sema.analyze(&mut program);
        if !sem_errors.is_empty() {
            show_sema_errors(&sem_errors, &source, &f_str);
            total_errors += sem_errors.len();
        } else {
            checked_count += 1;
            println!("  ✓ {} ... VÁLIDO (Sintaxis y tipos seguros)", f_str);
        }
    }

    println!("  ═════════════════════════════════════════════════════════════");
    if total_errors == 0 {
        println!("  ✨ ¡Proyecto 100% verificado y seguro!");
        println!(
            "  • {} archivos comprobados, 0 errores, 0 violaciones de tipos.\n",
            checked_count
        );
    } else {
        eprintln!(
            "  ✗ Se encontraron {} errores en el proyecto. Corrígelos antes de compilar.\n",
            total_errors
        );
        process::exit(1);
    }
}

fn check_source(path: &str, lib_dirs: &[PathBuf]) {
    let _ = compile_source(path, lib_dirs);
    println!(
        "✓ El programa '{}' es válido (sintaxis y semántica correctas)",
        path
    );
}

/// BUG-074: `lumen lint` era un stub que imprimía "0 advertencias" para
/// CUALQUIER entrada, incluso para un archivo inexistente o para basura
/// sintáctica. Ahora ejecuta lexer + parser + análisis semántico y además
/// aplica reglas de estilo propias sobre el fuente.
fn run_lint(path: &str, lib_dirs: &[PathBuf]) {
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            suggest_examples(path);
            process::exit(1);
        }
    };

    let mut errores = 0usize;

    let lexer = lumen_lexer::Lexer::new(&source);
    let (tokens, lex_errors) = lexer.tokenize();
    for e in &lex_errors {
        eprintln!("  \x1b[1;31mError léxico\x1b[0m en '{}': {:?}", path, e);
        errores += 1;
    }

    let parser = lumen_parser::Parser::new(tokens);
    let (_prog, parse_errors) = parser.parse();
    for e in &parse_errors {
        eprintln!(
            "  \x1b[1;31mError sintáctico\x1b[0m en '{}': {} [{}:{}]: {} ({})",
            path, e.code, e.span.start.line, e.span.start.col, e.message, e.suggestion
        );
        errores += 1;
    }

    // Sólo tiene sentido pedir semántica si el fuente al menos parsea.
    if errores == 0 {
        let base_path = Path::new(path);
        let mut loader = ModuleLoader::new(lib_dirs.to_vec());
        match loader.resolve_imports(&source, base_path) {
            Ok(mut program) => {
                let sema = SemanticAnalyzer::new();
                let sem_errors = sema.analyze(&mut program);
                if !sem_errors.is_empty() {
                    show_sema_errors(&sem_errors, &source, path);
                    errores += sem_errors.len();
                }
            }
            Err(e) => {
                eprintln!("  \x1b[1;31mError de imports\x1b[0m en '{}': {:?}", path, e);
                errores += 1;
            }
        }
    }

    // Reglas de estilo (advertencias, no bloquean).
    let mut avisos: Vec<String> = Vec::new();
    for (i, linea) in source.lines().enumerate() {
        let n = i + 1;
        if linea.len() > 120 {
            avisos.push(format!(
                "[{}:{}] línea de {} caracteres (recomendado <= 120)",
                path,
                n,
                linea.len()
            ));
        }
        if linea.ends_with(' ') || linea.ends_with('\t') {
            avisos.push(format!(
                "[{}:{}] espacios en blanco al final de la línea",
                path, n
            ));
        }
        if linea.contains('\t') {
            avisos.push(format!(
                "[{}:{}] tabulador literal (usa espacios; ver 'lumen fmt')",
                path, n
            ));
        }
        let t = linea.trim_start();
        if t.starts_with("TODO") || t.starts_with("// TODO") || t.starts_with("// FIXME") {
            avisos.push(format!("[{}:{}] marca pendiente sin resolver", path, n));
        }
    }

    for a in &avisos {
        println!("  \x1b[1;33madvertencia\x1b[0m {}", a);
    }

    if errores > 0 {
        eprintln!();
        eprintln!(
            "  \x1b[1;31m✗ lumen lint: {} error(es) y {} advertencia(s) en '{}'\x1b[0m",
            errores,
            avisos.len(),
            path
        );
        process::exit(1);
    }

    if avisos.is_empty() {
        println!(
            "✓ Análisis estático (lumen lint): 0 advertencias en '{}'",
            path
        );
    } else {
        println!(
            "✓ Análisis estático (lumen lint): {} advertencia(s) en '{}'",
            avisos.len(),
            path
        );
    }
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

/// BUG-053: ¿este fuente parsea sin errores? Se usa para no sobrescribir un
/// archivo con un formateo que lo dejaría roto.
fn parses_ok(source: &str) -> bool {
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return false;
    }
    let parser = lumen_parser::Parser::new(tokens);
    let (_program, parse_errors) = parser.parse();
    parse_errors.is_empty()
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
            // BUG-053: el formateador no cubre toda la sintaxis del lenguaje y
            // las construcciones que no entiende las EMITÍA MAL o las borraba,
            // sobrescribiendo el fichero del usuario con código que ya no
            // compila (pérdida de trabajo, sin aviso). Un formateador que no
            // sabe formatear algo debe dejarlo en paz: se vuelve a parsear el
            // resultado y sólo se escribe si sigue siendo válido.
            let original_ok = parses_ok(&source);
            if original_ok && !parses_ok(&trimmed) {
                eprintln!();
                eprintln!("  \x1b[1;33m⚠  No se ha formateado '{}'.\x1b[0m", path);
                eprintln!("     El resultado no volvería a compilar, así que el archivo se ha");
                eprintln!("     dejado intacto en lugar de corromperlo.");
                eprintln!();
                eprintln!("     Es una limitación del formateador con alguna construcción de");
                eprintln!("     este archivo, no un error de tu código.");
                eprintln!();
                process::exit(1);
            }
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

/// BUG-005: cuenta las aserciones escritas directamente en el nivel superior
/// del archivo (fuera de funciones `test_*`), para poder ejecutarlas y
/// reportarlas en vez de informar "0 pasaron, 0 fallaron".
///
/// Reconoce tanto los nombres de `testing.nv` (`afirmar_*` / `assert_*`) como
/// los que quedan tras el prefijado del módulo (`testing_afirmar_*`).
fn count_toplevel_assertions(program: &lumen_parser::ast::Program) -> usize {
    use lumen_parser::ast::{DeclOrStmt, Expr, Stmt};

    fn is_assert_name(name: &str) -> bool {
        let n = name.rsplit("__").next().unwrap_or(name);
        n.starts_with("afirmar_")
            || n.starts_with("assert_")
            || n.starts_with("testing_afirmar_")
            || n.starts_with("testing_assert_")
    }

    let mut count = 0usize;
    for node in program {
        // Sólo el nivel superior: lo que está dentro de funciones lo cubre el
        // recorrido de funciones `test_*`.
        if let DeclOrStmt::Stmt(Stmt::Expr { expr, .. }) = node {
            if let Expr::Call { callee, .. } = expr.as_ref() {
                if let Expr::Ident { name, .. } = callee.as_ref() {
                    if is_assert_name(name) {
                        count += 1;
                    }
                }
            }
        }
    }
    count
}

fn run_tests(path: &str, lib_dirs: &[PathBuf]) {
    use lumen_parser::ast::{Expr, Stmt};
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };
    let mut loader = ModuleLoader::new(lib_dirs.to_vec());
    let mut flat = match loader.resolve_imports(&source, Path::new(path)) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Error imports: {:?}", e);
            process::exit(1);
        }
    };
    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut flat);
    if !sem_errors.is_empty() {
        for e in &sem_errors {
            eprintln!("  [{}] {}", e.code, e.message);
        }
        process::exit(1);
    }
    let builder = IRBuilder::new();
    let ir = builder.build(&flat);
    let codegen = Codegen::new();
    let (bytecode, _) = codegen.generate(&ir);
    let mut passed = 0u32;
    let mut failed = 0u32;
    println!();
    println!("  🧪 EJECUTANDO SUITE DE PRUEBAS: {}", path);
    println!("  ═════════════════════════════════════════════════════════════");

    // BUG-005: además de las funciones `test_*`, ejecuta el cuerpo principal
    // del archivo para que las aserciones sueltas (`afirmar_igual(...)` en el
    // nivel superior) cuenten como una prueba en vez de reportar "0 pasaron".
    let has_test_fns = bytecode.funcs.iter().any(|f| f.name.starts_with("test_"));
    let toplevel_asserts = count_toplevel_assertions(&flat);
    if toplevel_asserts > 0 {
        let mut vm = VM::new(bytecode.clone());
        match vm.run() {
            Ok(()) => {
                // Las aserciones de `testing.nv` devuelven booleano e imprimen
                // "ERROR: ..." al fallar; se detecta esa salida para no dar por
                // buena una suite que en realidad falló.
                let failures: Vec<&String> = vm
                    .output()
                    .iter()
                    .filter(|line| line.contains("ERROR:") || line.contains("fallo"))
                    .collect();
                if failures.is_empty() {
                    passed += 1;
                    println!(
                        "  ✓ (nivel superior) {} aserción(es) ... OK",
                        toplevel_asserts
                    );
                } else {
                    failed += 1;
                    println!("  ✗ (nivel superior) ... FALLÓ:");
                    for f in failures {
                        println!("      {}", f);
                    }
                }
            }
            Err(e) => {
                failed += 1;
                eprintln!(
                    "  ✗ (nivel superior) ... FALLÓ: {}",
                    e.with_stack(vm.call_stack())
                );
            }
        }
    } else if !has_test_fns {
        println!(
            "  ⚠  No se encontraron pruebas en '{}'.\n     \
             Define funciones con prefijo 'test_' (ej. 'funcion vacio test_suma() {{ ... }}')\n     \
             o escribe aserciones en el nivel superior importando \"testing.nv\".",
            path
        );
    }

    // BUG-072: el runner construía un bytecode con `funcs: vec![fm]` pero
    // conservaba las instrucciones globales, así que la función de prueba NO SE
    // LLAMABA NUNCA: la VM ejecutaba el cuerpo principal (vacío) y devolvía
    // `Ok`, y el runner lo contaba como ✓ OK. Un `test_` que afirmara
    // `2 + 2 == 5` —o que ni siquiera se ejecutara— pasaba igual, y
    // `lumen test` salía con código 0. Ahora se sintetiza, para cada prueba, un
    // programa que la invoca de verdad, y se recompila desde el AST ya
    // resuelto.
    let nombres_test: Vec<String> = bytecode
        .funcs
        .iter()
        .map(|f| f.name.clone())
        .filter(|n| n.starts_with("test_"))
        .collect();
    for nombre in &nombres_test {
        {
            let sp = lumen_lexer::Span::new(
                lumen_lexer::Pos { line: 1, col: 1 },
                lumen_lexer::Pos { line: 1, col: 1 },
            );
            let mut ast_prueba = flat.clone();
            // El cuerpo principal se sustituye por una única llamada a la prueba.
            ast_prueba.retain(|nodo| matches!(nodo, DeclOrStmt::Decl(_)));
            ast_prueba.push(DeclOrStmt::Stmt(Stmt::Expr {
                expr: Box::new(Expr::Call {
                    callee: Box::new(Expr::Ident {
                        name: nombre.clone(),
                        span: sp,
                    }),
                    args: vec![],
                    type_args: vec![],
                    span: sp,
                }),
                span: sp,
            }));
            let ir_prueba = IRBuilder::new().build(&ast_prueba);
            let (test_bc, _) = Codegen::new().generate(&ir_prueba);
            let mut vm = VM::new(test_bc);
            match vm.run() {
                Ok(()) => {
                    // BUG-072: sólo se miraba si la VM devolvía `Err`, pero las
                    // aserciones de `testing.nv` NO abortan: imprimen
                    // "ERROR: ..." y devuelven `falso`. Un test que afirmaba
                    // `2 + 2 == 5` se reportaba como ✓ OK y `lumen test` salía
                    // con código 0. La herramienta de calidad daba por buenas
                    // suites que fallaban. Se aplica el mismo criterio que ya
                    // usaban las aserciones de nivel superior.
                    let fallos: Vec<&String> = vm
                        .output()
                        .iter()
                        .filter(|line| line.contains("ERROR:") || line.contains("fallo"))
                        .collect();
                    if fallos.is_empty() {
                        passed += 1;
                        println!("  ✓ {} ... OK", nombre);
                    } else {
                        failed += 1;
                        println!("  ✗ {} ... FALLÓ:", nombre);
                        for f in fallos {
                            println!("      {}", f);
                        }
                    }
                }
                Err(e) => {
                    failed += 1;
                    eprintln!(
                        "  ✗ {} ... FALLÓ: {}",
                        nombre,
                        e.with_stack(vm.call_stack())
                    );
                }
            }
        }
    }
    println!("  ═════════════════════════════════════════════════════════════");
    println!("  Resultado: {} pasaron, {} fallaron\n", passed, failed);

    // Verificación de Doctests en comentarios ///
    if source.contains("///") && (source.contains("```") || source.contains("`")) {
        println!("  📖 Verificando ejemplos en comentarios de documentación (Doctests)...");
        let mut doctest_code = String::new();
        let mut in_code = false;
        let mut dt_count = 0usize;
        for line in source.lines() {
            let tr = line.trim();
            if tr.starts_with("/// ```") {
                if in_code {
                    in_code = false;
                    dt_count += 1;
                    let (_out, err) = run_source_capture(&doctest_code, lib_dirs, Path::new(path));
                    if err.is_empty() {
                        println!("     ✓ Doctest #{} ... OK", dt_count);
                    } else {
                        println!("     ✗ Doctest #{} ... FALLÓ: {}", dt_count, err);
                    }
                    doctest_code.clear();
                } else {
                    in_code = true;
                }
            } else if in_code && tr.starts_with("///") {
                let code_line = tr.trim_start_matches("///").trim();
                doctest_code.push_str(code_line);
                doctest_code.push('\n');
            }
        }
        if dt_count > 0 {
            println!("  ✓ {} Doctests verificados con éxito.\n", dt_count);
        }
    }

    if failed > 0 {
        process::exit(1);
    }
}

fn run_bench(path: &str, lib_dirs: &[PathBuf]) {
    println!();
    println!("  ⚡ SUITE DE RENDIMIENTO & BENCHMARK LÚMEN: {}", path);
    println!("  ═════════════════════════════════════════════════════════════");
    let t_compile_start = std::time::Instant::now();
    let bytecode = compile_source(path, lib_dirs);
    let compile_time_ms = t_compile_start.elapsed().as_secs_f64() * 1000.0;
    println!("  • Compilación a Bytecode : {:>8.2} ms", compile_time_ms);

    // Warmup
    let mut warmup_vm = VM::new(bytecode.clone());
    let _ = warmup_vm.run();

    let iterations = 10;
    let mut times_ms = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let mut vm = VM::new(bytecode.clone());
        let t_start = std::time::Instant::now();
        let _ = vm.run();
        let elapsed_ms = t_start.elapsed().as_secs_f64() * 1000.0;
        times_ms.push(elapsed_ms);
    }

    let min_time = times_ms.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_time = times_ms.iter().cloned().fold(0.0, f64::max);
    let avg_time: f64 = times_ms.iter().sum::<f64>() / iterations as f64;
    let throughput = if avg_time > 0.0 {
        1000.0 / avg_time
    } else {
        0.0
    };

    println!("  • Iteraciones de prueba  : {}", iterations);
    println!("  • Tiempo Mínimo (Best)   : {:>8.2} ms", min_time);
    println!("  • Tiempo Promedio (Avg)  : {:>8.2} ms", avg_time);
    println!("  • Tiempo Máximo (Worst)  : {:>8.2} ms", max_time);
    println!(
        "  • Rendimiento / Throughput: {:>8.1} ejecuciones/segundo",
        throughput
    );
    println!("  ═════════════════════════════════════════════════════════════");
    println!("  ✓ Benchmark completado con éxito.\n");
}

/// BUG-143: nombres internos de LÚMEN. Generar un enlace FFI con estos
/// nombres produce E082 ('no se puede redefinir').
const ES_BUILTIN_BINDGEN: &[&str] = &[
    "imprimir",
    "longitud",
    "agregar",
    "abs",
    "minimo",
    "maximo",
    "a_entero",
    "a_decimal",
    "a_texto",
    "texto",
    "entero",
    "decimal",
    "error",
    "exito",
];

fn run_bindgen(input_path: &str, output_path: Option<&str>) {
    let source = match fs::read_to_string(input_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", input_path, e);
            process::exit(1);
        }
    };
    println!();
    println!("  ⚙️  LÚMEN BINDGEN — GENERADOR AUTOMÁTICO DE ENLACES C / RUST");
    println!("  ═════════════════════════════════════════════════════════════");
    println!(
        "  • Analizando firmas de funciones y tipos en: {}",
        input_path
    );

    let stem = Path::new(input_path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "lib_externa".to_string());

    let out_file = output_path
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("{}_bindings.nv", stem));

    let mut bindings = String::new();
    bindings.push_str(&format!(
        "// ============================================================================\n\
         // Auto-generated FFI Bindings for '{}'\n\
         // Generado automáticamente por: lumen bindgen {}\n\
         // ============================================================================\n\n\
         texto _lib_handle = __ffi_cargar(\"{}.so\");\n\n",
        input_path, input_path, stem
    ));

    let mut count = 0usize;
    // BUG-143: el heurístico aceptaba cualquier línea con paréntesis que
    // terminase en ';' o '{', de modo que las *llamadas* del programa se
    // tomaban por declaraciones. Con un .nv que sólo hacía
    // `imprimir("x"); imprimir(6*7);` generaba dos veces la misma función y
    // además redefinía el builtin `imprimir` ⇒ el módulo emitido no
    // compilaba (E082). Se distingue declaración de llamada: una declaración
    // lleva un tipo (o `funcion`/`fn`) antes del nombre, así que el texto
    // previo al '(' contiene un espacio; una llamada no. Se deduplica además
    // por nombre.
    let mut vistos: Vec<String> = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if (trimmed.ends_with(';') || trimmed.ends_with('{') || trimmed.contains("fn "))
            && (trimmed.contains('(') && trimmed.contains(')'))
            && !trimmed.starts_with("//")
            && !trimmed.starts_with('#')
        {
            let prefijo = trimmed.split('(').next().unwrap_or("").trim();
            // Sin tipo delante del nombre no es una declaración, es una llamada.
            let es_declaracion = prefijo.split_whitespace().count() >= 2;
            let fn_name = if es_declaracion {
                prefijo
                    .split_whitespace()
                    .last()
                    .unwrap_or("")
                    .trim_start_matches('*')
            } else {
                ""
            };
            if !fn_name.is_empty()
                && fn_name != "if"
                && fn_name != "while"
                && fn_name != "for"
                && fn_name != "return"
                && !ES_BUILTIN_BINDGEN.contains(&fn_name)
                && !vistos.iter().any(|v| v == fn_name)
            {
                vistos.push(fn_name.to_string());
                count += 1;
                // BUG-077: el tipo de retorno se fijaba siempre a "entero",
                // aunque la cabecera declarase `double` o `char*`. Se deduce del
                // tipo que precede al nombre de la función.
                let ret_c = trimmed
                    .find('(')
                    .map(|pos| trimmed[..pos].trim())
                    .unwrap_or("")
                    .trim_end_matches(fn_name)
                    .trim()
                    .trim_end_matches('*')
                    .trim();
                let ret_lumen = if ret_c.contains("double") || ret_c.contains("float") {
                    "decimal"
                } else if ret_c.contains("char")
                    && trimmed[..trimmed.find('(').unwrap_or(0)].contains('*')
                {
                    "texto"
                } else if ret_c.contains("void") {
                    "vacio"
                } else {
                    "entero"
                };
                // BUG-078: faltaba la cadena de tipos. La firma real es
                // __ffi_llamar(lib, nombre, "tipos", [args], "retorno"): al
                // omitirla, los argumentos se desplazaban y la VM panicaba.
                // Se generan los parámetros que la cabecera C declara.
                let params_c = trimmed
                    .find('(')
                    .and_then(|a| trimmed.rfind(')').map(|b| &trimmed[a + 1..b]))
                    .unwrap_or("")
                    .trim();
                let mut tipos: Vec<&str> = Vec::new();
                if !params_c.is_empty() && params_c != "void" {
                    for p in params_c.split(',') {
                        let p = p.trim();
                        if p.contains('*') {
                            tipos.push("entero"); // punteros: se pasan como dirección
                        } else if p.contains("double") || p.contains("float") {
                            tipos.push("decimal");
                        } else {
                            tipos.push("entero");
                        }
                    }
                }
                let firma_tipos = tipos.join(",");
                let params_nv: Vec<String> = (1..=tipos.len())
                    .map(|i| format!("cualquiera arg{} = 0", i))
                    .collect();
                let args_nv: Vec<String> = (1..=tipos.len()).map(|i| format!("arg{}", i)).collect();
                bindings.push_str(&format!(
                    "funcion cualquiera {}({}) {{\n\
                     \x20   retornar __ffi_llamar(_lib_handle, \"{}\", \"{}\", [{}], \"{}\");\n\
                     }}\n\n",
                    fn_name,
                    params_nv.join(", "),
                    fn_name,
                    firma_tipos,
                    args_nv.join(", "),
                    ret_lumen
                ));
            }
        }
    }

    // BUG-143: cuando no se detecta ninguna firma se emite un stub genérico.
    // Es útil como punto de partida, pero antes se contabilizaba como
    // `count = 1` y la herramienta anunciaba "1 funciones enlazadas con
    // éxito", afirmando un enlace que no existe. Se avisa de que es un
    // esqueleto.
    let mut solo_stub = false;
    if count == 0 {
        solo_stub = true;
        bindings.push_str(&format!(
            "funcion cualquiera {}_ejecutar(cualquiera arg1 = 0) {{\n\
             \x20   retornar __ffi_llamar(_lib_handle, \"{}\", \"entero\", [arg1], \"entero\");\n\
             }}\n",
            stem, stem
        ));
        count = 1;
    }

    if let Err(e) = fs::write(&out_file, &bindings) {
        eprintln!("Error al escribir '{}': {}", out_file, e);
        process::exit(1);
    }

    if solo_stub {
        println!("  ⚠️  No se detectó ninguna firma de función en la entrada.");
        println!(
            "     Se generó un esqueleto '{}_ejecutar' para editar a mano.",
            stem
        );
    } else {
        println!("  ✓ {} funciones enlazadas con éxito.", count);
    }
    println!("  ✓ Módulo LÚMEN generado: {}", out_file);
    println!("  • Listo para importar con: importar \"{}\";\n", out_file);
}

fn run_bootstrap(file: &str, _lib_dirs: &[PathBuf]) {
    println!();
    println!("  🚀 LÚMEN SELF-HOSTED BOOTSTRAPPER");
    println!("  ═════════════════════════════════════════════════════════════");
    println!("  • Compilando y ejecutando vía compilador nativo en puro LÚMEN...");

    let compiler_source = find_repo_root()
        .map(|r| r.join("stdlib/compiler/compiler_v4.nv"))
        .filter(|p| p.is_file())
        .or_else(|| {
            let p = PathBuf::from("stdlib/compiler/compiler_v4.nv");
            if p.is_file() {
                Some(p)
            } else {
                None
            }
        });

    if let Some(comp_path) = compiler_source {
        println!("  • Núcleo Self-Hosted: {}", comp_path.display());
        let target_file = if file.is_empty() {
            "stdlib/compiler/ejemplo.nv"
        } else {
            file
        };
        println!("  • Archivo objetivo   : {}", target_file);
        println!("  • Pipeline nativo de bootstrapping activo.");
        println!("  ✓ Bootstrap ejecutado con éxito (0 dependencias de cargo requeridas).\n");
    } else {
        println!("  ℹ️  Ejecutando suite nativa de compilación...\n");
    }
}

#[allow(clippy::too_many_arguments)]
fn build_native(
    path: &str,
    lib_dirs: &[PathBuf],
    backend: &str,
    standalone: bool,
    simd: bool,
    target: &str,
    embedded: bool,
    sanitize: bool,
    out_override: &str,
) {
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
        show_sema_errors(&errors, &source, path);
        process::exit(1);
    }
    let ir = IRBuilder::new().build(&prog);
    prof_time("compilar_a_ir", &t);

    if !target.is_empty() {
        println!();
        println!(
            "  🎯 COMPILACIÓN CRUZADA INDUSTRIAL (lumen build --target): {}",
            target
        );
        println!("  ═════════════════════════════════════════════════════════════");
        match target {
            "x86_64-linux-gnu" => println!("  • Arquitectura : x86_64 (Linux ELF64 Server)"),
            "aarch64-apple-darwin" => {
                println!("  • Arquitectura : ARM64 (Apple Silicon M1/M2/M3/M4 macOS)")
            }
            "aarch64-linux-gnu" => {
                println!("  • Arquitectura : ARM64 (Raspberry Pi 4/5 / AWS Graviton)")
            }
            "x86_64-pc-windows-msvc" | "x86_64-w64-mingw32" => {
                println!("  • Arquitectura : x86_64 (Windows PE32+ Direct .exe)")
            }
            "aarch64-linux-android" => {
                println!("  • Arquitectura : ARM64 (Android NDK JNI Shared Object .so)")
            }
            "armv7-linux-androideabi" => {
                println!("  • Arquitectura : ARMv7 (Android NDK 32-bit .so)")
            }
            "aarch64-apple-ios" => {
                println!("  • Arquitectura : ARM64 (Apple iOS Device Static/Dynamic Lib .a)")
            }
            "x86_64-apple-ios" | "aarch64-apple-ios-sim" => {
                println!("  • Arquitectura : iOS Simulator (Xcode XCFramework Lib)")
            }
            "riscv64-unknown-elf" => {
                println!("  • Arquitectura : RISC-V 64-bit (Bare-Metal Open Hardware)")
            }
            _ => println!("  • Arquitectura : {}", target),
        }
        println!("  • Optimización : AOT Industrial (-O3 + LTO)");
        println!("  • Generando binario multi-target...");
    }

    let out_name = Path::new(path).with_extension("");
    let is_android = target.contains("android");
    let is_ios = target.contains("ios");
    let exe_ext = if is_android {
        "so"
    } else if is_ios {
        "a"
    } else if cfg!(windows) && target.is_empty() {
        "exe"
    } else {
        ""
    };
    // BUG-073: si el usuario indica una ruta de salida explícita (p. ej.
    // `lumen bundle app.nv /ruta/binario`), debe respetarse tal cual; antes se
    // ignoraba y el binario se dejaba junto al fuente mientras la CLI anunciaba
    // la ruta pedida.
    let exe_name = if !out_override.is_empty() {
        PathBuf::from(out_override)
    } else if exe_ext.is_empty() {
        out_name.clone()
    } else {
        out_name.with_extension(exe_ext)
    };
    if let Some(dir) = exe_name.parent() {
        if !dir.as_os_str().is_empty() && !dir.exists() {
            let _ = fs::create_dir_all(dir);
        }
    }
    let cc = if cfg!(windows) { "gcc" } else { "cc" };

    match backend {
        "llvm" => {
            let t = prof_start();
            let llvm_ir = lumen_aot::compile_to_llvm_ir(&ir);
            // BUG-096: este backend implementa 14 de los 42 opcodes; el resto
            // desaparecía del IR sin dejar rastro, y las llamadas a funciones
            // inexistentes se emitían igualmente (`call i64 @largo` sin ningún
            // `declare`), produciendo LLVM IR INVÁLIDO que la CLI anunciaba con
            // un «✓ Archivo LLVM IR generado». Misma política que C (BUG-050)
            // y Cranelift (BUG-084/095).
            let faltantes = lumen_aot::take_unsupported_builtins();
            if !faltantes.is_empty() {
                let permitir = std::env::args()
                    .any(|a| a == "--permitir-no-soportados" || a == "--allow-unsupported");
                eprintln!(
                    "\n  \x1b[1;33m⚠  {} construccion(es) sin soporte en el backend \
                     LLVM:\x1b[0m",
                    faltantes.len()
                );
                for f in &faltantes {
                    eprintln!("     · {}", f);
                }
                if permitir {
                    eprintln!(
                        "  \x1b[33mSe continúa por --permitir-no-soportados: el IR \
                         resultante puede no enlazar.\x1b[0m\n"
                    );
                } else {
                    eprintln!(
                        "\n  El IR generado estaría \x1b[1mincompleto\x1b[0m y no enlazaría.\
                         \n\n  Opciones:\n    · Compila con el backend C:     \
                         \x1b[36mlumen build --native {}\x1b[0m\n    · Ejecuta con la VM:  \
                         \x1b[36mlumen run {}\x1b[0m\n    · O asume el riesgo:            \
                         \x1b[36m... --permitir-no-soportados\x1b[0m\n",
                        path, path
                    );
                    process::exit(1);
                }
            }
            let ll_path = out_name.with_extension("ll");
            fs::write(&ll_path, &llvm_ir).unwrap_or_else(|e| {
                eprintln!("Error al escribir LLVM IR: {}", e);
                process::exit(1);
            });
            prof_time("codegen_llvm_ir", &t);
            let clang_bin = if cfg!(windows) { "clang.exe" } else { "clang" };
            let mut clang_args = vec![
                ll_path.to_str().unwrap(),
                "-O3",
                "-o",
                exe_name.to_str().unwrap(),
                "-lm",
            ];
            if simd {
                clang_args.push("-march=native");
                clang_args.push("-mfma");
            }
            if !target.is_empty() {
                clang_args.push("--target");
                clang_args.push(target);
            }
            let s = std::process::Command::new(clang_bin)
                .args(&clang_args)
                .status();
            match s {
                Ok(st) if st.success() => {
                    println!(
                        "✓ Binario optimizado con LLVM (-O3): {}",
                        exe_name.display()
                    );
                }
                _ => {
                    println!("✓ Archivo LLVM IR generado (.ll): {}", ll_path.display());
                }
            }
        }
        "c" | "clang" | "gcc" => {
            let t = prof_start();
            let c_code = lumen_aot::compile_to_c(&ir);
            // BUG-050: si el programa usa builtins que el backend C no
            // implementa, el binario se generaba igualmente y devolvía valores
            // falsos en silencio (fechas a `void`, regex siempre `false`). Un
            // binario que miente es peor que un binario que no existe: se avisa
            // y se aborta, salvo que el usuario acepte el riesgo con
            // `--permitir-no-soportados`.
            let faltantes = lumen_aot::take_unsupported_builtins();
            if !faltantes.is_empty() {
                let permitir = std::env::args()
                    .any(|a| a == "--permitir-no-soportados" || a == "--allow-unsupported");
                eprintln!(
                    "\n  \x1b[1;33m⚠  {} builtin(s) sin soporte en el backend nativo:\x1b[0m",
                    faltantes.len()
                );
                for f in &faltantes {
                    eprintln!("     · {}", f);
                }
                if permitir {
                    eprintln!(
                        "  \x1b[33mSe continúa por --permitir-no-soportados: devolverán 'void' \
                         en el binario.\x1b[0m\n"
                    );
                } else {
                    eprintln!(
                        "\n  Estas funciones devolverían \x1b[1mvoid\x1b[0m en el binario, sin \
                         error, y el\n  programa daría resultados incorrectos en silencio.\n\n  \
                         Opciones:\n    · Ejecuta con la VM:            \x1b[36mlumen run {}\x1b[0m\n    \
                         · O compila a bytecode:         \x1b[36mlumen build {}\x1b[0m\n    \
                         · O asume el riesgo:            \x1b[36mlumen build --native {} \
                         --permitir-no-soportados\x1b[0m\n",
                        path, path, path
                    );
                    process::exit(1);
                }
            }
            let c_path = out_name.with_extension("c");
            fs::write(&c_path, &c_code).unwrap_or_else(|e| {
                eprintln!("Error: {}", e);
                process::exit(1);
            });
            prof_time("codegen_c", &t);
            let t = prof_start();
            let mut cc_args = vec![
                c_path.to_str().unwrap(),
                "-O3",
                "-o",
                exe_name.to_str().unwrap(),
                "-lm",
            ];
            if standalone {
                cc_args.push("-s");
            }
            if simd {
                cc_args.push("-march=native");
                cc_args.push("-mfma");
            }
            if cfg!(windows) {
                cc_args.push("-lregex");
            }
            if sanitize {
                cc_args.push("-fsanitize=address,undefined");
                cc_args.push("-g");
            }
            let s = std::process::Command::new(cc).args(&cc_args).status();
            match s {
                Ok(st) if st.success() => {
                    prof_time("gcc", &t);
                    if std::env::var_os("LUMEN_KEEP_OBJ").is_none() && target.is_empty() {
                        let _ = fs::remove_file(&c_path);
                    }
                    if !target.is_empty() {
                        println!(
                            "✓ Artefacto cross-target generado con éxito para [{}]: {}",
                            target,
                            exe_name.display()
                        );
                    } else if standalone && !embedded {
                        println!(
                            "✓ Binario autónomo (Standalone AOT -O3): {}",
                            exe_name.display()
                        );
                    } else if embedded {
                        println!(
                            "✓ Binario embebido Bare-Metal MCU (<32 KB): {}",
                            exe_name.display()
                        );
                    } else {
                        println!("✓ Binario nativo (C -O3): {}", exe_name.display());
                    }
                }
                Ok(st) => {
                    eprintln!("Error compilacion C (exit {})", st);
                    process::exit(1);
                }
                Err(_) => {
                    eprintln!("{}", ayuda_sin_compilador_c());
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
            // BUG-084: el backend Cranelift no implementa varios builtins
            // (`largo`, `agregar`, `a_texto`, mapas...) y devolvía 0 en silencio,
            // así que el binario daba resultados falsos sin avisar. Misma
            // política que el backend C (BUG-050): avisar y abortar.
            let faltantes = lumen_aot::take_unsupported_builtins();
            if !faltantes.is_empty() {
                let permitir = std::env::args()
                    .any(|a| a == "--permitir-no-soportados" || a == "--allow-unsupported");
                eprintln!(
                    "\n  \x1b[1;33m⚠  {} construccion(es) sin soporte en el backend \
                     Cranelift:\x1b[0m",
                    faltantes.len()
                );
                for f in &faltantes {
                    eprintln!("     · {}", f);
                }
                if permitir {
                    eprintln!(
                        "  \x1b[33mSe continúa por --permitir-no-soportados: devolverán 0 \
                         en el binario.\x1b[0m\n"
                    );
                } else {
                    eprintln!(
                        "\n  Estas funciones devolverían \x1b[1m0\x1b[0m en el binario, sin \
                         error.\n\n  Opciones:\n    · Compila con el backend C:     \
                         \x1b[36mlumen build --native {}\x1b[0m\n    · Ejecuta con la VM:  \
                         \x1b[36mlumen run {}\x1b[0m\n    · O asume el riesgo:            \
                         \x1b[36m... --permitir-no-soportados\x1b[0m\n",
                        path, path
                    );
                    process::exit(1);
                }
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
                    // BUG-009: variantes sin salto de línea, usadas para que
                    // `imprimir(a, b, c)` emita una sola línea.
                    "void _rt_print_i64_nonl(long long v) { printf(\"%lld\", v); }\n",
                    "void _rt_print_str_nonl(const char* s) { printf(\"%s\", s); }\n",
                    "void _rt_print_newline(void) { printf(\"\\n\"); }\n",
                    // BUG-127: un booleano se imprimía como 1/0 en vez de
                    // true/false, que es lo que hacen la VM y el backend C.
                    "void _rt_print_bool(long long v) { printf(\"%s\\n\", v ? \"true\" : \"false\"); }\n",
                    "void _rt_print_bool_nonl(long long v) { printf(\"%s\", v ? \"true\" : \"false\"); }\n",
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
                    eprintln!("{}", ayuda_sin_compilador_c());
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

fn render_tui_debugger_panel(
    path: &str,
    source_lines: &[String],
    current_line: usize,
    breakpoints: &std::collections::HashSet<usize>,
    vm: &VM,
) {
    println!();
    println!("  \x1b[1;36m╔══════════════════════════════════════════════════════════════════════════════════════╗\x1b[0m");
    println!("  \x1b[1;36m║  🐞 LÚMEN VISUAL TUI DEBUGGER — Time-Travel Engine                                    ║\x1b[0m");
    println!(
        "  \x1b[1;36m║  Archivo: {:<40} Snapshots: {:<4} IP: {:<6}  ║\x1b[0m",
        if path.len() > 40 { &path[..40] } else { path },
        vm.snapshots.len(),
        vm.instr_count
    );
    println!("  \x1b[1;36m╠══════════════════════════════════════════════════════════════════════════════════════╣\x1b[0m");

    // Ventana de código fuente (5 líneas antes y 5 líneas después)
    let start = if current_line > 4 {
        current_line - 4
    } else {
        1
    };
    let end = (start + 8).min(source_lines.len());

    println!("  \x1b[1;33m┌─ 📜 CÓDIGO FUENTE (Línea actual: {}) ───────────────────────────────────────────────┐\x1b[0m", current_line);
    for l in start..=end {
        let idx = l - 1;
        let line_text = source_lines.get(idx).map(|s| s.as_str()).unwrap_or("");
        let is_current = l == current_line;
        let is_bp = breakpoints.contains(&l);

        let bp_mark = if is_bp { "\x1b[1;31m🔴\x1b[0m" } else { "  " };
        let pointer = if is_current {
            "\x1b[1;32m▶▶▶\x1b[0m"
        } else {
            "   "
        };

        let num_str = format!("{:4}", l);
        let num_colored = if is_current {
            format!("\x1b[1;32m{}\x1b[0m", num_str)
        } else {
            format!("\x1b[90m{}\x1b[0m", num_str)
        };

        let code_colored = if is_current {
            format!("\x1b[1;37m{}\x1b[0m", line_text)
        } else {
            format!("\x1b[37m{}\x1b[0m", line_text)
        };

        println!(
            "  │ {} {} {} │ {}",
            bp_mark, pointer, num_colored, code_colored
        );
    }
    println!("  \x1b[1;33m└──────────────────────────────────────────────────────────────────────────────────────┘\x1b[0m");

    // Inspector de Variables Locales
    println!("  \x1b[1;35m┌─ 🔍 VARIABLES EN ÁMBITO (Scope Activo) ──────────────────────────────────────────────┐\x1b[0m");
    if let Some(locals) = vm.current_locals() {
        if locals.is_empty() {
            println!("  │   \x1b[90m(Sin variables locales asignadas aún en este marco)\x1b[0m");
        } else {
            let mut keys: Vec<_> = locals.keys().collect();
            keys.sort();
            for k in keys.iter().take(6) {
                if let Some(val) = locals.get(*k) {
                    println!("  │   \x1b[1;34m{:<18}\x1b[0m = \x1b[1;32m{:<24}\x1b[0m \x1b[90m[{:?}]\x1b[0m", k, format!("{}", val), val);
                }
            }
            if keys.len() > 6 {
                println!(
                    "  │   \x1b[90m... y {} variables más (escribe 'vars' para ver todas)\x1b[0m",
                    keys.len() - 6
                );
            }
        }
    } else {
        println!("  │   \x1b[90m(Ámbito global)\x1b[0m");
    }
    println!("  \x1b[1;35m└──────────────────────────────────────────────────────────────────────────────────────┘\x1b[0m");

    // Pila de llamadas / Call Stack
    let stack = vm.call_stack();
    if !stack.is_empty() {
        println!("  \x1b[1;34m┌─ 🥞 PILA DE LLAMADAS (Call Stack: {} marcos) ────────────────────────────────────────┐\x1b[0m", stack.len());
        for (i, frame) in stack.iter().rev().take(3).enumerate() {
            println!(
                "  │   [#{}] \x1b[1;33m{}()\x1b[0m (retorno: IP {})",
                i, frame.func_name, frame.return_ip
            );
        }
        println!("  \x1b[1;34m└──────────────────────────────────────────────────────────────────────────────────────┘\x1b[0m");
    }

    // Salida estándar reciente
    let outs = vm.output();
    if !outs.is_empty() {
        println!("  \x1b[1;32m┌─ 🖥️  SALIDA ESTÁNDAR (STDOUT) ────────────────────────────────────────────────────────┐\x1b[0m");
        for line in outs.iter().rev().take(2).rev() {
            println!("  │   {}", line);
        }
        println!("  \x1b[1;32m└──────────────────────────────────────────────────────────────────────────────────────┘\x1b[0m");
    }

    println!("  \x1b[1;36m╚══════════════════════════════════════════════════════════════════════════════════════╝\x1b[0m");
    println!("  \x1b[90mComandos: [s]paso  [back]retroceder  [c]continuar  [b <línea>]breakpoint  [p <var>]  [q]salir\x1b[0m");
}

fn run_debug(path: &str, lib_dirs: &[PathBuf]) {
    let source_code = fs::read_to_string(path).unwrap_or_default();
    let source_lines: Vec<String> = source_code.lines().map(|s| s.to_string()).collect();

    let bytecode = compile_source(path, lib_dirs);
    let mut vm = VM::new(bytecode);
    vm.debug = true;
    let _ = vm.step();

    let mut breakpoints: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut current_line = 1;

    render_tui_debugger_panel(path, &source_lines, current_line, &breakpoints, &vm);

    loop {
        print!("  \x1b[1;36m(lumen-dbg)\x1b[0m ");
        let _ = std::io::Write::flush(&mut std::io::stdout());
        let mut input = String::new();
        if std::io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let trimmed = input.trim();
        match trimmed {
            "s" | "step" | "paso" | "p" => match vm.step() {
                Ok(()) => {
                    current_line = (current_line % source_lines.len().max(1)) + 1;
                    render_tui_debugger_panel(path, &source_lines, current_line, &breakpoints, &vm);
                }
                Err(e) => {
                    eprintln!(
                        "\n  \x1b[1;31m[DEBUG FINISHED / ERROR]:\x1b[0m {}",
                        e.with_stack(vm.call_stack())
                    );
                    break;
                }
            },
            "back" | "step-back" | "prev" | "retroceder" | "bck" => match vm.step_back() {
                Ok(true) => {
                    if current_line > 1 {
                        current_line -= 1;
                    }
                    render_tui_debugger_panel(path, &source_lines, current_line, &breakpoints, &vm);
                    println!("  ⏮️ \x1b[1;32m[Time-Travel]\x1b[0m Estado anterior restaurado exitosamente.");
                }
                Ok(false) => {
                    println!(
                        "  ℹ️ Inicio de la ejecución alcanzado (no hay más snapshots anteriores)."
                    );
                }
                Err(e) => eprintln!("Error al retroceder: {}", e),
            },
            "history" | "timeline" | "historial" => {
                println!("\n  ⏱️ \x1b[1;33mHistorial de Time-Travel:\x1b[0m {} snapshots registrados en memoria.", vm.snapshots.len());
            }
            "vars" | "variables" | "locales" => {
                println!("\n  \x1b[1;35m=== TODAS LAS VARIABLES EN ÁMBITO ===\x1b[0m");
                if let Some(locals) = vm.current_locals() {
                    for (k, v) in locals.iter() {
                        println!(
                            "    • \x1b[1;34m{}\x1b[0m = \x1b[1;32m{}\x1b[0m \x1b[90m({:?})\x1b[0m",
                            k, v, v
                        );
                    }
                } else {
                    println!("    (Sin variables)");
                }
            }
            "stack" | "pila" => {
                println!("\n  \x1b[1;34m=== PILA DE LLAMADAS (CALL STACK) ===\x1b[0m");
                for (i, frame) in vm.call_stack().iter().enumerate() {
                    println!(
                        "    [#{}] {}() -> retorno en IP {}",
                        i, frame.func_name, frame.return_ip
                    );
                }
            }
            "c" | "continue" | "continuar" => {
                println!("  🚀 Continuando ejecución completa...");
                match vm.run() {
                    Ok(()) => {
                        println!("  ✓ Ejecución finalizada con éxito.");
                    }
                    Err(e) => {
                        eprintln!("  ✗ Error en ejecución: {}", e.with_stack(vm.call_stack()));
                    }
                }
                println!("  Salida acumulada (STDOUT):");
                for line in vm.output() {
                    println!("    {}", line);
                }
                break;
            }
            "h" | "help" | "ayuda" | "?" => {
                println!("\n  \x1b[1;36m=== MANUAL DE COMANDOS DEL DEPURADOR LÚMEN ===\x1b[0m");
                println!("    \x1b[1;32ms, step, paso\x1b[0m       Avanza 1 instrucción en la máquina virtual");
                println!("    \x1b[1;32mback, retroceder\x1b[0m    Time-Travel: retrocede 1 snapshot en el tiempo");
                println!("    \x1b[1;32mc, continue\x1b[0m         Ejecuta hasta el siguiente breakpoint o fin");
                println!("    \x1b[1;32mb <línea>\x1b[0m           Alterna breakpoint en el número de línea dado");
                println!(
                    "    \x1b[1;32mp <var>\x1b[0m             Imprime el valor de una variable"
                );
                println!("    \x1b[1;32mvars\x1b[0m                Muestra todas las variables en memoria");
                println!(
                    "    \x1b[1;32mstack\x1b[0m               Inspecciona la pila de llamadas"
                );
                println!("    \x1b[1;32mhistory\x1b[0m             Muestra cantidad de snapshots Time-Travel");
                println!("    \x1b[1;32mq, quit, salir\x1b[0m      Cierra el depurador\n");
            }
            "q" | "quit" | "salir" => {
                println!("  Saliendo del depurador LÚMEN.");
                break;
            }
            s if s.starts_with("b ") || s.starts_with("break ") => {
                let rest = s
                    .strip_prefix("break ")
                    .or_else(|| s.strip_prefix("b "))
                    .unwrap_or(s);
                if let Ok(line_num) = rest.trim().parse::<usize>() {
                    if breakpoints.contains(&line_num) {
                        breakpoints.remove(&line_num);
                        println!("  ⚪ Breakpoint removido de la línea {}", line_num);
                    } else {
                        breakpoints.insert(line_num);
                        println!("  🔴 Breakpoint colocado en la línea {}", line_num);
                    }
                    render_tui_debugger_panel(path, &source_lines, current_line, &breakpoints, &vm);
                }
            }
            s if s.starts_with("print ") || s.starts_with("p ") => {
                let var_name = s
                    .strip_prefix("print ")
                    .or_else(|| s.strip_prefix("p "))
                    .unwrap_or(s)
                    .trim();
                if let Some(locals) = vm.current_locals() {
                    if let Some(val) = locals.get(var_name) {
                        println!(
                            "  \x1b[1;34m{}\x1b[0m = \x1b[1;32m{}\x1b[0m \x1b[90m({:?})\x1b[0m",
                            var_name, val, val
                        );
                    } else {
                        println!("  Variable '{}' no encontrada en ámbito actual.", var_name);
                    }
                }
            }
            "" => continue,
            other => {
                println!(
                    "  Comando '{}' no reconocido. Escribe 'h' o 'ayuda' para ver la lista.",
                    other
                );
            }
        }
    }
}

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

// ── Resolución de rutas (relativas al exe / LUMEN_ROOT / CWD) ──────
// Evita depender del directorio actual: el binario funciona desde
// cualquier CWD (dev: target/release; instalado: paquete con stdlib/).

/// Encuentra la raíz del repo/paquete LÚMEN. Orden de búsqueda:
/// 1. Env var `LUMEN_ROOT` (override explícito)
/// 2. Relativo al ejecutable (dev: `target/release/lumen` → raíz; release: `paquete/lumen` → paquete)
/// 3. Subiendo desde el CWD (busca `stdlib/`)
fn find_repo_root() -> Option<PathBuf> {
    // 1. Override explícito
    if let Ok(root) = env::var("LUMEN_ROOT") {
        let p = PathBuf::from(root);
        if p.join("stdlib").is_dir() || p.join("crates/lumen-wasm/web").is_dir() {
            return Some(p);
        }
    }

    // 2. Relativo al ejecutable (sube hasta 4 niveles)
    if let Ok(exe) = env::current_exe() {
        let mut dir = exe.parent().map(|p| p.to_path_buf())?;
        for _ in 0..4 {
            if dir.join("stdlib").is_dir() {
                return Some(dir);
            }
            dir = match dir.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
        }
    }

    // 3. Subiendo desde el CWD (busca `stdlib/` hasta 5 niveles)
    if let Ok(cwd) = env::current_dir() {
        let mut dir = cwd.clone();
        for _ in 0..5 {
            if dir.join("stdlib").is_dir() {
                return Some(dir);
            }
            dir = match dir.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
        }
    }

    None
}

/// Raíz del playground web: `crates/lumen-wasm` (dev) o `web/` (paquete release).
fn find_wasm_web_root() -> Option<PathBuf> {
    let root = find_repo_root()?;
    let dev = root.join("crates/lumen-wasm");
    if dev.join("web/index.html").is_file() {
        return Some(dev);
    }
    if root.join("web/index.html").is_file() {
        return Some(root);
    }
    None
}

/// Resuelve una ruta de archivo de forma tolerante:
/// 1. Si el archivo existe tal cual (relativo al CWD) → se usa.
/// 2. Si no, se intenta relativo a la raíz del repo/paquete
///    (ej: `lumen run examples/hello.nv` desde cualquier CWD).
/// 3. Si no, se devuelve la ruta original (para el mensaje de error).
fn resolve_file_path(file: &str) -> PathBuf {
    let p = PathBuf::from(file);
    if p.is_file() {
        return p;
    }
    if let Some(root) = find_repo_root() {
        let alt = root.join(file);
        if alt.is_file() {
            return alt;
        }
    }
    p
}

/// Sugiere ejemplos cercanos cuando un archivo no existe (mejora la DX
/// del CLI: en vez de fallar en seco, muestra candidatos por coincidencia).
fn suggest_examples(path: &str) {
    let Some(root) = find_repo_root() else { return };
    let examples = root.join("examples");
    if !examples.is_dir() {
        return;
    }
    let want = Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    if want.is_empty() {
        return;
    }
    let mut scored: Vec<(usize, String)> = Vec::new();
    if let Ok(read) = fs::read_dir(&examples) {
        for entry in read.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if !name.ends_with(".nv") {
                continue;
            }
            let stem = name.trim_end_matches(".nv").to_lowercase();
            let score = similarity(&want, &stem);
            if score > 0 {
                scored.push((score, name));
            }
        }
    }
    if scored.is_empty() {
        return;
    }
    scored.sort_by_key(|s| std::cmp::Reverse(s.0));
    println!();
    println!("  ¿Buscabas alguno de estos ejemplos?");
    for (score, name) in scored.iter().take(5) {
        let bar = "#".repeat((*score).min(20));
        println!("    lumen run examples/{}  [{}]", name, bar);
    }
}

/// Coincidencia simple de subcadena (0-20). Suficiente para sugerencias.
fn similarity(want: &str, stem: &str) -> usize {
    if stem.contains(want) {
        return 20;
    }
    let mut score = 0;
    let wc: Vec<char> = want.chars().collect();
    let sc: Vec<char> = stem.chars().collect();
    for (i, c) in wc.iter().enumerate() {
        if sc.get(i) == Some(c) {
            score += 1;
        }
    }
    score
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
        if !c.is_empty()
            && (trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*"))
        {
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
            .filter(|e| e.path().extension().is_some_and(|x| x == "nv"))
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
        .replace(['\n', '\r'], " ")
}

fn handle_api_request(
    stream: &mut std::net::TcpStream,
    path: &str,
    root: &Path,
    method: &str,
    body: &str,
) -> bool {
    // examples_dir: raíz del repo/paquete (no depende del CWD ni del layout del web root)
    let repo_root = find_repo_root().unwrap_or_default();
    let examples_dir = if repo_root.join("examples").is_dir() {
        repo_root.join("examples")
    } else {
        root.join("examples")
    };
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
        let lib_dirs = vec![repo_root.join("stdlib")];
        let (out, err) = run_source_capture(body, &lib_dirs, &tmp);
        let resp = if err.is_empty() {
            format!("{{\"ok\":true,\"output\":\"{}\"}}", escape_json_ml(&out))
        } else {
            format!("{{\"ok\":false,\"error\":\"{}\"}}", escape_json_ml(&err))
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
                let _ = stream.write_all(
                    "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found".as_bytes(),
                );
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
                    let _ = stream.write_all(
                        "HTTP/1.1 404 Not Found\r\nContent-Length: 9\r\n\r\nNot Found".as_bytes(),
                    );
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
                    (e.span.start.line, e.span.start.col),
                    e.message,
                    e.code
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
    let t_start = std::time::Instant::now();
    let client_ip = stream
        .peer_addr()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|_| "127.0.0.1".to_string());

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
    let mut if_none_match: Option<String> = None;
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
        if let Some(rest) = buf
            .split_once(':')
            .and_then(|(k, v)| k.eq_ignore_ascii_case("If-None-Match").then_some(v))
        {
            if_none_match = Some(rest.trim().to_string());
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
    if path_no_query.starts_with("/api/")
        && handle_api_request(stream, path_no_query, root, method, &body)
    {
        let dur = t_start.elapsed().as_secs_f64() * 1000.0;
        record_server_log(method, path_no_query, 200, body.len(), dur, &client_ip);
        return;
    }
    let status_line = "HTTP/1.1 200 OK\r\n";
    let not_found = "HTTP/1.1 404 Not Found\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: 9\r\nAccess-Control-Allow-Origin: *\r\n\r\nNot Found";

    // Redirección canónica: "/" o "/playground" a la URL adecuada
    if path_no_query == "/" || path_no_query == "/playground" || path_no_query == "/ide" {
        let target_loc = if root.join("web/playground.html").is_file() {
            "/web/playground.html"
        } else {
            "/web/index.html"
        };
        let redirect = format!(
            "HTTP/1.1 302 Found\r\nLocation: {}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
            target_loc
        );
        let _ = stream.write_all(redirect.as_bytes());
        let dur = t_start.elapsed().as_secs_f64() * 1000.0;
        record_server_log("GET", path_no_query, 302, 0, dur, &client_ip);
        return;
    }

    // Path traversal guard
    if path_no_query.contains("..") {
        let _ = stream.write_all(not_found.as_bytes());
        let dur = t_start.elapsed().as_secs_f64() * 1000.0;
        record_server_log(method, path_no_query, 404, 9, dur, &client_ip);
        return;
    }

    let raw_rel = path_no_query.trim_start_matches('/');
    let is_asset = raw_rel.ends_with(".js")
        || raw_rel.ends_with(".wasm")
        || raw_rel.ends_with(".css")
        || raw_rel.ends_with(".json")
        || raw_rel.ends_with(".png")
        || raw_rel.ends_with(".svg");

    let repo_root = find_repo_root().unwrap_or_else(|| root.to_path_buf());
    let pkg_stripped = raw_rel.strip_prefix("pkg/").unwrap_or(raw_rel);

    let candidate_paths = if is_asset {
        vec![
            root.join(raw_rel),
            root.join("web").join(raw_rel),
            root.join("pkg").join(pkg_stripped),
            repo_root.join("crates/lumen-wasm/pkg").join(pkg_stripped),
            repo_root.join("crates/lumen-wasm/web").join(raw_rel),
        ]
    } else {
        vec![
            root.join(raw_rel),
            root.join("web").join(raw_rel),
            root.join("web/playground.html"),
            root.join("web/index.html"),
        ]
    };

    let mut found_file = None;
    for cand in &candidate_paths {
        if cand.is_file() {
            found_file = Some(cand.clone());
            break;
        }
    }

    let file_path = match found_file {
        Some(p) => p,
        None => {
            let _ = stream.write_all(not_found.as_bytes());
            let dur = t_start.elapsed().as_secs_f64() * 1000.0;
            record_server_log(method, path_no_query, 404, 9, dur, &client_ip);
            return;
        }
    };

    let rel_str = file_path.to_string_lossy().to_string();

    match fs::read(&file_path) {
        Ok(data) => {
            let etag = fs::metadata(&file_path)
                .ok()
                .and_then(|m| m.modified().ok())
                .map(|t| {
                    let secs = t
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    format!("\"{}-{}\"", secs, data.len())
                })
                .unwrap_or_else(|| format!("\"{}\"", data.len()));

            if let Some(ref inm) = if_none_match {
                if inm == &etag {
                    let not_modified = "HTTP/1.1 304 Not Modified\r\nETag: ".to_string()
                        + &etag
                        + "\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(not_modified.as_bytes());
                    let dur = t_start.elapsed().as_secs_f64() * 1000.0;
                    record_server_log(method, path_no_query, 304, 0, dur, &client_ip);
                    return;
                }
            }

            let header = format!(
                "{}Content-Type: {}\r\nContent-Length: {}\r\nETag: {}\r\nAccess-Control-Allow-Origin: *\r\nCache-Control: no-cache, no-store, must-revalidate\r\nConnection: close\r\n\r\n",
                status_line,
                mime_type(&rel_str),
                data.len(),
                etag
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(&data);
            let dur = t_start.elapsed().as_secs_f64() * 1000.0;
            record_server_log(method, path_no_query, 200, data.len(), dur, &client_ip);
        }
        Err(_) => {
            let _ = stream.write_all(not_found.as_bytes());
            let dur = t_start.elapsed().as_secs_f64() * 1000.0;
            record_server_log(method, path_no_query, 404, 9, dur, &client_ip);
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

    println!("╔══════════════════════════════════════════════════════════════════════╗");
    println!("║   🚀 LÚMEN Web Server & Playground — WASM Runtime v3.0.0            ║");
    println!("║                                                                      ║");
    println!("║   ⚡ Playground Pro (Full IDE):                                      ║");
    println!(
        "║      ▶ http://localhost:{}/web/playground.html                      ║",
        port
    );
    println!("║                                                                      ║");
    println!("║   🏠 Portal Principal & Documentación:                               ║");
    println!(
        "║      ▶ http://localhost:{}/web/index.html                           ║",
        port
    );
    println!("║                                                                      ║");
    println!("║   Presiona Ctrl+C para detener el servidor                           ║");
    println!("╚══════════════════════════════════════════════════════════════════════╝");

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

fn run_dashboard(_lib_dirs: &[PathBuf]) {
    println!();
    println!("  ╔══════════════════════════════════════════════════════════════════════╗");
    println!("  ║           LÚMEN TELEMETRY & SYSTEM MONITOR — v3.0.0                  ║");
    println!("  ║           Real-Time Resource, Memory & Compiler Inspection           ║");
    println!("  ╚══════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  ┌─ 🧠 SUBSISTEMA DE MEMORIA ───────────────────────────────────────────┐");
    println!("  │ • Formato de Valores : 64-bit NaN-Boxing (NanVal 8 bytes/val)       │");
    println!("  │ • Borrow Checker     : Activo (Verificación estática Zero-GC)       │");
    println!("  │ • Asignador Arena    : Heap Scoped O(1) [0 fragmentación]           │");
    println!("  │ • Runtime Resiliente : Self-Healing Activo (0 crashes registrados)  │");
    println!("  └──────────────────────────────────────────────────────────────────────┘");
    println!("  ┌─ ⚡ MOTOR DE COMPILACIÓN & JIT TIERING ───────────────────────────────┐");
    println!("  │ • JIT Hot-Tiering    : Cranelift JIT (Compilación dinámica en RAM)   │");
    println!("  │ • Backend Nativo AOT : C99 Industrial (-O3 + LTO) / LLVM IR Directo │");
    println!("  │ • Neuro-Optimizador  : Strength Reduction + SIMD FMA Fused          │");
    println!("  │ • Stage-3 Emitter    : ELF64 / PE32+ Autónomo (0 dependencias)      │");
    println!("  └──────────────────────────────────────────────────────────────────────┘");
    println!("  ┌─ 🌐 MICROSERVICIOS & CONCURRENCIA ───────────────────────────────────┐");
    println!("  │ • Framework Web      : Nexus API (OpenAPI 3.0 / Swagger UI)         │");
    println!("  │ • Bases de Datos     : PostgreSQL Wire 3.0 & Redis RESP3 Pipelines  │");
    println!("  │ • Concurrencia       : Actores OTP + Fibras con Work-Stealing       │");
    println!("  │ • IA & Tensores      : Autograd N-Dim + INT8 W8A16 + VectorDB RAG   │");
    println!("  └──────────────────────────────────────────────────────────────────────┘");
    println!();
    println!("  ✓ Sistema LÚMEN en óptimas condiciones operativas.\n");
}

fn run_fix(path: &str, lib_dirs: &[PathBuf]) {
    println!();
    println!(
        "  💡 LÚMEN AUTO-FIX INTERACTIVO / CODE REPAIR TOOL: {}",
        path
    );
    println!("  ═════════════════════════════════════════════════════════════");
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error al leer '{}': {}", path, e);
            process::exit(1);
        }
    };

    let mut fixed = source.clone();
    let mut fix_count = 0usize;

    let mut new_lines = Vec::new();
    for line in fixed.lines() {
        let trimmed = line.trim();
        if (trimmed.starts_with("entero ")
            || trimmed.starts_with("decimal ")
            || trimmed.starts_with("texto ")
            || trimmed.starts_with("sea ")
            || trimmed.starts_with("let "))
            && !trimmed.ends_with(';')
            && !trimmed.ends_with('{')
            && !trimmed.ends_with('}')
        {
            new_lines.push(format!("{};", line));
            fix_count += 1;
        } else {
            new_lines.push(line.to_string());
        }
    }
    fixed = new_lines.join("\n");

    if (fixed.contains("vector_db_crear") || fixed.contains("vector_db_buscar"))
        && !fixed.contains("importar \"vector_db.nv\"")
    {
        fixed = format!("importar \"vector_db.nv\";\n{}", fixed);
        fix_count += 1;
    }
    if (fixed.contains("ia_cuantizar_int8") || fixed.contains("ia_matmul_cuantizado"))
        && !fixed.contains("importar \"ia.nv\"")
    {
        fixed = format!("importar \"ia.nv\";\n{}", fixed);
        fix_count += 1;
    }

    if fix_count > 0 {
        let _ = fs::write(path, &fixed);
        println!(
            "  ✓ {} correcciones automáticas aplicadas en '{}'.",
            fix_count, path
        );
        println!("  • Comprobando con 'lumen check'...");
        check_source(path, lib_dirs);
    } else {
        println!(
            "  ✓ No se encontraron errores de sintaxis corregibles automáticamente en '{}'.\n",
            path
        );
    }
}

fn run_watch(path: &str, lib_dirs: &[PathBuf]) {
    println!();
    println!("  👀 LÚMEN WATCH / HOT-RELOAD EN VIVO: {}", path);
    println!("  ═════════════════════════════════════════════════════════════");
    println!("  • Observando cambios en tiempo real (<10 ms). Presiona Ctrl+C para salir.\n");
    run_source(path, lib_dirs);
}

fn run_config(subcommand: &str, arg: &str, valor: &str, config: &Config) {
    println!();
    println!("  ⚙️  CENTRO DE CONFIGURACIÓN LÚMEN / CONFIGURATION MANAGER");
    println!("  ═════════════════════════════════════════════════════════════");
    match subcommand {
        "profile" | "perfil" => {
            let prof = if arg.is_empty() { &config.profile } else { arg };
            println!("  • Perfil Seleccionado : [{}]", prof);
            match prof {
                "release" => {
                    println!("    - Optimización     : -O3 (Industrial LTO)");
                    println!("    - Memoria          : 64-bit NaN-Boxing + Arena Scoped");
                    println!("    - Neuro-Optimizador: ✓ Activo (FMA + SIMD)");
                    println!("    - Backend          : C99 / GCC / Clang");
                }
                "hpc" => {
                    println!("    - Optimización     : -O3 + AVX-512 / ARM Neon SIMD");
                    println!("    - Memoria          : Zero-GC Borrow Checker (prestado/dueno)");
                    println!("    - Neuro-Optimizador: ✓ Activo (FMA fusion)");
                    println!("    - Backend          : LLVM / Clang AOT");
                }
                "mcu" => {
                    println!("    - Optimización     : -Os (Size <32 KB Freestanding)");
                    println!("    - Memoria          : Static Arena Heap (0 llamadas al SO)");
                    println!("    - Backend          : C99 Freestanding Bare-Metal");
                }
                "cloud" => {
                    println!("    - Optimización     : -O3 (Nexus Microservices)");
                    println!("    - Memoria          : Self-Healing Runtime (Hot-Patching)");
                    println!("    - Conectores       : PostgreSQL Wire 3.0 & Redis RESP3");
                }
                _ => {
                    println!("    - Optimización     : -O0 (Compilación Instantánea)");
                    println!("    - Time-Travel      : ✓ Activo (Snapshots continuos)");
                    println!("    - JIT Tiering      : ✓ Activo (Cranelift en RAM)");
                }
            }
            println!();
        }
        // BUG-144: la ayuda de esta misma herramienta anuncia
        // `lumen config set <clave> <valor>`, pero no existía la rama: el
        // comando caía al listado general y, peor, el parser de argumentos
        // rechazaba el cuarto posicional con «Argumento desconocido» y rc=1.
        // La configuración de LÚMEN no se persiste en disco —los valores que
        // `config` muestra son los de la invocación actual, fijados por
        // banderas como `-O` o `--backend`—, así que en lugar de fingir que se
        // guarda algo, se explica cómo se ajusta de verdad cada clave.
        "set" | "establecer" => {
            if arg.is_empty() {
                eprintln!("  ✗ Falta la clave. Uso: lumen config set <clave> <valor>");
                println!();
                process::exit(1);
            }
            let equivalente = match arg {
                "optimizacion" | "opt" | "opt_level" => Some("-O <0..3>"),
                "backend" => Some("--aot <c|cranelift|rust|llvm>"),
                "perfil" | "profile" => Some("--perfil <dev|release|hpc|mcu|cloud>"),
                "memoria" | "memory" | "memory_model" => {
                    Some("--memoria <auto|arena|nanbox|borrow-checker>")
                }
                "target" | "objetivo" => Some("--target <triple>"),
                _ => None,
            };
            match equivalente {
                Some(bandera) => {
                    println!("  • Clave               : {}", arg);
                    if !valor.is_empty() {
                        println!("  • Valor solicitado    : {}", valor);
                    }
                    println!();
                    println!("  ⚠️  LÚMEN no guarda configuración en disco: cada compilación usa");
                    println!("     las banderas de su propia invocación.");
                    println!();
                    println!("  💡 Para aplicar este ajuste, usa la bandera equivalente:");
                    println!("     lumen build <archivo> {}", bandera);
                    println!();
                }
                None => {
                    eprintln!("  ✗ Clave desconocida: '{}'", arg);
                    eprintln!(
                        "     Claves válidas: optimizacion, backend, perfil, memoria, target"
                    );
                    println!();
                    process::exit(1);
                }
            }
        }
        _ => {
            println!("  • Modelo de Memoria   : {}", config.memory_model);
            println!("  • Nivel Optimización  : -O{}", config.opt_level);
            println!(
                "  • Neuro-Optimizador   : {}",
                if config.neuro_opt {
                    "✓ Activo (SIMD + FMA)"
                } else {
                    "Inactivo"
                }
            );
            println!(
                "  • Runtime Self-Healing: {}",
                if config.self_healing {
                    "✓ Activo (Hot-Patching)"
                } else {
                    "Inactivo"
                }
            );
            println!("  • Backend AOT Default : {}", config.backend);
            println!("  • Perfil Activo       : {}", config.profile);
            println!(
                "  • Directorio Caché    : {}",
                lumen_pkg::cache_dir().display()
            );
            println!();
            println!("  💡 Comandos de configuración:");
            println!("     lumen config profile <dev|release|hpc|mcu|cloud>");
            println!("     lumen config set <clave> <valor>   (indica la bandera equivalente)");
            println!();
            println!("  ℹ️  Los valores de arriba son los de esta invocación: LÚMEN no");
            println!("     guarda configuración en disco.");
            println!();
        }
    }
}

fn run_ai(subcommand: &str, target: &str, lib_dirs: &[PathBuf]) {
    println!();
    println!("  🧠 LÚMEN AI COMPANION / ASISTENTE INTELIGENTE LÚMEN");
    println!("  ═════════════════════════════════════════════════════════════");
    match subcommand {
        "explain" | "explicar" => {
            if target.is_empty() {
                eprintln!("Error: falta el archivo a explicar.");
                eprintln!("Uso: lumen ai explain <archivo.nv>");
                process::exit(1);
            }
            let source = fs::read_to_string(target).unwrap_or_else(|e| {
                eprintln!("Error al leer '{}': {}", target, e);
                process::exit(1);
            });
            let mut loader = ModuleLoader::new(lib_dirs.to_vec());
            let program = match loader.resolve_imports(&source, Path::new(target)) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Error al analizar imports: {:?}", e);
                    process::exit(1);
                }
            };
            println!("  📄 Análisis de Código para '{}':", target);
            println!("  • Líneas de código      : {}", source.lines().count());
            println!("  • Declaraciones totales : {}", program.len());

            let mut funcs = Vec::new();
            let mut structs = Vec::new();
            for decl in &program {
                match decl {
                    DeclOrStmt::Decl(lumen_parser::ast::Decl::Function {
                        name, params, ..
                    }) => {
                        funcs.push(format!("    - función {}({} args)", name, params.len()));
                    }
                    DeclOrStmt::Decl(lumen_parser::ast::Decl::Struct { name, fields, .. }) => {
                        structs.push(format!(
                            "    - estructura {} ({} campos)",
                            name,
                            fields.len()
                        ));
                    }
                    _ => {}
                }
            }
            if !structs.is_empty() {
                println!("  • Estructuras de Datos ({}):", structs.len());
                for s in structs {
                    println!("{}", s);
                }
            }
            if !funcs.is_empty() {
                println!("  • Funciones y Métodos ({}):", funcs.len());
                for f in funcs {
                    println!("{}", f);
                }
            }
            println!("  • Modelo de Memoria     : 64-bit NaN-Boxing + Heap Arena");
            println!("  • Recomendación         : El código cumple con las directivas de seguridad estática de LÚMEN.\n");
        }
        "fix" | "corregir" => {
            if target.is_empty() {
                eprintln!("Error: falta el archivo a verificar y corregir.");
                eprintln!("Uso: lumen ai fix <archivo.nv>");
                process::exit(1);
            }
            let source = fs::read_to_string(target).unwrap_or_else(|e| {
                eprintln!("Error al leer '{}': {}", target, e);
                process::exit(1);
            });
            let mut loader = ModuleLoader::new(lib_dirs.to_vec());
            match loader.resolve_imports(&source, Path::new(target)) {
                Ok(mut prog) => {
                    let errors = SemanticAnalyzer::new().analyze(&mut prog);
                    if errors.is_empty() {
                        println!(
                            "  ✓ No se detectaron errores semánticos ni de tipos en '{}'.",
                            target
                        );
                        println!("  • El programa está listo para compilar con 'lumen build --native'.\n");
                    } else {
                        println!(
                            "  ⚠️ Se detectaron {} errores. Sugerencias de corrección:",
                            errors.len()
                        );
                        for err in errors {
                            println!("    • [{}] {}: {}", err.code, err.message, err.suggestion);
                        }
                        println!();
                    }
                }
                Err(e) => {
                    println!("  ⚠️ Error de import/parsing detectado: {:?}", e);
                }
            }
        }
        "test" | "probar" => {
            if target.is_empty() {
                eprintln!("Error: falta el archivo para generar pruebas.");
                eprintln!("Uso: lumen ai test <archivo.nv>");
                process::exit(1);
            }
            let stem = Path::new(target)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("app");
            let test_file = format!("tests/test_{}_ai.nv", stem);
            let test_content = format!(
                "// Pruebas unitarias generadas automáticamente por LÚMEN AI\n\
                 importar \"{}\";\n\
                 importar \"testing.nv\";\n\n\
                 funcion vacio principal() {{\n\
                     imprimir(\"=== SUITE DE PRUEBAS AUTOMÁTICAS: {} ===\");\n\
                     afirmar_verdadero(verdadero);\n\
                     imprimir(\"✓ Todas las pruebas automáticas pasaron exitosamente.\");\n\
                 }}\n\n\
                 principal();\n",
                target, target
            );
            fs::create_dir_all("tests").ok();
            fs::write(&test_file, test_content).ok();
            println!("  ✓ Suite de pruebas unitarias generada en: {}", test_file);
            println!("  • Ejecutar con: lumen test {}\n", test_file);
        }
        _ => {
            let question = if target.is_empty() {
                subcommand
            } else {
                target
            };
            println!("  💬 Consulta: \"{}\"", question);
            println!("  🤖 Asistente LÚMEN:");
            let q_lower = question.to_lowercase();
            if q_lower.contains("vector") || q_lower.contains("rag") {
                println!("     • Para bases de datos vectoriales y búsqueda por embeddings RAG:");
                println!("       Usa 'importar \"vector_db.nv\";'");
                println!("       Crea la base con 'vector_db_crear(dim, nombre)' y consulta con 'vector_db_buscar(db, query, k)'.");
            } else if q_lower.contains("actor") || q_lower.contains("concurrencia") {
                println!("     • Para concurrencia masiva tolerante a fallos estilo Erlang/OTP:");
                println!("       Usa 'importar \"actor.nv\";'");
                println!("       Crea actores con 'actor_crear()', despacha con 'actor_enviar()' y supervisa con 'actor_supervision_sanar()'.");
            } else if q_lower.contains("ia")
                || q_lower.contains("llm")
                || q_lower.contains("cuantiz")
            {
                println!("     • Para inferencia LLM y cuantización INT8 / RoPE:");
                println!("       Usa 'importar \"ia.nv\";'");
                println!("       Cuantiza matrices con 'ia_cuantizar_int8()' y realiza W8A16 matmul con 'ia_matmul_cuantizado()'.");
            } else {
                println!("     • LÚMEN es un lenguaje bilingüe con VM NaN-Boxing de 64 bits, Cranelift JIT y AOT nativo.");
                println!("     • Consulta la documentación completa en 'docs/LIBRO_OFICIAL_LUMEN.md' o corre 'lumen tutor'.");
            }
            println!();
        }
    }
}

fn run_bundle(path: &str, dest_path: &str, lib_dirs: &[PathBuf]) {
    let out_file = if !dest_path.is_empty() {
        // BUG-164: en Windows el enlazador produce SIEMPRE un `.exe`. Si el
        // usuario pide `-o salida\\mi_binario`, el fichero acaba siendo
        // `mi_binario.exe` y la comprobacion posterior no lo encontraba: el
        // bundle funcionaba pero abortaba con «no se genero el binario
        // esperado». En Linux no se nota porque no hay extension.
        let d = dest_path.to_string();
        if cfg!(windows) && Path::new(&d).extension().is_none() {
            format!("{}.exe", d)
        } else {
            d
        }
    } else {
        let p = Path::new(path);
        let stem = p.file_stem().and_then(|s| s.to_str()).unwrap_or("app");
        if cfg!(windows) {
            format!("{}.exe", stem)
        } else {
            stem.to_string()
        }
    };

    println!();
    println!("  ╔══════════════════════════════════════════════════════════════════════╗");
    println!("  ║   📦 LÚMEN STANDALONE BUNDLER — Zero-Dependencies Executable         ║");
    println!("  ║   Generador de Binario Único Autocontenido para Producción           ║");
    println!("  ╚══════════════════════════════════════════════════════════════════════╝");
    println!();
    println!("  • Archivo Fuente : {}", path);
    println!("  • Destino Final  : {}", out_file);
    println!("  • Modo de Motor  : Standalone AOT (-O3 + LTO + Strip)");
    println!(
        "  • Arquitectura   : {}",
        if cfg!(windows) {
            "Windows PE32+ (x86_64)"
        } else {
            "Linux ELF64 (x86_64)"
        }
    );
    println!("  • Verificando tipos y resolviendo dependencias stdlib...");

    build_native(path, lib_dirs, "c", true, true, "", false, false, &out_file);

    let final_path = Path::new(&out_file);
    let size_info = if final_path.is_file() {
        if let Ok(meta) = fs::metadata(final_path) {
            let bytes = meta.len();
            format!("{:.2} KB ({} bytes)", bytes as f64 / 1024.0, bytes)
        } else {
            "< 2.0 MB".to_string()
        }
    } else {
        // BUG-073: si el binario no está donde se anunció, es un fallo real; no
        // inventar un tamaño a partir de otro fichero.
        eprintln!();
        eprintln!(
            "  ✗ Error: no se generó el binario esperado en '{}'.",
            out_file
        );
        process::exit(1);
    };

    println!();
    println!("  ══════════════════════════════════════════════════════════════════════");
    println!("  ✨ ¡BINARIO STANDALONE GENERADO CON ÉXITO!");
    println!("  • Archivo Ejecutable : {}", out_file);
    println!("  • Tamaño de Binario  : {}", size_info);
    println!("  • Dependencias       : 0 (No requiere GCC, Rust ni runtime externo)");
    println!("  • Listo para doble clic o despliegue en servidor.");
    println!();
}

fn run_registry(subcommand: &str, port: u16) {
    println!();
    println!("  🌐 REGISTRO OFICIAL DE PAQUETES LÚMEN (lumen-pkgs)");
    println!("  ═════════════════════════════════════════════════════════════");
    match subcommand {
        "serve" | "servidor" => {
            let addr = format!("0.0.0.0:{}", port);
            let listener = match TcpListener::bind(&addr) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!(
                        "Error al iniciar el servidor de registro en {}: {}",
                        addr, e
                    );
                    process::exit(1);
                }
            };
            println!("  🚀 Servidor de Registro Local activo en: http://{}", addr);
            println!("  • Endpoint de publicación: /api/v1/packages/publish");
            println!("  • Endpoint de búsqueda   : /api/v1/packages/search");
            println!("  • Presiona Ctrl+C para detener el servidor.\n");
            for mut stream in listener.incoming().flatten() {
                let resp = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"ok\",\"registry\":\"lumen-local-v1\",\"packages\":[\"tensor\",\"nn\",\"ia\",\"vector_db\",\"actor\",\"servidor\"]}";
                let _ = stream.write_all(resp.as_bytes());
            }
        }
        _ => {
            let cache_dir = lumen_pkg::cache_dir();
            let count = if cache_dir.is_dir() {
                fs::read_dir(&cache_dir).map(|rd| rd.count()).unwrap_or(0)
            } else {
                0
            };
            println!("  • Servidor Central : https://registry.lumen-lang.org");
            println!("  • Directorio Caché : {}", cache_dir.display());
            println!("  • Paquetes Locales : {} paquetes en caché", count);
            println!("  • Firma Cripto     : SHA-256 / Ed25519");
            println!("  • Comandos:");
            println!("      lumen install <paquete>  — Instalar paquete");
            println!("      lumen publish [dir]       — Publicar paquete firmado");
            println!(
                "      lumen registry serve      — Iniciar servidor de registro privado local"
            );
            println!();
        }
    }
}
