# Guía de Contribución — LÚMEN v3.2.0 Producción Real

## Requisitos

- Rust 1.70+ (MSRV 1.82 para CI)
- `cargo test --workspace` debe pasar: **917 tests** (616 e2e + 9 production + resto) — ver `docs/produccion.md`
- `cargo bench -p lumen-bench` compila 8 benches (4 prod: fallthrough, defaults, matematicas, headless)
- `cargo fmt -- --check` + `cargo clippy --all -- -D warnings` limpios
- `LUMEN_HEADLESS=1 CI=1` para validación headless (`stdlib/graficos.nv:es_headless()`)
- 0 warnings de compilación

## Configuración

```bash
git clone https://github.com/omarPVP123131/Lumen.git
cd Lumen
cargo build --all
cargo test --workspace          # 917 tests (ver docs/produccion.md)
cargo bench -p lumen-bench -- --quick   # 8 benches smoke
LUMEN_HEADLESS=1 CI=1 cargo test --workspace   # valida headless (es_headless)
```

## Desarrollo

### Estructura del proyecto

El compilador está organizado en 16 crates dentro de `crates/`:

| Crate | Responsabilidad |
|-------|----------------|
| `lumen-lexer` | Análisis léxico |
| `lumen-parser` | Parsing y AST |
| `lumen-sema` | Análisis semántico + ModuleLoader |
| `lumen-ir` | Representación intermedia + optimizaciones |
| `lumen-codegen` | Generación de bytecode |
| `lumen-vm` | Máquina virtual bytecode |
| `lumen-cli` | Interfaz de línea de comandos (binario único) |
| `lumen-fmt` | Formateador de código |
| `lumen-repl` | REPL interactivo |
| `lumen-project` | Scaffolding de proyectos (`lumen new`) |
| `lumen-lsp` | Servidor LSP (diagnósticos, completado, hover, go-to-def) |
| `lumen-doc` | Generador de documentación HTML |
| `lumen-aot` | Compilación AOT (C transpiler + Cranelift) |
| `lumen-pkg` | Gestor de paquetes (`lumen install`) |
| `lumen-api` | API pública del compilador (usar LÚMEN como biblioteca) |
| `lumen-plugin` | Sistema de plugins para fases del compilador |

### Flujo para añadir una feature

1. **AST**: Añadir variante en `Expr`, `Stmt`, o `Decl` (`lumen-parser/src/ast.rs`)
2. **Tokens**: Si se necesita nueva palabra clave (`lumen-lexer/src/token.rs`)
3. **Parser**: Parsear la nueva sintaxis (`lumen-parser/src/parser.rs`)
4. **Sema**: Type checking y validación (`lumen-sema/src/sema.rs`)
5. **IR**: Compilar a instrucciones intermedias (`lumen-ir/src/builder.rs`)
6. **Codegen**: Emitir bytecode (`lumen-codegen/src/codegen.rs`)
7. **VM**: Ejecutar la nueva instrucción (`lumen-vm/src/vm.rs`)
8. **Value**: Si se necesita un nuevo tipo de valor (`lumen-vm/src/value.rs`)

### Estándares

- Código limpio, sin comentarios superfluos
- Seguir patrones existentes
- Tests obligatorios para código nuevo
- 0 warnings en todos los crates

## Pull Requests

1. Una feature por PR
2. Tests que pasan en CI
3. Documentación actualizada
4. Sin regresiones (todos los tests existentes deben pasar)

## Producción Real v3.1.4

Ver `docs/produccion.md` para checklist: `CHUNK_VERSION 7` (defaults persistidos), `builder last_significant()` + `label_counter` global, `vm bind_args` unificado, `stdlib/graficos.nv es_headless()` centralizado, bench 8, CI `headless-check` (`LUMEN_HEADLESS=1 CI=1`). `VERSION` 3.1.4 es source of truth.

## Reportar Issues

Incluir:
- Código mínimo que reproduce el problema
- Comportamiento esperado vs actual
- Versión de LÚMEN (`lumen --version` → debe coincidir con `VERSION` 3.1.4 y `CHUNK_VERSION 7`)
- Si es gráfico/headless: `LUMEN_HEADLESS=1` repro y `docs/produccion.md` §1.4

