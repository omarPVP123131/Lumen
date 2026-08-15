# LÚMEN — Herramientas y Desarrollo

**v2.4.2 — Guía del Desarrollador**

Guía completa de herramientas, comandos, CI/CD y flujo de trabajo.

---

## Índice

1. [Comandos CLI](#1-comandos-cli)
2. [Comandos Cargo](#2-comandos-cargo)
3. [Scripts CI/CD (Pre-Vuelo)](#3-scripts-cicd-pre-vuelo)
4. [Git Hooks](#4-git-hooks)
5. [Testing](#5-testing)
6. [Formateo](#6-formateo)
7. [Fuzzing](#7-fuzzing)
8. [Benchmarks](#8-benchmarks)
9. [Cobertura](#9-cobertura)
10. [Estructura del Proyecto](#10-estructura-del-proyecto)
11. [Flujo de Trabajo](#11-flujo-de-trabajo)

---

## 1. Comandos CLI

### `lumen run <archivo>`

Ejecuta código fuente `.nv` o bytecode compilado `.nvc`.

```bash
lumen run programa.nv
lumen run programa.nvc
```

### WASM Playground

```bash
cd crates/lumen-wasm
wasm-pack build --target web --out-dir pkg
python serve.py
# Abrir http://localhost:8080/web/index.html
```

Incluye editor con 19 ejemplos, consola JS interop, modo oscuro Catppuccin.

### `lumen build <archivo>`

Compila código fuente a bytecode `.nvc`.

```bash
lumen build programa.nv          # Genera programa.nvc
lumen build src/main.nv          # Con proyecto lumen.toml
```

### `lumen check <archivo>`

Verifica sintaxis y semántica sin ejecutar.

```bash
lumen check programa.nv
```

### `lumen test <archivo>`

Ejecuta funciones que comienzan con `test_` en el archivo.

```bash
lumen test tests.nv
```

### `lumen fmt <archivo>`

Formatea código fuente.

```bash
lumen fmt programa.nv             # Formatea y escribe en disco
lumen fmt --fix programa.nv       # Explícito: formatea y escribe
lumen fmt --check programa.nv     # Solo verifica, sin modificar
```

- Sin flags: formatea y escribe (comportamiento por defecto)
- `--fix`: igual que sin flags (explícito)
- `--check`: sale con código 1 si el archivo necesita formateo (útil en CI)

### `lumen install <paquete>`

Instala paquetes del registry LÚMEN.

```bash
lumen install coleccion        # Instala desde registry
lumen install --local ./ruta   # Instala desde ruta local
lumen install --help           # Muestra ayuda del comando
```

### `lumen repl`

Inicia el REPL interactivo.

```bash
lumen repl
> 2 + 2
=> 4
> imprimir("hola")
hola
> funcion texto saludo() { retornar "hola"; }
> saludo()
=> hola
```

### `lumen disasm <archivo>`

Desensambla bytecode `.nvc` para inspeccionar instrucciones.

```bash
lumen build programa.nv
lumen disasm programa.nvc
```

### Flags Globales

| Flag | Descripción |
|------|-------------|
| `-L <dir>` | Agrega directorio de búsqueda para módulos |
| `--lib-dir <dir>` | Igual que `-L` |
| `--version`, `-v` | Muestra la versión |
| `--help` | Muestra la ayuda |

---

## 2. Comandos Cargo

### Compilación

```bash
cargo build                          # Debug
cargo build --release                # Release optimizado
cargo check                          # Solo verificar (más rápido)
```

### Testing

```bash
cargo test                           # Todos los tests
cargo test --workspace               # Todo el workspace
cargo test -p lumen-vm               # Solo un crate
cargo test -p lumen-vm --test e2e    # Solo un archivo de test
cargo test test_string_index         # Solo un test específico
cargo test --no-fail-fast            # No parar en el primer fallo
```

### Formateo

```bash
cargo fmt                            # Formatear todo
cargo fmt --check                    # Verificar (CI)
cargo fmt -- --check -p lumen-vm     # Verificar un crate
```

### Linting

```bash
cargo clippy                         # Linter (requiere componente)
cargo clippy -- -D warnings          # Tratar warnings como errores
```

### Limpieza

```bash
cargo clean                          # Limpiar target/
```

### Documentación

```bash
cargo doc --open                     # Generar y abrir docs
cargo doc --no-deps                  # Solo este proyecto
```

---

## 3. Scripts CI/CD (Pre-Vuelo)

### `scripts/pre-vuelo.ps1`

Validación completa antes de push. Ejecuta:

```
Job: fmt → Job: check → Job: test → Job: coverage
```

```powershell
# Ejecución completa
.\scripts\pre-vuelo.ps1

# Saltar tests (solo fmt + check + coverage)
.\scripts\pre-vuelo.ps1 -SkipTests

# Saltar cobertura
.\scripts\pre-vuelo.ps1 -SkipCoverage
```

### `scripts/pre-commit.ps1`

Validación rápida antes de commit.

```
Job: fmt → Job: check
```

```powershell
.\scripts\pre-commit.ps1
```

### Reportes JSON

Ambos scripts generan reportes en `target/`:

| Archivo | Generado por |
|---------|-------------|
| `target/pre-vuelo-report.json` | `pre-vuelo.ps1` |
| `target/pre-commit-report.json` | `pre-commit.ps1` |

```powershell
# Leer reporte
Get-Content target/pre-vuelo-report.json | ConvertFrom-Json | Select-Object -ExpandProperty summary
```

---

## 4. Git Hooks

### Pre-Commit Hook

**Archivo**: `.git/hooks/pre-commit`

```powershell
#!/usr/bin/env pwsh
.\scripts\pre-commit.ps1
exit $LASTEXITCODE
```

Ejecuta `fmt` y `check` antes de cada commit.

### Pre-Push Hook

**Archivo**: `.git/hooks/pre-push`

```powershell
#!/usr/bin/env pwsh
.\scripts\pre-vuelo.ps1
exit $LASTEXITCODE
```

Ejecuta la suite completa antes de cada push.

### Instalación

```powershell
# Los hooks deben ser ejecutables
# Ya vienen configurados en el repositorio
# Verifica que PowerShell 7+ esté instalado (pwsh)
```

---

## 5. Testing

### Estructura de Tests

```
crates/
  lumen-lexer/
    src/lexer.rs           # Tests unitarios inline (#[cfg(test)])
    tests/proptest.rs      # Property-based tests
  lumen-parser/
    src/parser.rs          # Tests unitarios inline
  lumen-sema/
    src/sema.rs            # Tests unitarios inline
  lumen-ir/
    src/builder.rs         # Tests unitarios inline
  lumen-codegen/
    src/bytecode.rs        # Tests unitarios inline
    tests/proptest.rs      # Property-based tests (6 tests)
  lumen-vm/
    src/vm.rs              # Tests unitarios inline (45 tests)
    tests/e2e.rs           # End-to-end tests (117 tests)
    tests/proptest.rs      # Property-based tests (18 tests)
  lumen-repl/
    src/lib.rs             # Tests unitarios inline (21 tests)
```

### Conteo de Tests

| Crate | Tests | Tipo |
|-------|-------|------|
| lumen-lexer | 24 | unit |
| lumen-parser | 42 | unit |
| lumen-sema | 49 | unit |
| lumen-ir | 20 | unit + folding |
| lumen-codegen | 13 | unit |
| lumen-codegen | 5 | proptest |
| lumen-vm | 45 | unit |
| lumen-vm | 166 | e2e |
| lumen-fmt | 2 | unit |
| lumen-repl | 2 | unit |
| lumen-project | 1 | unit |
| lumen-aot | 1 | unit |
| lumen-doc | 1 | unit |
| lumen-pkg | 1 | unit |
| lumen-plugin | 1 | unit |
| lumen-api | 5 | unit |
| **Total** | **~378** | |

> Además: batería VM LÚMEN `test_vm.ps1` **39/40** (solo `stress_fecha` flaky timing) y `fuego.ps1` **117/117 compilan · 112 CORRECTOS** en la cadena 100% LÚMEN.

### Escribir Tests E2E

Los tests e2e están en `crates/lumen-vm/tests/e2e.rs`. Usan la función `run_source()`:

```rust
fn run_source(source: &str) -> Result<Vec<String>, String> {
    // Pipeline completo: Lexer → Parser → Sema → IR → Codegen → VM
    // Retorna la salida de imprimir() o Err con el mensaje de error
}

#[test]
fn test_mi_feature() {
    let output = run_source(r#"
        entero x = 42;
        imprimir(x);
    "#).unwrap();
    assert_eq!(output, vec!["42"]);
}
```

### Ejecutar Tests

```bash
# Todo el proyecto
cargo test --workspace

# Solo e2e
cargo test -p lumen-vm --test e2e

# Solo proptest VM
cargo test -p lumen-vm --test proptest

# Solo un test específico
cargo test test_string_index
```

---

## 6. Formateo

### `lumen fmt`

El formateador automático sigue estas reglas:

- Indentación: 4 espacios por nivel
- Llaves `{}` en líneas separadas para bloques
- Espacio después de `si`, `mientras`, `para`
- Espacios alrededor de operadores
- Coma seguida de espacio en listas/diccionarios
- Línea en blanco entre declaraciones top-level
- `retornar` sin punto y coma si es la última expresión

### Formatear Todo el Proyecto

```powershell
Get-ChildItem -Recurse -Filter *.nv | ForEach-Object {
    lumen fmt $_.FullName
}
```

### CI con `--check`

```yaml
# En GitHub Actions
- name: Check formatting
  run: |
    $files = Get-ChildItem -Recurse -Filter *.nv
    $bad = 0
    foreach ($f in $files) {
      lumen fmt --check $f.FullName
      if ($LASTEXITCODE -ne 0) { $bad++ }
    }
    if ($bad -gt 0) { exit 1 }
```

---

## 7. Fuzzing

### Fuzz Targets

Los fuzz targets están en `fuzz/`:

| Archivo | Objetivo |
|---------|----------|
| `fuzz_lexer.rs` | Lexer con entradas aleatorias |
| `fuzz_parser.rs` | Parser con secuencias de tokens |
| `fuzz_decoder.rs` | Decodificador de bytecode |

### Ejecutar Fuzzing

```bash
# Requiere cargo-fuzz instalado
cargo install cargo-fuzz

# Ejecutar fuzzer
cargo fuzz run fuzz_lexer
cargo fuzz run fuzz_parser
cargo fuzz run fuzz_decoder
```

---

## 8. Benchmarks

Los benchmarks están en `crates/lumen-bench/benches/benchmarks.rs`.

### Ejecutar Benchmarks

```bash
cargo bench -p lumen-bench
```

Los benchmarks miden:

- Compilación completa (lex → parse → sema → IR → codegen)
- Ejecución VM de programas típicos
- Operaciones individuales de VM (Add, Div, Call, etc.)

---

## 9. Cobertura

### Requisitos

- Rust **nightly** (`rustup default nightly` o `cargo +nightly`)
- `cargo-llvm-cov` (`cargo install cargo-llvm-cov`)

### Generar Reporte

```bash
# Cobertura completa del workspace
cargo llvm-cov --workspace --html

# Reporte en target/coverage/html/index.html
```

### Script Automatizado

```powershell
# pre-vuelo ya incluye cobertura
.\scripts\pre-vuelo.ps1

# O solo cobertura
.\scripts\pre-vuelo.ps1 -SkipTests
```

---

## 10. Estructura del Proyecto

```
LumenRust/
├── AGENTS.md              # Diario de construcción
├── Cargo.toml             # Workspace raíz
├── LENGUAJE.md            # Manual del lenguaje
├── HERRAMIENTAS.md        # Este documento
├── crates/
│   ├── lumen-lexer/       # Tokenizador
│   │   ├── src/token.rs       # Tipos de token + Span + Pos
│   │   ├── src/lexer.rs       # Lexer principal
│   │   ├── src/error.rs       # LexError
│   │   └── tests/proptest.rs  # Proptest lexer
│   ├── lumen-parser/      # Parser (AST)
│   │   ├── src/ast.rs         # Definiciones AST
│   │   ├── src/parser.rs      # Parser recursivo
│   │   └── src/error.rs       # ParseError
│   ├── lumen-sema/        # Análisis semántico
│   │   ├── src/sema.rs        # SemanticAnalyzer
│   │   ├── src/loader.rs      # ModuleLoader (imports)
│   │   └── src/error.rs       # SemError + ModuleError
│   ├── lumen-ir/          # Representación intermedia
│   │   ├── src/ir.rs          # Instr (opcodes IR)
│   │   └── src/builder.rs     # IRBuilder (AST → IR)
│   ├── lumen-codegen/     # Generación de bytecode
│   │   ├── src/bytecode.rs    # Bytecode + encode/decode
│   │   ├── src/codegen.rs     # Codegen (IR → bytecode)
│   │   ├── src/disasm.rs      # Disassembler
│   │   └── tests/proptest.rs  # Proptest bytecode
│   ├── lumen-vm/          # Máquina virtual
│   │   ├── src/vm.rs          # VM (stack-based)
│   │   ├── src/value.rs       # Value enum
│   │   ├── tests/e2e.rs       # Tests end-to-end
│   │   └── tests/proptest.rs  # Proptest opcodes
│   ├── lumen-cli/         # Interfaz de línea de comandos
│   │   └── src/main.rs        # CLI (run, build, check, fmt, etc.)
│   ├── lumen-fmt/         # Formateador
│   │   └── src/lib.rs         # format_source()
│   ├── lumen-repl/        # REPL interactivo
│   │   └── src/lib.rs         # Repl struct + eval
│   ├── lumen-project/     # Gestión de proyectos
│   │   └── src/lib.rs         # ProjectManifest
│   ├── lumen-lsp/         # Language Server Protocol
│   └── lumen-bench/       # Benchmarks
├── examples/
│   ├── junior/            # 50 ejemplos básicos
│   ├── senior/            # 80 ejemplos avanzados
│   └── real/              # 70 ejemplos de software real
├── stdlib/                # Librería estándar (.nv)
│   ├── texto.nv
│   ├── matematicas.nv
│   ├── coleccion.nv
│   ├── fecha.nv
│   └── archivos.nv
├── docs/
│   └── spec/              # Especificaciones técnicas
│       ├── grammar.ebnf
│       └── bytecode-format.md
├── fuzz/                  # Fuzz targets
│   ├── fuzz_lexer.rs
│   ├── fuzz_parser.rs
│   └── fuzz_decoder.rs
└── scripts/               # PowerShell CI/CD
    ├── pre-vuelo.ps1
    └── pre-commit.ps1
```

---

## 11. Flujo de Trabajo

### Desarrollo Diario

```bash
# 1. Crear feature branch
git checkout -b feature/mi-feature

# 2. Escribir código...

# 3. Formatear
cargo fmt
lumen fmt mi_archivo.nv

# 4. Verificar compilación
cargo check

# 5. Correr tests
cargo test --workspace

# 6. Pre-commit (antes de commit)
.\scripts\pre-commit.ps1

# 7. Commit
git add .
git commit -m "feat: mi feature"

# 8. Pre-push (antes de push)
.\scripts\pre-vuelo.ps1

# 9. Push
git push origin feature/mi-feature
```

### CI/CD Pipeline (GitHub Actions)

```yaml
name: CI
on: [push, pull_request]
jobs:
  fmt:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo fmt --check
      - run: Get-ChildItem -Recurse -Filter *.nv | ForEach-Object { lumen fmt --check $_.FullName }

  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --workspace
      - run: cargo clippy -- -D warnings

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace --no-fail-fast

  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo install cargo-llvm-cov
      - run: cargo llvm-cov --workspace --html
      - uses: actions/upload-artifact@v4
        with:
          name: coverage
          path: target/coverage/html/
```

### Publicación (Release)

```bash
# 1. Actualizar versión en Cargo.toml
# 2. Actualizar AGENTS.md, LENGUAJE.md
# 3. Pre-vuelo completo
.\scripts\pre-vuelo.ps1

# 4. Commit de release
git add .
git commit -m "release: v1.4.0"
git tag v1.4.0

# 5. Build release
cargo build --release

# 6. Push con tags
git push origin main --tags

# 7. Publicar en crates.io (opcional)
cargo publish -p lumen-cli
```

---

## Referencia Rápida de Comandos

| Tarea | Comando |
|-------|---------|
| Compilar | `cargo build` |
| Release | `cargo build --release` |
| Tests | `cargo test --workspace` |
| E2E tests | `cargo test -p lumen-vm --test e2e` |
| Proptests | `cargo test -p lumen-vm --test proptest` |
| Formatear Rust | `cargo fmt` |
| Formatear .nv | `lumen fmt archivo.nv` |
| Verificar .nv | `lumen fmt --check archivo.nv` |
| Linter | `cargo clippy -- -D warnings` |
| Fuzzing | `cargo fuzz run fuzz_lexer` |
| Bench | `cargo bench -p lumen-bench` |
| Cobertura | `cargo llvm-cov --workspace --html` |
| Pre-commit | `.\scripts\pre-commit.ps1` |
| Pre-push | `.\scripts\pre-vuelo.ps1` |
| Ejecutar .nv | `lumen run programa.nv` |
| Compilar .nv | `lumen build programa.nv` |
| Verificar .nv | `lumen check programa.nv` |
| Tests .nv | `lumen test tests.nv` |
| REPL | `lumen repl` |
| Desensamblar | `lumen disasm programa.nvc` |
| Instalar paquete | `lumen install coleccion` |
| Nuevo proyecto | `lumen new mi_proyecto` |

---

*LÚMEN v2.4.2 — Agosto 2026*
