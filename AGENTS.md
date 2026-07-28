# AGENTS.md — Diario de construcción de LÚMEN

**v1.7.0 — Released: Julio 2026**

---

## Testing (Actual)

| Crate | Tests | Tipo |
|-------|-------|------|
| lumen-lexer | 24 | unit |
| lumen-parser | 42 | unit |
| lumen-sema | 49 | unit |
| lumen-ir | 20 | unit + folding |
| lumen-codegen | 13 | unit |
| lumen-codegen | 5 | proptest |
| lumen-vm | 45 | unit |
| lumen-vm | 165 | e2e |
| lumen-fmt | 2 | unit |
| lumen-repl | 2 | unit |
| lumen-project | 1 | unit |
| lumen-aot | 1 | unit |
| lumen-doc | 1 | unit |
| lumen-pkg | 1 | unit |
| lumen-plugin | 1 | unit |
| **Total** | **~371** | |

**0 warnings, ~371 tests passing. 45/45 ejemplos funcionando. 165 e2e, 45 unit.**

---

## Fases completadas

### Fases 0-15: Infraestructura base ✅
Lexer, parser, sema, IR, bytecode, VM, CLI, arrays, control de flujo avanzado.

### Fase 16: Funciones avanzadas ✅
Parámetros default, Lambdas IIFE, Closures.

### Fase 17: Estructuras/Objetos ✅
`estructura`, inicialización, acceso, asignación de campos.

### Fase 18: Módulos ✅
`importar`, ModuleLoader, detección circular.

### Fase 19: Optimizaciones ✅
Constant folding, DCE, shared pools.

### Fase 20: v1.0 Release ✅

### Fases 21-27: Features del lenguaje ✅
For-Each, Resultado<T,E>, Opcion<T>, Enums, Tuplas, Destructuring, Genéricos.

### Fases 28-30: Stdlib + Archivos ✅
matematicas, texto, coleccion, fecha, archivos.

### Fase 31: Stack Traces ✅

### Fase 32-33: Mensajes de Error Mejorados ✅
Caret, ANSI, preview multi-línea, conteo de errores.

### Fase 34: Fuzzing ✅
3 targets cargo-fuzz (lexer, parser, decoder).

### Fase 35: Property-Based Testing ✅
Proptest en codegen.

### Fase 37: lumen fmt ✅
`lumen fmt` formatea código .nv. Crate `lumen-fmt`.

### Fase 38: lumen repl ✅
REPL interactivo. Crate `lumen-repl`.

### Fase 39: lumen test ✅
`lumen test` ejecuta funciones `test_*`.

### Fase 40: lumen.toml + lumen new ✅
`lumen new` scaffolding. Crate `lumen-project`.

### Fase 41: CI/CD + Releases ✅
GitHub Actions CI + release multiplataforma.

### Fases 42-57: Lenguaje & Sintaxis (Bloque 1) ✅
- 42: Inferencia de tipos (`x = 42`)
- 43: Métodos en structs (`impl Struct`)
- 44: Diccionarios (`diccionario<K,V>`)
- 45: String interpolation (`"Hola {nombre}"`)
- 46: Rangos (`0..5`, `0..=5`)
- 47: Constantes (`const`)
- 48: String indexing (`s[i]`)
- 49: Conversiones (`a_texto`, `a_entero`, `a_decimal`)
- 50: División entera (`entero/entero → entero`)
- 51: Concatenación mixta (`"x" + 42`)
- 52: Mejores errores (preview multi-línea)
- 53: Operador ternario (`?:`)
- 54: Loop labels (`romper etiqueta`)
- 55: Pattern matching exhaustivo + guardas
- 56: Genéricos con bounds (`<T: Rasgo>`)
- 57: Matrices 2D (`lista<lista<T>>` + stdlib matrices.nv)

### Fase 58: Enums Avanzados ✅
Variantes con datos (`Variant(entero)`).

### Fase 59: Closures Pro ✅
Captura por valor/referencia. Closures movibles.

### Fase 60: Async/Await ✅
Sintaxis `async funcion` / `esperar`. Sema + IR bases.

### Fase 65: Guard Let ✅
`sea patron = expr sino { romper/retornar/continuar }`. Desugaring en IR builder a JmpIf/Jmp.
`Stmt::GuardLet` en AST, `parse_guard_let()` en parser, sema + loader, IR builder.

### Fase 66: Operator Overloading ✅
Vía trait method convention (`impl Suma for Punto`). `Expr::Binary.resolved_method`.
Sema: `resolve_operator_overloads()` post-analysis walk con HashMap de inferencia de tipos.
IR builder: `resolved_method` → `Call` o `Binary` nativo.

### Fase 67: Extension Methods ✅
`impl Trait para TipoPrimitivo` (`impl Duplicable para entero`). `type_to_impl_name()` mapea
`entero`, `texto`, `decimal`, `booleano`, `lista`, `opcion`, `resultado`, `tupla`.

### Fase 68: Tipos Asociados en Traits ✅
`tipo Item;` en traits y `tipo Item = T;` en impl.
`AssociatedType` e `ImplAssociatedType` en AST, sema e IR.

### Fase 69: Where Clauses ⏭️
Saltado — `<T: Rasgo>` syntax ya soporta bounds.

### Fase 70: Impl Trait return ✅
`funcion impl Rasgo foo() { retornar expr; }`. `Type::ImplTrait(String)` en AST.
Parseo en `parse_type()`, mapea a `TypeInfo::TypeVar` en sema.

### Fases 71-74: LSP Server ✅
`lumen lsp` — Diagnósticos en vivo, Autocompletado, Go-to-definition, Hover de tipos.
Crate `lumen-lsp`. Protocolo JSON-RPC sobre stdin/stdout.

### Fase 75: lumen doc ✅
Generación de HTML desde comentarios `///`. Crate `lumen-doc`.

### Fase 76: Debugger ✅
Depurador interactivo con breakpoints, step, continue, inspect de variables.

### Fase 77: lumen fmt avanzado ✅
Soporte para `.lumen-fmt.toml` (`indent_spaces`, etc.). Crate `lumen-fmt`.

### Fase 78: lumen lint ✅
Análisis estático de código muerto y complejidad ciclomática.

### Fase 79: REPL Pro ✅
Historial persistente, multilínea, resaltado de sintaxis, autocompletado.

### Fase 80: Package Manager ✅
`lumen install`, registry central, lock file. Crate `lumen-pkg`.

### Fase 81: Build Incremental ✅
Caché de compilación incremental para builds más rápidos.

### Fase 82: Hot Reload ✅
Recarga automática de módulos en dev. `lumen serve`.

### Fase 83: Playground Web ✅
Editor online con ejecución en navegador.

### Fases 86-87: AOT Compilation ✅
- 86: Transpilación a C + gcc/clang -O3
- 87: Backend Cranelift (base)

### Fase 94: Single Binary ✅
`lumen` como binario único integrando run, build, check, fmt, repl, doc, lsp, install en un solo ejecutable sin spawn. Creadas libs `lumen_doc`, `lumen_lsp`, `lumen_pkg`.

### Fase 95: Installer ✅
Scripts `scripts/install.ps1` (Windows) y `scripts/install.sh` (Unix) con detección de release binaria y fallback a compilación desde fuente.

### Fase 84: Benchmarks ✅
Suite criterion para pipeline completo (lexer→parser→sema→IR→codegen→VM). `crates/lumen-bench/`.
- 4 benchmarks: `lexer_tokenize`, `parser_parse`, `pipeline_full`, `vm_fib_20`

### Fase 85: Plugins API ✅
Sistema de plugins para fases del compilador (pre-parse, post-sema, etc.).
- `Plugin` trait con hooks: `on_tokens`, `on_ast`, `on_sema`, `on_ir`
- `PluginRegistry` con registro y ejecución ordenada de plugins

### Fase 88: AOT LTO + Optimización ✅
Link-time optimization, dead code stripping, inlining agresivo en backend AOT.
- `opt_level = "speed_and_size"` en Cranelift
- `__attribute__((used))` + funciones `static` en transpilador C
- Tests de DCE y compilación básica

### Fase 96: WASM Playground ✅
Compilación a `wasm32-unknown-unknown` con feature flags. VM refactorizada:
- `call_core_builtin()` + `call_full_builtin()` extraídas del dispatch masivo
- Display impl para VmError
- `#[cfg(feature = "full")]` en fields TCP, cluster, scope, FFI de VM
- Stubs para crypto_ffi, gui_ffi, coro_ffi en modo minimal
- crate `lumen-wasm` con playground web en HTML

### Fases 97-130: Builtins Faltantes Implementados ✅
~70 builtins nuevos añadidos al VM (FFI, crypto, concurrencia, GUI, corrutinas, utilidades, fecha):
- **FFI**: `__ffi_cargar/load`, `__ffi_llamar/call`, `__ffi_asignar/alloc`, `__ffi_liberar/free`, `__ffi_escribir/write`, `__ffi_leer/read`, `__ffi_peek`, `__ffi_poke`
- **Crypto**: `__hash_sha256`, `__hash_sha512`, `__aes_encriptar/encrypt`, `__aes_desencriptar/decrypt`, `__jwt_codificar/encode`, `__jwt_decodificar/decode`
- **Concurrencia**: `__hilo_lanzar/spawn`, `__hilo_esperar/join`, `__mutex_nuevo/new`, `__mutex_bloquear/lock`, `__canal_nuevo/new`, `__canal_enviar/send`, `__canal_recibir/recv`, `__rwlock_nuevo/new`, `__rwlock_leer/read`, `__rwlock_escribir/write`, `__arc_nuevo/new`, `__arc_obtener/get`, `__arc_asignar/set`, `__tarea_lanzar/spawn`, `__tarea_esperar/await`, `__stream_desde/from`, `__stream_mapear/map`, `__stream_filtrar/filter`, `__stream_colectar/collect`, `__actor_nuevo/new`, `__actor_enviar/send`, `__actor_recibir/recv`, `__generador_nuevo/new`, `__generador_siguiente/next`, `__supervisor_nuevo/new`, `__supervisor_agregar/add`, `__supervisor_iniciar/start`, `__cluster_conectar/connect`, `__cluster_enviar/send`, `__scope_nuevo/new`, `__scope_lanzar/spawn`, `__scope_cancelar/cancel`, `__par_mapear/map`, `__par_unir/join`, `__dormir/sleep`, `__seleccionar/select`
- **GUI**: `__gui_ventana/window`, `__gui_mostrar/show`, `__gui_cerrar/close`, `__gui_id/hwnd`, `__gui_esperar/poll`
- **Corrutinas**: `__coro_crear/create`, `__coro_ceder/yield`, `__coro_reanudar/resume`
- **Utilidades**: `__tipo_de/typeof`, `__fs_listar/listdir`, `__env_listar/list`
- **Fecha**: `__tiempo_formatear/format`, `__tiempo_parsear/parse`, `__tiempo_diferencia/diff`
- **Conexión**: `crypto_ffi.rs`, `gui_ffi.rs`, `coro_ffi.rs` ahora incluidos vía `mod` en lib.rs
- **Stdlib dual ES/EN**: `texto.nv`, `fecha.nv`, `io.nv`, `crypto.nv` actualizados con aliases inglés
- **Async Runtime**: `async funcion` + `esperar` ahora funcional vía thread tasks. Builtins `__tarea_lanzar/spawn`, `__tarea_esperar/await`. `Expr::Esperar` desugara a `Call(__tarea_esperar)` en IR builder.
- **JS Interop**: `__js_eval/evaluar` y `__js_call/llamar` vía `JS_EVAL` static callback. `js-sys` en WASM runtime.
- **WASM Playground**: build con `wasm-pack`, playground web con ejemplos.
- **AES**: encriptar/desencriptar vía BCrypt AES-GCM
- **Timezone**: `__timezone_info` con soporte IANA
- **Duracion**: `__duration_new/secs`
- **Calendario**: Hijri (`__calendar_hijri`) y Persa (`__calendar_persian`)
- **Async File I/O**: `__leer/escribir_archivo_async`
- **Async Timer**: `__timer_delay`
- **Async TCP**: `__tcp_connect_async`
- **JS Interop**: `__js_eval/evaluar` y `__js_call/llamar` vía `JS_EVAL` static callback. `js-sys` en WASM runtime.
- **WASM Playground**: build con `wasm-pack`, playground web con ejemplos.
- **Docker**: Dockerfile multi-stage + docker-compose.yml
- **Logging**: stdlib/logging.nv con niveles ES/EN
- **Gráficos avanzados**: sprites, audio SDL2_mixer, partículas en LÚMEN puro
- **Canvas 2D**: círculos, líneas, triángulos, rectángulos redondeados, gradientes, bitmap font (stdlib/graficos_canvas.nv)
- **Tilemap**: sistema de mapas con cámara, colisiones, view culling (stdlib/graficos_tilemap.nv)
- **Charts**: gráficos de barras, líneas, pastel, dispersión (stdlib/graficos_charts.nv)
- **TUI Temas**: sistema de temas con preset Catppuccin, claro, oscuro, alto contraste (stdlib/tui_temas.nv)
- **demo_completo.nv**: 33 secciones cubriendo todas las features
- **WASI**: soporte wasm32-wasip1 target
- **Skills**: .opencode/agents/lumen-engineer.md + lumen-tester.md
- **165 e2e tests** (26 nuevos)

---

## Comandos CLI

| Comando | Descripción |
|---------|-------------|
| `lumen run <file>` | Ejecuta fuente .nv o bytecode .nvc |
| `lumen build <file>` | Compila a .nvc |
| `lumen check <file>` | Verifica sintaxis + semántica |
| `lumen disasm <file>` | Desensambla .nvc |
| `lumen fmt <file>` | Formatea código |
| `lumen repl` | Modo interactivo |
| `lumen new <name>` | Crea proyecto |
| `lumen test <file>` | Ejecuta tests |
| `lumen lint <file>` | Análisis estático |
| `lumen doc <file>` | Genera documentación HTML |
| `lumen debug <file>` | Inicia depurador |
| `lumen serve` | Hot reload + playground |
| `lumen lsp` | Servidor LSP |
| `lumen run -L <dir> <file>` | Ejecuta con ruta de librerías |
| `./scripts/install-hooks.ps1` | Instala git hooks (auto-tag en post-commit) |

---

## Bytecode (.nvc)

- **Version**: 6
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-46
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays
  - 33-34: Closures
  - 35-37: Structs
  - 38-40: Result
  - 41-42: Option
  - 43: Enum
  - 44-45: Tuples
  - 46: Mod

---

## Estructura del proyecto

```
crates/
  lumen-lexer/    → token.rs, lexer.rs, error.rs
  lumen-parser/   → ast.rs, parser.rs, error.rs
  lumen-sema/     → sema.rs, loader.rs, error.rs
  lumen-ir/       → ir.rs, builder.rs
  lumen-codegen/  → bytecode.rs, codegen.rs, disasm.rs
  lumen-vm/       → vm.rs, value.rs
  lumen-cli/      → main.rs (binario único)
  lumen-fmt/      → lib.rs
  lumen-repl/     → lib.rs
  lumen-project/  → lib.rs
  lumen-lsp/      → lib.rs
  lumen-doc/      → lib.rs
  lumen-aot/      → lib.rs
  lumen-pkg/      → lib.rs
  lumen-bench/    → benches/benchmarks.rs
docs/spec/        → grammar.ebnf, bytecode-format.md, error-codes.md, vm-spec.md
examples/         → *.nv (45 ejemplos funcionales)
stdlib/           → *.nv (texto, matematicas, coleccion, fecha, archivos, matrices)
scripts/          → PowerShell CI/CD (pre-commit, pre-vuelo, auto-tag, install-hooks, installers)
scripts/git-hooks/ → post-commit (auto-tag v{version} al commitear)
```
