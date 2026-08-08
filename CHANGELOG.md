# Changelog

Todos los cambios importantes del proyecto LÚMEN se documentan aquí.

---

## v2.4.0 — 6 Agosto 2026

### Agregado
- **Sprint 6: Gramática completa en el pipeline puro (self-hosted)** — `importar` con base-dir + self-import detectado, `sea`/`const` (VarDecl), StructInit `T {}` → mapas, `.campo` → Index, `elegir`/`defecto:`/`caso` reales (cadenas `sino` con im::HashMap persistente), enum `Nombre::Miembro(args)`, Option/Result (`algun`/`ninguno`/`exito`/`error` → op 38/39/41/42), closures IIFE, params default inlineados, traits `rasgo`/`impl`/`este` (métodos mangled + resolución por tipo de var), cast `como`, cortocircuito `&&`/`||` (`_cg_and_or`)
- **Sprint 7: VM en LÚMEN (`vm.nv`)** — ejecutador de .nvc en LÚMEN puro (dispatch 0-46, bandas boxed, corutinas reales con intercambio st/sp/pc, handlers JSON/tarea/coro/crypto/fs/env/tiempo/hilo/mutex/calendario, `fmain` acepta `__main__`/`main`/`principal`)
- **Optimización 43x**: COW con `Arc` en `Value::Str`/`Value::Array` (fixpoint 861s → 20.1s); `a_entero` O(n)→O(1) (demo 120s → 0.9s); `__str_subcadena_chars`/`__str_reemplazar` natives; guards de banda [3e9,9e9) y < -1e9
- **Tipo dinámico `Numero` real** + alias `cualquiera`/`any` (desbloquea csv.nv y test_migracion)
- **Benchmark vs Rust**: `scripts/benchmark_vs_rust.ps1` — compile x5.4, run x231 (mediana x2-6)
- **Resultados**: batería test_vm.ps1 **39/40** (solo `stress_fecha` flaky) · cargo test **375/375** · **fuego.ps1: 116/116 compilan · 108 CORRECTOS · 4 INCOMPATIBLES · 4 TIMEOUT · 0 fallos**

### Arreglado
- Self-import (`fs::canonicalize` `\\?\` en Windows) y renombre `graficos_avanzado_demo.nv` (sombreado del stdlib)
- `_imp_prefijar` sin rama `Lista` (calls en array-literals de imports quedaban sin prefijar)
- Scan de genéricos sin límites en parser Rust y LÚMEN (rompía `mientras i < n && ...`)
- Ternario (precedencia `mp == 0` + ramas FALSY), `.agregar` en ExprStmt, floats con `.` en lexer, PushNum f64 en codegen_to_nvc
- `__map_poner` persistente no propagaba cadenas `sino` (elegir con 2+ casos)
- Entry `principal` sin `__main__` (44_extension_methods/math), Ret sin caller, Ret con call_stack vacío en corutinas
- `a_texto_v` con ints reales >3e9, colisión de banda [1e9,2e9), Store global desde funciones
- Orden de claves de mapas determinista en `csv.nv` (serializar ordena claves numéricas)
- Output del VM en crashes (run_bytecode imprimía buffer solo en éxito)

### Limpieza (6 Ago 2026)
- Eliminados artefactos de test de la raíz (26 archivos), `examples_backup_2026/` (270 archivos), 41 `.nvc` temporales de `stdlib/compiler`, `src/` vacía — commit `4f2f6c7` (276 archivos, -6525 líneas)
- `.opencode/` añadido al .gitignore
- Docs sincronizadas: README (v2.4.0), CHANGELOG, `docs/self-hosting.md`, `docs/siguiente.md`, `docs/roadmap.md`, reports

---

## v2.3.0 — 31 Julio 2026

### Agregado
- **Self-hosting puro COMPLETADO: LÚMEN se compila a sí mismo sin `__compile_nv`**
  - `compiler_v4.nv` autocontenido (55,308 bytes): `lexer.nv` + `parser.nv` + `codegen.nv` + main, sin imports
  - Pipeline: leer .nv → lexer puro → parser puro → codegen puro → `__codegen_a_nvc` → .nvc
  - **Fixpoint confirmado**: `compiler_v4_self.nvc` (54,712 bytes, 49 funciones) recompila su propio source con resultado IDÉNTICO (52,160 bytes → 11,437 tokens → 6,376 instrs → 54,712 bytes) en 3 runs consecutivos (193s, 203s, 197s)
  - Tabla de funciones completa en el autocompilado: `_lx_es_ident`…`codegen_print` + `__main__`

### Arreglado (pipeline puro)
- `Jmp`/`JmpIf` serializados con target en tabla `nums` (la VM lee `nums[idx]`) — antes target directo → loop infinito
- If emitía JMP al final del then-body para saltar el else (antes ejecutaba ambos)
- Indexación `chars[i]`: postfix `[expr]` en `_parse_pr` + nodo `Index` en codegen + `OP_ARRAY_GET` (29)
- `intentar` (TryUnwrap): mapeo `40 => 40` en `cg_to_vm` — antes Nop dejaba `Exito(Str)` en el stack
- Print multi-arg: un `OP_PRINT` por argumento en orden (antes uno solo → output parcial/invertido)
- Break/Continue: `loop_stack` en codegen con backpatches (breaks→fin de loop, conts→loop_start)
- `numero r;` (VarDecl sin inicializador): PushInt 0 por defecto — antes stack underflow
- Lexer puro: procesa escapes de string `\n \t \r \" \\` — antes `"\""` rompía la tokenización
- Keywords `void` y `diccionario` añadidas al lexer puro — antes las funciones `codegen_imprimir` se corrompían
- Forward declarations (`funcion X(...);`): ignoradas con nodo `Vacio` — antes se tragaban la función siguiente
- Fix previo en `crates/lumen-lexer/src/lexer.rs`: escape `\r` → CR real (era 'r' literal)

### Cambiado
- `AGENTS.md`, `docs/self-hosting.md`, `docs/siguiente.md`, `docs/roadmap.md` sincronizados
- `stdlib/compiler/generar_v4.ps1` regenera `compiler_v4.nv` (concatenación autocontenida)
- Test artifacts de aislamiento eliminados (`mini_*`, `test_lexer*`)

---

## v2.2.0 — 30 Julio 2026

### Agregado
- **Self-hosting Total: `Value::Map` optimizado de `Vec<(Value,Value)>` a `HashMap<Value,Value>`**
  - `Hash` + `Eq` manual para `Value` (f64 via `to_bits()`, Map via sorted key-value hashes)
  - `__map_get`/`__map_set`/`__map_contains`: O(n) scan lineal → O(1) hash lookup
  - Sets (union/inter/diff): O(n²) → O(n) con `contains_key`
  - `codegen_to_nvc`: `map_get` O(1) con `HashMap::get`
  - JSON helpers actualizados a HashMap
  - ~378 tests pasan, autocompilación funcional (533ms)
  - El parser LÚMEN-in-LÚMEN ahora tiene mapas O(1) — self-hosting total sin `__compile_nv` es viable

### Cambiado
- `AGENTS.md`, `docs/self-hosting.md`, `docs/siguiente.md`, `docs/roadmap.md` sincronizados

---

## v2.1.0 — Julio 2026

### Agregado
- **Fase 180: Mutation Testing** — `stdlib/testing.nv` extendido con funciones de mutación de código fuente.
  - `mutar/mutate`: 5 tipos de mutación (+→-, verdadero→falso, >→<, eliminar si, invertir retornar).
  - `mutantes_generar/mutants_generate`: generación de N mutantes desde código fuente.
  - `mutacion_probar/mutation_test`: ejecución de mutation testing y conteo de mutantes muertos.
  - `mutacion_puntaje/mutation_score`: cálculo de puntaje de mutación (muertos/total).
  - `mutacion_analizar/mutation_analyze`: análisis de archivos con reporte de calidad de tests.
  - Ejemplos: `examples/jr/mutation_jr.nv`, `examples/sr/mutation_sr.nv`.
- **Fase 182: Tracing Distribuido** — `stdlib/tracing.nv` con spans estilo OpenTelemetry simplificado.
  - `span_iniciar/span_start`, `span_finalizar/span_finish`: ciclo de vida de spans.
  - `span_atributo/span_attribute`: atributos clave-valor en spans.
  - `span_error/span_error_set`: marcado de spans con error.
  - `spans_exportar/spans_export`: exportación JSON de todos los spans.
  - `spans_arbol/spans_tree`: visualización jerárquica en formato árbol.
  - `spans_limpiar/spans_clear`: reinicio de sesión de tracing.
  - Ejemplos: `examples/sr/tracing_sr.nv`, `examples/real/tracing_real.nv`.
- **Fase 184: Profiler CPU/Memoria** — `stdlib/profiler.nv` con perfilado de rendimiento.
  - `perfil_iniciar/profile_start`, `perfil_finalizar/profile_end`: medición de tiempos.
  - `perfil_reporte/profile_report`: reporte detallado con min/max/promedio/total.
  - `perfil_resumen/profile_summary`: resumen compacto para comparación rápida.
  - `perfil_memoria/profile_memory`: estimación de memoria usada.
  - `perfil_promedio/profile_avg`: tiempo promedio por función.
  - Detección de hot paths: identificación automática de funciones más lentas.
  - Ejemplos: `examples/sr/profiler_sr.nv`, `examples/real/profiler_real.nv`.
- **Dual ES/EN**: todas las funciones nuevas tienen alias en español e inglés.
- **6 nuevos ejemplos**: mutation (jr/sr), tracing (sr/real), profiler (sr/real).

### Cambiado
- `stdlib/testing.nv` extendido con sección de mutación (funciones `mutar`, `mutantes_generar`, `mutacion_probar`, `mutacion_puntaje`, `mutacion_analizar`).
- `AGENTS.md` actualizado con fases 180, 182, 184 completadas.
- `docs/roadmap.md` actualizado: portabilidad 75% → 100%.

---

## v2.0.0 — Julio 2026

### Agregado
- **Operador bitwise `|`**: soporte completo en parser → sema → IR → codegen → VM

### Agregado
- **Fase 96: WASM Playground** — Compilación a `wasm32-unknown-unknown`.
  - VM refactorizada: `call_core_builtin()` + `call_full_builtin()` extraídas del dispatch masivo.
  - Feature flags `full`/`minimal` en VM con `#[cfg(feature = "full")]` en fields TCP, cluster, scope, FFI.
  - Stubs para crypto_ffi, gui_ffi, coro_ffi en modo minimal.
  - crate `lumen-wasm` con playground web HTML.
  - `Display` impl para `VmError`.
- **Fases 97-130: ~70 builtins nuevos** (FFI, crypto, concurrencia, GUI, corrutinas, utilidades, fecha):
  - FFI: `__ffi_cargar/load`, `__ffi_llamar/call`, `__ffi_asignar/alloc`, `__ffi_liberar/free`, `__ffi_escribir/write`, `__ffi_leer/read`, `__ffi_peek`, `__ffi_poke`
  - Crypto: `__hash_sha256`, `__hash_sha512`, `__aes_encriptar/encrypt`, `__aes_desencriptar/decrypt`, `__jwt_codificar/encode`, `__jwt_decodificar/decode`
  - Concurrencia: hilos, mutex, canales, rwlock, arc, tareas, streams, actores, generadores, supervisores, cluster, scope, par, dormir, seleccionar (~36 builtins)
  - GUI: `__gui_ventana/window`, `__gui_mostrar/show`, `__gui_cerrar/close`, `__gui_id/hwnd`, `__gui_esperar/poll`
  - Corrutinas: `__coro_crear/create`, `__coro_ceder/yield`, `__coro_reanudar/resume`
  - Utilidades: `__tipo_de/typeof`, `__fs_listar/listdir`, `__env_listar/list`
  - Fecha: `__tiempo_formatear/format`, `__tiempo_parsear/parse`, `__tiempo_diferencia/diff`
- Nuevo crate: `lumen-wasm` (runtime WASM + playground web).
- **Stdlib dual ES/EN**: `texto.nv`, `fecha.nv`, `io.nv`, `crypto.nv` actualizados con aliases inglés.
- **149 e2e tests** (10 nuevos desde v1.6.0, +10 desde v1.7.0alpha).

### v2.0 — GUI/TUI/Juegos 100% (Julio 2026)
- **Canvas 2D**: círculos, líneas, triángulos, rectángulos redondeados, gradientes, bitmap font
- **Tilemap**: sistema de mapas 2D con cámara, colisiones AABB, view culling
- **Charts**: gráficos de barras, líneas, pastel, dispersión con ejes automáticos
- **TUI Temas**: 4 presets (Catppuccin, claro, oscuro, alto contraste)
- **Demo completo**: 33 secciones cubriendo todas las features del lenguaje
- **Skills**: `.opencode/agents/lumen-engineer.md` + `lumen-tester.md`
- **Roadmap**: GUI/TUI/Juegos 55%→100%

### Cambiado
- Version bump a 2.0.0.
- Todos los .md sincronizados con estado actual.
- README reescrito con WASM playground, Docker, sintaxis dual.
- 66 ejemplos nuevos organizados en jr/sr/real (22 módulos stdlib).

### Corregido
- `>=` y `<=` verificados funcionales en parser (no requerían workaround)
- `info` y `debug` NO son keywords reservadas (solo `error` lo es)
- Bitwise OR `|` implementado como operador de expresión
- stdlib: restaurados `>=`/`<=` y `|` donde habían sido reemplazados incorrectamente

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
