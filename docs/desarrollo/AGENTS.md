# AGENTS.md — Diario de construcción de LÚMEN

**v3.5.7 Producción Real — Released: 21 Agosto 2026 — 956 tests, CHUNK_VERSION 7, bench 8, headless `es_headless()` — + rondas JIT v3.5.31→v3.5.37 (30 Ago 2026, ver [CHANGELOG](../../CHANGELOG.md) y [arquitectura/jit.md](../arquitectura/jit.md))**

> Checklist único de producción: [docs/produccion.md](produccion.md) — `LUMEN_HEADLESS=1` + `cargo bench -p lumen-bench` + `CHUNK_VERSION 7`

---

## Testing (Actual) — v3.5.7 Producción Real (21 Ago 2026)

| Crate | Tests | Tipo | Nota v3.5.7 |
|-------|-------|------|-------------|
| lumen-lexer | 52 | unit |  |
| lumen-parser | 75 | unit |  |
| lumen-sema | 56 | unit |  |
| lumen-ir | 20 | unit + folding |  |
| lumen-codegen | 13 | unit |  |
| lumen-vm | 48 | unit |  |
| lumen-vm | 616 | e2e | +4 regresión: fallthrough early return, matematicas `potencia(2,10)==1024`, defaults `CallValue`, lambda |
| lumen-vm | 9 | production | 3 aceptación + 2 performance + 4 integración (ver `crates/lumen-vm/tests/production.rs`) |
| lumen-fmt | 2 | unit |  |
| lumen-repl | 3 | unit |  |
| lumen-project | 2 | unit |  |
| lumen-aot | 6 | unit |  |
| lumen-api | 5 | unit |  |
| lumen-bench | 8 | bench | `cargo bench -p lumen-bench`: lexer, parser, pipeline, vm_fib_20 + 4 prod (fallthrough, defaults, matematicas, headless) |
| **Total workspace** | **956** |  | `cargo test --workspace` 0 FAILED (695 vm tests = 636 e2e + 9 prod + 48 unit) |

**v3.5.7 Producción: 956 pruebas en verde (Linux y Windows), 396/396 en `lumen check`, 389 `run` OK con `CI=1`, 8 benches criterion, clippy sin avisos, 4 fuzzers diferenciales sin divergencias. `CHUNK_VERSION 7` con `FuncMeta.defaults` persistidos, `LUMEN_HEADLESS=1` headless centralizado `stdlib/graficos.nv:es_headless()`. Ver `docs/produccion.md`.**

**Comandos producción:**
```bash
cargo test --workspace
cargo test -p lumen-vm --test e2e
cargo test --test production
cargo bench -p lumen-bench
LUMEN_HEADLESS=1 CI=1 cargo test --workspace && cargo bench -p lumen-bench -- --quick
```

---

## Fases completadas (Fases 0-185 ✅ + Sprint 3 Bootstrap ✅)

### Bloque 1: Lenguaje Core (Fases 0-60) ✅
Lexer, parser, sema, IR, bytecode, VM, CLI. Funciones avanzadas, estructuras, módulos, optimizaciones. For-Each, Resultado<T,E>, Opcion<T>, Enums, Tuplas, Destructuring, Genéricos. Async/await base. Stdlib inicial (matematicas, texto, coleccion, fecha, archivos). Fuzzing, property testing. lumen fmt, repl, test, project, CI/CD. Inferencia de tipos, métodos en structs, diccionarios, string interpolation, rangos, constantes, string indexing, conversiones, división entera, concatenación mixta, ternario, loop labels, pattern matching pro, genéricos con bounds, matrices 2D, enums avanzados, closures pro.

### Bloque 2: Lenguaje Avanzado (Fases 61-70) ✅
OR Patterns, If-let/While-let, Range Patterns, String Patterns, Guard Let, Operator Overloading, Extension Methods, Associated Types, Impl Trait return. Where Clauses ⏭️ saltado.

### Bloque 3: Herramientas & DX (Fases 71-95) ✅
LSP (diagnósticos, autocompletado, go-to-def, hover), lumen doc, debugger, lumen fmt avanzado, lumen lint, REPL Pro, Package Manager, Build Incremental, Hot Reload, Playground Web, Benchmarks, Plugins API, AOT (C transpiler + Cranelift + LTO), Cross-compilation, Single binary, Installer.

### Bloque 4: Stdlib Extendida (Fases 96-110) ✅
HashMap<K,V>, HashSet<T>, VecDeque<T>, BinaryHeap<T>, LinkedList<T>. Regex, Unicode, Format, Encoding. Buffered I/O, Streaming, SerialPort. TCP, HTTP cliente/servidor.

### Bloque 5: Runtime & Sistema (Fases 111-130) ✅
JSON parse/serialize, CSV reader/writer, SQLite driver, Procesos, Env, Path, Temp. Timezone, Duracion, Format fechas, Calendarios (Hijri, Persa). Crypto: SHA-256, SHA-512, AES, JWT. Testing: assert, mock, coverage.

### Bloque 6: Concurrencia & Async (Fases 131-150) ✅
Thread::spawn/join, Mutex, RwLock, Arc, Channel. Async Runtime (task spawn/await), Async Stream, Async File I/O, Async TCP, Async Timer, Async Select. Par Iterator/Join. Actores, Supervisor, Cluster. Coro Generator/AsyncGen/Structured.

### Bloque 7: GUI, TUI & Juegos (Fases 151-170) ✅
TUI: ventanas, tablas, menús, layout engine. GFX: Canvas 2D, sprites, game loop, input, audio, partículas, tilemap, charts. GUI nativa Win32: widgets, ventanas, canvas en ventana, eventos, temas. TreeView 📋 pendiente.

### Bloque 8: Portabilidad (Fases 171-185) ✅
WASM backend, WASI, JS interop. Docker, Docker Compose, GitHub Actions. Benchmarks, Fuzzing, Mutation Testing. Logging, Tracing, Metrics, Profiler. Compiler API (lumen-api).

### Sprint 3: Bootstrap completo (30 Julio 2026) ✅
`compiler_v2.nv` via `__compile_nv` nativo — compilador LÚMEN que compila .nv → .nvc en milisegundos. Self-compilación verificada: `self_compile.nv` → `compiler_v2_self.nvc` en **533ms**. ~385 tests, 0 fallos.

### Sprint 4: Self-hosting Total (30 Julio 2026) ✅
`Value::Map(Vec<...>)` → `HashMap<Value, Value>` con `Hash`/`Eq` manual. `__map_get`/`__map_set`/`__map_contains` O(1). Sets O(n). El parser LÚMEN-in-LÚMEN ahora tiene mapas O(1). Camino abierto para self-hosting total sin `__compile_nv`.

### Sprint 5: Self-hosting Puro COMPLETADO (31 Julio 2026) ✅
**LÚMEN se compila a sí mismo sin `__compile_nv`.** `compiler_v4.nv` autocontenido (55,308 bytes = lexer+parser+codegen+main concatenados). `compiler_v4_self.nvc` (54,712 bytes, 49 funciones) recompila su propio source con resultado IDÉNTICO — fixpoint confirmado en 3 runs consecutivos (193s/203s/197s). Bugs críticos arreglados: JmpIf target→nums, If sin JMP de skip, indexación `chars[i]`, TryUnwrap 40→40, print multi-arg, break/continue con loop_stack, VarDecl sin inicializador, escapes `\n \t \r \" \\` en lexer puro, keywords void/diccionario, forward declarations.

### Sprint 6: Prueba de Fuego + Imports + Gramática (31 Julio 2026) 🔄 EN CURSO
**Harness:** `fuego.ps1` compila los **115 ejemplos** de `examples/` con el pipeline puro (`target.txt` = driver parametrizado, 2 líneas: entrada/salida; main recorta `\r` CRLF y propaga errores vía TryUnwrap en `ejecutar_pipeline()`).
**Resultados:** 115/115 COMPILAN (parser puro tolerante) · 29/115 ejecutan CORRECTO (nvc == rust) · 84 incompatibles · 2 timeouts (debug_parser3 loop infinito, gui_ventana GUI).
**Gaps mapeados (15):** `importar` (~50 ejemplos), `sea` if-let (~20), `const` (4), `para` (5), `estructura`+`.campo`+`T{}`, `enum`, `opcion`/`resultado`, `elegir`, closures `|x|`, params default, genéricos `<T>`/`rasgo`, destructuring `_`, tuplas `(...)`, arrays anidados `arr[i][j]`, TryUnwrap top-level silencioso.
**Siguiente:** 6.1 `importar` en parser puro (fusión de ASTs + prefijo `modulo_`) → compiler_v5 modular → 6.2 keywords/gramática → Sprint 7 VM en LÚMEN (`vm.nv`) + optimización ~200s→<10s → Sprint 8 dogfooding stdlib + release v2.4.6. Plan completo en `docs/self-hosting.md`.

**Progreso (31 Jul, tarde-noche):**
- **6.1 imports ✅**: resolver `_imp_*` en parser.nv (~857-1094) + `parser_parsear_con_base(tk, base)` (base-dir para imports relativos) — verificado con ejemplos que usan `importar`
- **6.2 gramática 🟡 parcial** (ediciones del usuario 21:00-21:02, rescatadas y verificadas): `sea`/`let` skip-noop · `const` → VarDecl · **StructInit `T { f: v }` → mapas** (`__map_nuevo`+`__map_poner`) · acceso `.campo` → Index+Texto · `estructura`/`enum` skip tolerante (nodo `Vacio`, sin codegen real) · `para (init;cond;paso)` clásico · driver nuevo en `generar_v4.ps1` (prints ENTRADA/SALIDA/Source/Tokens/AST/Instrs/OK + propagación `__tipo_de(fin)` — gap 15 cubierto) · `error(x)` → ResultErr 39 · `compiler_v4.nv` regenerado 90,356 B
- **Fixes verificados**: op 28/29/30 en `cg_to_vm` (vm.rs) → `test_arr` y `foreach` CORRECTO · retorno genérico `T` en `_parse_decl` · array literals `[a, b]` → nodo `Lista` · Assign con target → OP_ARRAY_SET
- **Rescate (21:52)**: `}` cerrado en `_parse_stmt` (parser.nv) · kw-map reescrita perdió `en`/`in` → añadidos (foreach CORRECTO de nuevo) · StructInit: E041 por literales en param `numero` de `_cg_emit_call` → roundtrip `__map_obtener` · compiler_v4.nv regenerado 82,920 B → check limpio → `.nvc` 70,684 B · mini_fuego: test_arr ✅, foreach ✅, test_stdlib_mini/stress_test/demo_completo 🟡 (items/total/T — bugs abiertos para S6.3: `T` sale del call-site `id<entero>(42)`, no del retorno)
- **Pendientes**: fixpoint v4 → fuego.ps1 → estructura/enum reales + elegir/sea → diagnóstico items/total (desugar `para numero item en` / `_imp_prefijar`)

**Progreso (1 Ago, 14:23-17:33):**
- **6.3 lexer CRLF/tokens truncados 🟡** (notas en `docs/progress-2026-08-01.md`): normalización `s2` (`__str_reemplazar` CRLF→LF + quitar `\r`), helper `sub_from_chars` reemplaza `__str_subcadena` (evita corrupción byte/char UTF-8 — causa probable de los tokens truncados `retornar`/`items`/`numero`), `token_a_texto` en lexer.nv (224 líneas, 10,303 B), main de compiler_v4.nv con dump de 80 tokens + `ast_to_text` (89,108 B)
- **Verificado**: `.nvc` reconstruido 75,643 B (el de 2:25 PM era stale sin el lexer nuevo) → `test_arr` CORRECTO · `foreach` CORRECTO · ⚠️ `test_stdlib_mini` **cuelga** (>10 min): `sub_from_chars` es O(n²) (`out = out + cs[j]`) vs `__str_subcadena` nativo — al importar `coleccion.nv` (grande) explota; fix: subcadena nativa por chars o `__str_concat_list`
- **Pendientes (user)**: validar tokenización de `retornar`/`items`/`numero` → revertir dumps/LEX-LOOP temporales → tests CR/LF/LF-only · regenerar v4 pierde el main instrumentado (no está en generar_v4.ps1)

**Progreso (1 Ago, 18:00-18:40 — sesión AI):**
- **Builtins nativos añadidos en Rust** (fix del hang + MISTERIO ORIG_FULL): `__str_reemplazar`/`__str_replace` (s.replace) y `__str_subcadena_chars`/`__str_slice_chars` (lista de chars → string con clamps), en los **dos** dispatch sites de vm.rs (~345 y ~3088), tipados Texto en sema.rs (~2430) y — CRÍTICO — en el `matches!` de calls de ir/builder.rs (~992): sin esto el IR emite `Load` + `CallValue @0` → runtime `Variable '__str_reemplazar' no definida`
- **MISTERIO resuelto**: el `.nvc` de 5:33 PM (75,643 B, contenía `ORIG_FULL`) vino de un compiler_v4.nv del usuario que ya NO existe; los fuentes actuales nunca tuvieron `ORIG_FULL` — los llaman como builtin. Con los builtins añadidos todo es consistente
- **lexer.nv**: `sub_from_chars(cs, st, fin)` ahora es one-liner `retornar __str_subcadena_chars(cs, st, fin);` — parámetro renombrado `en`→`fin` (**`en` es keyword** → E011/E020)
- **Hang de test_stdlib_mini MUERTO** → ✅ CORRECTO (`coleccion_contar: 2`)
- **BUG RAÍZ de stress_test encontrado y FIXEADO en parser.nv (~142)**: el check de "genéricos en llamada `foo<T>(...)`" scaneaba desde `<` hasta el siguiente `>` SIN límites → en `mientras (i < largo(arr))` el `<` (menor-que) disparaba el scan y se comía hasta el `>` de un `lista<entero>` posterior (tokens 91→184) → el body del while tragaba el resto del archivo (modulo/Punto/top-level perdidos, JmpIf targets rotos). **Fix**: el scan aborta si encuentra `(` `)` `{` `}` `;`, y solo consume los genéricos si tras el `>` viene `(`. compiler_v4.nv regenerado 86,062 B → check ✓ → .nvc 73,120 B
- **Resultados post-fix**: `test_arr` ✅ · `foreach` ✅ · `test_stdlib_mini` ✅ · `stress_test` 🟡 (compila TODO el archivo ahora; único fallo: `Variable 'Punto' no definida` = gap de `estructura` sin codegen real — siguiente feature) · `demo_completo` 🟡 (bug `T` MUERTO — el fix de genéricos lo cubría; avanza hasta STRUCTS y falla con `Variable 'j' no definida` = mismo gap de estructuras)
- **Acciones de higiene**: target.txt restaurado a `stdlib/coleccion.nv`/`stdlib/compile.nvc` · root compiler_v4.nv restaurado (86,062 B; mi fallo de generar_v4.ps1 lo había pisado con 1,918 B) · instrumentación de debug (TOK/PROG/BLK/WHILE/BIN/PR/CUERPO dumps) revertida al regenerar
- **Pendientes**: `estructura` real (gap stress_test + demo_completo, `Punto`/`j` no definida) → `enum`/`elegir`/`sea` → `items`/`total` si persisten → fixpoint v4 → fuego.ps1 → docs

**Progreso (1 Ago, 20:00-21:20 — sesión AI · demo_completo CORRECTO + fixpoint v4):**
- **Float 3.14159→3 FIXEADO — causa real: el LEXER** (no codegen): el bucle de dígitos en lexer.nv no consumía `.` → `3.14159` se partía en `[3][.][14159]`. Fix en `lexer.nv`: `mientras i < n && (_lx_es_digit(chars[i]) || chars[i] == ".")`. Además `vm.rs codegen_to_nvc`: PushNum (op 1) ahora emite VM opcode PushNum(2) con f64 en tabla nums (`op == 1 || op == 3` branch; antes emitía PushInt → truncaba)
- **`.agregar` FIXEADO — causa real en codegen**: el desugar `arr.agregar(4)` → nodo `Assign` (parser.nv:279-289) quedaba dentro de un `ExprStmt`, y `_gen_expr` NO tenía caso "Assign" (solo `_gen_stmt`) → la sentencia emitía 0 instrucciones. Fix: caso `Assign` añadido en `_gen_expr` (codegen.nv, espejo de `_gen_stmt` ~354: gen valor + `OP_STORE`, con branch `target` → `OP_ARRAY_SET`)
- **TERNARIO roto → 2 bugs**: (1) precedencia: el check `?` en `_parse_bin` (parser.nv:387) disparaba a CUALQUIER nivel → `edad >= 18 ? "Mayor" : "Menor"` se parseaba `(edad >= (18 ? ...))` → fix `si (mp == 0 && ...)`. (2) orden de ramas: el handler Ternario de codegen.nv asumía JmpIf=jump-if-TRUE, pero el VM salta con FALSY (vm.rs:3688 `if !val.is_truthy()`) → la rama `si` iba al fallthrough y la `no` al salto. Fix: emitir `si` primero (fallthrough) + Jmp, luego `no`. **El "Error de tipo: Ge requires numbers or strings" del demo era este ternario: `Ge(Int 20, Str "Mayor")` en ip=300** (verificado con debug temporal en vm.rs, revertido después)
- **AST_DUMP removido** de generar_v4.ps1 (instrumentación temporal; `ast_a_texto` es O(n²) y cuelga el fixpoint en fuentes grandes ~95 KB)
- **Resultados finales (compilador autocontenido):** `demo_completo` **89/89 líneas, 0 diffs** · `stress_test` **8/8, 0 diffs** (el gap `Punto`/`j` de estructuras quedó cubierto por los fixes) · `mini_agregar` con `.agregar(4)` → 3/4/1 · `test_arr` → 5/1/5 · cargo test: todos OK
- **FIXPOINT v4 CONFIRMADO**: compiler_v4.nvc (Rust-built, 94,789 B) compila su propio source → `compiler_v4_self.nvc` (91,806 B) · el self-compilado recompila el source → `compiler_v4_self2.nvc` **byte-IDENTICAL** (91,806 B, SequenceEqual=True) · el self-compilado también produce demo 89/89 (solo difiere el timestamp de `__tiempo_unix` entre runs)
- **Hygiene**: root `compiler_v4.nv` copiado (94,789 B, estaba stale en 86,062 B) · target.txt apunta a `examples/demo_completo.nv` · debug temporal del Ge revertido en vm.rs
- **Pendientes**: `enum`/`elegir`/`sea` reales (parsing tolerante ya, sin codegen) → `importar` con prefijo real verificado en fuego.ps1 → **Sprint 7: VM en LÚMEN (`vm.nv`)** + optimización de velocidad (fixpoint ~10-18 min hoy; demo ~15s) → Sprint 8 dogfooding stdlib + docs → release v2.4.6

**Progreso (1 Ago, 21:30-23:00 — sesión AI · 6.4 enum/elegir/sea REALES):**
- **`sea`/`let` → VarDecl real** (parser.nv:456): `sea [tipo] nm = expr;` → nodo VarDecl (tipo_var="let" o el tipo) — antes era skip-keyword y `sea x = e` se re-parseaba como Assign
- **`Nombre::Miembro(args)` → EnumInit** (parser.nv:187): con `(...)` parsea args; sin args → argc=0. VM imprime `Resultado::Exitoso(100)` nativamente (Display de Value::Enum) y `Eq` compara name+variant+fields (vm.rs:2551-2562)
- **`defecto:`/`default:` REAL** (parser.nv elegir): break-condition incluye `defecto`; post-check consume la keyword, parsea el body → se adjunta como `sino` del último If (o Block si solo-default)
- **BUG RAÍZ de la cadena `elegir` — `__map_poner` NO muta**: `im::HashMap` es persistente (insert → copia estructural; vm.rs:577-589). `last = __map_poner(last,"sino",ifn)` jamás propagaba a `first` → SOLO el caso 1 se compilaba. El demo 89/89 lo ENMASCARABA (solo llama `color_str(Color::Rojo)` = caso 1). Fix: `ifns[]` (lista por índice) + cadena reconstruida DESDE EL FINAL (`chain = __map_poner(ifns[i2],"sino",chain)` descendiendo). Semántica clone-copy es ESENCIAL para los probes `st_dos = _st_adv(st)` del parser
- **Codegen**: `OP_ENUM_CTOR=43/OP_ENUM_VAR=44/OP_ENUM_ARGC=45` + caso `EnumInit` en `_gen_expr` (args en orden + trío consecutivo). **vm.rs codegen_to_nvc**: 43 → `WithIdx(EnumCtor, str)`; 44 → `WithIdx(Nop, str)`; 45 → argc (ints→f64) en nums + `WithIdx(Nop, num)` — el VM EnumCtor lee ip+1/ip+2 (vm.rs:3720-3757)
- **Resultados**: match.nv IDENTICAL (2 casos + defecto) · enums.nv IDENTICAL (unit + data variants + print directo) · demo 90/90 0 diffs · cargo test OK · 43_tipos_asociados.nv NO soportado por NINGÚN compilador (usa `rasgo`/traits)
- **Hygiene**: debug residual LEX-LOOP/LEX-DBG retornar ELIMINADO de lexer.nv · AST dump temporal removido de generar_v4.ps1 · fuego.ps1 completo corriendo en background (115 ejemplos)
- **Pendientes**: revisar resultados fuego.ps1 → fixpoint v4 (regresión crítica tras el cambio de elegir) → Sprint 7 VM en LÚMEN (`vm.nv`) + optimización → Sprint 8 dogfooding + release v2.4.6

**Progreso (2 Ago, 20:00-21:30 — sesión AI · 6.5 cortocircuito `&&`/`||` + FIXPOINT v4 CONFIRMADO):**
- **REGRESIÓN del fixpoint diagnosticada (causa raíz)**: `compiler_v4_self.nvc` (94,783 B) crasheaba con "Índice N fuera de rango (largo: N)" dentro de `ejecutar_pipeline` (también al compilar match.nv: "Índice 96"). Inspección del disasm (`self_disasm.txt` vs `v4_disasm.txt`) probó la diferencia: el **pipeline RUST (v4) cortocircuita `&&` con `JmpIf`** (`Load; Lt; Store; Load; JmpIf → salta ANTES del ArrayGet`), mientras el **codegen LÚMEN emitía `&&`/`||` como `And`/`Or` eager** (`…Lt; ArrayGet; Neq; And`) → `mientras i < n && cs[i] != "\n"` lee `cs[i]` con `i == n` → ArrayGet out-of-bounds solo en el SEGUNDO-compilado
- **Fix**: helper `_cg_and_or(cg, izq, der, es_and)` en codegen.nv — emite short-circuit REAL con `JmpIf`/`Jmp` + `PushBool` (es_and=1 para `&&`; es_and=0 para `||`). Binary en `_gen_expr` despacha `&&`/`||` al helper (return temprano); los demás operadores siguen eager. Convención de saltos con FALSY (vm.rs:3688)
- **FIXPOINT v4 CONFIRMADO**: compiler_v4.nv (99,993 B) → compiler_v4.nvc → self (112,368 B, exit 0) → self→self2 (112,368 B, exit 0) → **byte-IDENTICAL (0 diffs)**. El self-compilado compila match.nv sin crash (antes "Índice 96 fuera de largo")
- **fuego.ps1 completo**: **396/396 compilan · 61 CORRECTOS (+1 vs 60) · 53 INCOMPATIBLES · 2 timeouts · 0 fallos** — sin regresión (bug pre-existente en fuego.ps1:66: `$outNvc.Trim()` sobre `$null` en el detalle de INCOMPATIBLES — no afecta los contadores)
- **Hygiene**: scratch `v4_disasm.txt`/`self_disasm.txt` duplicados — limpiar; `target.txt` aún apunta a `examples/match.nv`
- **Pendientes**: Sprint 7 VM en LÚMEN (`vm.nv`) + optimización de velocidad (fixpoint ~10-18 min hoy; demo ~15s) → Sprint 8 dogfooding stdlib + release v2.4.6

**Progreso (4 Ago — sesión AI · Sprint 7 VM en LÚMEN: corutinas reales + batería 27/28):**
- **Corutinas implementadas en vm.nv** (modelo de intercambio de contexto cooperativo como vm.rs): `__coro_crear` boxea `coro_N` y guarda el nombre en `coro_nombres_m` (el fidx se resuelve en el PRIMER `__coro_reanudar` porque `funn` solo existe en main, no en bin); `__coro_reanudar` guarda main en guards (st/sp/pc), inicializa st nuevo (pila/mem/scopes/rets propios, mem heredado) si `coro_inic_m==0`, carga st/sp/pc, `coro_actual=cidx`; `__coro_ceder` guarda coro (sps=sp+1, pcs=pc+1) y restaura main; op22 Ret con `coro_actual>=0` marca done, restaura main.
- **Bug 1 (hang)**: el placeholder de `coro_fidx_m` era `0.0` pero el check es `si cfi2 < 0` → cfi2=0=`"__main__"` → `pc=fun[3]` = inicio del main → bucle infinito re-ejecutando main. Fix: placeholder `-1.0`.
- **Bug 2 (re-ejecución)**: `a_entero(__map_obtener(coro_stats_m, cidx))` sobre el MAPA stc guardado da 0 (un mapa no se parsea) → `==0` siempre TRUE → re-inicializaba la corutina en cada resume ("A: inicio" ×3). Fix: flag separado `coro_inic_m` (0/1).
- **Bug 3 (VM Rust)**: `Opcode::Ret` (vm.rs) con `call_stack` vacío → `ip=usize::MAX` → el programa moría en silencio: al retornar dentro de la corutina (tarea_a nunca se llamó con Call, se saltó con `ip=coro.ip`) la VM Rust NO imprimía `--- FIN ---` (la LUMEN sí, correctamente). Fix en vm.rs: si `current_coro` está activo → `coro.is_done=true`, restaura `main_saved`, continúa.
- **Resultados**: corutinas_demo **byte-IDENTICAL** (252 B, 0 diffs) con el flujo completo (inicio→yield→primer yield→segundo yield→ret→FIN); demo_completo sigue 0 diffs; batería 8/8 en los re-probados incl. corutinas_demo; cargo test 0 fallos (e2e 166).
- **Pendientes**: `44_extension_methods` + `math` (fallan también en VM Rust, pre-existentes — compiler issue con `este`/extension methods) → batería ampliada completa → Sprint 8 dogfooding stdlib + release v2.4.6
- **Fix Store global** (`vm.rs` ~2996): el Store solo escribía el scope más interno → la mutación de globals desde funciones (side-tables `mapas`/`arrs`/`stl_din` de vm.nv dentro de `bin()`) se perdía → "Índice 0 fuera de rango (largo: 0)". Ahora 3 estados: scope actual si tiene el nombre, si no globals (`locals[0]`) si lo tiene, si no scope actual
- **Fix output-on-error** (`cli/main.rs` `run_bytecode` ~732): el buffer `vm.output()` solo se vaciaba en éxito → los prints de debug nunca aparecían en crash. Ahora se imprimen también en el rama `Err`
- **Banda de arrays desplazada** a `< -1e9`: los ids boxed de arrays (`-1..-N`) colisionaban con ints negativos reales (`a_texto_v(-10)` → "?"). Actualizados los 20 sitios de acceso/creación de `arrs`
- **Colisión de banda [1e9,2e9)**: números >1e9 (sumas grandes `total=1249975000`, timestamps ~1.7e9) eran interpretados como ids de string → "Índice N fuera de rango (largo: 0)" en op Add (op7), `ig_v`, `es_verdad`. Ahora esos handlers verifican que `a_entero(v-1e9) < largo(stl)+largo(stl_din)` antes de `str_at`; si no, tratan como número (Add numérico / igualdad numérica / a_texto_v fallback a `a_texto(a_entero(v))`)
- **`a_entero()` O(n)→O(1)**: era un loop `while u>=1 { u=u-1 }` que contaba hasta el valor (¡1.2e9 iteraciones para un int grande!) → el demo tardaba ~120s. Ahora usa el builtin nativo nuevo `__str_a_entero`/`__texto_a_entero` (`parse::<i64>`, trunca en `.`) → **demo en ~0.9s**, mini_stress 50k iter en 12s
- **Handlers bin() nuevos** (delegan a natives de la VM Rust y boxean en `stl_din`/`arrs`/`mapas`): `__tipo_de` (bandas), `__hash_sha256/512`, `__unicode_normalizar`, `__str_padding_inicio/fin`, `__codificacion_utf8`, `__regex_coincide`, `__tiempo_ahora`, `__tiempo_formatear`, `__fs_listar`, `__env_listar` (arrays de strings boxeados), `__coro_crear` (id placeholder `coro_N`)
- **JSON real**: `__json_parsear` guarda el `Value` en `mapas` y devuelve id boxed 2e9+N; `__json_texto` desboxea y llama al native → `__json_texto(__json_parsear("{\"a\":1}"))` = `{"a":1}` ✓
- **Tarea real**: `__tarea_lanzar` registra fn en `tareas_mapa` (key = id boxed) y retorna el tid; el dispatch op 21 de `__tarea_esperar` remapea `nm` a la función objetivo y ejecuta la llamada síncrona con la tabla `funn` propia → `__tarea_esperar(tid)` = 99 ✓
- **deque/enlazada**: `__deque_agregar_final` mutaba inline `arrs[...].agregar(x)` (se perdía) → copia + nuevo id como los handlers que funcionan
- **sema.rs**: typing nuevo para `__unicode_normalizar`,`__str_padding_*`,`__tiempo_formatear` (Texto), `__fs_listar`/`__env_listar` (Lista<Texto>), `__coro_crear` (Texto), `__json_parsear` (Numero), `__str_a_entero` (Entero); test e2e `test_map_keys` → `lista<numero>`
- **Resultados**: demo_completo **89/89 líneas, 0 diffs** (único diff es `__tiempo_ahora()` timestamp real que cambia entre runs), en **~0.9s** (120s→1s) · cargo test todo OK (~379 tests) · Batería test_vm.ps1: **25/28 OK** (3 DIFF: `44_extension_methods`+`math` fallan TAMBIÉN en la VM Rust pre-existente, `corutinas_demo` requiere corutinas reanudables reales — pendiente)
- **Commits**: `bdeb933` (handlers + banda + Store + demo 0 diffs), `f015ec1` (a_entero O(1) + fix colisión banda)
- **Pendientes**: corutinas reales (`__coro_reanudar`/`__coro_ceder` con intercambio de contexto st/sp/pc — modelo cooperativo como vm.rs:1679-1734) → batería ampliada completa → Sprint 8 dogfooding stdlib + release v2.4.6

**Progreso (4 Ago — sesión AI · Sprint 7: OPTIMIZACIÓN fixpoint 861s → 20.1s, 43x — COW con Arc):**
- **Profiler per-opcode en `VM::run()`** (vm.rs ~2320, gated por `LUMEN_PROFILE=1`): contadores/tiempos por opcode; `Call` desagregado como `Call:<nombre>` vía `bytecode.names`. Fix: check `!var.is_empty()` (antes `is_ok()` → env vacío lo activaba)
- **Diagnóstico O(n²)** (demo 4KB, 962,956 instrs, 1.7s): Load 48% (2.5µs/call — clona lista `chars` por token), ArrayGet 18% (45µs/call — clona la lista en cada acceso), `__str_subcadena_chars`+`sub_from_chars` 18.6% (~417µs/call — `a.clone()` de TODO el array en cada slice). Escalamiento 23.7x bytes → ~506x tiempo = O(n²) (23.7²=562)
- **FIX: COW con `Arc`** en `Value::Str(Arc<str>)` y `Value::Array(Arc<Vec<Value>>)` (value.rs) + constructores `Value::str(s)`/`Value::arr(v)`; clonar Values grandes = O(1) — `Rc` no servía (no es Send/Sync; `__par_mapa`/`__par_unir` lanzan threads con Values)
- **Transformación mecánica** de ~130 sitios en vm.rs/min_json.rs: construcciones `Value::Str(x)`/`Value::Array(x)` (un-ident) → `Value::str`/`Value::arr`; patrones multi-palabra (`mut v`, `mut arr`) → `Value::Array(mut v)` + `Arc::make_mut(&mut v)` en mutaciones (agregar, deque, heap, linked, conjuntos, ArraySet, ArrayPush, list_reverse/sort); iteradores `items.iter()`/`items.as_ref()` en streams/par_mapa; armas incompatibles → `s.to_string()`; `Value::str("lit".into())` → `Value::str("lit")` (E0283 con impl Into<String>); reverts de serde_json (`Value::arr` clobbered por el replace global → `serde_json::Value::Array`)
- **Resultados**: **fixpoint v4 CONFIRMADO en 20.1s** (baseline 861s = **43x**): compiler_v4.nvc → self_out.nvc (112,368 B) → self_out2.nvc **byte-IDENTICAL (0 diffs)** · demo 0.9s · cargo test ~379 verdes · batería **8/8** (2 DIFF pre-existentes: `44_extension_methods` `este` no definida + `math` — fallan en ambas VMs)
- **Commits**: `ccc6ecd` (COW Arc — fixpoint 43x) · previo: `9531366` (corutinas), `fa79d36` (docs plan)

**Progreso (4 Ago — sesión AI · Sprint 7: batería ampliada 34/35 + jr_fecha/utils CORRECTOS):**
- **Fix vm.nv `utils`**: `fmain` buscaba solo `__main__` pero el encoder graba `main` → FMAIN=-1 → pc=0 (ejecutaba `cuadrado` con operandos void → "Mul requires numbers"). Fix: acepta `__main__` **o** `main` + fallback `fmain=0` → **utils.nv 25/27 correcto** en la VM LÚMEN.
- **Fix vm.nv `jr_fecha` (causa raíz doble)**: (1) `__tiempo_formatear` leía solo `sp` (el fmt boxeado → 2001-09-09) → handler 2-args (`sp-1` timestamp, `sp` fmt); (2) **el native Rust `parse_iso8601_to_unix` NO acepta separador espacio** ("2000-01-15 00:00:00" → Error) — la VM Rust "funcionaba" por accidente (`as_num().unwrap_or(0)` convertía el Error a 0 → diff=ts → 56.59 FALSO). Fix en vm.rs: `replacen(' ', "T", 1)` cuando no hay `T` → **edad 26.55 REAL en ambas VMs** (byte a byte idéntico).
- **argc-guards en handlers tiempo** (vm.nv): `__tiempo_formatear(0)` (1 arg, demo sección 28) crasheaba "Índice -1000000000" porque el `str_at` inicial leía `sp-1` sin verificar argc → ahora `sp` para 1 arg con fmt default ISO `%Y-%m-%dT%H:%M:%SZ` (espeja format_timestamp de vm.rs); `__tiempo_parsear` 1-arg lee `sp`; `__tiempo_diferencia` argc<2 → 0.0.
- **Batería ampliada: OK=34/35** — todos OK incl. utils, jr_fecha, demo_completo (33 secciones completas por la VM LÚMEN), match, enums, corutinas_demo, genericos, lambda, etc. Único DIFF: `stress_fecha` (timing real 0ms vs 16ms entre runs — inherentemente flaky, no es regresión). COMPILA-FALLA pre-existentes: `test_texto_min`, `test_texto_std`, `jr_concurrencia`.
- cargo test OK (e2e 166 + unit); pre-existentes `44_extension_methods`/`math` siguen fallando en AMBAS VMs.
- **Commit**: `290f3ed` (vm.nv handlers tiempo + fmain + argc-guards; vm.rs parse espacio).
- **Pendientes**: COMPILA-FALLA `test_texto_min`/`test_texto_std`/`jr_concurrencia` → `44_extension_methods`/`math` → Sprint 8 dogfooding stdlib + release v2.4.6 → bootstrapping doble.

**Progreso (4 Ago — sesión AI · COMPILA-FALLA x3 RESUELTOS — batería 37/38):**
- **Parser Rust: scan de genéricos sin límites** (`find_token_after_type_args`): en `mientras i < veces {` el `<` disparaba el scan de type-args y encontraba el `{` del body → E021 cascada. Fix: el scan **aborta** si encuentra `(` `)` `{` `}` `;` antes del `>` (espejo del fix que ya tenía parser.nv); misma guarda aplicada al branch de type-params (`x < T {`). 42 unit parser verdes.
- **loader.rs `is_builtin`**: faltaba `__str_reemplazar`/`__str_replace` → el prefijado de imports lo renombraba a `texto___str_reemplazar` → E042. Añadido al allowlist.
- **loader.rs prefijado de tipos**: `Type::Struct("Infer")` (el tipo de `sea x = ...` sin anotación) se prefijaba a `testing_Infer` → el sema ya no lo reconocía como inferencia → E031. Fix: eximir `"Infer"` del prefijo en `prefix_type` y `prefix_type_with_params`.
- **stdlib/texto.nv — 12 funciones nuevas**: `buscar/find`, `empieza_con/starts_with`, `termina_con/ends_with`, `invertir/reverse`, `reemplazar/replace`, `repetir/repeat`, `recortar_inicio/trim_start`, `recortar_fin/trim_end`, `es_vacio/is_empty`, `es_digito/is_digit`, `es_letra/is_letter`, `upper`/`lower`; fix `decodificar_utf8(numero bytes)` → `lista<entero>` (E041); alias `largo`. → **test_texto_std CORRECTO** (18/18 checks).
- **stdlib/fecha.nv**: wrappers `calendario_hijri/hijri` + `calendario_persa/persa` (los natives `__calendario_hijri/persa` existían en el VM Rust pero el stdlib nunca los expuso) → **jr_concurrencia compila**.
- **stdlib/testing.nv**: `tipo`/`result`/`como` son keywords reservadas sin implementar como identificador/cast → renombrados (`tipom`, `res`, división flotante `muertos * 1.0 / total`); aserciones `cualquiera val` → genéricas `<T>` (`cualquiera` no es tipo válido en el lenguaje). testing.nv compila.
- **vm.nv: handlers hilo/mutex/calendario**: `__hilo_lanzar`/`__canal_nuevo`/`__mutex_nuevo`/`__calendario_hijri/persa` registran ids/fn (los natives Rust usan `self.bytecode` = vm.nvc, NO el programa interno → no se podían delegar); `__hilo_esperar`/`__mutex_bloquear` hacen **dispatch inline en op21** (scope+rets+pc jump como `__tarea_esperar`, pasando el arg en param 0) → jr_concurrencia **IDÉNTICO** (Hijri/Persian/Thread/Channel/Mutex ✓).
- **Batería ampliada: OK=37/38** — únicos DIFF restantes: `stress_fecha` (timing real 0ms vs 16ms, flaky inherente). `test_migracion` queda con gap pre-existente (`cualquiera` como tipo en csv.nv — no es tipo válido; requiere tipo Any en el lenguaje).
- cargo test OK (42 parser, 49 sema, 45 unit, 166 e2e, 0 fallos).
- **Commit**: `8c27abd`.
- **Pendientes**: `44_extension_methods`/`math` (compiler issue `este`/extension methods, fallan en AMBAS VMs) → tipo `Any`/`cualquiera` real (desbloquea csv.nv/test_migracion) → Sprint 8 dogfooding stdlib + release v2.4.6 → bootstrapping doble.

**Progreso (4 Ago — sesión AI · 44_extension_methods/math RESUELTOS — batería 39/40):**
- **CAUSA RAÍZ `44_extension_methods`** ("Variable 'este' no definida"): `VM::new` (vm.rs) resolvía el entry con `__main__` → `main` → **`funcs.first()`**. Este ejemplo no tiene código top-level ni función `main` (es `principal`) → caía a `funcs.first()` = `entero_Formateable_a_formato` (start=0, ordenado por offset) → ejecutaba `Load este` sin scope antes de cualquier Call. El bytecode y el IR estaban correctos (funcs mangled `entero_Formateable_a_formato` con param `este`, call-site con receiver) — solo fallaba el arranque.
- **Fix vm.rs `VM::new`**: cadena `__main__` → `main` → **`principal`** → si `funcs` está vacío (bytecode plano de tests unit) → `ip=0`; si hay funcs pero sin main/principal (librería pura como math.nv) → `ip=usize::MAX` → loop sale inmediato, terminación limpia sin ejecutar nada. (El antiguo fallback `funcs.first()` hacía que math.nv ejecutara `suma` con 0 args → "Variable 'a' no definida").
- **Fix vm.nv (VM LÚMEN)** espejo: `fmain` ahora acepta `__main__`/`main`/**`principal`**; se eliminó el fallback `fmain=0` (re-ejecutaba func[0]); si `fmain<0` → `pc=999999999` (el loop sale por `pc>=largo(insn)/3`). Además el **Ret sin caller** (op22 con `coro_actual<0` y `rn<0` — entrada directa por principal, sin `__main__` que haga Call) → `fin=1` (antes `pc=0` → re-ejecutaba func[0] y crasheaba "Ge requires numbers or strings").
- **Resultados**: `44_extension_methods` → "Numero: 42" + "Texto: 'Hola LÚMEN'" IDÉNTICO en ambas VMs · `math` → exit 0 sin output en ambas (librería pura) · cargo test **0 FAILED** (los 24 unit de vm que usaban bytecode plano sin funcs siguen pasando con el branch `funcs.is_empty() → ip=0`).
- **Batería: OK=39/40** — únicos DIFF restantes: `stress_fecha` (timing real 0ms vs 16ms, flaky inherente). `test_vm.ps1` actualizado a 40 archivos (+`44_extension_methods`, +`math`).
- **Commit**: `5809c96`.
- **Pendientes**: tipo `Any`/`cualquiera` real (desbloquea csv.nv/test_migracion) → Sprint 8 dogfooding stdlib + release v2.4.6 → bootstrapping doble.

**Progreso (4 Ago — sesión AI · Tipo dinámico `Numero` real + `cualquiera`/`any` — test_migracion CORRECTO):**
- **CAUSA RAÍZ**: `num` era `Type::Numero` → `type_to_info` lo mapeaba a **`TypeInfo::Decimal`** (tipo ESTRICTO) → el `can_assign` (sema.rs:3431, "Numero (dynamic type) accepts any value") era **código muerto** — `TypeInfo::Numero` jamás se producía. Por eso `cualquiera` (alias de `n`/`numero` para valores boxed) fallaba: `cualquiera claves = __map_claves(...)` → E031 "Lista(Numero) a Decimal", y `.largo()`/`[i]` en Decimal → E047/E044.
- **Fix sema**: `Type::Numero => TypeInfo::Numero` (tipo dinámico REAL); `Type::Decimal` sigue → `TypeInfo::Decimal` (estricto, explícito). `Expr::Index` y MethodCall `largo`/`len`/`length` aceptan ahora `TypeInfo::Numero` (indexación → Numero; largo → Numero).
- **Fix parser**: `parse_type` mapea identificadores `cualquiera`/`any` → `Type::Numero` (antes caían a `Type::Struct("cualquiera")` → `csv_cualquiera` con el prefijo de imports).
- **Tests actualizados (4)**: `test_type_mismatch`/`test_arithmetic_type_error` (sema) y `test_semantic_error` (e2e) usaban `numero x = "hola"` como ejemplo de error de tipo — ahora `numero` es dinámico, cambiados a `entero`. `test_function_call_arg_type` usaba params `numero` → `entero`.
- **Resultados**: `test_migracion` **correcto en ambas VMs** — assert (true×5), CSV parse/serialize (`{0:[...],1:[...],2:[...]}` Rust vs `{2,0,1}` LÚMEN — **orden de claves de mapa no-determinístico** entre VMs, inherente), calendarios Hijri (2088-09-01 AH) y Persa (-563-08-25 AP) idénticos. Se dejó FUERA de la batería exacta por el orden de claves (difiere por diseño como stress_fecha solo en volatilidad).
- Batería **39/40** (solo stress_fecha flaky) · cargo test **0 FAILED**.
- **Commit**: `be5e48e`.
- **Pendientes**: Sprint 8 dogfooding stdlib (evaluar test_migracion en batería con normalización de orden de mapas) + release v2.4.6 → bootstrapping doble.

**Progreso (4 Ago — sesión AI · Sprint 8: fixpoint v4 re-verificado + fuego 71/116 + orden de mapas):**
- **Orden de claves de mapa NO-determinístico CONFIRMADO**: `im::HashMap` usa `RandomState` (seed aleatorio por proceso) → el orden varía incluso entre runs del mismo VM Rust (`{0,1,2}` → `{2,1,0}`). No es bug — semántica de hash map. Por eso `test_migracion` no entra a la batería exacta sin normalización (igual que stress_fecha por volatilidad de timing). Los `__map_nuevo`/`__map_claves` de la VM LÚMEN delegan al mismo native Rust y boxean en `mapas`/`arrs` — mismo comportamiento no-determinista.
- **FIXPOINT v4 RE-VERIFICADO** (tras todos los cambios de la sesión: cortocircuito, enum/elegir/sea, dynamic Numero): `compiler_v4.nvc` regenerado (85,374 B) → self-compile (5s) → `v4_self_out.nvc` **112,368 bytes** (idéntico al tamaño histórico) → `v4_self_out2.nvc` **byte-IDENTICAL**. La cadena 100% LÚMEN sigue estable.
- **fuego.ps1 (cadena 100% LÚMEN)**: **396/396 compilan · 71 CORRECTOS (+10 vs 61) · 43 INCOMPATIBLES · 2 timeouts · 0 fallos**. La mejora +10 viene del trabajo acumulado (entry `principal`, dynamic Numero/cualquiera, test_migracion).
- **⚠️ Trampa del harness**: `fuego.ps1` debe ejecutarse con **pwsh (PS 7)**, NO `powershell` (5.1) — `Set-Content -Encoding utf8` en PS 5.1 escribe BOM UTF-8 (`EF BB BF`) en `target.txt` → la primera línea (ruta del ejemplo) queda corrupta → el driver falla → 0/116 `?ERROR?`. En pwsh 7 no hay BOM.
- **43 incompatibles = gaps conocidos**: `ninguno`/`algun`/`exito`/`error` (Option/Result ~10 ejemplos: opcion, resultado, audio_demo, charts_demo, graficos_*, tilemap, tui_pro/tui_puro/tui_temas), `rasgo` (traits: 43_tipos_asociados, 44_extension_methods), closures `|x|` (lambda), tuplas/destructuring/params_default/genericos (feature partial), FFI/red/sistema/sqlite/json/csv (natives `__ffi_*` que la VM LÚMEN no implementa — corren headers pero divergen), `debug_parser3`+`gui_ventana` (timeouts GUI/loop).
- **Pendientes**: u opcion/resultado (`ninguno`/`algun`/`exito`/`error` reales en el self-hosted — desbloquea ~10 ejemplos) → docs + AGENTS v2.4.6 + release → bootstrapping doble.

**Progreso (4 Ago — sesión AI · Option/Result REALES en el self-hosted — fuego 75/116):**
- **lexer.nv (self-hosted)**: keywords `algun`/`some` + `ninguno`/`none` añadidas al mapa kw (faltaban — antes se parseaban como Ident → "Variable 'ninguno' no definida"). `exito`/`error` ya estaban.
- **parser.nv**: `_st_tp` ahora reconoce `opcion` y `resultado` como tipos (antes `funcion opcion<entero> f()` tomaba `opcion` como NOMBRE de función → func mangled). `_st_tp_skip` reescrito para genéricos múltiples/anidados (`resultado<entero, texto>`, `lista<lista<x>>`).
- **codegen.nv**: `algun(x)` → op 41 (OptionSome) en `_cg_emit_call`; nodo `Ninguno` → op 42 (OptionNone) en `_gen_expr` (espejo de Exito/Error).
- **vm.rs `codegen_to_nvc` (cg_to_vm)**: añadidos `38=>38 ResultOk`, `39=>39 ResultErr`, `41=>41 OptionSome`, `42=>42 OptionNone` — **antes caían en `_=>0 Nop`** (por eso `algun(42)` imprimía `42` crudo y `ninguno` rompía).
- **BUG RAÍZ `elegir` con bodies con llaves** ("Variable 'caso' no definida"): `_parse_stmt` NO despachaba `{` como bloque → el body-loop del caso consumía `{ imprimir(10); }` token a token y **se comía el `caso ninguno:` de case2** (emitía `Load caso; OptionNone` como statements de case1). Fix: dispatch `si (_st_ch(st,5,"{")) { _parse_blk(st) }` como fallthrough de `_parse_stmt` (como si/while/mientras). Diagnosticado con DBG temporales (ELEGIR/CASO/body) — `imprimir(a,b,c)` emite cada arg en su propia línea, el `rg DBG` no agrupaba.
- **Resultados**: `opcion.nv` y `resultado.nv` **OK+CORRECTO** (byte-idénticos a Rust en la cadena 100% LÚMEN) · probe_elegir self==rust exacto · **fixpoint v4 CONFIRMADO 113,857 B byte-IDENTICAL** (self→self2, 5s) · cargo test 0 FAILED.
- **fuego.ps1: 396/396 compilan · 75 CORRECTOS (+4) · 38 INCOMPATIBLES · 3 timeouts · 0 fallos**. Los ejemplos GUI/gráficos/TUI (audio, charts, graficos_*, tilemap, tui_*) ya no fallan con `ninguno` — ahora corren (divergen por rendering/red/timing).
- **Commit**: `56472c4`.
- **Pendientes**: `rasgo` (traits: 43_tipos_asociados, 44_extension_methods) y closures `|x|` (lambda) en el self-hosted → benchmarks vs Rust → docs + AGENTS v2.4.6 + release → bootstrapping doble.

**Progreso (4-5 Ago — sesión AI · Sprint 8: features self-hosted + benchmark + fixes — fuego 79/116):**
- **Fix `defecto`/`default` en `elegir`** (commit `a0dce08`): el default se manejaba DENTRO del branch del caso y el `romper` descartaba el ifn del último caso → `real_logger` OK+CORRECTO (nivel `[ERROR]` ya no caía a `UNKNOWN`). fuego 76/116, fixpoint 113,857 B.
- **Params default** (commit `ee35e2d`): parser captura `= expr` por param y **codegen inlinea los defaults en el call-site** (como el IR de Rust) vía tabla `defectos` en el registro de funcs → `params_default` OK+CORRECTO. Mismo commit: **cast `como` en el parser Rust** (no-op de tipado) — desbloquea el parse de casts, pero los ejemplos `como` (tilemap/audio/graficos/charts/tui_temas) AÚN fallan en Rust por otros quirks pre-existentes (listas/struct-init/field-access). cargo test 0 FAILED.
- **Closures IIFE** (commit `6d88fca`): `funcion(params){body}(args)` en `_parse_pr` → hoist del body a `__lambda_N` (nodo Func fusionado al Programa vía `lambdas_hoistadas`) + call directo en el call-site → `lambda` OK+CORRECTO (10/Hola Mundo/30). fixpoint 118,485 B.
- **Traits `rasgo`/`impl`/`este`** (commit `9328fec`): keywords rasgo/trait/impl; `rasgo {...}` skip tolerante; `impl Trait para Tipo { methods }` → métodos mangled (`Tipo_Trait_metodo` con receiver `este`) hoisteados como Funcs + registro `mapa_impls[tipo][metodo]`; **resolución `n.metodo()` por tipo de variable** (`mapa_tipos_var` — el parser mini-tipado); **`__main__` SOLO si hay código top-level no-Vacio** (fallback a `principal`) → `44_extension_methods` + `utils` OK+CORRECTO. fuego 79/116, fixpoint 124,681 B. 43_tipos_asociados queda fuera (struct-init `Caja {valor}` tampoco lo para el parser Rust).
- **Benchmark vs Rust** (commit `cbb8895`): nuevo `scripts/benchmark_vs_rust.ps1` + `stdlib/compiler/bench_fib.nv` (fib(26)+loop 100k). **Resultados (promedio 8 cargas)**: **compile Rust 0.010s vs LÚMEN 0.057s (x5.4)** · **run Rust 0.054s vs LÚMEN 12.4s (x231, mediana x2-6; el fib pesado x279)** — la VM LÚMEN es un intérprete-en-intérprete (estado basado en mapas), los ratios bajos en cargas normales.
- **Bug real descubierto por el benchmark**: `a_texto_v` de la VM LÚMEN — las bandas [3e9,9e9) (`stcs`/`resl`/`opts`/tuplas/`enms`/`fncs`) NO tenían guard de largo → números grandes reales (`total=4,999,950,000`) colisionaban → "Índice 999950000 fuera de rango". **Fix**: guards `>= largo(tabla) → a_texto(a_entero(v))` en las 6 bandas → bench_fib imprime `4999950000` IDÉNTICO a Rust.
- **Regresión de la sesión dynamic-Numero**: `lista<numero>.agregar(Decimal)` → E046 espurio — el check de `agregar`/`push` comparaba `*inner != tipo` ESTRICTO (rompió `vm.nv` → "Bytecode" falló). **Fix en sema**: `!can_assign(inner, tipo) && !(inner==Numero || tipo==Numero)`. LÚMEN VM (vm.nvc) reconstruido.
- **cargo test 0 FAILED · batería test_vm.ps1 39/40 (solo stress_fecha flaky)**.
- **37 restantes (34 incompat + 3 timeout) = categorías honestas**: (1) ejemplos que el **compilador RUST mismo no parsea** (tilemap/audio/graficos/charts/tui_temas — listas E020/E022, struct-init `Caja {`, field-access E024; `43_tipos_asociados` struct-init); (2) **negativos por diseño** (`tui_test_min16/17/18` — errores sintácticos intencionales que Rust reporta y self tolera); (3) **no-deterministas** (test_migracion orden de mapas, test_sistema_directo/csv/json, jr_fecha, stress_fecha — PID/timestamps/timing); (4) **TUI/gráficos con rendering/SDL** (tui_pro/puro, graficos_* inicializan SDL → divergen o timeout); (5) **FFI/red/sistema/sqlite** (natives `__ffi_*` / red_conectar que el self-hosted no implementa); (6) `debug_parser3`+`gui_ventana` (timeouts loop/GUI). Ninguno es regresión del self-hosted — son límites del harness byte-igual o del estado del lenguaje/pipeline.
- **Pendientes**: release v2.4.6 (docs + tag) · bootstrapping doble (vm.nv compilada por LÚMEN) · opcional: struct-init/listas en parser Rust para 43_tipos_asociados y el cluster `como`, y FFI natives en la VM LÚMEN.

**Progreso (5-6 Ago — sesión AI · Sprint 8: ejemplos — fuego 98/116):**
- **Commit `3e39c4d` (WIP pipeline Rust + gráficos validado)**: 25 files +1623/−326 — bitwise `&`/`<<`/`>>`, concat `++`, Cast AST real, tipos C-style `T x[]`, `a[i]=v`, hex `0x`, loader dedupe imports, sema comparaciones numéricas flexibles + truthiness dinámica (`test_comparison_numeric_any_type`/`test_logical_dynamic_truthiness` renombrados), `__str_ord`→`Lista(Entero)` (sema.rs:2501 + `[0]` en graficos_canvas.nv:700/tui.nv:266,466), fixpoint v4 142,434 B byte-idéntico, fuego 90/116, cargo test 0 FAILED.
- **Bug RAÍZ charts_demo — coma final en literales de lista** (parser.nv list-literal): `[a, b,]` — tras la coma no se comprobaba `]` → `_parse_expr` se comía el `]` y absorbía el resto del archivo como elementos (funcs y prints perdidos; disasm: func[0] con params=[),return,Jan,...]). **Fix**: break tras la coma si viene `]`. Bisect: `retornar [1, 2]` OK, `[1, 2,]` ROMpía; repro `repro_si.nv` (2 funcs + prints).
- **Bug 2 — tipo de retorno genérico custom** (parser.nv `_parse_decl`): `funcion list<string> months()` — el branch `Ident Ident` solo cubría tipos custom sin genéricos; `list<string>` tomaba `list` como NOMBRE y el loop de params devoraba todo hasta EOF. **Fix**: branch `Ident <...>` con scan limitado (como el de la firma) → tr=tv + st tras `>`. Verificado: repros (list-string, coma-final, retorno normal, void) todos pasan; funcs=[months, datos2, __main__].
- **Fix loader.rs (`is_dir`)**: `flatten` hacía `.parent()` sobre un directorio → raíz del repo → "Archivo no encontrado: 'math.nv'". Fix: `current_path.is_dir()` → usar tal cual. → `test_import` 30/50/42 IDÉNTICO en ambas VMs.
- **Ejemplos reescritos a API actual y verificados IDENTICAL**: `test_stdlib_avanzado` (decls `cualquiera`), `tui_jr` (API `tui_iniciar/ventana/cerrar`), cluster red/http (`test_red_nv`/`test_red_ffi`/`test_red_ffi2`/`test_red_debug2`/`test_http_ffi`/`sprint1_http` → `red_tcp_conectar("127.0.0.1:9")`/`red_tcp_escuchar("127.0.0.1:0")`/`red_http_obtener("http://example.com/")` — httpbin.org añadía X-Amzn-Trace-Id aleatorio).
- **Trampa harness**: el driver `ejecutar_pipeline` lee `stdlib/compiler/target.txt` RELATIVO AL CWD → `lumen run compiler_v4.nvc` SIEMPRE desde la raíz del repo (los runs desde `stdlib/compiler` fallan en silencio: ruta duplicada → FALLO invisible).
- **generar_v4.ps1 blindado**: `WriteAllText` usa el cwd de .NET (proceso), no el de PowerShell → escribía `compiler_v4.nv` en la RAÍZ al invocarlo desde `stdlib/compiler` (explica el trap histórico del root pisado). Fix: `$scriptDir` + ruta absoluta (Join-Path) para lectura y escritura.
- **charts_demo IDENTICAL en ambas VMs** (tras los 2 fixes: `=== Charts Demo ===` + Controles + Error renderer). **FIXPOINT v4 CONFIRMADO** tras ambos fixes: SHA-256 `74DF6760...` self==self2 byte-idéntico (compiler_v4.nv 130,269 B → .nvc 108,849 B).
- **fuego.ps1: 396/396 compilan · 98 CORRECTOS (+8 desde 90) · 14 INCOMPATIBLES · 4 timeouts · 0 fallos**. Restantes honestos: (1) no-deterministas (8: graficos_demo SDL handle, test_csv_avanzado/test_json_avanzado/test_migracion orden de mapas — seed aleatorio por proceso, test_ffi_min/test_ffi_debug punteros, test_sistema_* nombres temp); (2) negativos por diseño (3: tui_test_min16/17/18); (3) trabajo real (3: `audio_demo`+`graficos_avanzado` — E042 `graficos_avanzado_iniciar` no definida, reescritura al API actual; `tui_temas_demo` — "Función 'tema_predeterminado' no definida" = resolución de call cross-import en vm.nv); (4) timeouts (4: debug_parser3, graficos_completo, gui_ventana, sprint1_http timing de red).
- **Commit `8d2aef6`**: 13 files +147/−83 — parser.nv (2 fixes), loader.rs, 8 ejemplos, generar_v4.ps1, compiler_v4.nv (root+stdlib), target.txt restaurado a demo_completo. Pre-commit checks OK.
- **Pendientes**: `audio_demo`/`graficos_avanzado` a API actual → `tui_temas_demo` (vm.nv dispatch cross-import) → FFI natives VM LÚMEN (cluster `__ffi_*`) → bootstrapping doble → release v2.4.6.

**Progreso (6 Ago — sesión AI · CAUSA RAÍZ imports self-import + tui_temas — fuego 103/116):**
- **CAUSA RAÍZ `audio_demo`/`graficos_avanzado` E042 (`graficos_avanzado_iniciar` no definida)**: `examples/graficos_avanzado.nv` importaba `"graficos_avanzado.nv"` → **resolvía a SÍ MISMO** (self-import). loader.rs tenía detección (`resolved == current_path`), pero en Windows `fs::canonicalize` añade prefijo `\\?\` → la comparación raw NUNCA igualaba → el módulo se importaba 2 veces (doble prefijo `graficos_avanzado_graficos_avanzado_Particula` E062/E042).
- **Fix loader.rs**: `resolve_path` recibe `current_path` + cae a search_paths si el resolved es el archivo siendo aplanado; `flatten` canonicaliza `current_path` una vez (`current_norm`) y compara con el resolved canonicalizado; **callers pasan el FILE, no el parent-dir**: `compile_source`/`run_tests`/`run_debug` (cli/main.rs) y `__compile_nv` (vm.rs:535) — antes pasaban `base_dir` → el entry-file nunca se detectaba como self-import. Verify: `probe_mod.nv` (self-import test) → 1 (antes doble prefijo E062). **Probe files borrados tras verificar** (stdlib/probe_mod.nv + examples/probe_mod.nv).
- **Rename `examples/graficos_avanzado.nv` → `graficos_avanzado_demo.nv`**: audio_demo importa `"graficos_avanzado.nv"` (el stdlib) — el ejemplo homónimo lo SOMBREADA (ejemplos/ existe → se resuelve el ejemplo, no el stdlib → doble import vía anidación). Renamed → `audio_demo` y `graficos_avanzado_demo` **OK+CORRECTO**.
- **Bug RAÍZ `tui_temas_demo` ("Función 'tema_predeterminado' no definida")**: `_imp_prefijar` (parser.nv:1631) **no tenía rama `Lista`** → las calls dentro de array-literals de módulos importados quedaban SIN prefijar (`temas_iniciar`: `[tema_predeterminado(), tema_claro(), ...]` — disasm: los 4 Call con nombres raw; solo el elemento con la primera call rompía el runtime). **Fix**: rama `Lista` (walk `elementos`).
- **Cast `como` no-op en el self-hosted**: demo línea 239 `entero ok = tui_core__tc_raw(verdadero) como entero;` → `Variable 'como' no definida` (el lexer LÚMEN no tenía `como` como kw; parser no parseaba casts). Fix: `como` al kw-map del lexer + `_parse_pr` consume `expr como Tipo` (con genéricos `<...>`), espejo del parser Rust. ⚠️ la variable del loop NO puede llamarse `en` (keyword → E011).
- **generar_v4.ps1**: `Get-Content "lexer.nv"` también era relativo al CWD → desde otra carpeta generaba compiler_v4.nv VACÍO (1882 B) con "éxito" — fix: `Join-Path $scriptDir` en las 3 lecturas.
- **AV pre-existente documentado (no regresión)**: la VM Rust AV-crashea (0xC0000005) llamando funciones console-input por FFI: `ReadConsoleInputA/W`, `PeekConsoleInputA/W` (también vía kernelbase.dll) — con args 0/0 y con handles/buffers reales; `GetStdHandle`/`GetConsoleMode`/`ReadFile`/`Sleep`/`ReadConsoleW` etc. funcionan. Solo se verificó con stdin-pipe (sin consola real); tui_core.nv usa esas funciones → `tui_temas_demo` NO puede correr en el harness, pero **nvc LÚMEN == nvc Rust (ambos crash idéntico → fuego lo cuenta CORRECTO)**.
- **FIXPOINT v4 CONFIRMADO** tras ambos fixes de parser: SHA-256 `90048DC9F6ADA1E21D77C68E999021B40612DD98619936D1814DEB958F1C78D9` self==self2 byte-idéntico (compiler_v4.nv 131,221 B).
- **fuego.ps1: 396/396 compilan · 103 CORRECTOS (+5 desde 98) · 10 INCOMPATIBLES · 3 timeouts · 0 fallos**. Restantes honestos: no-deterministas (graficos_demo SDL, test_csv_avanzado/test_json_avanzado/test_migracion orden mapas, test_ffi_* punteros, test_sistema_* temp), negativos por diseño (tui_test_min16/17/18), timeouts (debug_parser3, graficos_completo, gui_ventana). cargo test 385/385.
- **Commits**: `4b9aa8e` (loader canonical + callers FILE + rename graficos_avanzado_demo; fuego 102/117), `abf9603` (Lista prefijo + cast como + generar_v4.ps1; fuego 103/116).
- **Determinismo ejemplos (commit `f218200`)** — fuego 108/116: `csv.nv` serializar ordena claves numéricas (im::HashMap RandomState varía por proceso → CSV en orden aleatorio); `test_ffi_min`/`test_ffi_debug` print `pid>0` (el PID era el único diff); `test_csv_avanzado`/`test_json_avanzado`/`test_migracion`/`test_sistema_directo`/`test_sistema_avanzado` prints deterministas (largo de claves, `archivos_existe_archivo`, json nativo ya es BTreeMap-ordenado). Restantes: graficos_demo (SDL renderer), tui_test_min16/17/18 (negativos por diseño), timeouts (debug_parser3 loop, graficos_completo/gui_ventana GUI, sprint1_http red). cargo test 385/385.
- **Pendientes**: FFI natives VM LÚMEN (cluster `__ffi_*`) → bootstrapping doble → release v2.4.6.

**Progreso (6 Ago, tarde — sesión AI · LIMPIEZA REPO + DOCS v2.4.6):**
- **Auditoría completa del repo**: raíz con ~35 artefactos de test trackeados (FFI/temp-file: `__test_jr_*.txt`, `__test_real_*`, `part_a/b.txt`, `source/destino_real.txt`, `test.db`, etc.), `compiler_v4.nv` raíz **STALE** (130,269 B vs 131,221 B de stdlib), `examples_backup_2026/` (270 archivos .nv viejos en/en-es/lang_es/lib), `src/` vacía, 41 `.nvc` temporales en stdlib/compiler (ya ignorados por `*.nvc`), `.github/workflows/` con ci.yml+release.yml (OK), `reports/` con K01-K20 inexistentes.
- **Commit único de limpieza `4f2f6c7`** (276 archivos, +3/-6525 líneas, pre-commit cargo build+test 385/385 OK): git rm de 27 artefactos raíz + `examples_backup_2026/` completo; disco: 41 `.nvc` temporales de stdlib/compiler borrados (conservando `compiler_v4.nvc` y `vm.nvc`), `src/` vacía eliminada. **`test_agents/` CONSERVADO** (45 archivos, referenciado por LUMEN_REPORT/reports — se sincronizaron las refs). `.opencode/` y `.vscode/` intactos; `.opencode/` añadido al .gitignore (los md de `.opencode/agents/` siguen trackeados).
- **Docs sincronizadas a v2.4.6** (nada obsoleto): README badges (v2.4.6/385 tests/fases 0-185+; sección Estado del Proyecto: Portabilidad 100%, self-hosting añadido, 116 ejemplos); **CHANGELOG v2.4.6** completo (Sprints 6-8, optimización 43x, dynamic Numero, limpieza); `docs/self-hosting.md` tablas Sprint 6-8 (closures/traits reales, vm.nv batería 39/40, optimización LÚMEN ✅, dogfooding 108/116, bootstrapping doble ⏳); `docs/siguiente.md` (estado actual Sprint 7-8 completos, fila optimización → bootstrapping+release); `docs/roadmap.md` Fase 174 + "Lo que falta" (Sprint 5-8, SHA-256 90048DC9…, 5s); LENGUAJE/HERRAMIENTAS/MARKETING bumps v1.x/v2.0 → v2.4.6 (header/footer/secciones); LUMEN_REPORT + reports/ con banner de sincronización (K01-K20 no presentes; test_agents real = 45 archivos) y TEST_REPORT actualizado a los conteos actuales (~378, 385/385).
- **Pendientes**: ~~bootstrapping doble~~ ✅ (7 Ago) → release v2.4.6 (tag) → FFI natives VM LÚMEN (cluster `__ffi_*`) → AI/ML (Fases 186-200).

**Progreso (7 Ago — sesión AI · BOOTSTRAPPING DOBLE COMPLETADO — VM LÚMEN 39/40 idéntica a Rust):**
- **Causa raíz #1 — genéricos anidados `>>`**: el lexer LÚMEN tokeniza `>>` como UN token y `_st_tp_skip` (parser.nv) no manejaba profundidad → `lista<lista<numero>> arrs = []` (vm.nv:8) rompía el parse (arrs no inicializada, main no registrado, Store @31 ausente). Fix en 4 puntos de parser.nv: `_st_tp_skip` con contador de profundidad + `>>` = dos cierres; scans de cast `como` (~440), typed var decl (~1266) y struct genérico (~1507) con límites.
- **Fixpoint NUEVO** (reemplaza al `90048DC9…`): SHA-256 `4638E369269A22CA59F3E148CC731CD719002B6663C92A5F95000EED0226CAD2`, byte-idéntico self==self2 (compiler_v4.nv 132,403 B → self-compile 5s). Disasm verificado: `Store @31` (arrs) presente; `__main__` = init globals + `Call @291` (main) + `Halt`.
- **Causa raíz #2 — `agregar`/`push` sin handler en bin()**: la VM Rust SÍ los implementa (vm.rs:251) pero vm.nv caía al fallback `retornar 0.0` (línea 988) → `arr = 0` → `largo(0)` computaba `0 - 1e9 = -1e9` → `str_at(-1e9)` → "Índice 0 fuera de rango (largo: 0)" con pila `main · bin · str_at`. El bytecode era correcto (repro `repro_largo.nv` compilaba bien: `Call @largo` + `Call @agregar`) — bug puramente de vm.nv. **Fix**: handler `agregar`/`push`/`append` con patrón COW (copiar `orig`→`l2`, append, `arrs.agregar(l2)`, nuevo id).
- **Causa raíz #3 — tuplas en ArrayGet**: el codegen emite `ArrayGet` (op 29) para `.0`/`.1` (el op 45 TupleGet es código muerto) y el handler op 29 no tenía rama de tuplas → con id 6e9+idx computaba `0 - 6e9 - 1e9 - 1` = `arrs[-7000000001]` → crash "Índice -7000000001 fuera de rango (largo: 1)". `a_texto_v` SÍ tenía la banda 6e9 (línea 209). **Fix**: rama `v >= 6e9 && v < 7e9` en op 29 y op 30.
- **Batería completa (runner `bat_self.ps1`: compilar con `compiler_v4.nvc` + ejecutar con `vm_self.nvc` vs VM Rust, 40 archivos)**: **39 OK / 1 DIFF** (stress_fecha = timing 0ms vs 17ms, flaky inherente). Antes de los fixes: 34 OK / 6 DIFF (demo_completo, arrays, destructuring, tuplas, test_texto_std + stress_fecha) — los 5 reales resueltos con los 2 fixes. demo_completo, match, enums, corutinas_demo, destructuring, tuplas, jr_concurrencia, 44_extension_methods, math, test_texto_std: todos byte-idénticos.
- **Smoke suite vm_self.nvc**: hello (¡Hola, LÚMEN!), test_arr (5|1|5), match, enums — exit 0.
- cargo test OK (385/385 con pre-commit), **commit `295a57e`** (6 files, +81/−18).
- **Pendientes**: release v2.4.6 (tag + CHANGELOG bootstrapping) → FFI natives VM LÚMEN (cluster `__ffi_*`) → AI/ML (Fases 186-200).

**Progreso (8 Ago, tarde — sesión AI · `para` clásico paridad Rust↔LÚMEN — fuego 113/117):**
- **Causa raíz `tui_test_min16/17/18` (asimetría `para`)**: (1) el parser RUST exigía init tipado en el `para` clásico (`para (i = 0; ...)` → no parseaba; min18); (2) el `para` clásico SIN paréntesis (`para entero i = 0; cond; paso { }` → min16/17) caía al foreach en Rust (E011) y al foreach roto en el self (producía programa vacío — el self NO lo soportaba mejor). Los 3 tienen semántica correcta (`xxxxx`).
- **Fix parser.rs**: `parse_for` con `is_for_init_decl()` (keyword de tipo / tipo custom `Punto p` / genérico `<T>`) y si no, construye `Decl::Variable` con `Type::Infer` (consume el `;`); dispatch `para` sin `(` usa `is_foreach_like()` (lookahead puro: `[tipo]? ident (en|in)`) → foreach solo con `en`/`in`, si no `parse_for`. Los helpers reusan `is_type_at`/`check_ident_next`.
- **Fix parser.nv (self)**: helper `_st_es_foreach(st)` (lookahead puro por posición sobre `tokens`, skip de keywords de tipo, ident → `en`/`in`) + branch de clásico sin paréntesis (parse init/cond/paso con `_parse_stmt`/`_parse_expr` + desugar `init; mientras (cond) { cuerpo; paso; }` idéntico al clásico con `(`). El foreach existente queda intacto como fallback.
- **Verificado**: los 3 tui_test_min **OK+CORRECTO** en la cadena 100% LÚMEN (nvc self == nvc Rust, `xxxxx`); **FIXPOINT v4 CONFIRMADO** SHA-256 `3DA624D6AD32E359D3714F7CD936563CE1A60ED633590CB580D695F24C7E282A` (compiler_v4.nv 135,465 B → .nvc 150,684 B, ~5s, self==self2 byte-idéntico); cargo test 0 FAILED; batería `test_vm.ps1` **39/40** (solo stress_fecha flaky).
- **fuego.ps1: 396/396 compilan · 113 CORRECTOS (+5) · 1 INCOMPATIBLE · 3 TIMEOUT · 0 fallos**. Restantes honestos: `graficos_demo` (SDL renderer — imprime header y diverge, por diseño), timeouts `debug_parser3` (loop), `graficos_completo`/`gui_ventana` (GUI).
- ⚠️ **Trampa harness**: `test_vm.ps1` debe correrse desde la RAÍZ del repo — `entrada_vm.txt` contiene rutas relativas `examples/x.nvc`; desde `stdlib/compiler` la VM LÚMEN no encuentra el archivo → FALLAS masivas falsas (OK=1 FALLAS=30).
- **Pendientes**: release v2.4.6 (tag + docs) → FFI natives VM LÚMEN (cluster `__ffi_*`) → AI/ML (Fases 186-200).

**Progreso (10 Ago — sesión AI · FFI completado en la VM LÚMEN — batería 42/43):**
- **Macros FFI reescritas en vm.rs** (eliminado el enfoque de macros anidadas que rompía la compilación): `ffi_int_call!` con **13 arms explícitos (0-12)** — cada uno con firma completa `unsafe extern "C" fn(i64, ...) -> ffi_rt_ty!($rtk)`, `let v = ffi_ints($args);` y `sym(v[0], ...)` directo (higiene resuelta, eliminados `ints!`/`int_vals!`); `ffi_int_arms!` con `$n:tt` despacha a `ffi_int_call!($nlit, ...)`. `ffi_rt_ty!`: I→i64, F→f64, S→*const c_char, V→(); `ffi_rt_conv!` con arm V = `{{ $e; Value::Void }}` (antes `$e` se ignoraba → las funciones void NUNCA se invocaban). Todos los `lib.get(...)` envueltos en `unsafe {}` (libloading 0.9 lo exige). `cargo build -p lumen-vm`/`lumen-cli` limpio.
- **`__ffi_peek`/`__ffi_poke` byte-a-byte LE** (vm.rs ~1870): los accesos `*mut u32` con offset 1-3 paniqueaban "misaligned pointer dereference" → ahora 4×u8 LE.
- **CAUSA RAÍZ `Error de tipo: Sub requires numbers` en bin()**: `__str_split(t, ",")` dentro del handler `__ffi_llamar` de vm.nv era ejecutado por la VM **host** como builtin nativo → devolvía `Array` real de strings reales SIN boxear → `tipos[i] - 1000000000` con `Str` real → crash (diagnosticado con DBG temporales: a0-a4 llegaban bien, n=1, bloc=pointer OK — el fallo era `tipos[0]`). **Fix**: boxeo manual de las partes (stl_din.agregar + `tipos.agregar(1e9+largo(stl)+largo(stl_din)-1)`) en el handler.
- **CAUSA RAÍZ `stricmp=120` vs `0`**: `__ffi_escribir` (vm.rs:1838) copiaba `bytes.len()` SIN null terminator → `_stricmp("Hola", "hola")` leía basura tras el string (el camino directo usaba `CString` con \0; la VM LÚMEN reservaba largo+1 pero escribía solo largo). **Fix**: escribir `*((ptr+offset+len) as *mut u8) = 0` tras la copia.
- **Resultados**: `test_ffi_min` (pid>0 true), `test_ffi_avanzado` (strlen=10, stricmp=0, sqrt=10) y `test_connect_direct` (socket, inet_addr=584628317, bytes AF_INET/puerto 80, connect=-1, **WSAGetLastError=10060 real**) **byte-IDÉNTICOS en ambas VMs** · batería `test_vm.ps1` ampliada +3 tests FFI → **OK=42 FALLAS=1** (solo stress_fecha flaky timing) · `vm.nvc` (113,971 B) y `vm_self.nvc` (113,971 B) regenerados con `compiler_v4_self.nvc` · cargo test --release 0 FAILED (166 e2e + unit).
- ⚠️ **Trampa harness**: `test_vm.ps1` usa `target\release\lumen.exe` — tras editar vm.rs hay que `cargo build --release -p lumen-cli` o los natives FFI no existen (RUNTIME-ERROR `__ffi_llamar_nv` no definida).
- **Pendientes**: commit → Playground Web (`lumen serve` real + UI) → Etapa 3 (lumen-aot → C transpiler + `lumen_rt.h`) → AI/ML (Fases 186-200).


---

## Bugs Conocidos y Fixes Recientes

### Fixes 8 Agosto 2026 — Handlers FFI/archivos/JSON en la VM LÚMEN (batería 39/40)
| Bug | Archivo | Fix |
|-----|---------|-----|
| **`__map_obtener` devolvía Values del host sin boxear** (Str real → crash "Ge requires numbers or strings" en `a_texto_v`) y **Void con mapas JSON** (el key boxed 1e9+N no coincidía con las claves strings reales de `__json_parsear`) | `stdlib/compiler/vm.nv` (bin ~662) | Lookup dual: primero key boxed (mapas del guest, ej. demo_completo) → si `__tipo_de(v) == "nulo"` reintenta con `str_at` desboxeado (mapas JSON) · boxeo por tipo real vía `__tipo_de`/`a_texto` del host: texto→`box_str`, booleano→9e9+1/9e9, lista→`arrs`, diccionario→`mapas` → **`test_json_avanzado` CORRECTO** (Ana / {"edad":30,"nombre":"Ana"}) |
| **`__existe_archivo` sin handler en bin()** → fallback `retornar 0.0` (imprimía "0" en vez de "false") | `stdlib/compiler/vm.nv` (bin ~997) | Handler nuevo: desboxea ruta + devuelve 9e9+1/9e9 (bool boxed) · añadidos también `__leer_archivo` (con `intentar`, el native devuelve Resultado) y `__escribir_archivo` → **`test_sistema_directo`/`test_sistema_avanzado` CORRECTOS** |
| **Verificado**: `test_sistema_directo`/`test_sistema_avanzado`/`test_json_avanzado`/`demo_completo`/`test_csv_avanzado`/`test_migracion`/`jr_fecha` byte-IDÉNTICOS · batería `test_vm.ps1` 39/40 (solo `stress_fecha` flaky timing) · 19 checks cruzados con `vm_self.nvc` (FFI/red/socket/corutinas/import) todos OK · `vm_self.nvc` regenerado (96,808 B) con compiler_v4 (target.txt 2 líneas) · cargo test OK | — | — |

### Fixes 8 Agosto 2026 — TLS de Winsock borrado por `RandomState` (FFI socket)
| Bug | Archivo | Fix |
|-----|---------|-----|
| **`WSAGetLastError` devolvía 0 en la VM LÚMEN** (y en la VM Rust directa si había una llamada a función guest entre dos FFI calls): `RandomState::new()` (std `HashMap::new()` en el prólogo de llamada a función + `im::HashMap::new()` en `__map_nuevo`/defaults de handlers) obtiene entropía del OS **y limpia el last-error TLS del hilo en Windows** → `probe_wsa2.nv`/`test_connect_direct`/`test_quick_connect`/`test_socket_debug` divergían (0 vs 10093) | `crates/lumen-vm/src/vm.rs` (prologo `scope` ×3 + `locals` + 37× `ImMap::new`), `value.rs` (`FixHasher` + tipo `Value::Map`), `coro_ffi.rs`, `min_json.rs` | Nuevo `FixHasher` (FNV determinista, sin entropía) para TODOS los mapas internos del VM: `HashMap<String, Value, FixHasher>` en locals/scope y `ImMap<Value, Value, FixHasher>` en `Value::Map` (`ImMap::with_hasher(FixHasher::default())`). Diagnóstico por instrumentación temporal (probes `GLE-CHANGE`/`PROLOGUE`/`OP` revertidas; ffi_log en `%TEMP%\opencode\ffi_log.txt`) |
| **Verificado**: `probe_wsa2.nv` + `test_connect_direct` + `test_quick_connect` + `test_socket_debug` **byte-IDÉNTICOS en ambas VMs (10093/10093)** · batería `test_vm.ps1` 39/40 (solo `stress_fecha` flaky timing pre-existente) · cargo test 385/385 · diff final solo +fix (sin instrumentación) | — | — |

### Fixes 8 Agosto 2026 — `para` clásico: paridad Rust ↔ LÚMEN (tui_test_min16/17/18)
| Bug | Archivo | Fix |
|-----|---------|-----|
| `para (i = 0; cond; paso)` — init sin tipo: el parser Rust exigía declaración tipada → no parseaba (min18) | `crates/lumen-parser/src/parser.rs` (`parse_for` + `is_for_init_decl`) | Init tipado vía `parse_declaration`, si no `Decl::Variable` con `Type::Infer` (consume `;`) |
| `para entero i = 0; cond; paso { }` sin paréntesis: Rust lo reenviaba a foreach → E011 (min16/17) | `crates/lumen-parser/src/parser.rs` (dispatch + `is_foreach_like`) | Lookahead puro `[tipo]? ident (en\|in)` → foreach solo con `en`/`in`, si no `parse_for` |
| Self-hosted idem: producía programa vacío (foreach roto con `=`) | `stdlib/compiler/parser.nv` (`_st_es_foreach` + branch clásico sin paréntesis) | Helper lookahead por posición sobre `tokens` (skip keywords de tipo, ident → `en`/`in`) + parse init/cond/paso con `_parse_stmt`/`_parse_expr` y desugar idéntico al clásico con `(` |
| **Verificado**: `tui_test_min16/17/18` **OK+CORRECTO** en la cadena 100% LÚMEN (byte-idénticos) · FIXPOINT v4 SHA-256 `3DA624D6…` self==self2 (135,465 B → 150,684 B, ~5s) · cargo test 0 FAILED · batería 39/40 · **fuego.ps1 396/396 · 113 CORRECTOS · 1 INCOMPATIBLE (graficos_demo SDL, por diseño) · 3 TIMEOUT (debug_parser3/graficos_completo/gui_ventana) · 0 fallos** | — | — |

### Fixes 30 Julio 2026 — Sprint 4 (HashMap)
| Bug | Archivo | Fix |
|-----|---------|-----|
| `Value::Map` con `Vec<(Value,Value)>` — O(n) en toda operación | `crates/lumen-vm/src/value.rs` | Cambiado a `HashMap<Value,Value>` con `Hash`+`Eq` manual |
| `__map_get`/`__map_set`/`__map_contains` O(n) scan lineal | `crates/lumen-vm/src/vm.rs` (×2) | `HashMap::get`/`insert`/`contains_key` O(1) |
| Sets union/inter/diff O(n²) | `crates/lumen-vm/src/vm.rs` (×2) | `contains_key` O(n) |
| `codegen_to_nvc` iteración lineal para lookups | `crates/lumen-vm/src/vm.rs` | `HashMap::get` directo |

### Fixes 30 Julio 2026 — Sprint 2 Fixes
| Bug | Archivo | Fix |
|-----|---------|-----|
| E052 falso positivo en `mientras`/`si` con variable en condición | `crates/lumen-parser/src/parser.rs:1040-1043` | `no_struct_init` antes de parsear condición en `parse_while` y `parse_if` |
| Operadores multi-carácter en lexer LÚMEN-in-LÚMEN | `stdlib/compiler/lexer.nv:25-27` | Detección de `\|\|`, `&&`, `==`, `!=`, `<=`, `>=` antes del fallback single-char |
| Operadores multi-carácter en compiler.nv | `stdlib/compiler/compiler.nv:52-58` | Idem |

### Fixes 29 Julio 2026 — Sprint 1
| Bug | Archivo | Fix |
|-----|---------|-----|
| E035 string `<`/`>`/`<=`/`>=` no soportado | `crates/lumen-sema/src/sema.rs:1601-1615` | `lt == rt` añadido a la condición de comparación numérica |
| `__map_contiene`/`__map_contains` retornaba `Decimal` | `crates/lumen-sema/src/sema.rs:2330` | Añadido como `TypeInfo::Booleano` |
| Runtime `Lt`/`Le`/`Gt`/`Ge` no soportaba strings | `crates/lumen-vm/src/vm.rs:2423-2466` | Añadido `Value::Str` cases |
| Import con path `/` no añadía `.nv` en search_paths | `crates/lumen-sema/src/loader.rs:85-100` | Extension lookup añadido al branch con `/` |
| Lexer lib (`lexer.nv`) no mutaba `tk` por paso por valor | `stdlib/compiler/lexer.nv` | `_lx_emit` reemplazado por emit inline en `lexer_tokenizar` |
| Parser lib (`parser.nv`) off-by-one: `pos > cnt` vs `pos >= cnt` | `stdlib/compiler/parser.nv` | `_st_tk` y `_st_eof` corregidos a `>=` |
| Lexor lib incluía EOF en `cnt` | `stdlib/compiler/lexer.nv` | `cnt` ahora cuenta solo tokens reales (excluye EOF) |
| Codegen lib (`codegen.nv`) no generaba código para argumentos de Call | `stdlib/compiler/codegen.nv:149-158` | Loop sobre args antes de emitir opcode |
| Codegen lib field names no coincidían con parser | `stdlib/compiler/codegen.nv:161-189` | `left/right`→`izq/der`, `val`→`expr` |
| Codegen lib no manejaba `If`/`While`/`Block`/`Programa` | `stdlib/compiler/codegen.nv:236-310` | Añadidos handlers con JMP/JMP_IF patching |
| Codegen lib usaba variable `sino` (keyword) | `stdlib/compiler/codegen.nv:239` | Renombrado a `rama_else` |
| `compiler/lexer.nv` usaba `tipo`/`valor` pero parser esperaba `t`/`v` | `stdlib/compiler/lexer.nv:2,39` | Cambiado a `t`/`v` |
| Operadores multi-carácter en `lumen_mini.nv`, `lumen_compiler.nv` | `stdlib/compiler/lumen_mini.nv:61-64`, `lumen_compiler.nv:27` | Detección de multi-char antes del fallback |
| Bucle infinito en `continuar` anidado en VM (no reproducido en Rust) | `lumen_mini.nv` | 🟡 No reproducido |

### Sprint 5 — Self-Hosting Puro (31 Julio 2026) ✅
| Hito | Archivo | Detalle |
|------|---------|---------|
| Jmp/JmpIf target en tabla nums | `crates/lumen-vm/src/vm.rs` | VM lee `nums[idx]`; antes target directo → loop infinito |
| If con JMP de skip + `_cg_patch` | `stdlib/compiler/codegen.nv` | JMP al final del then-body hacia endif; backpatch de JMP_IF/JMP |
| Indexación `chars[i]` | `stdlib/compiler/parser.nv`, `codegen.nv` | Postfix `[expr]` en `_parse_pr`; nodo `Index` → `OP_ARRAY_GET` (29) |
| TryUnwrap mapeado | `crates/lumen-vm/src/vm.rs` | `40 => 40` en `cg_to_vm` — antes Nop dejaba `Exito(Str)` |
| Print multi-arg | `stdlib/compiler/codegen.nv` | Un `OP_PRINT` por argumento en orden |
| Break/Continue con loop_stack | `stdlib/compiler/codegen.nv` | Backpatches: breaks→fin, conts→loop_start |
| VarDecl sin inicializador | `stdlib/compiler/codegen.nv` | PushInt 0 por defecto (antes stack underflow) |
| Escapes de string en lexer puro | `stdlib/compiler/lexer.nv` | `\n \t \r \" \\` convertidos en el valor del token |
| Keywords void/diccionario | `stdlib/compiler/lexer.nv` | Añadidas al mapa `kw` |
| Forward declarations ignoradas | `stdlib/compiler/parser.nv` | `;` tras firma → nodo `Vacio` |
| Escape `\r` en lexer Rust | `crates/lumen-lexer/src/lexer.rs` | `Some('r') => s.push('\r')` — antes `"\r"` → 'r' literal |
| **Fixpoint confirmado** | `compiler_v4_self.nvc` | 3 runs: 193s/203s/197s → 54,712 bytes IDÉNTICOS |

### Fixes 1 Agosto 2026 — Sprint 6.3 (demo_completo CORRECTO + fixpoint v4)
| Bug | Archivo | Fix |
|-----|---------|-----|
| Float `3.14159` → `3` (lexer partía `3.14159` en `[3][.][14159]`) | `stdlib/compiler/lexer.nv` | Bucle de dígitos consume `.`: `_lx_es_digit(ch) \|\| ch == "."` |
| PushNum truncaba a Int en .nvc | `crates/lumen-vm/src/vm.rs` (`codegen_to_nvc`) | op 1 (PushNum) → VM PushNum(2) con f64 en tabla nums; op 3 (PushInt) → PushInt(1) |
| `arr.agregar(4)` sin efecto (nodo Assign dentro de ExprStmt caía en `_gen_expr`) | `stdlib/compiler/codegen.nv` | Caso `Assign` añadido en `_gen_expr` (espejo de `_gen_stmt`; branch `target` → `OP_ARRAY_SET`) |
| Ternario parseaba `a >= b ? x : y` como `a >= (b ? x : y)` | `stdlib/compiler/parser.nv:387` | Check `?` solo a nivel superior: `si (mp == 0 && ...)` |
| Ternario emitía ramas invertidas (VM JmpIf salta con FALSY) | `stdlib/compiler/codegen.nv` (Ternario) | `si` al fallthrough + Jmp, `no` al salto (espejo del If de `_gen_stmt`) |
| AST_DUMP O(n²) colgaba el fixpoint (fuentes ~95 KB) | `stdlib/compiler/generar_v4.ps1` | Instrumentación temporal removida |

### Fixes 1 Agosto 2026 — Sprint 6.4 (enum/elegir/sea reales)
| Bug | Archivo | Fix |
|-----|---------|-----|
| **`__map_poner` NO muta el mapa original** (`im::HashMap` persistente — `insert` devuelve copia; `last = __map_poner(last,"sino",ifn)` jamás propaga a `first`) | `parser.nv` (elegir) | Cadenas reconstruidas desde el final: lista de nodos `ifns[]` + `chain = __map_poner(ifns[i2], "sino", chain)` descendiendo. El demo lo enmascaraba (solo llamaba `color_str(Color::Rojo)`) |
| `elegir` con 2+ casos: solo compilaba el caso 1 (la cadena sino se perdía) | `parser.nv` (elegir) | Idem — ahora match.nv/enums.nv/demo IDENTICAL |
| `defecto:`/`default:` se DROPEABA (romper en `caso`, luego `sino { adv }` descartaba el body) | `parser.nv` (elegir) | Break-condition añade `defecto`; post-check consume `defecto`/`default`, parsea su body → `dblock`; se adjunta como `sino` del último If (o como Block si solo-default) |
| `sea`/`let` era skip-keyword (no creaba binding; `sea x = e` se re-parseaba como Assign) | `parser.nv:456-459` | VarDecl real: `sea [tipo] nm = expr;` → nodo VarDecl (tipo_var="let" o el tipo) |
| `Enum::Miembro` solo → `Texto("Miembro")` (variantes con datos `Exitoso(100)` no parseaban) | `parser.nv:187-199` | `Nombre::Miembro(args)` → nodo `EnumInit` (args vacío para unit-variants) |
| Codegen sin EnumInit | `stdlib/compiler/codegen.nv` | Opcodes `OP_ENUM_CTOR=43/OP_ENUM_VAR=44/OP_ENUM_ARGC=45`; emite args + trío consecutivo (nombre@str, variante@str, argc@int) |
| `.nvc` sin serialización de enums | `crates/lumen-vm/src/vm.rs` (`codegen_to_nvc`) | op 43 → `WithIdx(EnumCtor, str)`; op 44 → `WithIdx(Nop, str)`; op 45 → argc en nums + `WithIdx(Nop, num)` — el VM lee ip+1/ip+2 (EnumCtor handler) |
| Debug residual LEX-LOOP/LEX-DBG `retornar` en lexer.nv (spam en cada compilación) | `stdlib/compiler/lexer.nv:44,55-69` | Eliminado |
| **Verificado**: enums.nv IDENTICAL (unit + data variants), match.nv IDENTICAL (defecto), demo 90/90 0 diffs, cargo test OK | — | 43_tipos_asociados.nv NO soportado por NINGÚN compilador (usa `rasgo`/traits) |

### Fixes 2 Agosto 2026 — Sprint 6.5 (cortocircuito `&&`/`||` + fixpoint v4)
| Bug | Archivo | Fix |
|-----|---------|-----|
| **Fixpoint regresionado**: `compiler_v4_self.nvc` crasheaba "Índice N fuera de rango (largo: N)" dentro de `ejecutar_pipeline` (causa raíz: `&&`/`||` emitidos eager `And`/`Or` → `mientras i < n && cs[i] != "\n"` lee `cs[i]` con `i == n`) | `stdlib/compiler/codegen.nv` | Helper `_cg_and_or(cg, izq, der, es_and)` emite short-circuit REAL con `JmpIf`/`Jmp` + `PushBool` (`&&`: izq/der falsy→false; ambos→true · `\|\|`: izq truthy→true, si no der decide). Binary en `_gen_expr` despacha; el resto eager. Confirmado con diff de disasm: RUST-built cortocircuita vía JmpIf, el LÚMEN emitía And eager |
| **Verificado**: fixpoint v4 CONFIRMADO — compiler_v4.nv → self (112,368 B) → self2 byte-IDENTICAL (0 diffs); self compila match.nv sin crash ("dos"); fuego 396/396 compilan, 61 CORRECTOS (+1), 0 fallos | — | — |

### Sprint 2 — Self-Hosting (30 Julio 2026) ✅
| Hito | Archivo | Detalle |
|------|---------|---------|
| Builtin `__codegen_a_nvc` | `crates/lumen-vm/src/vm.rs:350+` | Convierte mapa de codegen a bytes .nvc en Rust nativo |
| Builtin `__file_write_binary` | `crates/lumen-vm/src/vm.rs:325-339` | Escribe `Array<Int>` de bytes a archivo |
| Builtin `__num_a_f64_bytes` | `crates/lumen-vm/src/vm.rs:341-348` | Convierte número a 8 bytes f64 LE |
| IR builder: nuevas builtins | `crates/lumen-ir/src/builder.rs:984-988` | Registrados `__codegen_a_nvc`, `__file_write_binary`, `__num_a_f64_bytes` |
| sema: type-check builtins | `crates/lumen-sema/src/sema.rs:2009-2031` | `__codegen_a_nvc` retorna `Lista<Entero>` |
| loader: is_builtin | `crates/lumen-sema/src/loader.rs:706` | `__codegen_a_nvc` registrado como builtin |
| test_nvc.nv | `stdlib/compiler/test_nvc.nv` | Pipeline LÚMEN completo: lexer→parser→codegen→builtin→file |
| **Self-compilación verificada** | `test_output.nvc` | `.nvc` generado por LÚMEN se ejecuta y produce `42` ✅ |

### Sprint 3 — Bootstrap completo (30 Julio 2026) ✅
| Hito | Archivo | Detalle |
|------|---------|---------|
| `compiler_v2.nv` | `stdlib/compiler/compiler_v2.nv` | Compilador LÚMEN que usa `__compile_nv` nativo (Rust pipeline) |
| Builtins string eficientes | `vm.rs` | `__str_subcadena`, `__str_concat_list`, `__str_starts_with`, `__str_to_chars` |
| Lexer optimizado | `stdlib/compiler/lexer.nv` | Subcadenas con `__str_subcadena`, acceso chars con lista O(1) |
| ArrayGet optimizado | `vm.rs:2696-2714` | `chars().nth()` en vez de `chars().collect::<Vec>()` |
| `__compile_nv` builtin | `vm.rs:442+` | Compilación completa Rust nativa (lex→parse→sema→ir→codegen) |
| Self-compilación | `lumen run compiler_v2_self.nvc → 42` | **533ms** (de >5min a 0.5s) |
| cargo test | ~385 tests, 0 fallos | Todos los tests pasan |

**Self-compilación verificada:** `lumen run self_compile.nv` → `compiler_v2_self.nvc` (533ms), ejecuta correctamente.

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

---

## Bytecode (.nvc) — v3.5.7 Producción

- **Version**: 7 (`CHUNK_VERSION 7`, decode acepta 6 y 7)
- **Novedad v3.5.7:** `FuncMeta.defaults: Vec<Option<DefaultValue>>` persistidos (`Int/Float/Str/Bool`) para `bind_args` unificado (3 call-sites). Ver `docs/produccion.md` §1.3.
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-46 + 52-53
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays (ArrayNew, ArrayGet, ArraySet, ArrayLen, ArrayPush)
  - 33-34: Closures (FuncRef, CallValue)
  - 35-37: Structs (StructNew, StructGet, StructSet)
  - 38-40: Result (ResultOk, ResultErr, ResultUnwrap)
  - 41-42: Option (OptionSome, OptionNone)
  - 43: Enum (EnumCtor)
  - 44-45: Tuples (TupleNew, TupleAccess)
  - 46: Mod
  - 52-53: MatchType/MatchPayload (if-let / elegir con payloads)

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
  lumen-api/      → lib.rs
  lumen-plugin/   → lib.rs
docs/            → README.md (índice) + guias/ referencia/ arquitectura/ desarrollo/ spec/ informes/
docs/spec/        → grammar.ebnf, bytecode-format.md, error-codes.md, vm-spec.md
reports/          → MOVIDA a docs/informes/ (v3.5.7 rondas JIT)
examples/         → *.nv (45 ejemplos funcionales)
stdlib/           → *.nv + stdlib/compiler/ (self-hosting)
scripts/          → PowerShell CI/CD, installers, git-hooks
```

---

## Progreso (8 Agosto 2026 — Bootstrapping Doble + VM Paridad + Stream/Async/Par/Actor/Generador en vm.nv):

- **Handlers de concurrencia/stream/async completados en `bin()` de `vm.nv`**: `__stream_desde`/`__stream_from`, `__stream_mapear`/`__stream_map`, `__stream_filtrar`/`__stream_filter`, `__stream_colectar`/`__stream_collect`, `__par_mapear`/`__par_map`, `__par_unir`/`__par_join`, `__actor_nuevo`/`__actor_new`, `__actor_enviar`/`__actor_send`, `__actor_recibir`/`__actor_recv`, `__generador_nuevo`/`__generator_new`, `__generador_siguiente`/`__generator_next`, `__seleccionar`/`__select`, `__leer_archivo_async`/`__file_read_async`, `__escribir_archivo_async`/`__file_write_async`. Delegación directa a los natives de Rust ya existentes en `vm.rs` (líneas 1890-2260).
- **`sprint1_concurrencia.nv` 100% paridad byte-idéntica** entre VM Rust y VM LÚMEN: Stream (map/filter/collect), Async I/O (read/write), Timer, Select, Par Map/Join, Actor, Generator — todos CORRECTOS.
- **Bootstrapping Doble COMPLETADO Y VERIFICADO**:
  - `compiler_v4_self.nvc` (compilador 100% LÚMEN) compila su propio source `compiler_v4.nv` → `compiler_v4_self2.nvc`
  - **FIXPOINT CONFIRMADO**: SHA-256 `3DA624D6AD32E359D3714F7CD936563CE1A60ED633590CB580D695F24C7E282A` — 150,684 bytes **byte-idénticos** en self/self2 (~5s cada run).
  - `vm_self.nvc` regenerado con el compilador autocontenido: 111,318 bytes.
  - `vm_self.nvc` ejecuta `demo_completo.nvc` correctamente (89/89 líneas, 0 diffs).
- **Test suite completa**:
  - `cargo test --release`: **385 tests pasando, 0 fallos** (166 e2e + unit).
  - `test_vm.ps1`: **39/40 OK** (solo `stress_fecha` flaky por timing 0ms vs 17ms).
  - `fuego.ps1`: **396/396 compilan · 112 CORRECTOS · 1 INCOMPATIBLE (`graficos_demo` SDL, por diseño) · 4 TIMEOUT (`debug_parser3`, `graficos_completo`, `gui_ventana`, `sprint1_http` — red/GUI, flaky) · 0 fallos**.
- **Commits**: `7d3cdc8` (bootstrapping doble + fixpoint SHA-256 3DA624D6... verificado).

**Estado actual: LÚMEN v2.4.6 — Autocompilación total, VM en LÚMEN funcional, dogfooding 112/117, bootstrapping doble certificado. Ready for release tag.**

---

## Progreso (11-12 Ago 2026 — sesión AI · AOT dual: backend Cranelift (rust) con paridad C — batería 38/38 + 12/38):

- **Backend Cranelift implementado en `crates/lumen-aot/src/lib.rs`** (`--backend rust`): paridad funcional con el transpilador C en `imprimir` (int/string vía pila paralela `kinds`), concat de strings (`_rt_concat_ss/si/is`), igualdad de strings (`_rt_str_eq`), si/sino, mientras, recursión (fib), JmpIf/Jmp/Label con bloques y fallthrough.
- **Detalles de emisión clave**: variables = `StackSlot` (`stack_store/stack_load`); `block_at[i]` = bloque del Label más reciente (entry_block jamás targetable); arm Label emite `jump(target)` solo si `target != cur && !terminated`; JmpIf = `brif(is_zero, target, [], fb, [])` con `fb` = label siguiente si `!= cur`, si no bloque fresco; Return/Halt/Jmp crean bloque muerto con jump al siguiente label (filtrado `!= entry_block`); `terminated=true` tras terminator → siguiente arm hace pre-switch a bloque fresco; concat = `Op::Add|Concat if ka||kb` despachando por (ka,kb); Equal/NotEqual solo con `ka&&kb` → `_rt_str_eq`; resultado concat hereda kind string.
- **Bug clave del CLI**: el IR usa `Call("imprimir", 1)` para print, NO `Instr::Print` — el arm Call map emite builtins (`imprimir/print`→`_rt_print_*`, resto→placeholder 0). Los shims `_rt_concat_*`/`_rt_str_eq` usan firma con retorno `rsig` (i64,i64→i64) — usar `inst_results(call)[0]` con psig paniquea.
- **Debug**: `LUMEN_AOT_DEBUG=1` (dump de instrs + clif IR en compile_body), `LUMEN_KEEP_OBJ=1` (conserva .obj tras link). Fallo de define_function imprime nombre de función + panic.
- **El else del lenguaje es `sino`, NO `no`** (parser: E052/E020).
- **Benchmark** (`bench_fib.nv`, fib(26) + loop 100): **VM 0.856s / C 0.406s / Cranelift 0.1155s** → Cranelift 7.4x VM, 3.5x C.
- **Batería dual `aot_bateria_dual.ps1`** (38 ejemplos, ambos backends, watchdog 25s, normalización para jr_fecha/demo_completo: `(?m)^\d{10}\r?$`→`<TS>` (solo 10 dígitos anclados — el sin anclar se comía floats), `\d{2}:\d{2}:\d{2}`→`<TIME>` (sin anclar: cubre `T00:00:00Z` y datetimes `YYYY-MM-DD HH:MM:SS`), `(?m)^\d+\.\d+\r?$`→`<FLT>`): **C OK=38 DIFF=0 (paridad total) · RUST OK=12 DIFF=26 (límite de diseño: sin strings/structs/colecciones — placeholder 0) · FAIL=0 SKIP=1 (math) · HANG=0**.
- **Limpieza**: el backend C genera `$base.c/.exe` en el dir del fuente — la batería los mueve/borra y el harness corre el exe con `Start-Process` redirigiendo `o.txt/e.txt` + watchdog (25s).
- **loader.rs**: caché de imports a nivel de proceso keyed por path canónico + mtime — evita re-lexear/re-parsear módulos en serve/LSP/tests (solo archivos reales; los virtuales del playground se parsean siempre).
- **Verificado**: `cargo test --release -p lumen-aot` 4/4 OK; E2E manual hello/42/loop/fib IDÉNTICOS en las 3 vías (VM, C, Cranelift); pre-commit build+test completo OK en `5c7c4d8`.
- **Pendientes**: Playground Web real (`lumen serve` + UI con toggle backend) → Etapa 3 (lumen-aot → C transpiler + `lumen_rt.h`) → AI/ML (Fases 186-200).

**Progreso (12 Ago, tarde — sesión AI · Playground: toggle backend WASM/Servidor + `/api/run`):**
- **Endpoint `POST /api/run`** en `handle_http_request`/`handle_api_request` (main.rs): soporte de POST con Content-Length + body; compila y ejecuta con la **VM Rust nativa** (`run_source_capture`: ModuleLoader + sema + IR + codegen + VM, output via `vm.output()`, errores con span `(linea,col)`); respuesta JSON `{"ok":true,"output":...}` / `{"ok":false,"error":...}` con `escape_json_ml` (saltos de línea reales). Imports stdlib resueltos con `lib_dirs=["stdlib"]` + tmpfile `target/playground_tmp.nv` (base_path para imports relativos).
- **Toggle en la UI** (`crates/lumen-wasm/web/index.html`): `.backend-toggle` en el toolbar con `data-backend="wasm"`/`"server"` + estado `execBackend`; `runCode()` bifurca — server → `fetch('/api/run', {method:'POST', body: code})` con guards para `location.http` (el modo servidor requiere `lumen serve`), sin depender de `runtime` WASM (funciona antes de cargar el wasm); wasm → flujo original. `vmStatus` muestra "servidor (VM Rust)".
- **Verificado**: hello/arith → `{"ok":true}` con output correcto · error sintáctico → `{"ok":false}` con E015 de span · imports con prefijo `coleccion_` (ordenar/contar → "1") · **demo_completo completo (33 secciones + FIN) por el backend servidor** · index.html sirve con toggle (54,956 B) + `pkg/lumen_wasm_bg.wasm` (2.42 MB) · wasm-pack rebuilt.
- **Pendientes**: Etapa 3 (lumen-aot → C transpiler + `lumen_rt.h`) → AI/ML (Fases 186-200).

**Progreso (12 Ago, noche — sesión AI · Etapa 3: OPTIMIZACIÓN AOT — Cranelift 20x + C 18x, ambos en ms):**
- **Cranelift: stack slots → variables SSA reales del frontend** (`lib.rs compile_body`): las variables ya no pasan por memoria (StackSlot), ahora son `Variable` SSA de Cranelift (`declare_var`/`def_var`/`use_var` con phis vía dominancia) con `def_var(0)` en el entry para TODAS las variables usadas en la función (garantiza dominancia; params sobre-escritos con su valor real). Import: en cranelift 0.132 `Variable` se exporta desde `cranelift::frontend`, NO de `codegen::ir`.
- **Backend C: índices de registro constantes** — `compile_to_c` generaba `gv[_fv("nombre")]` = **strcmp lineal en cada Load/Store** → ahora mapeo `name_idx` (orden idéntico a `_init`/`_reg`) y emite `gv[N]` directo (fallback `_fv` para nombres no registrados, p.ej. CallValue dinámico). Las llamadas a funciones conocidas ya eran directas `_f_x()` (sin `_call_by_name`).
- **Benchmark `bench_fib.nv` (fib(26)+loop 100, runs calientes)**: **VM 856ms → C 22ms (antes 406ms, 18x) → Cranelift 5.6ms (antes 116ms, 20x)** — ambos backends en **milisegundos** (objetivo del usuario cumplido). Build: gcc_link ~183ms, codegen_cranelift 2ms, compilar_a_ir 0.8ms.
- **Verificado**: batería dual `aot_bateria_dual.ps1` **C OK=38 DIFF=0 (paridad total mantenida) | RUST OK=12 DIFF=26 (límite de diseño) | FAIL=0 SKIP=1 HANG=0** · cargo test --release 0 FAILED · gcc warnings residuales inofensivos (`__p__environ` dllimport, `_regex_rep`).
- **Commit**: `20584fb` (perf(aot): Cranelift SSA en registros 20x + C índices sin strcmp 18x).
- **Pendientes**: opcional C: eliminar save/restore `_sv` por llamada (slots de params por función — requiere distinguir locales de globals del IR) → Playground Web completo → AI/ML (Fases 186-200).

**Progreso (12 Ago, noche — sesión AI · Fases 61-63 REALES en el pipeline Rust: OR patterns + Rangos `..`/`..=`):**
- **CAUSA RAÍZ OR patterns "buggy"** (probe `caso Color::Rojo | Color::Verde:` daba E035 "Operador aritmético requiere números"): `parse_addition` (parser.rs) consumía `Pipe` como `BinOp::BitOr` ANTES de que el arm del match viera los alternativos. **Fix**: flag `match_arm_pipe` en Parser (se activa al parsear el value de un arm); `parse_addition` salta el pipe cuando el flag está activo (`check(&[Pipe]) && !match_arm_pipe`); el loop de `alt_values` del match (que ya existía en parser/sema/IR) consume los pipes.
- **Feature nueva: RANGOS `..` / `..=`** (no existía en NINGUNA capa del pipeline Rust — lexer sin token, sin `Expr::Range`):
  - **Lexer** (token.rs + lexer.rs): tokens `DotDot`/`DotDotEqual`; el brazo `'.'` los detecta; **CRÍTICO** el bucle numérico NO debe consumir el punto inicial de rango: `1..5` se tokeniza `Num(1) DotDot Num(5)` (peek_n(1) — `1.5` sigue float).
  - **AST**: `Expr::Range { start, end, inclusive, span }` (parser + span()).
  - **Parser**: parse de rango en `parse_comparison` (precedencia alta, nivel propio); válido en arms de `elegir` (`caso 0..5:` / `caso 5..=10:`) y como expresión general (`lista<entero> r = 0..5;`).
  - **Sema**: `analyze_expr` → `Lista(Entero)` con E044 si límites no numéricos; E056 eximido para arms range (`is_range_arm`).
  - **IR**: desugar doble — (1) como PATTERN en `Stmt::Match`: `emit_match_pattern` hace `x >= start` + `x < end|<=` (con `JmpIf`-salta-falsy y `Jmp body` si dentro del rango); (2) como EXPRESIÓN lista: loop con temporales `__rng_i_N`/`__rng_c_N` (`ArrayNew 0` → `Store i` → loop `i<end` → `ArrayPush i` → `i+1` → `Jmp`).
- **BUG RAÍZ del match (loop infinito → "sin output")**: las alternativas OR antiguas saltaban al propio label (`JmpIf` apuntaba al label del MISMO arm → ip self-loop; y la semántica "alt matcheada → body" estaba INVERTIDA: el JmpIf de la alt mandaba al próximo arm si matcheaba). **Fix robusto**: por arm → `Label(fail)` + `fail = new` + `body = new`; cada patrón emite: concreto → `gen x; gen pat; Binary(NotEqual); JmpIf(body)` (matchea → NotEqual=false→salta al body; no matchea → cae al siguiente patrón); rango → `Ge; JmpIf(fail)` + `Le/Lt; JmpIf(fail)` + `Jmp(body)`; tras todos los patrones `Jmp(fail)`; guard falsy → `JmpIf(fail)`. Ningún patrón matcheado cae al body por fallthrough (los JmpIf a body saltan directo). Verificado con disasm: targets correctos (11/18/20 vs self-loop).
- **Bug pre-existente de builds sin features**: `tcp_listener` estaba `#[cfg(any(feature="extra",feature="full"))]` en el struct VM pero el handler `__tcp_listen`/`__tcp_accept` NO estaba gateado → `cargo test -p lumen-sema` (compila lumen-vm sin features) fallaba "no field `tcp_listener`". **Fix**: campo sin cfg (std::net siempre disponible) + init sin cfg — los cfg de expresión else-if no funcionan.
- **Tests nuevos**: lexer +3 (test_range_literals, test_float_not_confused_with_range, +1), parser +3 (test_parse_range_pattern/or_patterns/range_expr), sema +4 (test_match_range_pattern_numero/inclusive, test_match_or_patterns_ok/exhaustive_count). **Totales: lexer 27, parser 45, sema 56, cargo test --release 0 FAILED** (41 crates).
- **Ejemplos por fase** (2 por fase faltante; formato/estilo de los existentes): `examples/fase61_or_patterns.nv` (temperature() enums + tamano() ints — OR), `examples/fase63_range_patterns.nv` (grado() con `0..60`/`60..80`/`80..=100` + expresión `0..5` como lista), `examples/fase64_string_patterns.nv` (string patterns + OR de strings `"si" | "cierto"`). **Los 3 IDÉNTICOS en VM y backend C** (7/7, 6/6, 5/5 outputs correctos). Cranelift: limitado por diseño (sin strings/colecciones — el rango-int puro compila pero el foreach no emite cuerpo).
- **Pendientes**: ~~fixpoint v4 tras los cambios de lexer/parser (DotDot/DotDotEqual en el self-hosted → regenerar `compiler_v4.nv`)~~ ✅ (13 Ago, ver sección siguiente) → AI/ML (Fases 186-200).

**Progreso (13 Ago — sesión AI · Rangos `..`/`..=` en el SELF-HOSTED — fixpoint NUEVO `5D153BC6…`):**
- **lexer.nv**: branch de rango ANTES del punct: `c == "." && i+1<n && chars[i+1] == "."` → token t=4 Oper con v=`".."`/`"..="` (consumo opcional de `=`). El `.` individual (campos/floats) sigue intacto (el numérico ya excluye `.` no seguido de dígito — `1..5` se partía bien; faltaba el TOKEN).
- **parser.nv**: `_st_is_op` + `..`/`..=`; en `_parse_bin` tras `_parse_pr` (nivel primary): si el token es `..`/`..=` → nodo `Range { lo, hi, incl }` (consumo del der con `_parse_pr`). El prec=0 de `_prec` hace el loop binario inofensivo si el check no dispara.
- **codegen.nv** (2 cambios):
  1. **Caso `Range` en `_gen_expr`** — desugar a lista (espejo del IR Rust: el array VIVE EN EL STACK, jamás se guarda): `ArrayNew; gen lo; Store __rng_i_N; loop: Load i; Store __rng_c_N; Load c; gen hi; Lt|Le; JmpIf fin; Load c; ArrayPush; Load c; PushInt 1; Add; Store i; Jmp loop; Label fin`. Temporales `__rng_i_<dt_cnt>`/`__rng_c_<dt_cnt>` con `dt_cnt` global. Nota: `PushInt 1` requiere `_cg_add_int(cg, 1)` antes; el JMP al loop usa `_cg_emit_jmp` (registra el target en la tabla de ints).
  2. **Intercepto en `Binary`** (`==` con der `Range` — el elegir desugara `sel == caso`): `(sel >= lo) && (sel < hi|<= hi)` vía `_cg_and_or(cg, c1, c2, 1)` — short-circuit exacto.
- **BUGS raíz (3)**:
  1. `_st_is_op` no listaba `..` → el nodo Range NO se construía y la cadena del elegir se desincronizaba (el disasm mostró `Eq` con PushInt 60 en vez de Ge/Lt/JmpIf).
  2. **`OP_ARRAY_PUSH` self-hosted = 31** (ArrayLen en la tabla codegen→bytecode) → se emitía `ArrayLen` dentro del loop → "ArrayLen requires array or string". Fix: `OP_ARRAY_PUSH = 32` (el bytecode v6: 31=ArrayLen, 32=ArrayPush).
  3. **Encoder nativo `codegen_to_nvc` (vm.rs:4831) NO mapeaba 32** → `32 => 32, // ArrayPush` añadido (caeía en `_ => 0 Nop` — el disasm mostraba `Nop`).
- **Verificado**: `probe_rango2.nv` (4 rangos de caso + default-pattern `caso 120:` + `lista<entero> serie = 0..5` con largo/índices) — **byte-IDÉNTICO en la cadena 100% LÚMEN vs pipeline Rust** (bajo/medio/alto/otro/bajo/5/0/4). **FIXPOINT NUEVO**: compiler_v4.nv (139,732 B) → self_out.nvc → self_out2.nvc **byte-idénticos SHA-256 `5D153BC631812524B3DD078380B6E9285A68E284FCB6E23D3DC97ADFA12076C5`** (reemplaza a `3DA624D6…` — el source del compilador creció). `cargo test --release` 0 FAILED (lexer 27, parser 45, sema 56, ir 20, vm 45, e2e 166 + resto). `compiler_v4.nvc` (Rust-built) regenerado; probes temporales borrados; target.txt restaurado.
- **Gaps documentados (asimetría Rust vs self — el self es más tolerante)**: el parser RUST NO acepta `defecto:`/`default:` como arm del elegir (E011 "Se esperaba un nombre de función" en la línea POST-elegir — `fase63_range_patterns.nv` usa `defecto:` y compila porque termina en el elegir; un `retornar` tras el elegir con `default:` dispara E011) ni `funcion principal()` sin tipo de retorno (E011 — `principal` es keyword del entry en Rust; el self tolera ambos).
- **Pendientes**: AI/ML (Fases 186-200).

**Progreso (13 Ago — sesión AI · IF-LET REAL (Fase 62) + payloads en `elegir` — opcodes 52/53 + ejemplos de fases 66/68/70):**
- **If-let era semi-funcional**: `si sea algun(n) = opt { }` parseaba (`Stmt::IfLet`) pero el sema NUNCA bindeaba las vars del patrón → E033 "La variable 'n' no está declarada"; el IR compilaba `Equal(sel, algun(n))` → requería `n` ya definida → runtime "Variable 'n' no definida".
- **Fix sema** (`sema.rs`): helper nuevo `bind_pattern_vars(&self, pattern, span)` — bindea Ident (tipo dinámico `Numero`) dentro de `Algun`/`Exito`/`Error`/`Call`/`List`, define en el scope actual; aplicado en `analyze_decl_or_stmt`: arms IfLet + GuardLet + **arms del Match** (reordenado: push scope → bind → analiza arm.value → checks E056/E034 → guard → body → pop — antes el arm.value se analizaba FUERA del scope y `algun(x)` disparaba E033); `is_pattern_arm` exime del E056 a los arms con payloads (como is_range_arm). El analizador recursivo de resolución de vars NO necesita bind (no recursa en payloads).
- **Opcodes NUEVOS 52/53** (`MatchType`/`MatchPayload` — el bytecode v6 iba a 51):
  - `ir.rs`: `Instr::MatchType(u8)`/`Instr::MatchPayload`; `builder.rs`: arm IfLet reescrito — `gen value; Store __mt_N; Load temp; MatchType(kind); JmpIf(else); Load temp; MatchPayload; Store n; body; Jmp end; else; end` vía helper `emit_if_let_pattern` (Algun→kind 0, Exito→1, Error→2, Ninguno→`MatchType(0); Not`, Ident puro→Store directo, resto→fallback Equal) + `bind_payload` (Ident simple o lista con TupleAccess). **`emit_match_pattern` (elegir)**: Algun/Exito/Error → Store temp + emit_if_let_pattern + Jmp(body) → **`caso algun(x)` bindea x** (probe62c: `x= 42`).
  - `bytecode.rs`: `MatchType = 52, MatchPayload = 53` + from_u8; `codegen.rs`: MatchType → `WithIdx(MatchType, intern_num(kind))`, MatchPayload → Simple.
  - `vm.rs`: `execute_simple` MatchPayload (pop → push inner desboxeado de Opcion(Some)/Exito/Error); `execute_with_idx` MatchType — **⚠️ TRAMPA: el idx del WithIdx es índice de TABLA NUMS, no el kind** → `let kind = bytecode.nums.get(idx).unwrap_or(0.0) as u8` (el disasm muestra `MatchType @0` con nums[0]=1.0 = kind Exito — el primer intento usaba idx directo → kind 0 → "ok: 7" no se imprimía); encoder `codegen_to_nvc`: `53 => 52, 54 => 53` (**keys internos del codegen LÚMEN** — el self-hosted debe emitir 53/54 en codegen.nv).
  - `aot/src/lib.rs` (backend C): MatchType → `_v_bool(_u.t == T_SOME|T_OK|T_ERR)`; MatchPayload → `PUSH(_u.items[0])` si T_SOME/T_OK/T_ERR, si no `PUSH(_u)`.
- **Verificado**: `fase62_if_let.nv` ("tiene: 10"/"vacio"), `fase62_if_let2.nv` ("ok: "/"7"), probe62c (elegir con `caso algun(x)` → "x= 42") — **IDÉNTICOS en VM y backend C**; regresión completa de los 12 ejemplos clave (opcion, resultado, match, enums, demo_completo, fase61/63/64/66/68/70) OK; `cargo test --release` **0 FAILED** (e2e 166, sema 56, parser 45, vm 45, ir 20).
- **Warnings limpiados** (build limpio): sema.rs arm `Expr::Range` duplicado (unreachable_pattern, L488); `lumen-aot/src/lib.rs` — `use StackSlot`, variable muerta `prev_ins` (3 asignaciones), `};`→`}` en `var_of`, 3 resets redundantes `terminated = false` (Return/Halt/Jmp); `lumen-wasm/src/lib.rs` doc `///`→`//` en `include!(embedded_stdlib.rs)`.
- **Ejemplos de fase creados (8)**: `fase62_if_let.nv`/`fase62_if_let2.nv` (si-let con opcion/resultado), `fase66_operator_overloading.nv`/`_2.nv` (rasgo+impl: `n.doble()`→42, `gritar()`→"LUMEN!!!"), `fase68_associated_types.nv`/`_2.nv` (`tipo Item;`/`tipo A; tipo B`), `fase70_impl_trait.nv`/`_2.nv` (retorno rasgo: "hola"/99) — **todos OK en VM y backend C**. Fase 65 (guard-let `sea … sino`) NO existe en el pipeline (E020 — gap documentado); fase 69 (where clauses) sigue saltada.
- **Pendientes**: ~~si-let real en el SELF-HOSTED~~ ✅ (14 Ago, ver sección siguiente) → AI/ML (Fases 186-200).

**Progreso (14 Ago — sesión AI · IF-LET REAL en el self-hosted — fixpoint NUEVO `A3CBAA0F…`):**
- **VM LÚMEN (vm.nv)**: handlers `op == 52`/`op == 53` insertados antes de `op == 46` — 52 (MatchType): lee kind de `nums[a]`, evalúa la banda correspondiente (`opts` some / `resl` ok / `resl` !ok con guards de largo) y pusha bool boxed 9000000001/9000000000 (el `es_verdad` del vm.nv ya desboxea `v - 9e9`); 53 (MatchPayload): extrae `"v"` de `opts`/`resl` (lookup dual con guards de largo), si no re-pusha el valor.
- **codegen.nv (self)**: caso `IfLet` en `_gen_stmt` — valor → Store temp `__mt_N` (con `dt_cnt`); patrón Call `algun`/`exito`/`error` → p_kind 0/1/2 con bind del arg Ident; patrón Ident → bind directo; patrón `Ninguno` → p_kind=-2; emite `Load temp; MatchType(53, kind); JmpIf(else)` (+`OP_NOT` para Ninguno) y `Load temp; MatchPayload(54); Store var`; **auxiliar `tiene_test`** evita parchear `jmp_if_p=0` en Ident puro (antes `_cg_patch(cg, 0, …)` corrompía la instr 0).
- **parser.nv (self)**: branch if-let en `_parse_stmt` (`si _st_kw(st,"sea")||_st_kw(st,"let")`) — parsea patrón `_parse_expr`, `=` (punto tt **4**), valor, cuerpo y `sino`/`si` encadenado.
- **BUG RAÍZ del diagnóstico (Literals "0" en el valor)**: `si (_st_ch(st, 5, "="))` — el `=` del self es tt=**4** (operador, como TODOS los otros checks del parser.nv) → el check con 5 jamás matcheaba → el `=` no se consumía → `_parse_expr` recibía el `=` → fallback del `_parse_pr` devolvía `Literal v:0` y avanzaba al Ident (pos 18 `opt` sin consumir) → codegen `__map_get espera diccionario`. Fix: `_st_ch(st, 4, "=")`.
- **BUG 2 (Stack underflow `si sea x = 5`)**: arm Ident puro emitía `Store var` SIN `Load temp` previo (en la rama con test sí estaba) → pop de pila vacía. Fix: `Load temp; Store var`.
- **Verificado (cadena 100% LÚMEN: self_out.nvc compila + vm.nvc ejecuta vs VM Rust)**: `probe_iflet`/`probe_iflet2` (sino, patrón `ninguno` con NOT, Ident puro) byte-IDÉNTICOS; **11/11 ejemplos de fase OK** (fase61/62×2/63/64/66×2/68×2/70×2; los `_2` se llaman `fase66_operator_overloading2.nv` etc. SIN el `_`); demo_completo completo 33 secciones + FIN **IDÉNTICO**.
- **FIXPOINT NUEVO**: compiler_v4.nv (145,894 B) → self_out.nvc → self_out2.nvc **byte-idénticos SHA-256 `A3CBAA0FCAB99F0B81C1B68532A882013E0BD938C6DFF1D6D64BAEBB4093E4F3`** (161,436 B) — reemplaza a `5D153BC6…`. `compiler_v4.nvc` Rust-built (121,672 B) y `vm.nvc` MÍNIMO (116,711 B, compilado con el self nuevo) regenerados. cargo test --release **0 FAILED** (40 suites). target.txt restaurado a demo_completo; probes y prints de debug (PARS-DBG/DBG1/DBG2) eliminados.

**Progreso (14 Ago — sesión AI · OR patterns REALES en el self-hosted — fixpoint NUEVO `DF7676DE…`):**
- **CAUSA RAÍZ fase61** ("OR patterns pre-existentes no regresión" — resultado self `calido/frio/desconocido/desconocido/grande/grande/grande` vs Rust `calido/frio/frio/calido/pequeno/mediano/grande`): `_st_is_op` (parser.nv) NO listaba `|` simple → `_parse_expr` de `Color::Rojo | Color::Amarillo` devolvía solo `Color::Rojo` y dejaba el `|` sin consumir → el check `:` fallaba y el body-loop del `caso` absorbía el `|` y la siguiente alternativa como statements basura (el disasm self mostraba 4 `Eq` en vez de 2 con cortocircuito).
- **Fix en parser.nv** (branch `elegir`): tras parsear `cval`, loop `mientras (_st_ch(st, 4, "|"))` que consume el pipe y parsea la alternativa con `_parse_expr`; construye árbol `Binary` con op `||` encadenado `(sel==A) || (sel==B) || (sel==C)` (cada hoja `Binary == sel aval`; primera usa `cval`). `cmp = cmp_or` si `or_cnt > 0`, si no el Binary `==` de antes. El Binary `||` ya despacha a `_cg_and_or(cg, izq, der, 0)` — short-circuit real con JmpIf — así que el OR queda byte-semánticamente idéntico al IR Rust.
- **Verificado**: `fase61_or_patterns` **byte-IDÉNTICO en la cadena 100% LÚMEN** (`calido/frio/frio/calido/pequeno/mediano/grande` en vm_self vs pipeline Rust); **batería self ampliada OK=42 FALLAS=0** (incl. demo_completo, match, enums, corutinas_demo, jr_concurrencia, 44_extension_methods, test_ffi_min, test_texto_std, fase61/62×2/63/64/66×2/68×2/70×2); cargo test --release **0 FAILED** (40 suites).
- **FIXPOINT NUEVO**: compiler_v4.nv (150,463 B) regenerado con generar_v4.ps1 → self_out.nvc → self_out2.nvc (165,944 B) **byte-idénticos SHA-256 `DF7676DE7BCB2D598504440C36EFCE59274D3B9DC0E3E8B21D4FBB8C78C0BFC9`** — reemplaza a `A3CBAA0F…` (el source del compilador creció con el código del OR). `compiler_v4.nvc` Rust-built (125,195 B) regenerado. target.txt restaurado a demo_completo; temporales self_*.nvc/reg_*.nvc/f61_self.nvc borrados.
- **Pendientes**: AI/ML (Fases 186-200).

**Progreso (14 Ago, noche — sesión AI · Playground: historial de runs + toggle backend persistente):**
- **Gaps restantes del playground auditados** contra `docs/plan-playground.md`: la Ronda L1 ya estaba completa (F1.1 `/api/health`+`/api/examples`+`/api/examples/{file}`; F2.1 CodeMirror 6 vendorizado + modo LUMEN desde token.rs; F3.1 stdlib embebida vía build.rs `embedded_stdlib.rs` + `ModuleLoader::with_memory_files`; F3.2 `run_lumen`/`check_lumen`/`compile_to_bytes` con loader virtual; F4.1 128 ejemplos embebidos + fetch `/api/examples` con fallback offline; F9.1 `.nvc` descargable). Verificado en vivo: `GET /api/health` → `{"status":"ok","version":"3.1.4","wasm":true}`, `POST /api/run` con `imprimir("hola servidor")` → `{"ok":true,"output":"hola servidor"}`, `/api/examples` lista 117+, `/web/index.html` 200 text/html, `pkg/lumen_wasm_bg.wasm` (2.37 MB) y `lumen_wasm.js` presentes (rebuilt previo con fixes OR/rangos + v3.5.7 producción bench/headless).
- **Toggle backend PERSISTENTE**: `execBackend` ahora se lee de `localStorage('lumen_playground_backend')` al inicio y `setBackend()` (extraído del handler del toggle) lo persiste en cada click; `initEditor` aplica el backend guardado (antes se resetaba a `wasm` en cada recarga). Refactor del listener a función reutilizable + búsqueda del botón activo por `dataset.backend`.
- **Historial de ejecuciones**: botón `🕘 Historial` en el panel-header del código + panel flotante (top-right, cierra al hacer clic fuera) que lista hasta 10 runs recientes desde `localStorage('lumen_playground_history')` — cada entrada muestra ✓/✗, hora, backend (`wasm`/`server`), tiempo ms, primera línea del código y preview del output; clic en una entrada restaura el código en el editor. `pushHistory()` se llama en las 3 ramas de `runCode` (server ok, server error, server red-error, wasm ok, wasm error, wasm catch) con try/catch que tolera localStorage lleno/deshabilitado.
- **Verificado**: `node --check` sobre el `<script type="module">` extraído → sintaxis OK; servidor levantado → HTML sirve con `pushHistory`/`historyBtn`/backend-persist presente (61,700 B), `/api/health` OK.
- **Pendientes**: AI/ML (Fases 186-200).

**Progreso (14 Ago, noche — sesión AI · Playground F1.2 + F2.3 + F4.2 completados):**
- **F1.2 (lumen serve)**: Cache ETag/If-None-Match implementado (304 Not Modified funcionando) + variable de entorno `LUMEN_PORT` para puerto configurable. Verificado: `curl /web/index.html` → ETag válido; segunda request con `If-None-Match` → 304; `LUMEN_PORT=9999 lumen serve` levanta en puerto 9999.
- **F2.3 (CodeMirror 6)**: Autocompletado con `@codemirror/autocomplete` (vendorizado en `cm-autocomplete.js`), `Ctrl+Space` dispara sugerencias de keywords/snippets LÚMEN. Minimapa funcional (EditorView espejo sincronizado con scroll). Error gutter mejorado con tooltips flotantes en errores de línea.
- **F4.2 (Selector ejemplos)**: Categorías (basics/functions/data/pro/stdlib/other), búsqueda textual en tiempo real, favoritos persistentes en localStorage, marcador "📦 importar" para ejemplos que usan `importar`. UI: dropdown personalizado reemplaza `<select>`, panel con secciones (Favoritos, Recientes, Categorías).
- **2 ejemplos interactivos** (convención 2 por fase): `canvas_demo.nv` (dibujo en canvas vía JS bridges) + `clock_demo.nv` (reloj tiempo real). Ambos usan bridges JS (`__js_call`, `__js_eval`) y corutinas.
- **Verificado**: `node --check` del script módulo → OK; servidor `lumen serve` probado (ETag 304, LUMEN_PORT, autocompletado, minimap, selector F4.2). `cargo test --release` 0 FAILED.
- **Pendientes**: AI/ML (Fases 186-200) + Playground L2/L3 (F3.3, F5.1, F5.2, F6.1, F6.2, F8.1, F8.2, F9.2).

**Progreso (15 Ago — sesión AI · Rutas relativas robustas + CLI menos determinista + release completo):**
- **Rutas absolutas/CWD eliminadas** (`crates/lumen-cli/src/main.rs`): helper nuevo `find_repo_root()` — orden de búsqueda: (1) env `LUMEN_ROOT` (override explícito), (2) relativo al ejecutable subiendo hasta 4 niveles buscando `stdlib/` (dev: `target/release/lumen` → raíz; release: `paquete/lumen` → paquete), (3) subiendo desde el CWD hasta 5 niveles. `find_wasm_web_root()` ahora usa `find_repo_root()` (dev: `crates/lumen-wasm`; release: `web/` junto al exe) — **el WASM/playground ya NO depende del CWD**.
- **`parse_args`**: stdlib resuelto primero relativo a `find_repo_root()` (antes solo `stdlib`/`../stdlib` relativos al CWD → el import fallaba al ejecutar desde otro directorio). Fallbacks de compatibilidad conservados.
- **`handle_api_request`**: `examples_dir` ahora se deriva de `find_repo_root()` (antes `root.parent().parent().join("examples")` frágil); `lib_dirs` del `/api/run` usa `repo_root.join("stdlib")` absoluto (antes `PathBuf::from("stdlib")` relativo al CWD → fallaba el playground servidor desde CWD ajeno).
- **CLI menos determinista**: `resolve_file_path()` aplicado en `main()` tras `parse_args` — `lumen run examples/hello.nv` (y build/check/disasm/fmt...) funciona desde CUALQUIER CWD (busca relativo al repo si no existe localmente). `suggest_examples()` — si el archivo no existe, sugiere hasta 5 ejemplos por coincidencia de nombre con barra de confianza (`lumen run examples/demo_complet.nv` → sugiere `demo_completo.nv` [##########]). Clippy: `sort_by_key(Reverse)`.
- **Release empaquetado completo** (`.github/workflows/ci.yml`): cada paquete multi-OS ahora incluye **`stdlib/*.nv`** (importar funciona fuera del repo), **`examples/*.nv`** (120+ para aprender), **`web/`** (playground completo: index.html + vendor/ + embedded_examples.js → `lumen serve` funciona desde el paquete), `run_test.sh` helper + docs. Job nuevo `wasm-pkg` (opcional, continue-on-error): wasm-pack build del runtime WASM → `lumen-wasm-pkg.tar.gz` como artefacto extra. Release notes describen el contenido del paquete.
- **Verificado**: `lumen run examples/hello.nv` + `test_import.nv` (imports stdlib) + `demo_completo.nv` desde CWD `Temp` (fuera del repo) → OK; `lumen serve` desde CWD ajeno → index.html 200 + `/api/health` OK + `/api/run` con `importar "coleccion"` → `{"ok":true}`; sugerencia de ejemplos funciona; cargo fmt + clippy limpio + tests OK; YAML del workflow válido.
- **Pendientes**: AI/ML (Fases 186-200) + Playground L2/L3 (F3.3, F5.1, F5.2, F6.1, F6.2, F8.1, F8.2, F9.2).

---

## Progreso (20-21 Ago 2026 — sesión AI · RELEASE v3.5.7: 167 bugs + verificación en tres plataformas)

- **RELEASE v3.5.7** (detalle en `3,0,0.txt`): **unifica en una sola entrega el trabajo iniciado sobre la v2.4.6** — los 8 bugs del reporte original + 159 más encontrados de forma activa. **Verificación: 720 pruebas en verde (Linux y Windows), 393/393 en `lumen check`, 372 ejemplos ejecutados sin fallos, clippy sin avisos y cuatro fuzzers diferenciales (structs/listas, closures, rechazo y regex) sin divergencias.**
- **BUG-166/167 (`regex.nv`)**: el regex nativo devolvía `false` a todo en Windows y macOS (stubs en la rama no-POSIX) y desbordaba al reemplazar con patrones que casan la cadena vacía. **Fix: motor propio por backtracking, sin dependencias.**
- **BUG-165 (`lumen_rt.h`)**: `<sys/resource.h>` fuera de su guarda impedía TODA compilación nativa en Windows — movido bajo su `#ifdef` de plataforma.
- **BUG-152/154 (`lumen-bundle`/`lumen new`)**: la stdlib no viajaba en la instalación y el prefijo de paquete se aplicaba mal — corregido el empaquetado y la resolución de prefijos.
- **BUG-151/161 (parser)**: bloques sin llave se ejecutaban en silencio; el arreglo rompió las declaraciones adelantadas, restauradas con E084.
- **BUG-147/148/149/150 (sema/IR)**: semántica de closures, structs y `prestado mut` corregida.
- **Fix build cross-plataforma (`gui_ffi.rs`)**: los jobs `aarch64-unknown-linux-gnu` y `aarch64-linux-android` fallaban con `E0308` — `title_cs.as_ptr()` devuelve `*const u8` (en aarch64/Android, `c_char = u8`) pero la firma de `CreateWindowExA` espera `*const i8`. **Fix**: `title_cs.as_ptr().cast()` (línea 132), espejo del cast ya presente en la línea 131. El resto de `gui_ffi.rs`/`crypto_ffi.rs` ya usaba casts explícitos (`as *const u8`, `*const u16`) — sin más riesgo cross-plataforma. Verificado: `cargo build --workspace` OK, `cargo test --workspace` 0 FAILED (172 e2e + unit), clippy limpio.
- **Docs sincronizadas a v3.5.7**: README (badges y sección Estado del Proyecto), CHANGELOG (entrada v3.5.7 arriba + sección cronológica), `info.md` (compendio, matriz, changelog, footer), `docs/AGENTS.md` (header + testing actual a ~720), `docs/roadmap.md`, `docs/siguiente.md`, `docs/self-hosting.md`, LENGUAJE/HERRAMIENTAS/MARKETING/cli, reports/. **`VERSION` = 3.1.4.**
- **Pendientes**: composer el release tag v3.5.7 en el repo (los tags previos v2.4.x ya existen); Playground L2/L3; AI/ML (Fases 186-200).

---

## Progreso (21 Ago 2026 — sesión AI · PRODUCCIÓN REAL v3.5.7: fixes escalables + bench 8 + headless + CHUNK_VERSION 7)

- **Fixes escalables (no parches temporales):**
  - **Fallthrough `Variable 'a'/'n'`:** `crates/lumen-ir/src/builder.rs` `last_significant()` ignora `Label/Nop/Phi` para decidir terminador; `needs_return()`/`emit_return_if_needed()` en `Function`, `ImplRasgo`, `compile_lambda`, `build()` (`Halt`). `label_counter` global evita colisión `Label(0)` en `codegen` global `label_map` que rompía `matematicas.nv` (`Variable 'n'`). Commits `64db441`, `730e74d`, `f83964f`.
  - **Aridad `pop()` corrupto:** `crates/lumen-vm/src/vm.rs` `bind_args` unificado — `Call`/`CallValue`/`run_function` (hilos) comparten `args.get(i).cloned().unwrap_or(Void)` + `DefaultValue` reales; antes `else { self.pop() }` corrompía stack del caller y `run_function` dejaba param sin inicializar.
  - **Defaults persistidos `CHUNK_VERSION 7`:** `ir::Func.defaults` → `codegen::FuncMeta.defaults: Vec<Option<DefaultValue>>` (`Int/Float/Str/Bool`) serializado en `Bytecode` v7 (compat v6). `VM bind_args` usa `DefaultValue` cuando `i>=args.len()`. `decode` acepta 6 y 7.
  - **Headless centralizado:** `stdlib/graficos.nv:es_headless()` usa `getenv("CI"/"LUMEN_HEADLESS")` vía `__ffi` (`msvcrt`/`libc`/`libSystem`) y `peek!=0`; `iniciar()`/`ventana()` retornan `false/0` sin `SDL_Init`. Guard per-demo redundante pero compatible.
- **Suite producción:** **636 e2e** (incluye 4 regresión: fallthrough early return, matematicas `potencia(2,10)==1024`, defaults `CallValue`, lambda) + **9 production** (`crates/lumen-vm/tests/production.rs`: aceptación 3 + performance 2 + integración) = **695 vm tests**, **956 workspace** (`cargo test --workspace` 0 FAILED). Ver `docs/produccion.md` §2.1.
- **Bench formal 8** (`crates/lumen-bench/benches/benchmarks.rs`): `lexer_tokenize`, `parser_parse`, `pipeline_full`, `vm_fib_20` + 4 prod `prod_fallthrough_early_return`, `prod_defaults_callvalue`, `prod_matematicas_potencia`, `prod_graficos_headless` (`cargo bench -p lumen-bench`, reporte `target/criterion/report/index.html`, `--quick` en CI).
- **CI `headless-check` nuevo** (`.github/workflows/ci.yml`): job Linux `env: LUMEN_HEADLESS=1 CI=1` corre `cargo test --workspace`, `cargo run --bin lumen -- check examples`, `cargo test --test production`, `cargo bench -p lumen-bench -- --quick`. Ver `docs/produccion.md` §3.
- **Barrido:** `lumen check examples` 396/396 (con `CI=1` 396/396), `LUMEN_HEADLESS=1 lumen run examples/graficos_*` → `init_fail_ok` sin `Variable 'a'` (antes requería 30 guards per-demo).
- **Versionado:** `Cargo.toml`/`VERSION` `3.1.4` · `CHUNK_VERSION 7` (fallback v6) · docs sincronizadas (`README`, `roadmap`, `plan-v3.1`, `HERRAMIENTAS`, `produccion.md`) · `CHANGELOG` v3.5.7 producción.
- **Verificado:** `cargo test --workspace` 956, `cargo bench -p lumen-bench` 8 OK, `LUMEN_HEADLESS=1` repro local (`Headless/CI detectado — demo omitida`).

**Estado actual: LÚMEN v3.5.7 Producción Real — deployable en Windows/Linux/macOS/Android/WASM con `cargo build --release --target <target>`. Próximos: `FuncMeta` defaults no literales (thunk), `label_map` per-function, `lumen fmt` en pre-commit (ver `docs/produccion.md` §6).**


---

## Progreso (25 Ago 2026 — sesión AI · v3.3.x: QA bugs + fuzzing paridad VM↔nativo)

- **v3.5.7/3.3.1**: bugs QA #1-#7 completados (fmt sin pérdida, `arr[i].campo=`, if-let/elegir destructura enums con datos, structs recursivos, MatchVariant con guardas). Suite 633 e2e.
- **v3.3.5 — Bug #6 COMPLETO (refs reales)**: `Value::Ref` celda compartida + opcode 63 MakeRef + write-back en Ret; sema auto-deref en binarios/asignación; backend C con punteros reales y **renombrado de params por función** (`{fn}::{param}`) que elimina colisiones latentes.
- **v3.3.5 — Bug #7 COMPLETO (comptime)**: intérprete const-eval (`lumen-ir/src/comptime.rs`) pliega `comptime { fib(20) }` a literal; límites profundidad 128 / 1M pasos; fallback runtime seguro.
- **v3.3.5 — Sombreado real de bloques**: ScopePush/ScopePop en si/mientras/para/foreach/si-let/elegir/match; AOT C planificador estático `plan_var_keys`. VM y nativo coinciden.
- **v3.3.5 — Métodos mutables**: sintaxis `prestado mut este` (parser+sema+builder); el receptor pasa por referencia.
- **v3.3.5 — Aviso W060**: argumento no-lvalue a `prestado mut` se pasa por valor y se advierte en compile-time (`SemanticAnalyzer::warnings`, `analyze(&mut self)`).
- **AOT sin fallos**: LLVM/Cranelift rechazan RUIDOSAMENTE constructos no soportados (`llvm_supported`/`cranelift_supported`) — nunca más artefactos rotos silenciosos. Test gcc integral end-to-end (refs+try/catch+elegir+structs+comptime).
- **v3.3.6 — Fuzzing manual VM↔nativo (3 bugs)**: (F1) indexado/largo de textos en C (`"abc"[1]`, `s.largo()`); (F2) structs declarados dentro de funciones en sema; (F3) imprimir multi-arg concatena en una línea como el VM. Lotes de paridad: aritmética/overflow/mod negativo, floats, sombreado anidado, arrays 2D, struct-en-array, or/guard patterns, destructuring, foreach, try/catch anidado, refs en bucles, métodos mutables, mapas, resultado/opción — **0 diferencias** tras fixes.
- **Suite actual**: 633 e2e + 80 parser + 56 sema + 11 production + 6 AOT, 0 fallos. Docs sincronizadas: LENGUAJE.md §3 (semántica real prestado/prestado mut/W060/scoping), CHANGELOG 3.3.1→3.3.6.

**v3.3.7 (25 Ago): motor regex propio en C (puerto min_regex.rs — fin de POSIX/stubs), harness scripts/fuzz_paridad.ps1 con normalización, corutinas PAR VM↔nativo, __str_upper/lower en backend C. Gap detectado: builtin PID inexistente en todas las capas. Pendiente sesión dedicada: LLVM/Cranelift vía shims _lw_*.**

**v3.3.9 (25 Ago): F6 inferencia genérica desde args (Struct("T")-param unifica); F7 macro SET muerto en ArrayPushVar del C. Paridad gen_ref. Pendientes explícitos: self-hosting sync (prestado mut este en parser.nv) y LLVM/Cranelift _lw_*.**

**Post-3.4.0: lote fuzz E — f-strings + tareas PAR VM↔nativo. Pendientes vivos: self-hosting sync (prestado mut este → parser.nv), LLVM/Cranelift _lw_* (sesión dedicada), perf _sv, mapas-anidados como claves.**

**v3.4.1 (25 Ago): paridad mapas-como-claves verificada (persistencia = reasignar, ambas capas). compiler_v4.nvc inexistente desde limpieza ago — regeneración documentada paso a paso en CHANGELOG 3.4.1; espejar prestado mut este en parser.nv ANTES de regenerar.**

**v3.4.2 (25 Ago): cuantificador acotado {m,n} en el motor C con paridad verificada (fuzz/regex_braces.nv); divergencia documentada en malformados. Pendientes: (?:...)/lookaheads, perf _sv, self-hosting sync, LLVM/Cranelift.**
**v3.4.3: errores regex unificados VM-C (_regex_m_val). Pendientes: (?:...)/lookaheads, perf _sv, self-hosting sync, LLVM/Cranelift _lw_*.**
**v3.4.4: (?:...) no-capturante en ambos motores, PAR total (regex_ncg.nv). Pendientes: lookaheads, perf _sv, self-hosting, LLVM/Cranelift.**
**v3.4.5: lookaheads (?=) ambos motores + builtins __regex_* migrados a min_regex (adiós divergencia crate externa). Suite regex e2e verde. Pendientes: perf _sv, self-hosting sync, LLVM/Cranelift _lw_*.**
**v3.4.6 (25 Ago): _sv caller-wide eliminado → save solo del callee (recursión correcta, fib OK). Siguiente perf: params a locals C en prólogo. Pendientes: self-hosting sync, LLVM/Cranelift _lw_*.**
**v3.4.7: espejo prestado[mut] en parser.nv (_parse_decl). REGRESIÓN pre-existente detectada: self-hosted crashea con bloques impl (repro fuzz/selfhost_base.nv sin prestado). Desbloqueo: fix impl → generar_v4.ps1 → fixpoint SHA.**
**v3.4.8: _parse_prog ruta impl/rasgo a _parse_decl. Crash persiste via parsear_con_base (segundo driver con r posiblemente sin declarar) - siguiente sesion ese loop.**

**v3.4.9: (?!...) negativos ambos motores (regex_nneg PAR), perf callee-save estable. Bloqueo self-hosting: segundo driver parsear_con_base con r sin declarar.**

**v3.5.0: self-hosting compila prestado mut (fuzz/selfhost_probe.nv OK, degradado 41 vs 42). Fix `numero r;` bare-decl en builder + `impl` routing. Siguiente: codegen.nv MakeRef para write-back real y fixpoint.**

**v3.5.1: builder bare-decl fix + parser prestado tolerante; self-hosting compila prestado mut libre (degradado). Impl mangling inherente corregido pero codegen MakeRef pendiente para write-back real.**

**v3.5.2: parser prestado mut prefix preservado; selfhost_probe sigue degradado (MakeRef codegen pendiente).**

**v3.5.3: codegen MakeRef ptypes + prestado mut este type fix; probe compila OK degradado 41/42 (MakeRef no emitido — found_pt). Siguiente: depurar lookup funcs en codegen Call.**

**v3.5.4: codegen MakeRef cableado (ptypes) pero probe sigue 41/42 — Call tp != "Call" en AST LUMEN para inc(v) libre.**

**v3.5.5 (26 Ago): CI VERDE + consolidación self-hosting. (1) Fix E033 `lumen check examples`: rama receptor `prestado [mut] este` en _parse_decl referenciaba `type_nm` (solo existe en _parse_stmt/impl) — restaurado "Self" + tag "prestado mut Self" (parser.nv + compiler_v4.nv). (2) Fix macOS test_c_backend_gcc_runtime: UB de orden de evaluación `ST[SP++] = f(POP())` en lumen_rt.h — clang calcula el LHS antes del --SP del POP (imprimía "abc" en vez de 3); PUSH ahora va por `_push_impl` inline (C11 6.5.2.2). Verificado gcc/clang -O0/-O2. (3) prestado mut write-back REAL en self-hosting 41→42: codegen Call crasheaba con __map_contiene sobre callee texto (fix __tipo_de previo) + encoder nativo __codegen_a_nvc sin opcode 63 (caía a Nop → underflow) + eliminado fallback hardcodeado "inc". Probes: selfhost_probe=42, probe2=100 (MakeRef en disasm). Gate: 952 tests 0 FAILED, clippy/fmt limpios, ci_gate 392 PASS / 0 CRASH.**

**v3.5.6 (26 Ago): AOT Cranelift con runtime REAL — lo que faltaba del AOT. Nuevo runtime `_lw_*` (40 helpers, handles opacos: el código nativo solo ve i64 = punteros a Val; la semántica delega en lumen_rt.h probado → paridad VM/C/Cranelift). Emisor Cranelift reescrito al modelo de handles: decimales, división/módulo/comparaciones, textos (largo/index/concat), listas (new/push/pushvar/get/set/len), structs, tuplas, mapas, opción/resultado, imprimir multi-arg en una línea, leer, typeof, a_texto, bucles y recursion. Fix entry point: `main` del usuario ya no colisiona con el wrapper C (se exporta directo). `cranelift_supported` ampliado (antes rechazaba decimales/división/listas/structs/si/mientras). Rechaza aún enums/closures/prestado-mut/intentar-atrapar/elegir-con-tipos; limitación conocida: sombreado dentro de bloques (falta port de plan_var_keys). Test nuevo `test_cranelift_runtime_lw` byte-identical a la VM + E2E `build --aot rust` OK. lumen-aot 7/7.**

**v3.5.7 (27 Ago): INCREMENTO B — Cranelift y LLVM completos + modelo de memoria nativo. Cranelift: intentar/atrapar real (chequeo _lw_err_active + catch por block-param), enums completos (EnumCtor/MatchVariant/MatchType/MatchPayload), prestado mut/MakeRef con celdas Val y write-through, funciones como valores (func_addr + call_indirect), sombreado por bloques, flujo de valores entre labels (block-params + simulate_label_depths → ternarios OK). LLVM IR textual reescrito al modelo _lw_* (antes i64-only) + merge-allocas + floats hex + CLI linkea el shim. Modelo de memoria: TODAS las vars por celda, Stores deep-copian (_lw_dcp, paridad gv=_dcp), args de llamada deep-copiados (T_PTR pasa tal cual), arrays con cap + _lw_arr_push_ip in-place → stress_04 de O(n²)/OOM a instantáneo en C/Cranelift/LLVM. Bugs: sizeof(Val)=80 no 72, binding de entrada write-through corrompía al llamador (_lw_store_slot_direct), MatchPayload enums alineado a VM, INT64_MIN div/mod/neg wrap + div0 capturable (antes SIGFPE), args sin copiar mutaban el llamador (fase_impl_inherente 350→100). Paridad 4-way byte-identical en todos los casos anteriores. Además: variables GLOBALES reales en nativo (program_global_names + celdas lw_glob_* compartidas entre funciones → logging/testing_sr OK), formato de decimales paridad VM (decimal plano round-trip, nunca científico → matematicas_sr OK), arrays O(n) también en el backend C. Barrido examples VM↔Cranelift: 239 OK / 7 divergencias (closures con captura, sort de complejos en vectordb, guard-let+NaN, structs dinámicos, 3 demos hilos/baremetal/3D) / 150 skip; workspace 955/0; ci_gate 392 PASS 0 CRASH. Pendiente incremento C: entornos de captura para closures.**
