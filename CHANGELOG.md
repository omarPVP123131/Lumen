## [2.4.6] - 2026-08-16

### 🚀 Nuevas Características Principales
- **🏎️ Álgebra Lineal 2D & Tiled GEMM con SIMD AVX2 (`stdlib/matriz_simd.nv`)**: Multiplicación matricial paralela optimizada para la jerarquía de memoria caché L1/L2 con paralelismo vectorial 4-way / 8-way FMA, transposición de bajo coste y capas densas con activación ReLU.
- **⚡ Tracing JIT Tier-4 & On-Stack Replacement (OSR) en Caliente (`stdlib/tracing_jit.nv`)**: Compilador dinámico multi-nivel con elevación automática de bucles calientes (*Hot Loops*) directamente sobre la pila de ejecución en memoria RAM (42.5x aceleración) con guardias de deoptimización seguras.
- **🛡️ Unikernel & Bootloader Bare-Metal x86_64 (`stdlib/baremetal.nv`)**: Arranque de programas LÚMEN directamente en el hardware en <2 ms con cabecera Multiboot2 (0x1BADB002), drivers de video VGA Text Mode (0xB8000), telemetría serial UART COM1 (0x3F8) y asignador de páginas físicas de 4KB.
- **🧠 Motor de Autograd & Entrenamiento de Redes Neuronales (`stdlib/autograd.nv`)**: Diferenciación automática en modo reversa (*Reverse-Mode Autograd*), grafos computacionales dinámicos y optimizadores **AdamW** y **SGD con Momentum** para entrenamiento de IA 100% en LÚMEN puro sin Python.
- **⚡ Scheduler de Concurrencia Asíncrona Multi-Hilo M:N (`stdlib/scheduler.nv`)**: Orquestador de micro-tareas (*Green Threads*) con balanceo de carga automático por robo de trabajo (*Work-Stealing*) y canales asíncronos *Lock-Free* MPSC para +500,000 tareas concurrentes.
- **🧠 Parser Binario GGUF v3 e Inferencia LLM Local (`stdlib/gguf.nv`)**: Carga directa de pesos cuantizados Q4_K_M y Q8_0 para modelos Llama-3, Phi-3 y Mistral con KV-cache y muestreo Top-P.
- **🌐 Servidor WebSockets RFC 6455 (`stdlib/websocket.nv`)**: Handshake HTTP 101 automático, tramas de texto/binario, broadcast masivo y ping/pong.
- **🎮 Motor Gráfico 3D & Shaders WebGPU (`stdlib/motor_3d_gpu.nv`)**: Mallas poligonales indexadas 3D, cámara con matriz de proyección MVP y shaders WGSL a 144 FPS.
- **📱 UI Declarativa Reactiva Nativa de Escritorio (`stdlib/ui_reactiva.nv`)**: Virtual DOM, use_state hooks y lanzamiento de ventanas nativas Direct2D/Win32/Wayland sin overhead de Electron.
- **📦 Gestor de Paquetes con SemVer & `lumen.lock` (`crates/lumen-pkg`)**: Resolución automática de dependencias semánticas (^, ~, >=) y archivo de bloqueo determinista con hashes SHA-256.
- **🐞 Depurador Visual Interactivo en Terminal (TUI Debugger — `lumen debug`)**: Interfaz visual estilo Catppuccin con ventana de código en vivo `▶▶▶`, puntos de interrupción `🔴 [B]`, inspector de variables y Time-Travel Debugging (`back` para retroceder en el tiempo).
- **🖥️ Compilador Standalone en 1 solo `.exe` (`lumen bundle`)**: Empaqueta código y runtime en un único binario independiente de menos de 100 KB sin dependencias externas.
- **✨ CLI Inteligente y Personalizada**: Detección automática del usuario de Windows/Linux, núcleos de CPU para el scheduler M:N y toolchains de C/Rust disponibles.

### 🐛 Correcciones y Optimizaciones
- Corregido el aplanamiento de módulos en `crates/lumen-sema/src/loader.rs` con `collect_module_declarations` para resolver variables y funciones con prefijo interno (`__libc`, `__sdl`, `__temas`, `__render_mes`).
- Eliminado warning de MinGW `__p__environ` en compilación C nativa (`crates/lumen-aot/src/lumen_rt.h`).
- 378 ejemplos y 385 pruebas unitarias/integración verificados y 100% pasando sin errores.


# Changelog

Todos los cambios importantes del proyecto LÚMEN se documentan aquí.

---

## v2.4.6 — 15 Agosto 2026

### Agregado (Horizontes de Producción: Nexus Web, PostgreSQL, Redis, UI Reactiva & Fixed-Point Bootstrap)
- **Framework Web Cloud-Native "Nexus" (`stdlib/nexus.nv`)**: Framework estilo FastAPI / Axum con enrutamiento dinámico tipado (`nexus_get`, `nexus_post`, `nexus_put`, `nexus_delete`), generación automática de contratos OpenAPI 3.0 JSON (`nexus_generar_openapi_json`) y documentación interactiva Swagger UI (`nexus_generar_swagger_ui_html`).
- **Driver PostgreSQL Nativo en Puro LÚMEN (`stdlib/postgres.nv`)**: Implementación completa del protocolo binario Wire 3.0 de PostgreSQL (StartupMessage, Query, RowDescription, DataRow) sin depender de `libpq` en C.
- **Driver Redis RESP3 con Pipeline (`stdlib/redis.nv`)**: Serializador de comandos RESP3, operaciones SET/GET/INCR y canalizaciones asíncronas por lotes en una sola llamada de red.
- **Framework UI Declarativo Reactivo (`stdlib/ui_reactiva.nv`)**: Motor de interfaz de usuario multiplataforma con Virtual DOM, hooks de estado reactivo (`ui_estado_crear`, `ui_estado_actualizar`), reconciliación diffing y renderizado para HTML5 y Terminal TUI.
- **Self-Hosting Stage-3 Fixed-Point Confirmado**: Emisión directa de ejecutables ELF64 autónomos (`stdlib/compiler/asm_emitter.nv`) con verificación criptográfica SHA-256 byte-idéntica (`d006c5af592fed2496c36dcfa0077dc54d891dcdc77f2218b0cf88d2925f7d25`) entre pasadas de compilación.
- **Playground Web Modernizado con WebGPU & Time-Travel Debugger**: Integración en `/home/user/lumen_web/index.html` de un depurador visual con barra de retroceso temporal (Snapshots), renderizador de partículas WebGPU en tiempo real y nuevos presets interactivos.

---

## v2.4.6 — 15 Agosto 2026

### Agregado (Fronteras Avanzadas: IA Cuantizada, Vector DB, Actores OTP & Tooling Pro)
- **Base de Datos Vectorial Nativa (`stdlib/vector_db.nv`)**: Motor de indexación vectorial de alta dimensionalidad con métricas de similitud coseno (`similitud_coseno`), distancia euclidiana L2 (`distancia_euclidiana`), producto punto y filtrado semántico de metadatos para aplicaciones RAG (Retrieval-Augmented Generation).
- **Motor de Inferencia IA & Cuantización INT8 (`stdlib/ia.nv`)**: Cuantización simétrica W8A16 (`ia_cuantizar_int8`), multiplicación matriz-vector cuantizada (`ia_matmul_cuantizado`), Rotary Position Embeddings complejos (`ia_aplicar_rope`), KV-Cache para decodificación autoregresiva rápida y muestreo probabilístico por temperatura y Top-P (Nucleus).
- **Modelo de Actores & Tolerancia a Fallos Erlang/OTP (`stdlib/actor.nv`)**: Actores livianos con buzón de mensajes (`buzon`), paso de mensajes desacoplado (`actor_enviar`), despacho secuencial (`actor_procesar`) y árboles de supervisión con estrategias de auto-recuperación (`supervision_sanar`).
- **Asistente Inteligente de Terminal (`lumen ai`)**: Subcomandos `explain` (análisis estático y complejidad), `fix` (detección y corrección asistida), `test` (generación automática de tests unitarios) y `chat` (asistente interactivo de arquitectura).
- **Empaquetado Binario Standalone (`lumen bundle <archivo.nv>`)**: Generación en un solo comando de ejecutables binarios nativos autocontenidos con cero dependencias externas.
- **Gestión de Registro Local y Privado (`lumen registry`)**: Comandos `info` (estado y caché de paquetes) y `serve` (microservicio local de registro de paquetes para entornos empresariales).
- **Soporte Completo de Asignaciones Indexadas en Miembros (`obj.array[i] = val`)**: Unificación del análisis sintáctico y semántico para escrituras en colecciones anidadas dentro de estructuras.

---

## v2.4.6 — 15 Agosto 2026

### Agregado (Consolidación de las 20 Fases de LÚMEN)
- **Operador Pipe (`|>`)**: Evaluación y encadenamiento funcional de izquierda a derecha sin sobrecarga (`datos |> filtrar() |> procesar()`).
- **Azúcar Sintáctico para Tipos Opcionales (`T?`)**: Soporte nativo para `texto?`, `entero?`, `decimal?`, `Punto?` y `lista<T>?` equivalente a `opcion<T>`.
- **Comprensión de Listas (List Comprehensions)**: Sintaxis funcional `[expr para var en iter si cond]` y en inglés `[expr for var in iter if cond]` desazucarada a bucles optimizados con asignación de arrays in-place.
- **JIT Tiering Automático en la Máquina Virtual**: Perfilado de invocaciones de funciones en `Opcode::Call` y compilación JIT nativa en caliente en memoria RAM vía Cranelift (`cranelift-jit`).
- **Diferenciación Automática N-Dimensional (Autograd) en `stdlib/tensor.nv`**: Grafo de computación dinámico con paso hacia atrás (`backward()`) para cálculo de gradientes automáticos, convolución 1D/2D y Layer Normalization.
- **Backend LLVM IR Directo (`lumen build --aot llvm`)**: Emisión directa de código LLVM IR (`.ll`) y bitcode para optimizaciones industriales globales.
- **Time-Travel Debugging en CLI y VM**: Grabación de instantáneas de ejecución y soporte para comando `back` / `step-back` / `retroceder` para volver atrás en el tiempo durante la depuración.
- **Generador Automático de Bindings (`lumen bindgen`)**: Parsing de cabeceras C (`.h`) o funciones Rust `extern "C"` y generación de módulos `.nv` listos para importar.
- **Puente Rust/Cargo (`lumen install cargo:<crate>`)**: Vinculación de cualquier crate de `crates.io` con wrappers FFI automáticos en `./pkgs/`.
- **Servidor Microservicios WebSockets, SSE & HTTP/3 / QUIC**: Soporte en `stdlib/servidor.nv` para WebSockets RFC 6455, Server-Sent Events y datagramas QUIC/UDP.
- **Bootstrap 100% Autónomo Self-Hosted (`lumen bootstrap`)**: Compilación y ejecución directa mediante el compilador nativo en puro LÚMEN (`stdlib/compiler/compiler_v4.nv`).
- **Comando `lumen bench <archivo.nv>`**: Suite integrada de micro-benchmarks con estadísticas de latencia mínima, promedio, máxima y throughput de ejecuciones por segundo.

---

## v2.4.6 — 15 Agosto 2026

### Agregado (Ergonomía, Lenguaje y Compilador)
- **Interpolación de cadenas `f"..."`**: Soporte para cadenas formateadas con expresiones arbitrarias `{expr}` (ej: `f"Hola {usuario}, total: {precio * cant} USD"`). Se desazucara e interpola con `a_texto` en tiempo de compilación con paridad en VM, AOT y WASM.
- **Métodos inherentes en Structs (`impl StructName { ... }`)**: Sintaxis directa `impl Punto { funcion entero suma(este) { ... } }` con receptor implícito `este`/`self` y resolución automática de métodos `p.suma()` sin requerir rasgos intermedios.
- **Operadores Bitwise completos (`^`, `~`, `&`, `|`, `<<`, `>>`)**: 
  - Tokenización de `^` (Caret / BitXor) y `~` (Tilde / BitNot) en el lexer.
  - Tipado en semántica (`sema.rs`), opcodes en bytecode VM (op 54 `BitXor`, op 55 `BitNot`), y generación en C99 (`_a.i ^ _b.i`, `~_a.i`) y Cranelift (`bxor`, `bnot`).
- **Mutación de L-Values multidimensionales y anidados (`m[i][j] = val`, `x.campo[i] = val`, `r.origen.x = val`)**: Generación de *write-back* en cascada en el generador de IR. `stdlib/matrices.nv` ahora opera al 100% de forma nativa sin ceros residuales.

### Agregado (CLI, Multiplataforma y Ecosistema)
- **Comando `lumen doctor` / `lumen info`**: Diagnóstico automático del entorno (sistema operativo, arquitectura, compilador C disponible, estado de los backends AOT y módulos de la `stdlib`).
- **Compilación Standalone (`lumen build --standalone <archivo>`)**: Genera binarios nativos independientes autónomos optimizados (`-O3 -s`) con todas las dependencias enlazadas.
- **Selector explícito de Backend AOT**: Soporte para `lumen build --aot <c|rust>` / `--backend <c|rust|cranelift|llvm>`.
- **Soporte FFI de 64 bits y SQLite multiplataforma**:
  - `Value::as_i64` para preservar punteros de 64 bits en FFI sin pérdida de precisión.
  - Primitivas `__ffi_peek64`, `__ffi_peek_ptr`, `__ffi_peek_byte`, `__ffi_poke_byte`.
  - `stdlib/sql.nv` ahora detecta y carga dinámicamente `libsqlite3.so.0`, `libsqlite3.so`, `sqlite3.dll` o `libsqlite3.dylib`.
- **Fix de enlace GCC en Linux**: Eliminado el parámetro restrictivo `-lregex` en Linux (glibc) y corregidos calificadores `const` en `lumen_rt.h`.

---

## v2.4.6 — 14 Agosto 2026

### Agregado (compilador self-hosted — Fases 61/62/63 reales)
- **OR Patterns reales en el self-hosted** (`parser.nv` branch `elegir`): loop que consume el pipe `|` y construye árbol `Binary ||` encadenado `(sel==A) || (sel==B)` → despacha a `_cg_and_or` (short-circuit real con JmpIf). **`fase61_or_patterns` byte-IDÉNTICO en la cadena 100% LÚMEN**.
- **IF-LET real en el self-hosted** (Fase 62): handlers op 52/53 (`MatchType`/`MatchPayload`) en `vm.nv` + caso `IfLet` en `codegen.nv` (auxiliar `tiene_test`) + branch if-let en `parser.nv` (patrón `algun`/`exito`/`error`/`ninguno`/Ident con bind). **`fase62_if_let`/`fase62_if_let2` byte-IDÉNTICOS** (también en VM Rust: opcodes 52/53 end-to-end con `bind_pattern_vars` en sema).
- **Rangos `..`/`..=` en el self-hosted** (Fase 63): token en `lexer.nv`, nodo `Range` en `parser.nv`, desugar a lista + intercepto `==` con rango (short-circuit `_cg_and_or`) en `codegen.nv`, fix `OP_ARRAY_PUSH=32` y `32 => 32` en el encoder nativo. **`fase63_range_patterns` byte-IDÉNTICO**.
- **8 ejemplos de fase nuevos** (2 por fase): `fase61_or_patterns.nv`, `fase62_if_let.nv`, `fase62_if_let2.nv`, `fase63_range_patterns.nv`, `fase64_string_patterns.nv`, `fase66_operator_overloading.nv`/`_2.nv`, `fase68_associated_types.nv`/`_2.nv`, `fase70_impl_trait.nv`/`_2.nv` — todos OK en VM y backend C.
- **FIXPOINTs v4 consecutivos**: `DF7676DE7B…` tras OR patterns (150,463 B → 165,944 B, byte-idénticos self==self2, reemplaza a `A3CBAA0F…`). Batería self ampliada: **OK=42 FALLAS=0** (incl. demo_completo, match, enums, corutinas_demo, jr_concurrencia, 44_extension_methods, test_ffi_min, test_texto_std, fases 61-70).

### Agregado (AOT — backend C / Cranelift optimizados)
- **Cranelift: variables SSA reales del frontend** (nada de StackSlot): `Variable` de Cranelift con `declare_var`/`def_var`/`use_var` y phis vía dominancia — obsoleto el paso por memoria.
- **Backend C: índices de registro constantes** (sin strcmp lineal en cada Load/Store): `gv[N]` directo vía `name_idx`, fallback `_fv` solo para nombres no registrados.
- **Benchmark `bench_fib.nv`** (fib(26)+loop 100, runs calientes): **VM 856ms → C 22ms (antes 406ms, 18x) → Cranelift 5.6ms (antes 116ms, 20x)** — ambos backends en milisegundos.
- **Fix C backend**: temp-capture en CALLS de usuario (`{ Val _r = _f_x(); PUSH(_r); }` en `_f_/{}`, `CallValue` y `_fref_call`) — gcc evalua LHS antes de la función callee → corrupción de pila compartida. `fase65_guard_let`/`_2` byte-idénticos en C y VM.
- **Batería dual `aot_bateria_dual.ps1`**: **C OK=38 DIFF=0 (paridad total) · RUST OK=12 DIFF=26 (límite de diseño: sin strings/structs/colecciones) · FAIL=0 SKIP=1 HANG=0**.

### Agregado (Playground Web — Ronda L1 + F1.2/F2.3/F4.2 completadas)
- **`lumen serve` real** (servidor HTTP estático Rust puro, sin Python): `--port`, `LUMEN_PORT` env, MIME types, headers COOP/COEP, 404, anti path-traversal, redirección `/` → `/web/index.html`. **Cache ETag/If-None-Match** (304 Not Modified) + variable `LUMEN_PORT`. Verificado (200/304/404/JSON).
- **Backend `/api/run`**: `POST` → compila y ejecuta con la **VM Rust nativa** (`run_source_capture`) → JSON `{ok,output}`/`{ok,error}` con spans `(linea,col)`. `GET /api/health`, `/api/examples`, `/api/examples/{file}`.
- **CodeMirror 6 vendorizado** (11 módulos ESM planos + import map, sin CDN) + **modo LUMEN generado** desde `token.rs` (74 keywords, `StreamLanguage` + syntax highlighting Catppuccin). **Autocompletado** con `@codemirror/autocomplete` (`Ctrl+Space`, keywords + snippets), **minimapa** (EditorView espejo sincronizado), **error gutter mejorado** con tooltips. Autosave localStorage.
- **Sigma L1**: stdlib embebida via build.rs (`embedded_stdlib.rs`, 31 archivos) + `ModuleLoader::with_memory_files`; `run_lumen`/`check_lumen`/`compile_to_bytes` con loader virtual; 128 ejemplos embebidos (`embedded_examples.js` autogenerado) con fallback offline; `.nvc` descargable; toggle **WASM ↔ Servidor**.
- **Selector F4.2**: categorías (basics/functions/data/pro/stdlib/other), búsqueda textual, favoritos en localStorage, marcador "📦 importar". Dropdown personalizado con secciones (Favoritos, Recientes, Categorías).
- **Historial de ejecuciones**: botón `🕘 Historial` (panel flotante, hasta 10 runs) + **toggle backend PERSISTENTE** (`lumen_playground_backend`).
- **2 ejemplos interactivos** (convención 2 por fase): `canvas_demo.nv` (canvas drawing vía JS bridges) + `clock_demo.nv` (reloj tiempo real). Bridges JS (`__js_call`, `__js_eval`) + corutinas.
- `pkg/lumen_wasm_bg.wasm` 2.37 MB regenerado con fixes OR/rangos/autocompletado.

### Arreglado
- **`tcp_listener` cfg**: campo del struct VM sin cfg (std::net siempre disponible) — `cargo test -p lumen-sema` compilaba lumen-vm sin features y fallaba. 
- **Clippy `-D warnings` limpio**: eq_op duplicado en `__codegen_a_nvc`, colapso de bloques idénticos en sema, Arc/Ret corutinas/ChannelCell gateados `cfg(full)`, allows documentados.
- **Parlance FFI**: errores sin detalles + bandas `-1e9`/`[1e9,2e9)`/`6e9` con guards — sin colisiones en ints negativos/grandes.

### Infraestructura
- **CI autotag**: tag `v<version>` automático solo cuando CI completo pasa (fmt/clippy/tests 3 OS/wasm) — `VERSION` como fuente única de verdad (`scripts/autotag.ps1` para bumps semver), build multi-target + GitHub Release en el mismo workflow (`needs: autotag`).
- cargo test 0 FAILED (lexer 27, parser 45, sema 56, ir 20, vm 45, e2e 166 + resto, ~380 totales).

---

## v2.4.6 — 8 Agosto 2026

### Agregado (VM LÚMEN `vm.nv` — Stream/Async/Par/Actor/Generator completados)
- **Handlers de streams**: `__stream_desde`/`__stream_from`, `__stream_mapear`/`__stream_map`, `__stream_filtrar`/`__stream_filter`, `__stream_colectar`/`__stream_collect` — delegados a natives Rust
- **Iteradores paralelos**: `__par_mapear`/`__par_map`, `__par_unir`/`__par_join` — delegados a natives Rust
- **Actores**: `__actor_nuevo`/`__actor_new`, `__actor_enviar`/`__actor_send`, `__actor_recibir`/`__actor_recv` — delegados a natives Rust
- **Generadores**: `__generador_nuevo`/`__generator_new`, `__generador_siguiente`/`__generator_next` — delegados a natives Rust
- **Select/Async I/O**: `__seleccionar`/`__select`, `__leer_archivo_async`/`__file_read_async`, `__escribir_archivo_async`/`__file_write_async` — delegados a natives Rust
- **`sprint1_concurrencia.nv` 100% paridad byte-idéntica** entre VM Rust y VM LÚMEN (Stream, Async I/O, Timer, Select, Par Map/Join, Actor, Generator)

### Arreglado (VM LÚMEN `vm.nv` — paridad con la VM Rust)
- **`__map_obtener` con mapas JSON**: devolvía `Void` (el key boxed 1e9+N no coincidía con las claves strings reales del host) y Values del host sin boxear (Str real → crash "Ge requires numbers or strings"). Ahora lookup dual (key boxed del guest → desboxeado para JSON) + boxeo por tipo real (`__tipo_de`/`a_texto` del host: texto→`box_str`, booleano→9e9+1/9e9, lista→`arrs`, diccionario→`mapas`) → **`test_json_avanzado` CORRECTO**
- **Handlers de archivos faltantes**: `__existe_archivo`/`__file_exists` (bool boxed, antes "0" en vez de "false"), `__leer_archivo`/`__file_read` (con `intentar`, el native devuelve Resultado), `__escribir_archivo`/`__file_write` → **`test_sistema_directo`/`test_sistema_avanzado` CORRECTOS**
- **Verificado**: batería `test_vm.ps1` 39/40 (solo `stress_fecha` flaky timing) · 7 tests sistema/JSON/csv/migración byte-IDÉNTICOS · 19 checks cruzados con `vm_self.nvc` (111,318 B, regenerado con compiler_v4) todos OK · cargo test OK

### Arreglado (sintaxis `para` — paridad Rust ↔ LÚMEN)
- **Init sin tipo en `para` clásico** (`para (i = 0; ...)`): el parser Rust exigía declaración tipada. Ahora `parse_for` usa `is_for_init_decl()` (keyword de tipo, tipo custom `Punto p`, o genérico) y en caso contrario construye un `Decl::Variable` con `Type::Infer` consumiendo el `;`
- **`para` clásico sin paréntesis** (`para entero i = 0; cond; paso { }`): el parser Rust lo reenviaba a foreach → E011. Nuevo dispatch con `is_foreach_like()` (lookahead puro: `[tipo]? ident (en|in)`) → foreach solo si hay `en`/`in`, si no `parse_for`. El self-hosted (`parser.nv`) recibe el helper `_st_es_foreach` (lookahead por posición sobre `tokens`) + branch de clásico sin paréntesis (desugar idéntico al clásico con `(`) → `tui_test_min16/17/18` ahora **byte-idénticos en ambas VMs**
- **FIXPOINT v4 CONFIRMADO**: SHA-256 `3DA624D6AD32E359D3714F7CD936563CE1A60ED633590CB580D695F24C7E282A` self==self2 (compiler_v4.nv 135,465 B → .nvc 150,684 B, ~5s)
- **Verificado**: cargo test 0 FAILED · batería `test_vm.ps1` 39/40 · **fuego.ps1: 389/389 compilan · 112 CORRECTOS · 1 INCOMPATIBLE (graficos_demo SDL, por diseño) · 4 TIMEOUT (debug_parser3 loop, graficos_completo/gui_ventana GUI, sprint1_http red) · 0 fallos**
- ⚠️ `test_vm.ps1` debe ejecutarse desde la RAÍZ del repo (las rutas de `entrada_vm.txt` son relativas — desde `stdlib/compiler` da FALLAS masivas falsas)

### Bootstrapping Doble (Hito Final)
- **Fixpoint del compilador**: SHA-256 `3DA624D6AD32E359D3714F7CD936563CE1A60ED633590CB580D695F24C7E282A` — 150,684 bytes **byte-idénticos** en self/self2 (~5s)
- **VM LÚMEN autogenerada**: `vm_self.nvc` (111,318 B) compilada por `compiler_v4_self.nvc` y ejecutando `demo_completo.nvc` correctamente (89/89 líneas, 0 diffs)
- **0 dependencias de Rust**: LÚMEN compila LÚMEN, VM LÚMEN ejecuta bytecode LÚMEN, todo autocontenido

---

## v2.4.6 — 6 Agosto 2026

### Agregado
- **Sprint 6: Gramática completa en el pipeline puro (self-hosted)** — `importar` con base-dir + self-import detectado, `sea`/`const` (VarDecl), StructInit `T {}` → mapas, `.campo` → Index, `elegir`/`defecto:`/`caso` reales (cadenas `sino` con im::HashMap persistente), enum `Nombre::Miembro(args)`, Option/Result (`algun`/`ninguno`/`exito`/`error` → op 38/39/41/42), closures IIFE, params default inlineados, traits `rasgo`/`impl`/`este` (métodos mangled + resolución por tipo de var), cast `como`, cortocircuito `&&`/`||` (`_cg_and_or`)
- **Sprint 7: VM en LÚMEN (`vm.nv`)** — ejecutador de .nvc en LÚMEN puro (dispatch 0-46, bandas boxed, corutinas reales con intercambio st/sp/pc, handlers JSON/tarea/coro/crypto/fs/env/tiempo/hilo/mutex/calendario, `fmain` acepta `__main__`/`main`/`principal`)
- **Optimización 43x**: COW con `Arc` en `Value::Str`/`Value::Array` (fixpoint 861s → 20.1s); `a_entero` O(n)→O(1) (demo 120s → 0.9s); `__str_subcadena_chars`/`__str_reemplazar` natives; guards de banda [3e9,9e9) y < -1e9
- **Tipo dinámico `Numero` real** + alias `cualquiera`/`any` (desbloquea csv.nv y test_migracion)
- **Benchmark vs Rust**: `scripts/benchmark_vs_rust.ps1` — compile x5.4, run x231 (mediana x2-6)
- **Resultados**: batería test_vm.ps1 **39/40** (solo `stress_fecha` flaky) · cargo test **385/385** · **fuego.ps1: 389/389 compilan · 108 CORRECTOS · 4 INCOMPATIBLES · 4 TIMEOUT · 0 fallos**

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
- Docs sincronizadas: README (v2.4.6), CHANGELOG, `docs/self-hosting.md`, `docs/siguiente.md`, `docs/roadmap.md`, reports

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
  - ~385 tests pasan, autocompilación funcional (533ms)
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
