# Changelog

Todos los cambios importantes del proyecto LÚMEN se documentan aquí.

---

## v1.7.0 — Julio 2026

### Agregado
- **Fase 96: WASM Playground** — Compilación a `wasm32-unknown-unknown`.
  - VM refactorizada: `call_core_builtin()` + `call_full_builtin()` extraídas del dispatch masivo.
  - Feature flags `full`/`minimal` en VM con `#[cfg(feature = "full")]` en fields TCP, cluster, scope, FFI.
  - Stubs para crypto_ffi, gui_ffi, coro_ffi en modo minimal.
  - crate `lumen-wasm` con playground web HTML.
  - `Display` impl para `VmError`.
- Nuevo crate: `lumen-wasm` (runtime WASM + playground web).
- VM: 139 e2e tests (26 nuevos desde v1.6.0).
- VM: 0 dependencias externas obligatorias (solo `libloading` con feature flag).

### Cambiado
- Version bump a 1.7.0 en workspace.
- VM struct fields con `#[cfg(feature = "full")]` para compilación WASM.
- `AGENTS.md` actualizado con fases completadas y test counts.

---

## v1.6.0 — Julio 2026

### Agregado
- **Fase 96-110: Librería Estándar Extendida** — HashMap, HashSet, VecDeque, BinaryHeap,
  LinkedList, Regex, Unicode Normalization, String Padding, UTF-8 Encoding,
  Buffered I/O, Streaming, TCP, HTTP, Serial Port (stub).
  ~70 builtins nuevos en VM (handlers Call/CallValue). Stdlib modules: `coleccion.nv`,
  `texto.nv`, `io.nv`, `red.nv`. Registrados en sema/loader/IR builder.
- **Fase 94: Single Binary** — `lumen` como binario único integrando `run`, `build`, `check`,
  `fmt`, `repl`, `doc`, `lsp`, `install` sin spawnear procesos hijos.
  Creadas libs `lumen_doc`, `lumen_lsp`, `lumen_pkg`.
- **Fase 95: Installer** — `scripts/install.ps1` (Windows) + `scripts/install.sh` (Unix).
  Detección de release binaria con fallback a compilación desde fuente.
- **Fase 65: Guard Let** — `sea patron = expr sino { romper/retornar/continuar }`.
  Desugaring en IR builder a JmpIf/Jmp. `Stmt::GuardLet` en AST + parser + sema.
- **Fase 66: Operator Overloading** — `impl Suma for Punto` con método `fn sumar(self, otro)`.
  - `Expr::Binary` ahora tiene `resolved_method: Option<String>`.
  - Sema: `resolve_operator_overloads()` post-analysis con `HashMap<String, TypeInfo>`.
  - IR builder: emite `Call` en lugar de `Binary` cuando hay overload resuelto.
  - Traits `Suma`, `Resta`, `Multiplica`, `Divide` implementables en structs.
- **Fase 67: Extension Methods** — `impl Rasgo para TipoPrimitivo`.
  - `type_to_impl_name()` soporta resolución de tipos nativos: `entero`, `texto`, `decimal`, etc.
- **Fase 68: Tipos Asociados en Traits** — `tipo Item;` en rasgos y `tipo Item = T;` en impl.
  - AST: `AssociatedType` e `ImplAssociatedType`. Sema e IR completos.
  - `resolve_trait_method_mangled()` ahora sustituye tipos asociados contra el impl concreto.
  - `examples/senior/associated_types.nv` — demo funcional con 2 impls distintos.
- **Fase 70: Impl Trait return** — `funcion impl Rasgo foo() { retornar expr }`.
  - `Type::ImplTrait(String)` en AST. Parseo en `parse_type()`.
- **Fases 71-74: LSP Server (`lumen-lsp`)** — Diagnósticos en vivo, Autocompletado,
  Go-to-definition y Hover. Protocolo JSON-RPC sobre stdin/stdout.
- **Fase 75: lumen doc (`lumen-doc`)** — Generador de documentación HTML desde `///`.
- **Fase 76: Debugger** — Depurador interactivo con breakpoints, step, continue e inspect.
- **Fase 77: lumen fmt avanzado** — Soporte para `.lumen-fmt.toml` (`indent_spaces`, etc.).
- **Fase 78: lumen lint** — Análisis estático: código muerto y complejidad ciclomática.
- **Fase 79: REPL Pro** — Historial persistente, multilínea, resaltado, autocompletado.
- **Fase 80: Package Manager (`lumen-pkg`)** — `lumen install`, registry, lock file.
- **Fase 81: Build Incremental** — Caché de módulos para builds más rápidos.
- **Fase 82: Hot Reload** — `lumen serve` con recarga automática en dev.
- **Fase 83: Playground Web** — Editor online con ejecución en navegador.
- **Fases 86-87: AOT Compilation (`lumen-aot`)** — Transpilador C + backend Cranelift (base).
- **Roadmap fusionado** — Combinación del historial v1.0-v1.2 con el roadmap extendido v3.0.

### Cambiado
- Version bump a 1.6.0 en workspace.
- Trait impl functions reciben `"self"` como primer parámetro automáticamente.
- `docs/roadmap.md` expandido con tablas detalladas hasta fase 220 (v3.0).
- `AGENTS.md` actualizado con fases 94-95 completadas.
- `docs/cli.md` actualizado con comando `install`.
- `HERRAMIENTAS.md` actualizado con nuevos scripts de instalación.

### Corregido
- Encoding UTF-8 en scripts de CI/CD (pre-commit PowerShell).
- Pipeline de docs actualizado con referencias a crates nuevos.

---

## v1.2.0 — Julio 2026

### Agregado
- **Stdlib** — Módulos nativos: `matematicas`, `texto`, `coleccion`, `fecha`.
- **E/S de Archivos** — Builtins `leer_archivo`, `escribir_archivo`, `existe_archivo`.
- **Stack Traces** — Pila de llamadas completa en errores de runtime.
- **Mensajes de Error Mejorados** — Subrayado exacto con caret (`^^^^`) y colores ANSI.
- **Fases 42-57** — Inferencia de tipos, métodos en structs, diccionarios, string interpolation,
  rangos, constantes, string indexing, conversiones, operador ternario, loop labels,
  pattern matching exhaustivo + guardas, genéricos con bounds, matrices 2D.
- **Fase 58** — Enums avanzados con datos asociados.

### Corregido
- Advertencias de Clippy (CI verde en todos los targets).

---

## v1.1.0 — Julio 2026

### Agregado
- **Fase 21: For-Each Loop** — `para x en expr` / `for x in expr`. 31 tests.
- **Fase 22: Opcion<T>** — `opcion<T>` con `algun(valor)` y `ninguno`. 15 tests.
- **Fase 23: Enums/Tipos Suma** — `enum Nombre { Variante, Variante(tipo) }`. 20 tests.
- **Fase 24: Tuplas** — `(tipo, tipo)` y acceso `.0`, `.1`. 4 tests.
- **Fase 25: Destructuring** — `entero x, texto y = expr`, wildcard `_`. 14 tests.
- **Fase 26: Genéricos Básicos** — `<T, U>` en funciones y structs. 17 tests.

### Cambiado
- Workspace version a 1.1.0.
- CI corre en branches `master` y `main`.
- MSRV actualizado a 1.82.

---

## v1.0.0 — Julio 2026

Release inicial de LÚMEN. Lenguaje de programación educativo en español con pipeline completo
Lexer → Parser → Sema → IR → Codegen → VM. 21 fases completadas.
