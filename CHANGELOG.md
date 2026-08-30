# Changelog de LÚMEN

## [3.5.7 + Rondas JIT v3.5.31→v3.5.37] - 2026-08-30

> Trabajo de rendimiento benchmark-driven posterior a la v3.5.7 de producción:
> JIT VM + AOT, con paridad byte-a-byte ON/OFF en cada ronda. Detalle técnico en
> [docs/arquitectura/jit.md](docs/arquitectura/jit.md); números en
> [docs/informes/BENCHMARK.md](docs/informes/BENCHMARK.md) y
> [benchmarks/results/informe.md](benchmarks/results/informe.md).

### Rendimiento (TOTAL de benchmarks, min-of-15, release): 590 → 267.1 ms (5.8× vs intérprete)
- **Tier-R (v3.5.34)**: recursión auto-nativa en registros — fib 74 → **4.4 ms** (23× vs intérprete, ~2× el C)
- **Tier-2 (v3.5.31+)**: bucles con aritmética de pila nativa sobre la arena de slots; arrays y strings nativos; super-opcodes Fused de 3 y 6 instrucciones; JmpIf nativo (v3.5.35); análisis estático de tipos VTag con concat rápido `lj_concat` y Load/Store nativos por etiqueta (v3.5.37)
- **Tier-1**: delegación por shims con prólogo de 1 call por nombre (`lj_probe_int`) y `lj_call_fast` para nombres no-builtin (decisión estática)
- **VM (intérprete, v3.5.36)**: pools de buffers de scope (sin alloc por llamada) + invalidación SELECTIVA de la caché de variables → fib OFF 107.7 → 100.4 ms

### Bugs reales encontrados y arreglados por las rondas
- v3.5.33 — constant folder IR: `i64::MIN / -1` paniqueaba y `%` era truncante (no `rem_euclid`) — paridad exacta con el runtime
- v3.5.34 — folder optimize.rs: modelo de pila por delta NETO → `f(3) + 1` borraba el argumento y el `Add` ("Stack underflow" en ambos modos)
- v3.5.35 — Tier-2: puntero `flat` obsoleto tras realocación (calls a usuario asignan slots) → lecturas nativas a memoria liberada (primes daba 1 o bucles infinitos)
- v3.5.37 — Load/Store nativos por etiqueta indexaban `slots[nidx]` sin verificar resolución del prólogo → panic en el fixpoint

### Estado de verificación (2026-08-30)
`cargo fmt --check` 0 · `clippy --all -- -D warnings` 0 · 956/956 tests ×2 (JIT ON/OFF) ·
`lumen check examples` 396/396 · ci_gate 392 PASS / 0 crashes ×2 · fixpoint self-hosting byte-idéntico (sha256 `02b0460d…`)

### Limpieza y documentación
- Documentación reorganizada: `docs/` con índice central ([docs/README.md](docs/README.md)) y subcarpetas guias/referencia/arquitectura/desarrollo/spec/informes; `reports/` → `docs/informes/`; nuevo [docs/arquitectura/jit.md](docs/arquitectura/jit.md)
- Raíz del repo limpia: artefactos de tests de archivos eliminados y cubiertos por `.gitignore`; `scripts/update_docs.py` (obsoleto, regeneraba docs v1.6.0) eliminado; script de fixpoint actualizado a la nueva ruta

## [3.5.7] - 2026-08-29

### AOT Industrial — Cranelift/LLVM completos + memoria nativa (Incremento B)
- **Cranelift completo**: intentar/atrapar real (`_lw_err_active` + catch por block-param), enums completos (`EnumCtor`/`MatchVariant`/`MatchPayload`), `prestado mut`/`MakeRef` con celdas `Val` y write-through, funciones como valores (`func_addr` + `call_indirect`), sombreado por bloques, flujo de valores entre labels (block-params + `simulate_label_depths` → ternarios OK)
- **LLVM IR reescrito** al modelo `_lw_*` (antes i64-only) + `merge-allocas` + floats hex + shim `lumen_rt.h` y link con `clang -O3 -lm -lpthread`; CLI `lumen build --llvm` funcional
- **Memoria nativa**: todas las vars por celda, Stores con `_lw_dcp` (paridad `gv=_dcp`), args deep-copiados (`T_PTR` pasa tal cual), arrays con `cap` + `_lw_arr_push_ip` in-place → `stress_04_arrays` de O(n²)/OOM a instantáneo en C/Cranelift/LLVM; `sizeof(Val)=80`, `INT64_MIN` wrap, `div0` capturable, binding write-through fix, `MatchPayload` enums
- **Globales reales** en nativo (`program_global_names` + celdas `lw_glob_*` compartidas → `logging`/`testing_sr` OK), decimales paridad VM (round-trip), arrays O(n) también en C
- **Paridad 4-way**: VM↔C↔Cranelift↔LLVM byte-identical en `fuzz/*`, `examples/*`; barrido `239 OK / 7 divergencias` (closures con captura, sort complejos, guard-let+NaN, structs dinámicos, 3 demos hilos/baremetal/3D) / `150 skip`; `955` tests, `ci_gate 392 PASS 0 CRASH`
- **DX**: `cargo fmt` 0, `clippy --all -- -D warnings` 0, `396/396` ejemplos `lumen check` OK, bench `c -O3 -flto -march=native` (strings 0.199→0.045s)

### Fuzzing manual VM↔nativo — 3 bugs encontrados y arreglados

#### Bug F1 — indexado/largo de textos en backend C (paridad)
- `"abc"[1]` crasheaba el binario nativo; `s.largo()` daba 0. `_arr_get`/`_arr_len`
  ahora manejan T_STR (get valida rango y devuelve el carácter; set lanza error claro)

#### Bug F2 — structs declarados dentro de funciones
- `estructura P { ... }` dentro de un cuerpo de función no se registraba en sema
  ("El struct 'P' no está definido"). collect_structs ahora registra también los locales

#### Bug F3 — imprimir multi-argumento en backend C
- El nativo imprimía cada argumento en su propia línea; la VM los concatena.
  Ahora ambos producen UNA línea (`imprimir("a:", x)`); `imprimir()` sin args = línea vacía

### Verificación de paridad
- Lotes de fuzzing comparando salida VM vs binario nativo: aritmética/overflow/mod
  negativo, floats, strings, sombreado anidado, arrays 2D, struct-en-array field assign,
  elegir con or/guard, destructuring, foreach, try/catch anidado, refs en bucles,
  métodos mutables, mapas, resultado/opción — **0 diferencias** tras los fixes

## [3.3.5] - 2026-08-25

### Lenguaje — hacia producción sin limitantes

#### Sombreado de bloques real (VM + builder + AOT C)
- El builder ahora emite ScopePush/ScopePop en los cuerpos de si/sino,
  mientras, para, para cada, si-let, elegir y match (antes solo bloques `{}`)
- `sea x = 1; si ... { sea x = 2 }` conserva x=1 fuera del bloque en AMBOS motores
  (antes el VM y el C filtraban la declaración al scope exterior)
- Backend AOT C: planificador estático de slots (`plan_var_keys`) — cada sitio
  textual de declaración recibe su propio key gv[] con sombreado correcto;
  los bucles reusan su slot (planeo por instrucción, no por ejecución)

#### Métodos con receiver mutable: `prestado mut este`
- Nuevo soporte de sintaxis en parse_param: `prestado mut este|self|yo`
- Sema sustituye Self por el tipo objetivo del impl dentro de Prestado
- Builder emite MakeRef del receptor cuando el método lo declara mutable:
  las mutaciones del método SÍ afectan a la instancia llamadora (VM y AOT)

#### Aviso W060: prestado mut con argumento no-lvalue
- sema ahora acumula avisos no fatales (`SemanticAnalyzer::warnings`)
- Pasar `o.campo`, `arr[i]` o una expresión a un parámetro `prestado mut`
  imprime ⚠ W060 explicando que se pasa por valor (las mutaciones se pierden)
- `analyze(&mut self)` permite leer los avisos desde CLI/repl/lsp/api

## [3.3.1] - 2026-08-25

### Lenguaje — Bugs QA completados

#### Bug #6 COMPLETO — referencias reales `prestado mut` con write-back
- Nuevo `Value::Ref` con celda compartida (`Arc<Mutex<Value>>`): los alias nunca divergen
- Nueva instrucción IR `MakeRef` (opcode 63): el builder la emite cuando un argumento
  variable simple llega a un parámetro `prestado mut`; expresiones no-lvalue caen a valor
- `Load`/`Store` transparentes a través de la referencia; write-back al slot del llamador en `Ret`
- Reenvío de referencias f(g(x)) comparte la misma celda; frontera de hilos/tasks degrada a valor
- Sema: auto-deref en binarios y asignación (`n = n + 1` sobre `prestado mut entero`)
- Backend AOT C: punteros reales (`T_PTR`), escritura inmediata, exclusión de ref-targets del save/restore
- Renombrado de params por función (`{fn}::{param}`) en el backend C: elimina colisiones
  entre params del callee y variables del llamador (bug latente pre-existente)

#### Bug #7 COMPLETO — comptime con llamadas a funciones
- Nuevo intérprete const-eval (`lumen-ir/src/comptime.rs`): aritmética, comparaciones,
  strings, builtins puros (abs/min/max/piso/techo/redondear/raiz/potencia) y llamadas
  recursivas a funciones propias del programa (límites: profundidad 128, 1M pasos)
- Pre-paso en CLI y pipeline de tests: `comptime { fib(20) }` se pliega a literal `6765`
- Degradación segura: lo no evaluable se ejecuta normal en runtime

### Backends AOT — todos funcionan sin fallos
- Backend C: intentar/atrapar real vía bandera de error + chequeos estáticos (sin
  setjmp/longjmp, inmune a -O3); MatchVariant para elegir destructurante;
  guard POP contra underflow (funciones void ya no leen ST[-1])
- Test gcc integral: refs + try/catch + elegir + structs + comptime verificados end-to-end
- LLVM/Cranelift: rechazo ruidoso (`llvm_supported`/`cranelift_supported`) — los backends
  limitados ahora FALLAN con mensaje claro en compile-time en lugar de emitir
  silenciosamente artefactos rotos; sugerencia apunta al backend C completo

## [3.3.0] - 2026-08-24

### Lenguaje — Bugs QA v3.2.0 corregidos + features nuevas

#### Bug #1 — fmt borraba código (CRÍTICO)
- Eliminados catch-alls `_ => {}` en fmt_stmt/fmt_decl/fmt_expr
- 7 statements recuperados: FieldAssign, ArraySet, For clásico, IfLet, GuardLet, Destructure stmt+decl
- 8 expresiones recuperadas: Range, Algun/Ninguno/Exito/Error/Intentar, TupleAccess, Lambda
- Exhaustividad forzada por compilador (nuevos Stmt/Decl sin brazo = error de compilación)

#### Bug #2 — `arr[i].campo = valor` en runtime (CRÍTICO)
- Builder: stack realineado con temporal entre StructSet y ArraySet
- Variante profunda `o.items[i].campo = v` también arreglada con write-back en dos fases

#### Bug #3 — if-let destructura enums de usuario con datos (CRÍTICO)
- Nuevo opcode MatchVariant=62: compara solo variant del Value::Enum
- MatchPayload extendido para extraer fields de Value::Enum
- Builder maneja ambas formas: `Exitoso(x)` (sin calificar, Expr::Call) y `Enum::Variante(x)` (calificada, Expr::EnumCtor)
- Multi-arg con ArrayGet por índice

#### Bug #4 — structs recursivos via opcion<Self> (CRÍTICO)
- collect_structs pre-registra nombres antes de resolver campos
- can_assign usa tipado nominal para Struct de usuario (mismo nombre = mismo tipo)

#### Bug #5 — imports transitivos (confirmado NO-bug funcional)
- Sistema funciona; confusión por convención de prefijos con nombres que empiezan por {stem}_

#### Bug #6 parcial — auto-deref Prestado<T> en sema
- FieldAccess, MethodCall, FieldAssign, Index, ArraySet resuelven a través de Prestado { inner }
- Elimina falsos errores E060/E047/E044; runtime sigue value-semantics (documentado)

#### Otros fixes
- 79 builtins sin typing: fallback Decimal→Numero (elimina falsos E031)
- __regex_reemplazar/__regex_replace typing explícito Texto
- Overflow aritmético wrapping_* (sin panic en i64::MIN/MAX)
- Notación científica 1e5, 1.5E-3
- ArrayPushVar O(n²)→O(n) para .agregar() en variables
- intentar/atrapar captura errores de runtime
- ScopePush/ScopePop + StoreLocal para shadowing correcto
- Sintaxis params unificada: ambas formas `Tipo nombre` y `nombre: Tipo`
- --help documenta sintaxis real + env vars
- Cascada de errores mitigada (dedup por código+línea, cap 20)
- LUMEN_KEEP_C=1 conserva el C intermedio de build --native
- Ejemplo nuevo: examples/fase_impl_inherente.nv

## [3.2.0] - 2026-08-21

### Producción Real — Hardening y Escalabilidad — CERTIFICADO APTO sobre artefacto
- **Verificación de release (artefacto empaquetado, no árbol fuentes):**
  - `lumen-v3.2.0-windows-x64.zip` SHA-256 `d5cb2b99…` == `SHA256SUMS.txt` ✓ · `linux-x64.tar.gz` `559e468b…` ✓
  - Paquete: `lumen.exe` + 69 stdlib + **394 ejemplos** (389 + 5 stress) + docs + web playground
  - `ci_gate.py` sobre el binario del paquete con su stdlib/examples: **393 PASS / 0 FAIL / 1 TIMEOUT permitido (`test_quick_connect.nv` @interactive) / 0 CRASH — Gate PASSED**
  - Usuario común SIN `LUMEN_HEADLESS`: `demo_produccion_total.nv` → `✓ Inferencia Transformer completada (dim=8)` EXIT:0 (antes `Índice 1 fuera de rango` en `tensor_softmax`) · `stress_04_arrays.nv` 20k en **0.04s** (antes >120s) · `stress_05` value semantics OK · `stress_02` try/catch+wrap OK
  - `cargo test --release --workspace`: 48 unit + 621 e2e + 11 production, 0 FAILED · bench release: lexer 1.6µs / parser 4.4µs / pipeline 15.3µs / vm_fib_20 11ms
- **VM:** `Add/Sub/Mul/Shl/Neg/Div/Mod` con `wrapping_*` para evitar panic en `i64::MIN`/`MAX` (overflow definido, no crash)
- **Lexer:** soporte notación científica `1e5`, `1.5E-3` (antes `E012`)
- **IR/VM:** `ArrayPushVar` in-place O(n²)→O(n) (20k pushes 10s→1s, 100k timeout→0.48s) con `ScopePush/Pop` y `StoreLocal` para shadowing correcto
- **VM:** `intentar/atrapar` ahora captura errores de runtime (handler stack + `PushHandler`/`PopHandler` opcodes 57/58)
- **VM:** `__str_longitud` ahora `chars().count()` (emoji 6, no 9 bytes) y `__str_subcadena` clamp negativo con `end==-1` → len
- **VM:** `__ffi_escribir` fix overflow 1 byte, `ffi_allocations` tracking + `Drop`, bounds checks para `escribir/leer/peek/poke`
- **CI:** fix job release — eliminado `lumen-lang-2.4.6.vsix` duplicado/obsoleto que causaba `Not Found update-a-release-asset`
- **Tests:** 5 regresiones `try_catch`, `overflow`, `agregar`, `scientific` → 621 e2e + 11 production (675 vm)

## [3.1.4] - 2026-08-21 — Producción Real (fixes escalables + bench + headless)

### ⚡ v3.1.4 Producción: listo para deploy real (escalable, sin parches temporales)

**Fixes escalables (no parches por demo):**
- **Builder fallthrough `Variable 'a'/'n'`:** `last_significant()` ignora `Label/Nop/Phi` para decidir terminador; `needs_return()`/`emit_return_if_needed()` en `Function`, `ImplRasgo`, `compile_lambda`, `build()` (`Halt`). `label_counter` global (no reseteo por función) evita colisión `Label(0)` en `codegen` global `label_map` que rompía `matematicas.nv` (`Variable 'n'`). Commits `64db441`, `730e74d`, `f83964f`.
- **Aridad `pop()` corrupto:** `bind_args` unificado — `Call`/`CallValue`/`run_function` (hilos) ahora comparten `args.get(i).cloned().unwrap_or(Void)` + `defaults` reales en vez de `self.pop()` divergente.
- **Defaults persistidos `FuncMeta.defaults` + `CHUNK_VERSION 7`:** `ir::Func.defaults: Vec<Option<Value>>` → `codegen::FuncMeta.defaults: Vec<Option<DefaultValue>>` (`Int/Float/Str/Bool`) serializado en `Bytecode` v7 (compat v6 → `vec![None; params.len()]`). `VM bind_args` usa `DefaultValue` cuando `i>=args.len()`. `decode` acepta 6 y 7 para compat con `.nvc` antiguos.
- **Headless centralizado `stdlib/graficos.nv:es_headless()`:** usa `getenv("CI"/"LUMEN_HEADLESS")` vía `__ffi` (`msvcrt/libc/libSystem`) y chequea `peek!=0`. `iniciar()` y `ventana()` retornan `false/0` sin `SDL_Init`/`SDL_CreateWindow`. Demos con `si !iniciar() { retornar; }` ya son suficientes; guard per-demo redundante pero compatible.

**Suite y bench formal:**
- **Tests:** 616 e2e (incluye 4 regresión: fallthrough early return, matematicas `potencia(2,10)==1024`, defaults `CallValue`, lambda) + 9 production (aceptación 3 + performance 2 + integración) = **673 vm tests**, **917 workspace** (`cargo test --workspace` 0 FAILED).
- **Bench 8** (`cargo bench -p lumen-bench`): `lexer_tokenize`, `parser_parse`, `pipeline_full`, `vm_fib_20` + 4 prod nuevos `prod_fallthrough_early_return`, `prod_defaults_callvalue`, `prod_matematicas_potencia`, `prod_graficos_headless` (reporte `target/criterion/report/index.html`, `--quick` en CI).
- **Barrido:** `cargo run --bin lumen -- check examples` 389/389 OK (con `CI=1`), `LUMEN_HEADLESS=1 lumen run examples/graficos_*` → `init_fail_ok` sin `Variable 'a'`.

**CI `headless-check` nuevo:**
- Job Linux `env: LUMEN_HEADLESS=1 CI=1` corre `cargo test --workspace`, `cargo run --bin lumen -- check examples`, `cargo test --test production`, `cargo bench -p lumen-bench -- --quick`. Ver `.github/workflows/ci.yml` y `docs/produccion.md`.

**Versionado:** `Cargo.toml`/`VERSION` `3.1.4` · `CHUNK_VERSION 7` (decode v6+7) · `is_known_prefixed` con `_` single · docs actualizadas (`README`, `AGENTS`, `roadmap`, `produccion.md`, etc.).

---

## [3.2.0] - 2026-08-21

### Producción Real — Hardening y Escalabilidad
- **VM:** `Add/Sub/Mul/Shl/Neg/Div/Mod` con `wrapping_*` para evitar panic en `i64::MIN`/`MAX` (overflow definido, no crash)
- **Lexer:** soporte notación científica `1e5`, `1.5E-3` (antes `E012`)
- **IR/VM:** `ArrayPushVar` in-place O(n²)→O(n) (20k pushes 10s→1s, 100k timeout→0.48s) con `ScopePush/Pop` y `StoreLocal` para shadowing correcto
- **VM:** `intentar/atrapar` ahora captura errores de runtime (handler stack + `PushHandler`/`PopHandler` opcodes 57/58)
- **VM:** `__str_longitud` ahora `chars().count()` (emoji 6, no 9 bytes) y `__str_subcadena` clamp negativo con `end==-1` → len
- **VM:** `__ffi_escribir` fix overflow 1 byte, `ffi_allocations` tracking + `Drop`, bounds checks para `escribir/leer/peek/poke`
- **Tests:** 5 regresiones `try_catch`, `overflow`, `agregar`, `scientific` → 621 e2e + 11 production (675 vm)

## [3.1.4] - 2026-08-20

### ⚡ v3.1.4: 167 bugs corregidos, unificación y verificación en tres plataformas

**Unifica en una sola entrega el trabajo iniciado sobre la v2.4.6: los 8 bugs del reporte original y 159 más encontrados de forma activa.**

### 🐛 Correcciones de Más Impacto
- **BUG-166/167 (`regex.nv`)**: el regex nativo devolvía `false` a todo en Windows y macOS (stubs en la rama no-POSIX) y desbordaba al reemplazar con patrones que casan la cadena vacía. **Motor propio por backtracking, sin dependencias.**
- **BUG-165 (`lumen_rt.h`)**: `<sys/resource.h>` fuera de su guarda impedía TODA compilación nativa en Windows.
- **BUG-152/154 (`lumen-bundle`/`lumen new`)**: la stdlib no viajaba en la instalación y el prefijo de paquete se aplicaba mal.
- **BUG-151/161 (parser)**: bloques sin llave se ejecutaban en silencio; el arreglo rompió las declaraciones adelantadas, restauradas con E084.
- **BUG-147/148/149/150 (sema/IR)**: semántica de closures, structs y `prestado mut`.
- **BUG-GUI (build cross-plataforma)**: `gui_ffi.rs` incompatibilidad de tipos `*const u8` vs `*const i8` en `CreateWindowExA` (aarch64 Linux/Android) — corregido con `title_cs.as_ptr().cast()`.

### ✅ Verificación
- **720 pruebas en verde** en Linux y Windows.
- **393/393** en `lumen check`.
- **372 ejemplos** ejecutados sin fallos.
- **clippy sin avisos** y **cuatro fuzzers diferenciales** (structs/listas, closures, rechazo y regex) sin divergencias.

---

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

Todo los archivos "changes manual" se documentan aquí.

---

## v3.1.4 - 20 Agosto 2026

### Corregido (Verificación en Tres Plataformas)
- Motor regex nativo propio por backtracking sin dependencias (BUG-166/167) — arregla stubs no-POSIX y desbordes con patrones de cadena vacía.
- Guardas de plataforma completas en `lumen_rt.h` (BUG-165) — `<sys/resource.h>` bajo su guarda, desbloquea la compilación nativa en Windows.
- stdlib empaquetada en instalaciones y prefijo de paquete correcto (BUG-152/154).
- Bloques sin llave ya no se ejecutan en silencio; declaraciones adelantadas restauradas con E084 (BUG-151/161).
- Semántica de closures, structs y `prestado mut` corregida (BUG-147/148/149/150).
- GUI nativa Win32 (`gui_ffi.rs`) compilable en aarch64-unknown-linux-gnu y aarch64-linux-android.

### Verificado
- 720 pruebas en verde (Linux y Windows), 393/393 `lumen check`, 372 ejemplos sin fallos, clippy limpio, 4 fuzzers diferenciales sin divergencias.

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


### Notas de plataforma y cobertura añadida (post-3.3.6)
- Regex nativas en el **backend C** usan POSIX regex: en Linux/macOS plenas; en
  Windows los stubs divergen del VM (que tiene motor propio por backtracking).
  Pendiente v3.3.7: portar el motor own-engine a lumen_rt.h.
- Tests formales nuevos: structs locales, métodos `prestado mut este`, refs en
  bucles, indexado/largo de textos en el nativo (test gcc integral extendido).
- Benchmarks criterion: `prod_ref_mut_writeback`, `prod_comptime_fold`.
- Docs spec: vm-spec opcode 63 MakeRef; error-codes W060; LENGUAJE §4 comptime
  con llamadas puras. LSP: hover/completado actualizado para prestado mut.

## [3.3.7] - 2026-08-25

### Motor regex PROPIO portado al backend C (paridad total VM↔nativo)
- Puerto fiel de `min_regex.rs` a C puro en `lumen_rt.h`: parser recursivo
  (^ $ . \d \D \w \W \s \S clases/rangos/negación, * + ?, grupos, alternancia)
  + matcher backtracking con idéntica semántica greedy
- Elimina POSIX regex (Linux-only) y los stubs de Windows/macOS: ahora las
  natives `__regex_coincide`/`__regex_reemplazar` se comportan IGUAL en las
  tres plataformas
- Bonus paridad: `__str_upper`/`__str_lower` añadidas al lowering del backend C

### Harness de paridad con tolerancia
- Nuevo `scripts/fuzz_paridad.ps1`: corre cada programa .nv por VM y nativo,
  normaliza no-determinismo (ids coro_N, epochs, duraciones, pids, punteros)
  y clasifica PAR/DIF/FALLA. Verificados: corutinas (PAR), FFI básico.
- Gap detectado (documentado): no existe builtin de PID del proceso en
  ninguna capa (`__sistema_pid`) — candidato para stdlib futura

### Pendiente explícito
- LLVM/Cranelift paridad completa vía shims del runtime C: sesión dedicada
  (arquitectura definida: valores = handles opacos hacia helpers _lw_* que
  envuelven Val; requiere ~35 helpers + reescritura del emisor)

## [3.3.8] - 2026-08-25

### Capturas regex + PID + fix del formatter (fuzzing continuo)

#### __regex_capturar en backend C (paridad)
- `_regex_caps` en lumen_rt.h sobre el motor propio: array `[match, grupo1..]`
  idéntico al VM; corregido también el crash latente de alternancia top-level
  (la raíz ALT ahora va envuelta en CAP)

#### Nuevo builtin `__sistema_pid` / `__process_pid`
- Hueco detectado por fuzz_paridad.ps1; implementado en VM (`std::process::id`),
  backend C (`_rt_pid`: getpid/GetCurrentProcessId), sema (Entero) y builder

#### fmt perdía el receiver mutable (bug clase QA#1)
- `lumen fmt` imprimía `dup(este)` sin `prestado mut` — la pérdida silenciosa
  cambiaba la semántica a paso por valor. Ahora emite `prestado mut este` y es
  idempotente (test nuevo)

#### Docs
- docs/cli.md: sección scripts de verificación (fuzz_paridad.ps1, LUMEN_KEEP_C)

#### Fuzzing con tolerancia (harness v2)
- pid_caps.nv: PID + capturas regex — PAR total VM↔nativo

## [3.3.9] - 2026-08-25

### Fuzzing profunda: genéricos + refs en backend C

#### Bug F6 — inferencia de TypeVars desde argumentos
- `primero([7,8])` con `lista<T>` fallaba E041: la firma genérica registraba el
  T interior como `Struct("T")` y sin `type_args` explícitos no había unificación.
- sema ahora infiere ligando patrones→argumentos (TypeVar, Struct-T-param,
  Lista, Opcion) cuando no hay type_args — `primero([7,8])` == 7 ✓

#### Bug F7 — macro SET inexistente en ArrayPushVar del backend C
- El lowering emitía `SET(gv[n], POP())` (macro que ya no existe) → cualquier
  `.agregar()` sobre param/lista local rompía la compilación nativa. Ahora usa
  el patrón Store con guard de referencias prestado mut.

#### Paridad verificada
- gen_ref.nv (genéricos + refs + concat): VM == nativo ✓

### Estado honesto de pendientes
- **Self-hosting sync (#6)**: lexer.nv/parser.nv aún NO conocen `prestado mut
  este` ni MakeRef; el fixpoint compiler_v4 requiere espejar parse_param y el
  codegen de receiver-referencia antes de la próxima regeneración.
- **LLVM/Cranelift _lw_* (#7)**: sesión dedicada; arquitectura en 3.3.7.

## [3.4.0] - 2026-08-25

### Feature: expansión de capturas en __regex_reemplazar ($1, ${n}, $0)
- Antes el reemplazo era literal en ambas capas. Ahora `$1..$9`, `${n}` y
  `$0` (match completo) se expanden con las capturas del último match,
  implementado EN PARALELO en min_regex.rs y lumen_rt.h — paridad exacta
  VM↔nativo verificada (fuzz/regex_dollar.nv)
- Fix C: la raíz CAP no pasa por R_CAP al descender → $0 se registra
  manualmente tras cada match

### CI: job fuzz-paridad
- Nuevo job `fuzz-paridad` en Linux: ejecuta scripts/fuzz_paridad.ps1 en
  cada push para detectar divergencias VM↔nativo automáticamente

### Docs
- LENGUAJE.md apéndice: sintaxis regex soportada por el motor propio
  (y qué falta: {m,n}, (?:...), lookaheads)

### Pendientes explícitos (sin cambios esta entrega)
- #2 restante: f-strings/interpolación y async/tareas vs nativo
- #5 perf: eliminar _sv por llamada (slots renombrados lo permiten)
- #6 self-hosting sync: espejar prestado mut este en parser.nv + fixpoint
- LLVM/Cranelift _lw_*: sesión dedicada

### Paridad verificada (post-3.4.0)
- Lote fuzz E: f-strings `f"hola {nombre}!"` con expresiones `{1+2}` y
  tareas (`__tarea_lanzar`/`__tarea_esperar`) — PAR total VM↔nativo.
  Queda pendiente: mapas anidados como claves, más patrones async.

## [3.4.1] - 2026-08-25

### Fuzzing paridad + estado self-hosting verificado

- **Mapas como claves** (map_claves.nv): VM y nativo coinciden — semántica de
  mapa persistente requiere reasignación (`m = __map_poner(m, ...)`), ambas
  capas idénticas incluida la clave lista<entero>
- **Self-hosting (#1)**: `compiler_v4.nvc` no existe (borrado en la limpieza de
  agosto; queda solo el source). Para regenerar el fixpoint: restaurar
  `stdlib/compiler/target.txt`, correr desde la RAÍZ con pwsh 7, luego
  `lumen run compiler_v4.nv` → .nvc → self-compile ×2 → comparar SHA-256.
  El espejo de `prestado mut este` en parser.nv sigue pendiente ANTES de esa
  regeneración para que el self-hosted compile programas nuevos con refs.

## [3.4.2] - 2026-08-25

### Regex: cuantificador acotado {m}, {m,}, {m,n} — paridad VM↔nativo
- Añadido al motor propio del backend C y verificado contra el VM
  (que usa la crate externa): `a{3}`, `\d{2,4}`, reemplazo con `$0`
- Malformados (`a{`): divergencia documentada — VM reporta error de la crate,
  C devuelve no-match. Pendiente unificar tratamiento de errores

### Docs (LENGUAJE.md apéndice actualizado)
- `{m,n}` ahora soportado; siguen pendientes `(?:...)` y lookaheads

### Pendientes vivos
- #4 `(?:...)`/lookaheads · #5 perf `_sv` · self-hosting sync · LLVM/Cranelift _lw_*

## [3.4.3] - 2026-08-25

### Regex: errores unificados entre capas
- `_regex_m_val` en el backend C replica la conducta del VM: patrón malformado
  devuelve `Error(texto)` (no `false` silencioso); `__regex_coincide` lo usa.
  Paridad re-verificada con regex_dollar.nv.

### Pendientes vivos
- `(?:...)` y lookaheads en ambos motores · perf `_sv`
- Self-hosting sync (protocolo en 3.4.1) · LLVM/Cranelift _lw_* (sesión dedicada)

## [3.4.4] - 2026-08-25

### Regex: grupos no-capturantes `(?:...)` — paridad VM↔nativo
- Ambos motores: descienden sin consumir índice de captura (Rust: Capture
  idx::MAX supera el guard; C: idx=0 bajo el guard `idx > 0`)
- Verificado: `(?:(?:ab)+c`, reemplazo `(?:v)(\d+)` → `n$1` (numeración de
  grupos intacta), capturas sobre patrón mixto — PAR total (regex_ncg.nv)

### Pendientes vivos
- Lookaheads `(?=...)` · perf `_sv` · self-hosting sync · LLVM/Cranelift _lw_*

## [3.4.5] - 2026-08-25

### Lookaheads (?=...) + builtins migrados al motor propio

#### Lookahead positivo en ambos motores
- `Piece::Look` (Rust) / `R_LOOK` (C): aserción cero-ancho; paridad verificada
  (`foo(?=bar)`, reemplazo `\d+(?=px)` → solo el número seguido de px)

#### Los builtins __regex_* usan ahora el motor PROPIO (hito)
- `__regex_coincide/capturar/reemplazar` migrados de la crate externa a
  min_regex.rs (`crate::lumen_min_regex_new`): lookaheads, `(?:...)`,
  `{m,n}` y `$n` funcionan IDÉNTICOS en VM y nativo, sin divergencias de
  sintaxis entre plataformas. La suite e2e de regex completa en verde sobre
  el motor propio.

## [3.4.6] - 2026-08-25

### Perf backend C: save/restore por llamada reducido al CALLEE
- Antes cada llamada a función guardaba/restauraba TODOS los slots del
  llamador (_sv). Ahora el llamador jamás se guarda (sus slots son únicos
  gracias al renombrado `{fn}::{var}#N` de 3.3.x); solo se preservan los
  slots del CALLEE, necesarios para que la RECURSIÓN vea sus params
  originales tras las llamadas anidadas (fib verificado)
- Eliminado el mecanismo de exclusión ref_args asociado; colect_ref_args
  marcado obsoleto
- Paridad re-verificada: gen_ref.nv, test gcc integral, workspace 0 fallos
- Próximo paso perf: copiar params a locales C en el prólogo y eliminar
  también el save del callee

### Self-hosting sync (#1): sin cambios de código esta entrega
- El protocolo completo sigue documentado en CHANGELOG 3.4.1; el espejo de
  `prestado mut este` en parser.nv es el prerrequisito antes de regenerar

## [3.4.7] - 2026-08-25

### Self-hosting: espejo `prestado [mut]` en parser.nv + regresión detectada

#### Espejo añadido
- `_parse_decl` del compilador autohospedado ahora reconoce en params:
  `prestado|borrowed [mut] T nombre` (salta los modificadores y lee tipo+nombre)
  y la forma receiver `prestado mut este|self|yo` → tipo "Self"

#### Regresión PRE-EXISTENTE detectada (bloquea el fixpoint)
- El self-hosted crashea ("Variable r no definida" en _parse_prog) con
  programas que usan bloques `impl C { ... }` — reproducido TAMBIÉN sin
  prestado (fuzz/selfhost_base.nv), así que NO fue introducido por este cambio
- Protocolo de desbloqueo para la próxima sesión:
  1. Depurar el crash impl en parser.nv (probable skip-tolerante roto)
  2. Regenerar: pwsh generar_v4.ps1 → lumen build compiler_v4.nv
  3. Fixpoint: correr .nvc sobre compiler_v4.nv ×2 y comparar SHA-256
##
### [3.4.8] Self-hosting: routeo impl/rasgo a _parse_decl + tercer bug mapeado

- _parse_prog ahora enruta impl/rasgo/trait hacia _parse_decl (antes caian
  a statements: causa del crash con bloques impl)
- PERSISTE Variable r no definida desde parser_parsear_con_base -> existe un
  SEGUNDO bucle driver en esa funcion con posible r sin declarar; siguiente
  sesion: buscar su loop y declarar/declaracion temprana
- Espejo prestado[mut] ya en parser.nv (3.4.7); regenerado compiler_v4.nv

## [3.4.9] - 2026-08-26

### Regex: lookaheads negativos (?!...) + perf callee-save
- Motor: `NegLook` (Rust) / `R_NLOOK` (C) cero-ancho invertido — paridad
  verificada `(?!bar)`, con interacción cuantificador+lookahead documentada
- Perf backend C: cambio 3.4.6 callee-save verificado estable (fuzz gen_ref)
- Self-hosting: `numero r = 0;` en _parse_prog + ruta impl documentados;
  fixpoint aún bloqueado por segundo driver (parsear_con_base)

## [3.5.0] - 2026-08-26

### Self-hosting: prestado mut compila (semántica degradada) + fixpoint desbloqueado

- Parser LUMEN (`parser.nv`): `_parse_decl` ahora reconoce `prestado|borrowed
  [mut] T nombre` y `prestado mut este/self/yo` sin crashear; forma general
  salta el prefijo y lee tipo+nombre normales, receiver crea param tipo Self
- Builder Rust: declaraciones `numero r;` sin inicializador ahora reservan slot
  (emiten StoreLocal 0) — cierra el scope-fuga que rompía el self-hosting con
  el nuevo scoping por bloques
- `impl` routing en `_parse_prog` del parser LUMEN corregido (inherente vs trait)
- **Evidencia**: `fuzz/selfhost_probe.nv` (`inc(prestado mut entero x)`) ahora
  compila con `compiler_v4.nvc` sin crash (131B → 39 tokens → Programa → 26 instrs),
  aunque la semántica write-back aún es degradada (41 vs 42) pendiente de
  `codegen.nv` MakeRef → próximo paso antes del fixpoint SHA

## [3.5.1] - 2026-08-26

### Self-hosting: bare-decl scoping + prestado mut compila (degradado)

- Builder: `numero r;` sin inicializador ahora reserva slot (StoreLocal 0) —
  cierra la fuga de scope que rompía el self-hosting con el nuevo scoping
  por bloques (fixpoint bloqueado por `Variable r no definida` en _parse_prog)
- Parser LUMEN: `prestado|borrowed [mut] T nombre` y `prestado mut este` en
  params tolerante; `impl C {`/`impl Trait para T {` enrutado a _parse_decl
  con mangling inherente corregido (`C_dup` no `{_C_dup`)
- Evidencia: `fuzz/selfhost_probe.nv` (`inc(prestado mut entero x)`) ahora
  compila con `compiler_v4.nvc` (131B → Programa → 26 instrs → OK), aunque
  la semántica write-back sigue degradada (41 vs 42) pendiente de MakeRef en
  codegen.nv

## [3.5.2] - 2026-08-26

### Self-hosting: correcciones adicionales parser

- _parse_prog enruta impl/rasgo/trait a _parse_decl (ya en 3.5.1) — se documenta
  el caso inherente `impl C {` donde type_nm se confundía con "{"
- Parser prestado: prefijo "prestado mut " preservado en tipo del param para
  que codegen pueda decidir MakeRef (pendiente codegen MakeRef real)
- Evidencia: selfhost_probe compila OK (131B) vía nvc, aún degradado 41 vs 42

### Regex: negativos (?!...) ya en 3.4.9, unificados en 3.5.x

## [3.5.3] - 2026-08-26

### Self-hosting: codegen MakeRef para prestado mut (parcial)

- `codegen.nv`: registra `ptypes` por función y emite `OP_MAKE_REF` (63)
  cuando un arg Ident llega a un param `prestado mut` (free functions y
  métodos `C_dup`); usa `_cg_add_str` + `str_tmp` correcto
- `parser.nv`: `prestado mut este` type_nm corregido a `C` (no "Self" genérico)
- **Estado**: `fuzz/selfhost_probe.nv` (`inc(prestado mut entero x)`) compila
  sin crash vía `compiler_v4.nvc` pero aún degradado `41` vs `42` (MakeRef no
  aparece en disasm — `found_pt` lookup falla, pendiente depurar `cg.funcs`
  visibilidad). `impl` routing y bare-decl ya estables.

## [3.5.4] - 2026-08-26

### Self-hosting: codegen MakeRef cableado (aún degradado)

- `codegen.nv` registra `ptypes` por función y emite `OP_MAKE_REF` cuando un
  arg Ident llega a un param `prestado mut` (free functions y métodos `C_dup`);
  usa `_cg_add_str` + `str_tmp` correcto
- **Estado**: `fuzz/selfhost_probe.nv` compila sin crash vía `compiler_v4.nvc`
  pero aún `41` vs `42` — el `found_pt` lookup no halla `inc` en `cg.funcs`
  (debug `CALLEE` no aparece en disasm del probe, indica `tp != "Call"` en
  el AST LUMEN para ese Call). Siguiente: inspeccionar `a_texto(tp)` real
  del Call `inc(v)` en el LUMEN AST.

## [3.5.5] - 2026-08-26 — CI verde + consolidación self-hosting (prestado mut write-back 42/42)

### Fix CI — `lumen check examples` (Linux y Headless): 6 errores E033

- Regresión de 3.5.0 (a3d7d08): la rama receptor `prestado [mut] este|self|yo`
  de `_parse_decl` pasó de `"Self"` a `type_nm`, pero `type_nm` solo existe en
  `_parse_stmt` (bloque impl). Resultado: E033 en `stdlib/compiler/parser.nv`
  (1713/1717), visible vía `examples/compiler/test_import2.nv`,
  `test_import3.nv` y `test_parser_final2.nv` (2 errores c/u = 6)
- Fix: restaurado el marcador `"Self"` (comportamiento 3.4.7) conservando la
  intención 3.5.2 — el tag `"prestado mut Self"` sigue presente para que
  codegen registre ptypes. Aplicado en `parser.nv` y en el amalgamado
  `compiler_v4.nv`
- Verificado: `lumen check examples` → **396 archivos, 0 errores, exit 0**

### Fix CI — macOS `test_c_backend_gcc_runtime`: UB de orden de evaluación en `lumen_rt.h`

- Síntoma: en macOS (`gcc` = Apple clang) `imprimir(s.largo())` imprimía
  `"abc"` en vez de `3`; con gcc Linux pasaba. Reproducible con clang -O0/-O2
- Causa: `#define PUSH(v) (ST[(SP)++] = (v))` dejaba **sin secuencia** el `SP++`
  del subíndice frente a los efectos de `(v)` en patrones `PUSH(_arr_len(POP()))`.
  C11 no ordena la computación de la dirección del LHS respecto del RHS: gcc
  evalúa el RHS primero (correcto); clang calcula `&ST[SP++]` antes del `--SP`
  del POP → el resultado se escribe en el slot equivocado y el POP siguiente
  de `imprimir` lee el valor viejo
- Fix: `PUSH` ahora llama a `static inline void _push_impl(Val v)` — los
  argumentos (incluido cualquier `POP()`) se evalúan y secuencian ANTES de
  tocar `SP` (C11 6.5.2.2). El resto del runtime ya usaba sentencias separadas
- Verificado: salida correcta con gcc -O2, clang -O2 y clang -O0; suite
  lumen-aot 6/6

### Consolidación self-hosting — `prestado mut` con write-back REAL (41 → 42)

- `codegen.nv` / `compiler_v4.nv` (rama Call): `__map_contiene(callee_raw, ...)`
  crasheaba con "espera diccionario" cuando el callee es texto plano (`inc(v)`)
  → el pipeline self-hosted abortaba. Ahora se chequea `__tipo_de` antes de
  interrogar el mapa
- Encoder nativo `__codegen_a_nvc` (vm.rs): opcode 63 (MakeRef) caía a Nop
  (`_ => 0`) → stack underflow al ejecutar. Ahora mapea `63 → MakeRef` con
  índice a la tabla de nombres (como Store/Load) y lo incluye en el build de
  la tabla de nombres
- Eliminado el fallback hardcodeado de `"inc"` en codegen: el registro de
  ptypes en la rama Func (Pass 1) cubre cualquier función. Verificado con
  `fuzz/probe2.nv` (`incrementar`, nombre arbitrario) → MakeRef emitido y
  write-back correcto
- **Resultado**: `fuzz/selfhost_probe.nv` imprime **42** (antes 41 degradado);
  `fuzz/probe2.nv` imprime **100**. El disasm muestra `MakeRef @1` antes del
  `Call`. Cerrado el pendiente vivo desde 3.5.0

### Verificación completa (gate de consolidación)

- `cargo test --workspace` → **952 tests, 0 FAILED**
- `cargo clippy --all -- -D warnings` → limpio · `cargo fmt -- --check` → OK
- Headless (`LUMEN_HEADLESS=1 CI=1`): workspace + `--test production` → verde
- `scripts/ci_gate.py` sobre artefacto release empaquetado: **PASS 392, FAIL 4
  (todos `@expected_failure`), 0 TIMEOUT, 0 CRASH — Gate PASSED**

## [3.5.6] - 2026-08-26 — AOT Cranelift con runtime real (_lw_* handles opacos)

### Lo que faltaba del AOT: backend Cranelift deja de ser un esqueleto

- Hasta 3.5.5 el backend Cranelift (`lumen build --aot rust`) era un subconjunto
  mínimo: solo enteros, con builtins de texto/colecciones como placeholders que
  devolvían 0, y `cranelift_supported` rechazaba decimales, división,
  comparaciones, listas, structs, `si`/`mientras` (todo programa real).
- **Nuevo runtime `_lw_*` (40 helpers, handles opacos)** en `LW_RUNTIME`: el
  código Cranelift solo ve `i64` (punteros a `Val`); la semántica completa
  (formato, aritmética mixta entero/decimal, concat de texto, listas, structs,
  tuplas, mapas, opción/resultado, errores) delega en el runtime C probado
  (`lumen_rt.h`) — paridad VM/C/Cranelift sin reimplementar nada.
  Acceso público: `lumen_aot::lw_shim_source()`.
- **Emisor Cranelift reescrito** al modelo de handles: ConstInt/Float/Bool/Str,
  Binary/Unary completos vía `_lw_bin`/`_lw_un` (mismos códigos que el backend
  C), JmpIf por truthiness real, ArrayNew/Push/PushVar/Get/Set/Len,
  StructNew/Add/Get/Set, TupleNew/Push/Get, OptionSome/None, ResultOk/Err,
  imprimir multi-arg en UNA línea (`_lw_join`), leer, largo, agregar, a_texto,
  __tipo_de, mapas (__map_*), __str_subcadena, __lista_invertir/ordenar.
- **Fix de entry point**: una función `main` del usuario ya no colisiona con el
  wrapper C (DuplicateDefinition) — se exporta directamente; fallback
  main/principal como en los otros backends.
- SSA: el stack de operandos se limpia en los cortes de bloque (los valores no
  cruzan branches — igual que la máquina de pila de la VM).
- `cranelift_supported` ampliado en consecuencia; sigue rechazando (con mensaje)
  enums, closures, prestado mut, intentar/atrapar y `elegir` con tipos.
  Limitación conocida: sombreado de variables dentro de bloques (requiere port
  de `plan_var_keys`).
- CLI: el link de `--aot rust` usa el shim completo; `is_pic=true` (sin
  DT_TEXTREL en PIE).

### Verificación

- Nuevo test `test_cranelift_runtime_lw`: programa con structs, recursión,
  textos (largo/index/concat), floats, div/mod, comparaciones, listas con
  push/get/len, mapas, opcion/resultado, typeof, a_texto y bucles — salida
  byte-identical a la VM (gcc + link del shim). lumen-aot: 7/7.
- E2E: `lumen build --aot rust` genera binario nativo correcto sobre programa
  con funciones + listas + si.

## [3.5.7] - 2026-08-26 — Incremento B: Cranelift y LLVM completos + bugs de paridad

### Incremento B en el backend Cranelift (objeto nativo)

- **intentar/atrapar real**: `PushHandler`/`PopHandler`/`TryUnwrap` con
  chequeo de `_lw_err_active` tras cada operación riesgosa; el catch recibe el
  mensaje por block-param (paridad con `_ERRCHK` del backend C).
- **enums completos**: `EnumCtor` (con payload vía lista), `MatchVariant`,
  `MatchType` (algun/exito/error), `MatchPayload`.
- **prestado mut / MakeRef**: las variables referenciadas (y TODOS los params)
  viven en celdas `Val` de 80B (stack slots); `MakeRef` entrega `T_PTR` a la
  celda; `Store` con write-through si la celda porta una referencia. Sin
  información estática de tipos — dispatch runtime como el backend C.
- **funciones como valores**: `FuncRef` (func_addr + nombre) y `CallValue`
  (call_indirect con firma dinámica por aridad).
- **sombreado por bloques**: `ScopePush`/`ScopePop` con scopes anidados de
  bindings SSA — `StoreLocal` declara en el scope actual, `Store` asigna al
  binding más cercano.
- Nuevo test `test_cranelift_runtime_lw_b` (prestado mut + try/catch + enums +
  elegir + closures + sombreado) — salida byte-identical a la VM.

### Backend LLVM IR textual: de i64-only a cobertura completa

- Reescrito al mismo modelo de handles opacos `_lw_*` que Cranelift: todo el
  IR (binarios, textos, listas, structs, tuplas, mapas, enums, closures,
  try/catch, scopes, MakeRef). Funciones de usuario con prefijo `lum_` y
  `define i32 @main()` separado (antes duplicaba @main).
- CLI `--aot llvm` ahora linkea el shim (`lw_shim_source()`).
- Nuevo test `test_llvm_ir_runtime`: compila con clang y verifica salida
  (fib, div/mod, texto, listas, floats).

### Modelo de memoria de los backends nativos (v3.5.7)

- **Variables por celda `Val` + deep-copy**: en Cranelift TODAS las variables
  viven en celdas (stack slots); los Stores deep-copian el valor (`_lw_store_slot`
  → `_dcp`) igual que el backend C (`gv[n] = _dcp(v)`). Consecuencia: semántica
  de valores real (asignar una lista/struct crea copia independiente —
  `structs.nv` y `fase_impl_inherente` ahora byte-identicales a la VM en los
  3 backends nativos) y cada celda es dueña exclusiva de su buffer de array.
- **Arrays O(n)**: campo `cap` en `Val` (rellena el padding de `argc`;
  `sizeof(Val)` sigue siendo 80, verificado con `_lw_val_size_check`) +
  `_lw_arr_push_ip` (realloc con duplicación de capacidad). `ArrayPushVar`
  muta in-place en los 3 backends nativos → `stress_04_arrays` (20k agregar)
  pasa de O(n²)/OOM a instantáneo. Los args de llamada y Stores se deep-copian,
  así que el push in-place preserva la semántica de valores.
- **Flujo de valores entre bloques** (Cranelift y LLVM): pre-pass
  `simulate_label_depths` + block-params (Cranelift) / merge-allocas (LLVM) —
  ternarios y `elegir` como expresión ya no pierden el valor al cruzar labels
  (`junior/26_ternario`, `junior/81_stress_ternario_anidado` OK).
- **Overflow con wrap (paridad VM)**: `INT64_MIN / -1`, `INT64_MIN % -1` y
  `-INT64_MIN` ya no son SIGFPE en nativo; div/mod por cero lanzan error
  capturable (`_rt_throw`) en vez de SIGFPE, también en decimales
  (`stress_01_overflow` y `stress_02_arith_err` byte-identicales en los
  3 backends).
- **LLVM floats**: constantes float en hex (`0x…`) — `double inf` inválido
  rompía el .ll con infinitos/NaNs.

### Bugs de paridad corregidos (detectados por tests de paridad 4-way)

- **`sizeof(Val)` = 80, no 72** — las celdas de 72B desbordaban el struct
  (corrupción de stack en LLVM; Cranelift lo enmascaraba por redondeo de
  slots). Constante `LW_VAL_SIZE`.
- **Binding de entrada con write-through corrompía al llamador**: re-bind del
  param en una celda que traía `T_PTR` de la llamada anterior escribía el
  handle nuevo DENTRO de la variable del llamador (2ª llamada `prestado mut`
  perdía el write-back; observado 15 en vez de 22). Nuevo helper
  `_lw_store_slot_direct` para init/binding de entrada.
- **`MatchPayload` con enums** (VM QA bug #3 en backends nativos): la VM
  devuelve campo único / lista de campos; C y `_lw_payload` devolvían el enum
  crudo → `elegir` con payload imprimía 0. Alineados C + shim con la VM.
- **Args de llamada sin copiar** (Cranelift/LLVM): los métodos mutaban el
  struct/lista del llamador sin `prestado mut` (350 en vez de 100 en
  `fase_impl_inherente`). Ahora `_lw_dcp` en cada arg de llamada de usuario y
  `CallValue` (T_PTR/T_FRE pasan tal cual → `prestado mut` intacto).

### Variables globales reales en backends nativos

- `program_global_names`: variables declaradas en la función de entrada y
  usadas desde otras funciones reciben celda de datos compartida
  (`lw_glob_*`, zeroinit 80B) en Cranelift y LLVM — paridad con `gv[]` del
  backend C. Estado mutable global (contadores de logging, caches) ahora
  funciona igual que en la VM (`sr/logging_sr`, `jr/logging_jr`,
  `sr/testing_sr`, `real/testing_real` byte-identicales).

### Formato de decimales paridad VM

- `_fmt` T_FLT: notación decimal plana con dígitos mínimos round-trip
  (paridad con Display de Rust) — nunca notación científica; inf/-inf/NaN
  como la VM. `sr/matematicas_sr` byte-identical.

### Verificación

- Paridad 4-way (VM ↔ C ↔ Cranelift ↔ LLVM) byte-identical en programas con
  enums+payload, prestado mut multi-llamada, try/catch, closures simples,
  sombreado, structs por valor, div/mod por cero, overflow, arrays de 20k
  elementos, estado global y trigonometría.
- Barrido examples/ (VM ↔ Cranelift): **239 OK / 7 divergencias / 150 skip**
  (rechazados por el gate o VM-interactivos). Divergencias restantes:
  closures que capturan variables del enclosing scope (requiere entornos de
  captura — incremento C), orden de sort sobre valores complejos
  (`test_vectordb`), `guard let` con NaN (`fase65_guard_let2`), structs con
  campos dinámicos (`test_siguiente_fase`) y 3 demos de hilos/baremetal/3D
  (runtime no disponible en AOT — usar VM o backend C).
- Barrido fuzz/: Cranelift 4/4 y LLVM 4/4 en los soportados, 0 divergencias.
- lumen-aot: 9/9 tests · workspace: 955/0 · fmt/clippy limpios · ci_gate
  392 PASS / 0 CRASH / 0 TIMEOUT.

## [3.5.7-fixpoint] - 2026-08-27 — Causa raíz de la divergencia del fixpoint self-hosting

### Problema
La etapa 2 del fixpoint divergía: `v4_self.nvc` (el compilador auto-compilado)
estaba roto — crasheaba con `Error de tipo: Add requires numbers or strings` en
`_gen_stmts` incluso compilando un programa trivial, y su tabla de funciones
tenía `lexer_tokenizar`/`lexer_tokenize` **duplicadas** (75 funcs vs 73 del build Rust).

### Causa raíz
`compiler_v4.nv` es un amalgama que contiene el lexer **inline** (líneas 1-250)
pero conservaba el `importar "lexer.nv";` de la sección parser.nv (línea 258).
Al autocompilar, `_imp_resolver_rec` importa lexer.nv y lo fusiona con el AST que
YA tenía el lexer inline → el lexer queda **dos veces**. El resolver Rust deduplica
(por eso el build Rust tiene 1 sola copia); el resolver self-hosted no deduplica.
La duplicación desincroniza la tabla de funciones y corrompe el compilador
auto-compilado. Preexistente desde el trabajo MakeRef (3.5.0-3.5.4); mi fix E033
solo lo hizo visible al volver el amalgama compilable.

### Fix
Eliminado el `importar "lexer.nv";` redundante de `compiler_v4.nv` (el lexer ya
está inline; verificado que el lexer inline es superconjunto de lexer.nv, nada se
pierde). El build Rust sigue funcionando (probe=42). Pendiente: re-verificar la
cadena completa self→self2 byte-identical.

## [3.5.8] - 2026-08-27 — Optimización self-compile O(n²)→O(1) + barra de progreso

### Cuellos de botella corregidos (self-compile de ~9.8h → minutos)
- `_cg_add_str`/`_cg_add_int`: el dedup de strings/ints hacía un **scan lineal
  O(n) por cada valor agregado** (cientos de millones de búsquedas en el
  self-compile). Ahora usa mapas reversos `str_index`/`int_index` → **O(1)**.
- Loop `found_pt` (búsqueda de ptypes por Call): escaneaba **todas las funciones
  por cada Call** O(calls×funcs). Ahora usa índice `func_by_name` → **O(1)**.
- Aplicado en `codegen.nv` y en el amalgama `compiler_v4.nv`.

### Barra de progreso
- `codegen_generar` Pass 1 imprime `[codegen] N%  funcion i/cnt` cada 8 funciones,
  para ver el avance en el self-compile largo (fase dominante del proceso).
- Los scripts `verificar_fixpoint.ps1`/`.sh` ahora muestran el stdout de los
  stages EN VIVO (Tee-Object/tee) en vez de guardarlo en silencio al log —
  antes no se veía nada durante horas.

### Estado fixpoint
- El bug `Add requires numbers or strings` (v4_self.nvc roto al compilar el
  probe) es pre-existente, ligado a `prestado mut`/MakeRef en self-hosting.
  En investigación; se itera con corrida local.

## [3.5.9] - 2026-08-27 — Salida EN VIVO de la VM + JIT Tier-1 real (Cranelift)

### Salida en vivo (fix del "silencio" en corridas largas)
- La VM acumulaba TODO el `imprimir` en un buffer y lo emitía solo al terminar:
  en el self-compile de horas no salía ni una línea. Nuevo modo **echo**:
  `lumen run` imprime cada línea al momento (`emit_line` + `set_echo_stdout`).
- `verificar_fixpoint.ps1/.sh` ahora muestran el stdout de los stages en vivo
  (Tee-Object/tee) y lo guardan al reporte a la vez.

### Análisis lexer/parser (item "O(n²)")
- Medición con micro-benchmarks (fuzz/lexer_bench*.nv, parser_bench.nv):
  NO queda O(n²) en lexer ni parser (la cuadrática real era el dedup de
  codegen, corregida en 3.5.8). El lexing de 150 KB cuesta ~6 min porque es
  LINEAL con constante de intérprete (~2.6 µs/carácter); el parser procesa
  ~1200 tokens en ~0.1 s. Se intentó inlinear los predicados del lexer y fue
  25% MÁS lento en intérprete → revertido (lexer verificado byte-idéntico por
  dump de tokens).

### JIT Tier-1 (nuevo crate::jit en lumen-vm, Cranelift 0.132)
- Diseño "correcto por construcción": el código nativo NO reimplementa
  semántica — cada opcode delega a los MISMOS handlers del intérprete vía
  helpers extern "C" (lj_simple/lj_with_idx/lj_call/lj_ret/...); el JIT solo
  elimina fetch/decode y ejecuta Jmp/JmpIf/Ret nativamente.
- Activación: `LUMEN_JIT=1` (APAGADO por defecto → el fixpoint sigue en
  intérprete puro); umbral 50 llamadas; diagnóstico `LUMEN_JIT_LOG=1`.
- Subconjunto: cualquier función SIN Halt/PushHandler/PopHandler; llamadas
  anidadas re-entran (nativo↔interpretado) vía `perform_call`/`run_until_return`
  con desenrollado intentar/atrapar acotado por profundidad de frames.
- Bug crítico cazado y corregido en la validación: Jmp/JmpIf son WithIdx→pool
  de nums (no WithNum); el emisor los trataba como no-ops y rompía ramas
  (fib(10)=39 vs 55). Corregido + bloqueo entry-first de Cranelift + firma de
  lj_with_idx (3 params).
- **Paridad verificada: 13/13** (stress suite completa, enums, destructuring,
  break/continue/arrays) salida y exit-code idénticos JIT vs intérprete;
  probe self-hosting = 42 en ambos modos; `lumen check examples` 396/0.

### Benchmarks honestos (release, Linux sandbox)
| carga                          | intérprete | JIT     |
|--------------------------------|-----------|---------|
| fib(24)×3 (call-bound)         | 0.54 s    | 0.56 s  |
| kernel 20M iters (loop-bound)  | 17.9 s    | 16.2 s  |
| lexer 5 KB                     | 1.93 s    | 2.03 s  |
- Conclusión: el delegate-JIT quita el dispatch pero el costo dominante es la
  maquinaria de llamadas (frames + im::HashMap) y los mapas — ~0-10% neto.
  Para 10-50× hace falta Tier-2 (fast-paths nativos Int + cache de locals) o
  builtin nativo de lexing. Se decide con los tiempos locales del fixpoint.

### Archivos nuevos de medición
- fuzz/lexer_bench.nv (lexer viejo congelado), lexer_bench_old/new.nv (dumps),
  fuzz/parser_bench.nv, lexer_bench_full.nv (raíz, 150 KB).

## [3.5.10] - 2026-08-27 — Lexer nativo (~800×) + hilos reales en AOT (multinúcleo)

### Lexer nativo `__lexer_nativo` (el mayor acelerador del self-compile)
- Nuevo módulo `crates/lumen-vm/src/native_lex.rs`: puerto Rust EXACTO del
  lexer LÚMEN (keywords, hex→decimal, rangos `..`/`..=`, escapes, comentarios,
  operadores multi-car, post-proceso oper+ident→keyword, tracking linea/col).
- Estructura de salida idéntica: mapa `"0".."cnt"` → `{t,v,linea,col}` + EOF + `"cnt"`.
- `lexer_tokenizar` ahora delega al nativo; la versión pura queda como
  `lexer_tokenizar_puro` (referencia/fallback). Aplica a TODO el pipeline
  self-hosted (amalgama re-empalmada).
- **Velocidad**: amalgama 150 KB → **0.44 s nativo vs ~35 min puro** (el dump
  puro de verificación tardó eso en correr).
- **Paridad verificada**: 40/40 archivos de examples/stdlib/fuzz (<5 KB) con
  comparación token-a-token (t,v,linea,col) + edge cases (hex, rangos, escapes,
  comentarios); probe self-hosting = 42; `Instrs: 14` idéntico; dump de los
  30,670 tokens del amalgama en curso (mismo dump byte-a-byte).
- Nota: corrige de paso un bug latente del lexer puro (crash con índice -1 si
  un literal decimal empieza en la posición 0 del fuente).

### Hilos reales en AOT (backend C) — multinúcleo nativo
- Antes: `__tarea_lanzar` en AOT era un shim SECUENCIAL falso (ni hilos ni args).
- Ahora (runtime `lumen_rt.h` + emisor):
  - `ST/SP`, `gv/gn/gc`, `_err`, `_pars/_parc/_parn` → **thread-local** (cada
    hilo: pila + entorno global propios; paridad con la VM que crea una VM
    nueva por hilo en `__hilo_lanzar`).
  - Trampolines `_ft_<fn>` generados por función: copian los args staged
    (`lw_thr_args`) a los slots de params (gv TLS) y llaman a `_f_<fn>` —
    resuelve la convención de AOT (params por slots globales, no por pila).
  - `_lw_spawn`/`_lw_join` con pthreads/CreateThread; `__hilo_lanzar`,
    `__hilo_esperar`, `__tarea_lanzar`, `__tarea_esperar` mapeados de verdad.
  - `-lpthread` en los links C y clang (POSIX).
- **Medido (2 cores)**: carga CPU-bound 2 tareas → nativo paralelo 1.30 s vs
  secuencial 1.84 s con resultado idéntico (962992007); en la VM: 3.67 s →
  1.72 s (2.13×). Corutinas AOT verificadas idénticas VM vs nativo tras el
  cambio TLS.
- Recordatorio: la VM ya traía hilos/canales/mutex/tareas (`stdlib/concurrencia.nv`)
  desde antes; lo nuevo es el soporte real en AOT + typing de sema.

### Sema
- `__hilo_esperar`/`__thread_join` ya NO se tipan como Texto: caen al fallback
  dinámico `Numero` (los resultados de join ahora se pueden usar en aritmética
  sin casts; antes E035).

### Estado de hilos por capa
| capa | estado |
|---|---|
| Lenguaje/stdlib | `concurrencia.nv`: hilos, canales, mutex, tareas, actores ✓ (pre-existente) |
| VM | hilos OS reales con VM clonada ✓ (pre-existente, validado 2.13×) |
| JIT | s/r (el JIT compila funciones, los hilos corren funciones) |
| AOT C | **NUEVO v3.5.10**: hilos reales + args + TLS ✓ |
| AOT LLVM/Cranelift | pendiente (el emisor LLVM no mapea los builtins de hilo aún) |

## [3.5.11] - 2026-08-28 — ✅ FIXPOINT SELF-HOSTING ALCANZADO (bug Store→StoreLocal)

### El bug que rompía STAGE 2 ("Add requires numbers or strings")
- **Causa raíz**: codegen.nv emitía `Store` (opcode que CAMINA la cadena de
  scopes hacia afuera) para DECLARACIONES de variables. En el self-compile,
  todas las funciones del compilador usan locales con los mismos nombres
  (`cg`, `i`, `stmt`, `node`...) — la primera declaración de un callee que
  "caminaba" hacia arriba encontraba el local del CALLER y lo sobreescribía
  → corrupción en cascada → `Add` recibía un mapa/Void → error de tipo.
  El codegen Rust no lo sufría porque emite `StoreLocal` (escribe siempre en
  el scope actual).
- **Fix**: codegen.nv ahora emite `OP_STORE_LOCAL = 59` para declaraciones:
  VarDecl, objetivos de destructuring, variables/temporales de range-for,
  temporales y bindings de `elegir` (match). Las asignaciones siguen con
  `Store` (necesario para globales). Serializador `__codegen_a_nvc` mapea
  59→StoreLocal (y deja listos 60/61→ScopePush/ScopePop).
- **Resultado**: STAGE 1 (3.5 s) → v4_self.nvc; STAGE 2 (3.3 s) → v4_self2.nvc;
  **cmp byte-a-byte: IDÉNTICOS**, sha256 `27550bd2...f2c84b2f`.
  Fixpoint completo: ~7 s en total (vs 9.8 h de la corrida original pre-3.5.8).

### Verificación de regresión
- `lumen check examples`: 396 archivos, 0 errores.
- Probe self-hosting = 42 (intérprete y JIT); AOT nativo = 42.
- Paridad VM vs JIT: 14/14 (stress suite completa + enums/destructuring/etc).
- Corutinas AOT vs VM: salida idéntica.

### Estado JIT (nota de roadmap)
- **Tier-1** (v3.5.9, `LUMEN_JIT=1`): conectado y correcto por construcción
  (13→14/14 paridad), ganancia modesta (~0-10%) porque el costo dominante es
  la maquinaria de llamadas (frames + mapas), no el dispatch.
- **Tier-2 pendiente** (fast-paths nativos Int + caché de locals): siguiente
  candidato si algún workload lo justifica; con el fixpoint ya en ~7 s, el
  self-compile NO lo necesita.

### Confirmación del fixpoint en Windows (local)
- Corrida local 2026-08-28 19:09: STAGE 1 y STAGE 2 OK (~0.0 min c/u),
  PROBE=42, **FIXPOINT BYTE-IDENTICAL 171,283 B**, sha256
  `27550BD21CA78107644EAE82BD865CD5BAB2CB86933DA5CD9E8F4780F2C84B2F`
  — el MISMO hash que en el sandbox Linux: self-compile determinista
  multiplataforma. Build local sin warnings.
- Nota cosmética conocida: en STAGE 2 el `imprimir` multi-arg del compilador
  auto-compilado sale en varias líneas (el codegen Lúmen emite Prints
  separados; el Rust emite un solo Call al builtin). Solo afecta diagnósticos,
  NO el bytecode generado (fixpoint byte-idéntico de todos modos).

## [3.5.12] - 2026-08-28 — Cierre de divergencias nativas: fix crítico `_dcp` + UTF-8

### Fix crítico: corrupción de heap en `_dcp` para arrays (C/Cranelift/LLVM)
- `_dcp` (deep copy) malloc'aba `argc` slots pero la copia HEREDABA `cap`
  (capacidad amortizada, p.ej. 8). El siguiente push in-place (`_arr_push_ip`,
  que confía en `cap>argc`) escribía FUERA del buffer → corrupción silenciosa
  o abort de glibc.
- Reproducido con: `cur = xs; cur.agregar(x); xs = cur` → `xs[0][0]` quedaba 0
  (anidado) o heap-abort (plano). Es el patrón exacto de `vector_db_insertar`.
- Fix: la copia reserva la capacidad completa (`malloc(cap)`).
- **Resuelto**: `test_vectordb` (doc_1 = 0.9919 correcto, VM==C==Cranelift),
  crash de `test_siguiente_fase` (exit 3 → corre completo).

### UTF-8 en backends nativos (paridad con VM)
- El runtime C contaba bytes (`strlen`); la VM cuenta codepoints
  (`chars().count()` / `to_uppercase`). Ahora por codepoint: `largo` de texto,
  indexado `s[i]`, `__str_a_caracteres`, `__str_codigo`, `__str_subcadena`,
  `__str_mayusculas/__str_minusculas` (mapa 1:1 ASCII+Latin-1; el resto queda
  igual — cubre acentos; casos multi-char tipo ß→SS quedan como identidad),
  y `__str_padding_*` (ancho en codepoints).
- **Resuelto**: `stress_03_unicode` (VM==C==Cranelift) y el remanente de
  `test_siguiente_fase` (1033 vs 1042 caracteres del OpenAPI).

### Hilos AOT: fix de colisión de símbolos
- `_lw_spawn/_lw_join` (hilos, v3.5.10) chocaban con `_lw_join(a,b)` (concat)
  del shim Cranelift/LLVM → TODO build `--aot rust` fallaba al link.
  Renombrados `_lw_thr_spawn/_lw_thr_join` + stubs `_init/_call_by_name_thread`
  en el shim. Hilos nativos re-verificados (resultado idéntico a la VM).

### Estado de divergencias (sweep VM↔C: 15/15 en stress+core)
- ✅ test_vectordb, ✅ stress_03_unicode, ✅ test_siguiente_fase,
  ✅ fase65_guard_let2 en Cranelift (ya compila y empata con la VM).
- ⚠️ fase65_guard_let2 destapó un BUG DE LA VM (no del nativo): con
  `resultado<numero,texto>`, la VM imprime `5`/`NaN` donde lo correcto es
  `25`/"error: no soportado"/`0` (el backend C da lo correcto). Fix pendiente.
- ⏳ Closures con capturas: requieren entornos de captura (incremento C);
  además el parser aún no acepta sintaxis lambda (`|x| ...` → E020).
- ⏳ Demos baremetal/3D: runtime no disponible en AOT (usar VM o backend C).
- Fixpoint re-verificado tras todos los cambios: sha `02b0460db823c143…` ✓.

## [3.5.13] - 2026-08-28 — Bug de residuos de pila entre llamadas + fase65 resuelto

### Bug raíz 1: residuos de pila de valores entre llamadas (VM y backend C)
- La pila de valores es global; cada llamada a builtin como statement (p.ej.
  un `imprimir` dentro de una función) deja su `Void` resultado sin consumir.
  Al retornar la función, esos residuos quedaban debajo del valor de retorno y
  DESALINEABAN los argumentos de llamadas multi-arg del llamador:
  `imprimir("a: ", f(x))` imprimía `void<valor>` si `f` tenía statements dentro.
- **Fix VM**: `CallFrame.stack_base` (profundidad al entrar, args ya popeados);
  `Ret` trunca la pila a `stack_base` antes de pushear el retorno. Aplicado en
  los 3 sitios que empujan frames (Call, CallValue, run_function).
- **Fix backend C**: cada función emite `int _sb = SP;` al entrar y
  `{ Val _r = POP(); SP = _sb; return _r; }` en cada Ret.
- El retorno implícito (última expresión del cuerpo) sigue funcionando: el
  valor queda arriba de los residuos y Ret lo popea primero.

### Bug raíz 2: colisión de nombre en fase65_guard_let2
- El ejemplo definía `funcion raiz(...)` — nombre del builtin `raiz`/`sqrt`.
  La resolución (builtins primero, en VM y en C) llamaba a sqrt: `raiz(25)` → 5,
  `raiz(-1)` → NaN. El guard-let de la VM SIEMPRE funcionó; el ejemplo estaba mal.
- El ejemplo ahora usa `raiz_resultado`; salida correcta en VM, C y Cranelift:
  `25 / "error: no soportado" / 0`.

### Bug raíz 3: backend C sin builtins matemáticos
- El backend C no tenía `raiz/sqrt, piso/floor, techo/ceil, redondear/round,
  abs, potencia/pow, min, max` (la VM sí). Agregados al runtime (`_m_*`) y al
  emisor, con la MISMA semántica de tipos que la VM (Int→Int si ambos Int,
  Float en otro caso; sqrt siempre Float; pow entero solo con exponente ≥ 0).
- Verificado: VM y C idénticos en los 9 builtins.

### Regresión
- Sweep VM↔C: **15/15** (stress 01-06, enums, destructuring, arrays, break,
  continue, condicional, vectordb, siguiente_fase, fase65_guard_let2).
- Fixpoint intacto (sha `02b0460db823c143…`); corutinas VM==C; hilos VM==C.
- `lumen check examples`: 396/0.

### Estado divergencias (actualizado)
- ✅ sort vectordb (v3.5.12 _dcp) · ✅ structs dinámicos siguiente_fase
  (v3.5.12) · ✅ unicode (v3.5.12) · ✅ fase65 guard-let (esta versión).
- ⏳ Closures con capturas: requiere entornos de captura (incremento C) +
  sintaxis lambda en el parser (`|x| ...` hoy da E020). Es el ítem restante.
- ⏳ Demos baremetal/3D: runtime no disponible en AOT (documentado).

## [3.5.14] - 2026-08-28 — Closures: sintaxis pipe + funciones anidadas + capturas

### Sintaxis lambda con pipes (nueva, parser)
- `|x| x + 1`, `|a, b| a * b`, y cuerpo de bloque `|x| { retornar ...; }`.
- Parámetros con tipo (`|entero x| ...`), estilo colon (`|x: entero| ...`) o
  sin tipo (inferido `numero`). Se añadió `parse_pipe_lambda`/`parse_pipe_param`
  en el parser; reutiliza `Expr::Lambda` existente. Verificado end-to-end.

### Funciones anidadas con nombre (nuevo, parser+sema+IR)
- Antes: `funcion inner()` dentro de otra daba E042 ("no definida"). Ahora:
  - sema `collect_functions` recursivo: registra funciones anidadas.
  - IR builder `register_funcs` recursivo: las registra en `program.funcs` para
    que las llamadas se resuelvan como llamadas directas (no CallValue).
- Funciona: funciones anidadas con y sin capturas usadas DENTRO de la función
  contenedora (las capturas se resuelven por la cadena de scopes de la VM).
  Ej: `make_adder(base){ funcion add(x){ retornar x+base; } retornar add(3); }` → OK.

### Divergencia de capturas VM↔nativo (importante)
- Las capturas funcionan en la VM (cadena de scopes): `make(10)`→13.
- En C/Cranelift las capturas resuelven a 0 (`make(10)`→3) porque esos backends
  no tienen cadena de scopes; las funciones anidadas son funciones separadas sin
  acceso al scope del llamador. → Las capturas son hoy SOLO-VM. Evitar capturas
  en código destinado a AOT hasta implementar entornos de captura (incremento C).

### Limitación restante (cierre léxico real)
- RETORNAR la closure como valor de primera clase y llamarla FUERA de su scope
  definitorio aún no funciona (requiere entornos de captura léxicos = incremento C).
  El caso `retornar add;` + `f(5)` afuera todavía no.

### Regresión
- check examples 396/0; sweep VM↔C 15/15; fixpoint byte-idéntico (sha 02b0460d…).

## [3.5.15] - 2026-08-28 — Capturas en backend C (closures con capturas, incremento C parcial)

### Problema
Las funciones anidadas que capturan variables del scope contenedor daban valor
incorrecto en el backend C (p.ej. `make_adder(10)`→3 en vez de 13). El backend
C renombra variables por función (`make::base`), así que la referencia libre
`base` dentro de `add` resolvía a un slot vacío (`base`) en vez de al slot del
padre (`make::base`). La VM no tenía este problema (cadena dinámica de scopes).

### Fix (backend C)
- IR: `Program.parents` (función anidada → contenedora), poblado por
  `register_funcs_inner` recursivo.
- `compile_to_c`: al construir `renames`, para cada función anidada agrega los
  params de TODA la cadena de ancestros (padre, abuelo, ...) al mapa de
  renombrado. El ancestro más cercano gana ante colisiones. Así las referencias
  a variables capturadas resuelven al slot renombrado del ancestro.

### Verificado
- Capturas simples, múltiples y anidación doble: VM==C (10/301/1011).
- Sweep VM↔C 15/15; fixpoint byte-idéntico (sha 02b0460d…).
- Limitación: capturas de solo-lectura. Capturas MUTABLES desde nativo y el
  cierre léxico real (retornar la closure y llamarla fuera) siguen pendientes.

### Estado closures (actualizado)
| caso | VM | C | Cranelift |
|---|---|---|---|
| lambda sin captura | ✅ | ✅ | ✅ |
| anidada sin captura | ✅ | ✅ | ✅ |
| captura (usada dentro) | ✅ | ✅ (3.5.15) | ✅ (3.5.16) |
| capture múltiple/abuelo | ✅ | ✅ (3.5.15) | ✅ (3.5.16) |
| closure retornada (fuera) | ❌ | ❌ | ❌ |

## [3.5.16] - 2026-08-28 — Capturas en Cranelift (closures vía celdas globales)

### Fix
Se extendió la resolución de capturas (v3.5.15) al backend Cranelift. Las
variables capturadas se promueven a celdas globales mangadas `{ancestro}::{var}`:
- `compute_captures` identifica variables capturadas por funciones anidadas.
- En `compile_body`, `cap_cell_for(n)` resuelve Load/Store/StoreLocal de una
  variable capturada a su celda global.
- El binding de entrada guarda los parámetros capturados en la celda global.
- Las celdas de captura se agregan a `global_names`.

### Verificado
- Capturas simple/múltiple/abuelo: VM==Cranelift (10/13/301/1011).
- Sweep VM↔Cranelift 13/14 (el único fallo, stress_03_unicode, es pre-existente:
  Cranelift no soporta `__str_a_caracteres`/`__str_mayusculas`/`__str_padding_inicio`).
- Sweep VM↔C 6/6; fixpoint byte-idéntico; check examples 396/0.

### Pendiente
- Closure léxica real (retornar la closure y llamarla fuera del scope) requiere
  entornos de captura reales en los backends. Sigue pendiente.
- Hilos en Cranelift/LLVM siguen pendientes (tabla nombre→función-nativa + linkage).

## [3.5.17] - 2026-08-28 — Hilos reales en Cranelift + concurrencia completa en nativo

### Hilos en Cranelift (el pendiente de v3.5.16)
- Trampolines `__lumen_ft_<fn>` emitidos por Cranelift (Export, uno por
  función): leen cada argumento con `_lw_thr_arg_handle(k)` (deep-copy del
  Val estagiado en el TLS del hilo hijo) y llaman a la función nativa.
- Shim de link con conocimiento del programa: `lw_shim_source_for(program)`
  arma la tabla `_lft_names/_lft_ptrs` + `_call_by_name_thread` que consume
  el runtime pthread/Win32 de lumen_rt.h. `lw_shim_source()` (stubs) queda
  solo para el path LLVM.
- Nuevos helpers de handles: `_lw_cstr`, `_lw_thr_spawn_h` (estagia handles
  → Val[] y crea el hilo), `_lw_thr_join_h`, `_lw_thr_arg_handle`.
- `Instr::Call` de Cranelift mapea `__hilo_lanzar/__hilo_esperar`
  (+alias `__tarea_*`/`__thread_*`); `cranelift_supported` los acepta y
  `llvm_supported` los rechaza explícitamente (el emisor LLVM no los mapea).
- CLI: el link de `--rust` ahora pasa `-lpthread` (no-Windows).

### Canales y mutexes nativos (paridad VM en C y Cranelift)
- lumen_rt.h: `LwChan` (buffer circular + condvar, recv bloqueante),
  `_lw_chan_new/_lw_chan_send/_lw_chan_recv`, `_lw_mutex_new`,
  `_lw_mutex_lock_call` (estagia 1 arg en `lw_thr_args` TLS y ejecuta la
  función nombrada bajo el cerrojo vía `_call_by_name_thread`).
- Backend C: mapeo directo de `__canal_*`/`__mutex_*` a la capa `_rt_*_v`.
- Cranelift: helpers `_lw_chan_*_h`/`_lw_mutex_*_h` + mapeo en `Instr::Call`.
- VM: los registros de canales/mutexes ahora son `Arc<Mutex<..>>` COMPARTIDOS
  entre la VM madre y las VMs de `__hilo_lanzar` (antes cada hilo tenía su
  registro y `__canal_recibir` entre hilos colgaba para siempre).

### Otras paridades nativas
- Calendarios `__calendario_hijri/__calendario_persa` porteados al runtime
  (aritmética idéntica a la VM) y mapeados en C y Cranelift.
- Tiempo: `__tiempo_ahora/__tiempo_formatear/__tiempo_diferencia/
  __tiempo_parsear` mapeados en Cranelift (ya estaban en C).

### Fixes de semántica en backend C (detectados al verificar hilos)
- CAPTURA DE LOCALES: las variables `sea` declaradas en el scope base de un
  ancestro ahora se capturan igual que los params (seed de captura en
  `plan_var_keys`, orden de planes padres→hijos, `base_bindings` replica el
  nombrado exacto de keys). Antes: la anidada veía void (VM/Cranelift OK).
- GLOBALES: variables `sea` de la entrada usadas por otras funciones van a
  UN slot compartido (key cruda), paridad con `program_global_names` de
  Cranelift/LLVM. Antes cada función veía su propio slot y las mutaciones
  se perdían (`total=0` en vez de `3`).
- SAVE/RESTORE de llamada: `name_sets` ahora conserva solo slots PROPIOS
  (`{fn}::...`); globales y slots capturados ya no se restauran tras la
  llamada (se deshacían mutaciones legítimas).

### Verificado
- Hilos: spawn/join con args enteros y texto, 3 y 8 hilos — VM==C==Cranelift
  (11555324 y 2200278), determinista en 5 corridas.
- Canal ENTRE hilos (productor/consumidor con recv bloqueante): 142 en los
  3 backends.
- `examples/jr_concurrencia.nv` (hilos+canal+mutex+calendarios): idéntico
  en VM, C y Cranelift.
- `cargo test -p lumen-aot`: 10/10 (incluye test nuevo `test_cranelift_threads`).
- check examples 396/0.
- Pendiente: closure léxica real (retornar la closure y llamarla fuera);
  stress_03_unicode en Cranelift (builtins de string, pre-existente).

## [3.5.18] - 2026-08-28 — Closures léxicas reales + stress_03 unicode + suite de benchmarks

### Closures léxicas: retornar la closure y llamarla fuera del scope
El pendiente histórico de v3.5.12-3.5.16 queda cerrado para el caso canónico:
```
funcion entero contador() {
    sea n = 0;
    funcion entero inc() { n = n + 1; retornar n; }
    retornar inc;            // la función ESCAPA como valor
}
sea f = contador();
imprimir("c1:" + f());       // 1
imprimir("c2:" + f());       // 2 — el estado capturado persiste
```
- **sema**: las funciones anidadas se pre-registran como VALORES
  (`TypeInfo::Func`) en el scope del padre; `E058` ("no es una función") se
  convierte en llamada dinámica (tipo Numero) para habilitar variables que
  guardan closures.
- **builder**: `Expr::Ident` que nombra una función registrada emite
  `FuncRef` (antes siempre `Load`); las llamadas sobre variables ya usaban
  `Load + CallValue` (sin cambio).
- **VM**: nuevo `Value::Closure { name, env }`. FuncRef captura los bindings
  visibles en CELDAS COMPARTIDAS (Arc<Mutex<Value>>); CallValue inyecta el
  entorno como scope sintético de `Value::Ref` (write-through) y `Ret` lo
  desapila (CallFrame.is_closure). Cada instancia de la factory tiene su
  PROPIO entorno: semántica léxica completa.
- **C / Cranelift / LLVM**: usan la maquinaria ya existente (FuncRef→T_FRE
  con puntero nativo, CallValue indirecto) + las celdas de captura de
  v3.5.15/3.5.16, que son estáticas y sobreviven al retorno del padre.
  Limitación v1 documentada: múltiples instancias de la misma factory
  COMPARTEN las celdas (VM sí las separa por instancia).

### stress_03_unicode resuelto en Cranelift (pre-existente desde v3.5.16)
- Nuevos helpers: `_lw_str_chars_h`, `_lw_str_upper_h`, `_lw_str_lower_h`,
  `_lw_str_pad_h` sobre `_to_chars/_case_str/_pad_str` del runtime.
- Mapeados: `__str_a_caracteres`, `__str_mayusculas/__str_minusculas`,
  `__str_padding_inicio/__str_padding_fin` (+alias ingleses).
- fuzz/stress_03_unicode.nv: idéntico en VM, C y Cranelift (UTF-8 correcto:
  "HOLA ÑANDÚ", chars de "añó" = 3, padding con relleno).

### Suite de benchmarks multi-lenguaje (benchmarks/)
- Tareas idénticas en Lúmen/C/C++/Rust/Python: fib(30), suma 50M, primos
  <100k (división de prueba), 200k strings construidos, array de 2M.
- Harness `benchmarks/run_bench.py`: mide tiempo de pared + RSS pico
  (wait4) por tarea e implementación (VM, AOT-C, Cranelift, LLVM si hay
  clang) y genera results/benchmark.csv + results/informe.md.
- Cargas calibradas para que todas las implementaciones completen: fib(26),
  suma 10M, primos<20k, 200k strings, arrays de 200k. La calibración misma
  es un hallazgo: el límite lo pone la memoria del runtime de handles.

### Fix de runtime descubierto por el benchmark: `_fmt` (backend C/Cranelift)
- `_fmt` malloc-aba 8192 bytes POR LLAMADA y los dejaba ir (leak de 8-40KB
  por iteración en bucles de strings) → strings 30k: 599MB/0.40s ANTES,
  16MB/0.02s DESPUÉS.
- HEAP-OVERFLOW potencial: T_STR hacía memcpy sin límite al buffer de 8KB
  (textos >8KB corrompían el heap); T_ARR/T_TUP/T_STT/T_MAP concatenaban sin
  crecer. Ahora: buffers de tamaño exacto por caso, T_STR=strdup directo y
  crecimiento con realloc (_fmt_grow) para colecciones.

## [3.5.19] - 2026-08-28 — Velocidad: las 4 mejoras del informe de benchmark

### ① Promoción de registros en el backend C (el gran salto de AOT)
Los locales propios que no escapan (sin `MakeRef`, sin captura por funciones
anidadas, sin ser globales) salen de `gv[]` y se convierten en variables
locales C (`Val _lv0, _lv1, ...`): GCC los mantiene en REGISTROS y optimiza
los bucles como C nativo. Consecuencias:
- Load/Store de locales ya no toca memoria global por instrucción.
- El save/restore alrededor de llamadas se encoge (los locales promovidos
  viven en el stack de C, per-call — la recursión es correcta por
  construcción, sin restaurar gv).
- Params y globales siguen en gv (ABI de hilos, CallValue, capturas).

### ② Caché inline de resolución de variables en la VM
`Load`/`Store` resolvían por scan de scopes con lookup de HashMap por nivel
en CADA instrucción. Ahora una caché name-idx→(gen, scope) resuelve en un
solo lookup; la generación se invalida en todo cambio estructural de
`locals` (push/pop de scopes, frames, truncate, declaraciones nuevas —
14 puntos + inserciones de StoreLocal/Store). Fallback seguro al scan
completo si la entrada no aplica.

### ③ Arena bump TLS para el runtime de handles (Cranelift/LLVM)
`_lw_box` mallokeaba por cada valor cajado (hot-path de cada operación).
Ah usa arenas bump de 4MB thread-local: asignación ~10× más barata. La
memoria sigue sin liberarse (GC real sigue pendiente — limitación #1
documentada en el informe), pero el costo por operación cae drásticamente.

### ④ Optimizador IR: constant folding (beneficia a TODOS los backends)
Nuevo `lumen-ir/src/optimize.rs`: pliega `Const a; Const b; Binary op` →
`Const r` (y `Unary`) para aritmética entera/float/comparaciones con
semántica EXACTA (wrapping, sin plegar división por cero ni MIN/-1, sin
tocar Concat/strings). Conectado en los 4 puntos de construcción de IR del
CLI (build nvc, backends, run, suite de pruebas). Menos instrucciones =
VM más rápida, menos ops de stack en C, menos box/unbox en Cranelift.

### Nota fixpoint
El bytecode de compiler_v4.nvc cambia (el pipeline Rust ahora pliega
constantes); v4_self/v4_self2 los produce el compilador self-hosted SIN el
optimizador, así que el fixpoint byte-idéntico se preserva.

### ⑤ Fusión de instrucciones en el emisor C (la optimización más grande)
El patrón canónico de bucle `Load/Const; Load/Const; Binary; Store|JmpIf`
(esqueleto de `x = x + 1`, `acc = acc + i`, `si (i < n)`) ahora se emite
como UNA sentencia C sin tráfico de la pila de valores ST[]. Resultado:
**sum 10M pasó de 0.796s a 0.121s (6.6×)**; AOT-C ya supera a Python en
fib/sum/primes.

### ⑥ Peephole en la VM (fusión de opcodes en el dispatch)
- `Add/Sub/Mul` (Int,Int) inspeccionan la siguiente instrucción: si es
  `Store`, escriben directo sin push/pop ni dispatch del Store.
- `Lt/Le/Gt/Ge/Eq/Neq` (Int,Int): si sigue `JmpIf`, saltan directo sin
  push del booleano ni dispatch del salto.
- `step_instr` con `#[inline(always)]` (el loop de `run` ya no paga la
  llamada al dispatcher).
- Refactor: `do_store_by_idx` comparte la lógica de Store con el peephole.
Ganancia ~5-10% (el dispatch sigue dominando; super-opcodes de bytecode son
la siguiente fase).

### Resultados benchmark v3.5.19 (benchmarks/results/informe.md)
- AOT-C vs Python: fib 0.026s vs 0.031s · sum 0.121s vs 0.693s (5.7×) ·
  primes 0.022s vs 0.026s.
- Cranelift: fib -18% tiempo y -16% RSS (arena).
- Regresión: tests aot 10/10, vm 695/695, sweep 25/25, fixpoint idéntico,
  closures y concurrencia intactos en los 3 backends.

## [3.5.20] - 2026-08-28 — Ultra-velocidad: emisor por expresiones, always-inline, super-opcodes VM y GC en Cranelift

### AOT-C: emisor por pila de expresiones (eliminación de ST[] en tramos rectos)
El backend C ya no pasa cada valor por la pila de valores ST[] en los tramos
sin saltos: mantiene una PILA DE EXPRESIONES C (`_bin(1, _deref(acc), _deref(i))`)
y materializa en UNA sentencia por Store/JmpIf/Return. GCC optimiza el bucle
completo (registros, CSE, strength reduction).
- **Freshness tracking**: si el valor a guardar es demostrablemente FRESCO
  (resultado de operación/llamada, sin alias), se asigna SIN `_dcp` — ahorra
  la copia profunda de arrays/strings recién creados.
- `LW_HOT` (`always_inline`) en `_deref`, `_bin`, `_truthy`, `_v_int`, etc.:
  la struct Val de 80B por valor hacía que la heurística de GCC rechazara el
  inlining → cada op del bucle pagaba una llamada real. Sin esto, el emisor
  por expresiones no rendía (45× más lento que C puro).
- Resultado: **sum 10M de 0.796s (v3.5.18) a 0.007s (114×)** — a 7× de C
  nativo; primes a 1.5× de C; fib/arrays 5-6×.

### VM: super-opcodes de bytecode (~18-20% en bucles)
Nuevas instrucciones fusionadas (solo el pipeline Rust las emite; el
compilador self-hosted sigue el formato clásico → fixpoint intacto):
- `FusedBinK/FusedBin` (d = a OP b|k) y `FusedCmpKJmp/FusedCmpJmp`
  (if !(a OP b|k) goto) — 4 instrucciones IR → 1 bytecode, sin push/pop.
- Fast-paths Int/Int y Float en la VM; fallback completo (strings, mezcla).
- Encode/decode con tags 5-8; disasm y JIT actualizados (el JIT cede esas
  funciones a la VM, cuyos brazos rápidos ganan).
- `do_load_by_idx` unificado con la caché inline de variables.
- sum VM: 3.15s → 2.66s.

### Cranelift: GC conservador mark-sweep con reutilización (adiós OOM)
- Arena bump TLS + freelist: cuando la asignación acumulada supera 8MB,
  mark-sweep. Raíces: scan conservador del STACK nativo (incluye los slots y
  spills de Cranelift, donde viven los handles i64) + registros vía setjmp.
  Los boxes NUNCA apuntan a otros boxes → marcado de un nivel.
- Tope de stack real por hilo: Windows vía TEB StackBase; Linux vía
  /proc/self/maps (cache TLS). Evita leer memoria no mapeada.
- Cada hilo barre solo su arena TLS; los valores cruzan hilos como Val por
  valor (join/canales), nunca como handle → sound entre hilos.
- Resultado: sum 10M en Cranelift de **OOM a 14MB**; fib 274→14MB; todos los
  benchmarks con resultados correctos.
- Fix incluido: el wrapper `main` de Cranelift devolvía el handle del
  resultado como exit code (salidas basura tipo 48/112); ahora retorna 0.

### Verificación
- sweep paridad 25/25 (VM==C==Cranelift) · fixpoint byte-idéntico ·
  tests aot 10/10 · hilos/canales/closures en Cranelift con GC activos.

## [3.5.21] - 2026-08-28 — Enteros sin tag en AOT-C (closing the gap con C)

### Análisis de tipos IR + emisión `long long` nativa
- `int_promotion_analysis`: punto fijo GLOBAL sobre el programa.
  - Locales: enteros si todas sus escrituras son expresiones enteras
    (ConstInt/aritmética entera/cargas de otra variable entera).
  - Parámetros: enteros si todos los llamadores ESTÁTICOS pasan enteros y el
    cuerpo no los reasigna; se copian del slot gv UNA vez al entrar.
  - Salvaguardas: funciones alcanzables solo dinámicamente (`__hilo_lanzar`,
    CallValue, FuncRef, `__mutex_bloquear`, `__coro_crear`) NO promocionan
    parámetros; quedan fuera capturadas, objetivos de MakeRef y globales.
- Emisor C: los bucles enteros operan con `long long` reales — wrapping
  exacto vía aritmética unsigned (paridad con `wrapping_add` de la VM),
  shifts con máscara `& 63` (paridad `wrapping_shl`), comparaciones como
  test C directo (sin `_truthy`/`_bin`). La conversión `_v_int` ocurre solo
  en las fronteras.
- Interacción con lo anterior: se combina con la pila de expresiones
  (v3.5.20) y los `always_inline`; los locales promovidos a Val (v3.5.19) y
  los enteros son excluyentes.

### Resultados (benchmarks/results/informe.md)
- sum 10M: **0.796s (v3.5.18) → 0.005s = 159×** (~5× de C, ~110× más rápido
  que Python). primes 20k: **a la par de C**. fib/arrays ~6-8× de C (el gap
  restante es el ABI de llamadas, no la aritmética).
- Regresión controlada: el bug de promocionar params de funciones sin
  llamadores estáticos (hilos) se detectó en el sweep (jr_concurrencia/
  hilos_stress divergían: el param texto llegaba vacío) y se corrigió con el
  conjunto de llamadores dinámicos.
- sweep paridad 25/25 · fixpoint byte-idéntico · tests aot 10/10.

## [3.5.22] - 2026-08-28 — Move-semantics en argumentos de llamada (análisis de último uso)

### El cambio
- `last_uses` por función: índice del último Load/MakeRef/ArrayPushVar de cada
  variable (las keys capturadas por anidadas quedan excluidas).
- Las llamadas a funciones de usuario ahora se emiten desde el camino de
  expresiones: cada argumento FRESCO o de ÚLTIMO USO se materializa SIN
  `_dcp` (`Val _mvN_K = expr;`), el resto con `_dcp` (conservador).
- Semántica idéntica a la VM: si la variable no se vuelve a leer, copiar o
  mover es indistinguible; si se lee después, se copia como antes.
- Bugs cazados en el camino:
  - temporales `_mva0` repetidos entre llamadas → contador por función;
  - hazard de sincronización XE↔ST: con la pila de expresiones pendiente,
    el resultado de una llamada en ST[] desordenaba los POPs posteriores
    (`"move:" + f(x)` imprimía invertido) → el brazo de llamada hace
    `xe_spill` del resto antes de emitir.

### Resultados
- fib(26): 0.016 → **0.012s** (recursión sin _dcp de args).
- strings 200k: 0.148 → **0.088s**. arrays 200k: 0.019 → 0.017s.
- Nuevos benchmarks: `passargs` (500k llamadas con array+texto) 0.038s;
  `movebig` (array de 100k elementos pasado por move, suma incluida) 0.012s.
- Acumulado sesión v3.5.18 → v3.5.22: sum 10M **0.796s → 0.005s (159×)**;
  primes a la par de C; fib ~6× C (el resto del gap es la representación Val).
- sweep paridad 25/25 · fixpoint byte-idéntico · tests aot 10/10.

## [3.5.23] - 2026-08-28 — ABI de argumentos en registros C (cerrando el gap)

### El refactor
Las funciones que NUNCA se llaman por nombre reciben sus argumentos como
PARÁMETROS C nativos (`Val _pv0` / `long long _lli0` directamente en la
firma), sin staging por gv[] ni save/restore de params; la recursión es
nativa de C.
- Elegibilidad: fuera `dyn_named` (hilos/mutex/corutinas/FuncRef detectados
  con walk de pila abstracta sobre `__hilo_lanzar` etc. con cualquier nº de
  args) y fuera params capturados por anidadas (deben vivir en gv).
- `_fw_<fn>(void)`: wrapper gv→registros para `_call_by_name` y el camino
  legacy; `_ft_<fn>` de hilos delega en el wrapper.
- El brazo XE de llamada convierte tipos en el sitio (LL→int param directo,
  Val→`(long long)_asf(...)`, LL→`_v_int(...)`), aplica move-semantics a los
  args Val, y emite la llamada directa con save/restore SOLO de locales gv.
- Bug del análisis de enteros corregido: las claves de param en el mapa de
  locales enmascaraban el mapa de params (fib no promocionaba su param).
- `prestado mut` preservado: el Load de param Val emite `_deref` (T_PTR).
- Bugs cazados: `windows(2)` no veía el nombre en `__hilo_lanzar` con 2+
  args (hilos divergían a 0); firmas `()` vs `(void)`; `_mva` duplicados.

### Resultados (acumulado v3.5.18 → v3.5.23)
| Tarea | v3.5.18 | v3.5.23 | Mejora | vs C |
|---|---|---|---|---|
| sum 10M | 0.796s | **0.002s** | **~400×** | ~2× |
| fib(26) | 0.032s | **0.009s** | 3.6× | ~4.5× |
| primes 20k | 0.027s | **0.003s** | 9× | **a la par** |
| strings 200k | 0.103s | 0.103s | — | ~10× |
| arrays 200k | 0.089s | 0.018s | 5× | ~6× |

- sweep paridad 25/25 · fixpoint byte-idéntico · tests aot 10/10.

## [3.5.24] - 2026-08-28 — Las 3 palancas finales: noinline condicional, strings rápidos, returns-int

### 1. Análisis no-throw + NOINLINE condicional
- `no_throw_analysis` (punto fijo decreciente, siembra optimista → la
  recursión se resuelve): una función no lanza si no contiene Div/Mod,
  ArrayGet/Set, TupleAccess ni llamadas lanzadoras fuera de sus `intentar`.
- Las funciones no-lanzantes se emiten SIN `LUMEN_NOINLINE` (gcc las
  inlinea) y SIN ERRCHK (muertos por definición); las llamadas a callees
  no-lanzantes tampoco emiten ERRCHK.

### 2. Strings: concat directo + adopción de buffers
- `_concat2(a, b)`: STR+STR en un solo malloc con 2 memcpys (sin ida y
  vuelta por `_fmt`); tipos mixtos delegan en `_bin(2, ...)`. Se usa en el
  camino de expresiones Y en el legacy.
- `_v_str_take(char*)`: `a_texto` adopta el buffer de `_fmt` sin recopiarlo
  (antes: doble copia + leak del buffer intermedio).

### 3. Análisis no-mutación → argumentos compartidos sin _dcp
- `no_mutate_analysis` (punto fijo): una función no muta si no tiene
  ArrayPushVar/ArraySet/StructSet/MakeRef, no recibe prestados ni llama a
  mutantes. Sus argumentos se pasan SIN `_dcp` — misma semántica que la VM
  (que comparte vía Arc) y ahorro de copias profundas.

### 4. Retornos `long long` para funciones que siempre devuelven entero
- `returns_int_analysis`: punto fijo sobre los valores de retorno. Las
  funciones que siempre devuelven entero retornan `long long` nativo (8B en
  vez de Val de 80B por llamada): fib pasa de devolver structs de 80B en
  cada una de sus ~400k llamadas recursivas a enteros en registro.
- Wrappers `_fw_` envuelven el resultado en Val para `_call_by_name`/
  trampolines de hilos; callers XE reciben el resultado como LL (kind 1).

### Resultados (acumulado v3.5.18 → v3.5.24)
| Tarea | v3.5.18 | v3.5.24 | vs C | vs Rust | vs Python |
|---|---|---|---|---|---|
| fib(26) | 0.032s | **0.005s** | 2.5× | **a la par** | **6× más rápido** |
| sum 10M | 0.796s | **0.002s** | 2× | **a la par** | **310× más rápido** |
| primes 20k | 0.027s | **0.003s** | 1.5× | **a la par** | 9× más rápido |
| arrays 200k | 0.089s | 0.018s | 6× | 9× | 2× más rápido |
| strings 200k | 0.103s | 0.124s | 12× | 10× | 0.5× (Python gana) |

fib/sum/primes: **rendimiento de Rust/C++**. Verificación: sweep 25/25,
fixpoint byte-idéntico, tests aot 10/10, hilos/canales/closures intactos.

## [3.5.25] - 2026-08-28 — Arena de strings, arrays fast-path, enteros nativos en Cranelift, dispatch VM

### Frente A — Arena de strings (TLS bump)
- `_sa_alloc`: arena bump thread-local para buffers de texto; `_v_str` y
  `_concat2` la usan. Los strings de LÚMEN son inmutables y nunca se liberan,
  así que la arena es semánticamente exacta y elimina malloc por
  concatenación. strings 200k en AOT-C: 0.124 → **0.083s** (y menos RSS).

### Frente B — Arrays: fast-paths en el runtime
- `_arr_get` inline con fast-path T_ARR (`__builtin_expect`) y camino
  genérico restaurado para tuplas (el refactor inicial lo borró → divergencia
  detectada por el sweep y corregida); `_arr_push_ip` inline con expect.

### Frente C — Enteros nativos en Cranelift (fusión sobre slots i64)
- `int_vars_by_name`: punto fijo decreciente sobre la pila abstracta con
  pops/pushes EXACTOS por instrucción (`instr_pops_pushes`) — los deltas
  netos dejaban residuos que promocionaban variables falsas (detectado con
  opcion.nv/arrays.nv y corregido).
- Locales siempre-enteros (no capturados, no MakeRef, no globales, declaración
  única) viven en slots i64 de 8B; fusión de patrones
  Load/Const;Load/Const;Arith/Cmp;Store/JmpIf → código nativo sin boxing,
  con saltos vía `brif` directo (bloque fall-through nuevo).
- Brazos Load/Store de int-locals: box/unbox solo en el límite (`_lw_h2i`).
- Bugs cazados y corregidos en el camino: el bucle convertido de `for` a
  `while` colgaba en `continue` (el for avanzaba solo); globales
  promocionados por error; MakeRef targets; análisis desincronizado.
- sum en Cranelift: **2.13s → 0.85s (2.5×)**; strings 0.19 → 0.15.

### Frente D — Dispatch de la VM
- `#[inline(always)]` en step_instr y `#[inline]` en los 4 ejecutores:
  mejora marginal (~2-3%); el threaded-dispatch real requiere computed-goto
  (nightly) — documentado como límite estructural del intérprete en stable.

### Verificación
- sweep paridad 25/25 · fixpoint byte-idéntico · tests aot 10/10 ·
  vm 695/695 · benchmark oficial 35/35 OK.

## [3.5.26] - 2026-08-29 — Arrays de enteros unboxed en AOT-C (gap de arrays cerrado)

### Análisis de arrays de enteros puros (`arr_vars_by_name`)
- Punto fijo con kinds {Int, Arr, Not}: un local es array-de-enteros si se
  declara `sea xs = []` y solo recibe `agregar` de enteros, lecturas con
  índice y `largo`, sin escapar jamás (llamadas, print, asignación-alias,
  ArraySet lo degradan).
- Emisión: `long long* xs_d; long long xs_n, xs_c;` con crecimiento
  amortizado por realloc; ventanas de fusión en el camino de expresiones:
  - `Load xs; Load j|ConstInt; ArrayGet` → lectura nativa con bounds-check
    en statement-expression (un solo uso del índice, sin doble evaluación).
  - `Load xs; ArrayLen` → `xs_n` directo.
  - `ArrayPushVar xs` → push nativo inline SIN apilar el resultado-residuo
    (el residuo inundaría ST[16384] — bug detectado: segfault a los 16k
    elementos; en la VM el residuo es inofensivo porque la pila crece).
- Prioridad de promociones: la promoción entera (v3.5.23) tiene prioridad
  sobre la promoción Val (v3.5.19) — antes se disputaban i/j y ganaba Val.

### Resultados
- arrays 200k: **0.018s → 0.003s (6×) = C = C++ exacto**.
- Batería: tests aot 10/10 · sweep paridad 25/25 · fixpoint byte-idéntico.
- El único gap nativo restante: strings (0.082 vs C 0.010) por la
  representación de texto (malloc+memcpy por concat); requiere builders con
  buffer reutilizable a nivel de IR.

## [3.5.27] - 2026-08-29 — Gap de strings CERRADO en AOT-C (0.082s → 0.013s, a la par de C)

El benchmark strings construía 200k textos (`"item-" + a_texto(i) + "-fin"`)
y sumaba sus largos. Costes detectados: el bucle caía al camino legacy ST[]
(los builtins `a_texto`/`largo` cortaban el camino de expresiones), `_bin(1)`
para STR hacía `_fmt(a)+_fmt(b)+malloc+_v_str` (3 mallocs CON LEAK y 3-4
copias por concat), `_fmt` de enteros usaba snprintf (~100-300ns), y cada
literal se copiaba al arena por iteración.

### Runtime (lumen_rt.h)
- `_bin` op 1 STR: STR+STR → `_concat2` (1 alloc arena); mixto → solo se
  formatea el lado no-texto y se concatena en el arena. Corrige además 3
  leaks por concatenación (los buffers de `_fmt` y del resultado intermedio
  nunca se liberaban).
- `_itoa_ll`/`_itoa_sa`: itoa manual ~10-20× más rápido que snprintf("%lld").
  `_fmt` T_INT lo usa; `_itoa_sa` escribe directo en el arena.
- `_to_text`/`_to_text_ll`: `a_texto()` como función del runtime con
  fast-path entero (arena, sin malloc ni snprintf).
- `_largo_ll`: `largo()` como función (misma tabla que la emisión legacy).
- `_v_str_lit`: los literales de texto se envuelven SIN copia al arena
  (son inmutables y viven en .rodata) — antes: strlen+alloc+memcpy por
  iteración en bucles con literales.
- Funciones calientes de texto marcadas `inline`.

### Emisor C (lib.rs)
- Brazos XE para los builtins `a_texto`/`largo` (argc 1): ya no cortan la
  cadena de expresiones — el bucle de texto se queda en registros C sin
  tráfico ST[] (PUSH/POP de Vals de 80B).
- `int_promotion_analysis`: `largo` builtin propaga IKind::Int → los
  acumuladores `total = total + largo(s)` se promocionan a `long long`.
- `ConstStr` (XE y legacy) emite `_v_str_lit`.
- Legacy `a_texto` y dispatch `_call_by_name` → `_to_text`; shim Cranelift
  `_lw_to_text` → `_to_text` (strings en Cranelift: 0.15s → 0.078s).

### Resultados (AOT-C)
| Tarea | v3.5.26 | v3.5.27 | C | C++ | Rust |
|---|---|---|---|---|---|
| strings 200k | 0.082s | **0.013s (6×)** | 0.010s | 0.008s | 0.009s |

Las 5 tareas quedan a la par de C/C++/Rust: fib 0.006 · sum 0.002 ·
primes 0.002-0.003 · strings 0.013 · arrays 0.002-0.003.

### Verificación
- tests workspace 956/956 · sweep paridad 25/25 · fixpoint byte-idéntico
  (170985 B) · edge-cases texto (INT64_MIN, unicode, concat mixto, bools,
  floats) VM vs C idénticos · benchmark oficial 35/35 OK.

## [3.5.28] - 2026-08-29 — ABI de enteros crudos en Cranelift (gap de llamadas cerrado)

El backend Cranelift boxea/desboxea cada valor como handle opaco i64 en cada
operación. v3.5.25 introdujo slots i64 y fusión, pero las LLAMADAS seguían
pagando box/unbox/deep-copy y un ERRCHK por instrucción. Esta versión porta el
ABI de enteros del backend C a Cranelift.

### ABI de enteros crudos (interprocedural)
- `cr_call_graph`: `direct_callers` (Call estáticos) y `dyn_named`
  (FuncRef + hilos/corutinas/mutex por nombre). Las funciones dinámicas
  reciben handles arbitrarios → nunca se especializan.
- `cr_params_int_analysis`: punto fijo — un parámetro es entero si NINGÚN
  llamador estático le pasa no-entero y no se reasigna; pliega las mismas
  exclusiones que compile_body (ref_targets/capturas/celdas/globales/stored)
  para que llamador y llamado decidan EXACTAMENTE lo mismo.
- `cr_returns_int_analysis`: punto fijo — funciones que SIEMPRE devuelven
  entero retornan i64 crudo.
- Llamada directa: params int reciben el i64 CRUDO (sin box ni dcp); el
  resto recibe handle (dcp solo de handles). Resultado int → i64 crudo.
- Stack con kinds `(Value, 0=handle|1=i64|2=b8)`: aritmética/comparación
  entera nativa; Div/Mod nativos con zero-check inline (`_lw_throw_div`,
  paridad "Error: Division por cero" + INT64_MIN/-1); los consumidores
  boxean solo si lo necesitan.
- ERRCHK solo si el callee puede lanzar (`no_throw_analysis`, misma que C):
  fib pasa a llamadas limpias.
- Params enteros SIN celda Val de 80B (solo slot i64) y void0 perezoso:
  fib ya no paga `_lw_void`+`_lw_store_slot_direct` por frame.

### Bugs corregidos en el camino
- Colisión de nombre param↔global (`program_global_names`): el análisis
  decía "int" y compile_body "handle" → el llamador pasaba i64 crudo y el
  llamado leía handle → SEGFAULT. Ahora params_int es la única fuente de
  verdad. (regresión: fuzz/cr_colision_global.nv)
- Div/Mod pasaban operandos CRUDOS a `_lw_bin` (espera handles) → SEGFAULT.
  (regresión: fuzz/cr_divmod_entero.nv)
- Bloque de label vacío al final de función (`elegir` donde todos los casos
  retornan) → el verificador de Cranelift lo rechaza; se rellena con return.

### Resultados (Cranelift)
| Tarea | v3.5.27 | v3.5.28 | C | Rust |
|---|---|---|---|---|
| fib | 0.095s | **0.003s (32×)** | 0.005s | 0.004s |
| primes | 0.054s | **0.004s (13×)** | 0.002s | 0.002s |
| sum | 0.735s | **0.027s (27×)** | 0.001s | 0.001s |

fib y primes quedan **a la par de C/Rust**. Frontera restante: sum (locales
enteros viven en stack slots → tráfico de memoria por iteración; C los
promueve a registros) y strings/arrays (boxing de handles por concat/push;
requiere portar el string-builder y los arrays unboxed).

### Verificación
- tests workspace 956/956 · sweep paridad 27/27 · fixpoint byte-idéntico
  (170985 B) · Div/Mod con try/catch e INT64_MIN/-1 idénticos VM vs C vs
  Cranelift · benchmark oficial OK.

## [3.5.29] - 2026-08-29 — Arrays de enteros sin boxear en Cranelift + fix de arrays literales

### Arrays de enteros sin boxear en Cranelift (paridad con C)
- Reutiliza `arr_vars_by_name` (v3.5.26) para detectar arrays de enteros
  que no escapan, y los materializa en tres slots i64 (ptr/len/cap) en vez
  de handles de 80B:
  - `agregar` → `_lw_iarr_push` (crecimiento amortizado ×2, sin boxing del
    elemento ni deep-copy del array).
  - `xs[j]`/`xs[0]` → fusión `Load xs; Load j|ConstInt; ArrayGet` →
    `_lw_iarr_get` nativo con bounds-check (lanza "Indice fuera de rango"
    capturable por intentar/atrapar).
  - `xs.largo()` → fusión `Load xs; ArrayLen` → carga de len nativa.
  - `sea xs = []` → slots inicializados a 0 en el entry (sin LW_ARR_NEW).
- `int_vars_by_name` ahora lleva pila con fuente (kind, variable) para que
  `xs[j]` sobre array promovido propague Int y los acumuladores
  (`acc = acc + xs[j]`) se promocionen a slots i64.

### Bug corregido — arrays literales locales (backend C y Cranelift)
- `arr_vars_by_name` promocionaba `ArrayNew(n)` para cualquier n, pero el
  camino nativo solo inicializa arrays VACÍOS: un literal local
  `sea xs = [1, 2, 3]` quedaba promovido sin elementos → "Indice fuera de
  rango". Ahora solo `ArrayNew(0)` se promociona (intención documentada en
  v3.5.26); los literales con elementos usan el camino Val. Detectado por
  el nuevo fuzz/arrays_nativos.nv.

### Resultados (Cranelift)
| Tarea | v3.5.28 | v3.5.29 | C | C++ | Rust |
|---|---|---|---|---|---|
| arrays 200k | 0.088s | **0.003s (29×)** | 0.002s | 0.002s | 0.002s |
| primes 20k | 0.059s | **0.003s** | 0.002s | 0.002s | 0.002s |
| fib(26) | 0.044s | **0.002s** | 0.005s | 0.008s | 0.002s |

Cranelift queda **a la par de C/C++/Rust en fib, primes y arrays**.
Frontera restante: sum (slots i64 con load/store por iteración vs
registros de C) y strings (boxing de handles por concatenación).

### Verificación
- tests workspace 956/956 · sweep paridad 28/28 (nuevo
  fuzz/arrays_nativos.nv: push/get/bounds-catch/literales) · fixpoint
  byte-idéntico (170985 B) · benchmark oficial OK.
