# AGENTS.md — Diario de construcción de LÚMEN

**v2.3.0 — Released: Julio 2026**

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

**0 warnings, ~378 tests passing. 45/45 ejemplos funcionando. 166 e2e, 45 unit.**

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
`compiler_v2.nv` via `__compile_nv` nativo — compilador LÚMEN que compila .nv → .nvc en milisegundos. Self-compilación verificada: `self_compile.nv` → `compiler_v2_self.nvc` en **533ms**. ~378 tests, 0 fallos.

### Sprint 4: Self-hosting Total (30 Julio 2026) ✅
`Value::Map(Vec<...>)` → `HashMap<Value, Value>` con `Hash`/`Eq` manual. `__map_get`/`__map_set`/`__map_contains` O(1). Sets O(n). El parser LÚMEN-in-LÚMEN ahora tiene mapas O(1). Camino abierto para self-hosting total sin `__compile_nv`.

### Sprint 5: Self-hosting Puro COMPLETADO (31 Julio 2026) ✅
**LÚMEN se compila a sí mismo sin `__compile_nv`.** `compiler_v4.nv` autocontenido (55,308 bytes = lexer+parser+codegen+main concatenados). `compiler_v4_self.nvc` (54,712 bytes, 49 funciones) recompila su propio source con resultado IDÉNTICO — fixpoint confirmado en 3 runs consecutivos (193s/203s/197s). Bugs críticos arreglados: JmpIf target→nums, If sin JMP de skip, indexación `chars[i]`, TryUnwrap 40→40, print multi-arg, break/continue con loop_stack, VarDecl sin inicializador, escapes `\n \t \r \" \\` en lexer puro, keywords void/diccionario, forward declarations.

### Sprint 6: Prueba de Fuego + Imports + Gramática (31 Julio 2026) 🔄 EN CURSO
**Harness:** `fuego.ps1` compila los **115 ejemplos** de `examples/` con el pipeline puro (`target.txt` = driver parametrizado, 2 líneas: entrada/salida; main recorta `\r` CRLF y propaga errores vía TryUnwrap en `ejecutar_pipeline()`).
**Resultados:** 115/115 COMPILAN (parser puro tolerante) · 29/115 ejecutan CORRECTO (nvc == rust) · 84 incompatibles · 2 timeouts (debug_parser3 loop infinito, gui_ventana GUI).
**Gaps mapeados (15):** `importar` (~50 ejemplos), `sea` if-let (~20), `const` (4), `para` (5), `estructura`+`.campo`+`T{}`, `enum`, `opcion`/`resultado`, `elegir`, closures `|x|`, params default, genéricos `<T>`/`rasgo`, destructuring `_`, tuplas `(...)`, arrays anidados `arr[i][j]`, TryUnwrap top-level silencioso.
**Siguiente:** 6.1 `importar` en parser puro (fusión de ASTs + prefijo `modulo_`) → compiler_v5 modular → 6.2 keywords/gramática → Sprint 7 VM en LÚMEN (`vm.nv`) + optimización ~200s→<10s → Sprint 8 dogfooding stdlib + release v2.4.0. Plan completo en `docs/self-hosting.md`.

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
- **Pendientes**: `enum`/`elegir`/`sea` reales (parsing tolerante ya, sin codegen) → `importar` con prefijo real verificado en fuego.ps1 → **Sprint 7: VM en LÚMEN (`vm.nv`)** + optimización de velocidad (fixpoint ~10-18 min hoy; demo ~15s) → Sprint 8 dogfooding stdlib + docs → release v2.4.0

**Progreso (1 Ago, 21:30-23:00 — sesión AI · 6.4 enum/elegir/sea REALES):**
- **`sea`/`let` → VarDecl real** (parser.nv:456): `sea [tipo] nm = expr;` → nodo VarDecl (tipo_var="let" o el tipo) — antes era skip-keyword y `sea x = e` se re-parseaba como Assign
- **`Nombre::Miembro(args)` → EnumInit** (parser.nv:187): con `(...)` parsea args; sin args → argc=0. VM imprime `Resultado::Exitoso(100)` nativamente (Display de Value::Enum) y `Eq` compara name+variant+fields (vm.rs:2551-2562)
- **`defecto:`/`default:` REAL** (parser.nv elegir): break-condition incluye `defecto`; post-check consume la keyword, parsea el body → se adjunta como `sino` del último If (o Block si solo-default)
- **BUG RAÍZ de la cadena `elegir` — `__map_poner` NO muta**: `im::HashMap` es persistente (insert → copia estructural; vm.rs:577-589). `last = __map_poner(last,"sino",ifn)` jamás propagaba a `first` → SOLO el caso 1 se compilaba. El demo 89/89 lo ENMASCARABA (solo llama `color_str(Color::Rojo)` = caso 1). Fix: `ifns[]` (lista por índice) + cadena reconstruida DESDE EL FINAL (`chain = __map_poner(ifns[i2],"sino",chain)` descendiendo). Semántica clone-copy es ESENCIAL para los probes `st_dos = _st_adv(st)` del parser
- **Codegen**: `OP_ENUM_CTOR=43/OP_ENUM_VAR=44/OP_ENUM_ARGC=45` + caso `EnumInit` en `_gen_expr` (args en orden + trío consecutivo). **vm.rs codegen_to_nvc**: 43 → `WithIdx(EnumCtor, str)`; 44 → `WithIdx(Nop, str)`; 45 → argc (ints→f64) en nums + `WithIdx(Nop, num)` — el VM EnumCtor lee ip+1/ip+2 (vm.rs:3720-3757)
- **Resultados**: match.nv IDENTICAL (2 casos + defecto) · enums.nv IDENTICAL (unit + data variants + print directo) · demo 90/90 0 diffs · cargo test OK · 43_tipos_asociados.nv NO soportado por NINGÚN compilador (usa `rasgo`/traits)
- **Hygiene**: debug residual LEX-LOOP/LEX-DBG retornar ELIMINADO de lexer.nv · AST dump temporal removido de generar_v4.ps1 · fuego.ps1 completo corriendo en background (115 ejemplos)
- **Pendientes**: revisar resultados fuego.ps1 → fixpoint v4 (regresión crítica tras el cambio de elegir) → Sprint 7 VM en LÚMEN (`vm.nv`) + optimización → Sprint 8 dogfooding + release v2.4.0

**Progreso (2 Ago, 20:00-21:30 — sesión AI · 6.5 cortocircuito `&&`/`||` + FIXPOINT v4 CONFIRMADO):**
- **REGRESIÓN del fixpoint diagnosticada (causa raíz)**: `compiler_v4_self.nvc` (94,783 B) crasheaba con "Índice N fuera de rango (largo: N)" dentro de `ejecutar_pipeline` (también al compilar match.nv: "Índice 96"). Inspección del disasm (`self_disasm.txt` vs `v4_disasm.txt`) probó la diferencia: el **pipeline RUST (v4) cortocircuita `&&` con `JmpIf`** (`Load; Lt; Store; Load; JmpIf → salta ANTES del ArrayGet`), mientras el **codegen LÚMEN emitía `&&`/`||` como `And`/`Or` eager** (`…Lt; ArrayGet; Neq; And`) → `mientras i < n && cs[i] != "\n"` lee `cs[i]` con `i == n` → ArrayGet out-of-bounds solo en el SEGUNDO-compilado
- **Fix**: helper `_cg_and_or(cg, izq, der, es_and)` en codegen.nv — emite short-circuit REAL con `JmpIf`/`Jmp` + `PushBool` (es_and=1 para `&&`; es_and=0 para `||`). Binary en `_gen_expr` despacha `&&`/`||` al helper (return temprano); los demás operadores siguen eager. Convención de saltos con FALSY (vm.rs:3688)
- **FIXPOINT v4 CONFIRMADO**: compiler_v4.nv (99,993 B) → compiler_v4.nvc → self (112,368 B, exit 0) → self→self2 (112,368 B, exit 0) → **byte-IDENTICAL (0 diffs)**. El self-compilado compila match.nv sin crash (antes "Índice 96 fuera de largo")
- **fuego.ps1 completo**: **116/116 compilan · 61 CORRECTOS (+1 vs 60) · 53 INCOMPATIBLES · 2 timeouts · 0 fallos** — sin regresión (bug pre-existente en fuego.ps1:66: `$outNvc.Trim()` sobre `$null` en el detalle de INCOMPATIBLES — no afecta los contadores)
- **Hygiene**: scratch `v4_disasm.txt`/`self_disasm.txt` duplicados — limpiar; `target.txt` aún apunta a `examples/match.nv`
- **Pendientes**: Sprint 7 VM en LÚMEN (`vm.nv`) + optimización de velocidad (fixpoint ~10-18 min hoy; demo ~15s) → Sprint 8 dogfooding stdlib + release v2.4.0

**Progreso (4 Ago — sesión AI · Sprint 7 VM en LÚMEN: corutinas reales + batería 27/28):**
- **Corutinas implementadas en vm.nv** (modelo de intercambio de contexto cooperativo como vm.rs): `__coro_crear` boxea `coro_N` y guarda el nombre en `coro_nombres_m` (el fidx se resuelve en el PRIMER `__coro_reanudar` porque `funn` solo existe en main, no en bin); `__coro_reanudar` guarda main en guards (st/sp/pc), inicializa st nuevo (pila/mem/scopes/rets propios, mem heredado) si `coro_inic_m==0`, carga st/sp/pc, `coro_actual=cidx`; `__coro_ceder` guarda coro (sps=sp+1, pcs=pc+1) y restaura main; op22 Ret con `coro_actual>=0` marca done, restaura main.
- **Bug 1 (hang)**: el placeholder de `coro_fidx_m` era `0.0` pero el check es `si cfi2 < 0` → cfi2=0=`"__main__"` → `pc=fun[3]` = inicio del main → bucle infinito re-ejecutando main. Fix: placeholder `-1.0`.
- **Bug 2 (re-ejecución)**: `a_entero(__map_obtener(coro_stats_m, cidx))` sobre el MAPA stc guardado da 0 (un mapa no se parsea) → `==0` siempre TRUE → re-inicializaba la corutina en cada resume ("A: inicio" ×3). Fix: flag separado `coro_inic_m` (0/1).
- **Bug 3 (VM Rust)**: `Opcode::Ret` (vm.rs) con `call_stack` vacío → `ip=usize::MAX` → el programa moría en silencio: al retornar dentro de la corutina (tarea_a nunca se llamó con Call, se saltó con `ip=coro.ip`) la VM Rust NO imprimía `--- FIN ---` (la LUMEN sí, correctamente). Fix en vm.rs: si `current_coro` está activo → `coro.is_done=true`, restaura `main_saved`, continúa.
- **Resultados**: corutinas_demo **byte-IDENTICAL** (252 B, 0 diffs) con el flujo completo (inicio→yield→primer yield→segundo yield→ret→FIN); demo_completo sigue 0 diffs; batería 8/8 en los re-probados incl. corutinas_demo; cargo test 0 fallos (e2e 166).
- **Pendientes**: `44_extension_methods` + `math` (fallan también en VM Rust, pre-existentes — compiler issue con `este`/extension methods) → batería ampliada completa → Sprint 8 dogfooding stdlib + release v2.4.0
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
- **Pendientes**: corutinas reales (`__coro_reanudar`/`__coro_ceder` con intercambio de contexto st/sp/pc — modelo cooperativo como vm.rs:1679-1734) → batería ampliada completa → Sprint 8 dogfooding stdlib + release v2.4.0

**Progreso (4 Ago — sesión AI · Sprint 7: OPTIMIZACIÓN fixpoint 861s → 20.1s, 43x — COW con Arc):**
- **Profiler per-opcode en `VM::run()`** (vm.rs ~2320, gated por `LUMEN_PROFILE=1`): contadores/tiempos por opcode; `Call` desagregado como `Call:<nombre>` vía `bytecode.names`. Fix: check `!var.is_empty()` (antes `is_ok()` → env vacío lo activaba)
- **Diagnóstico O(n²)** (demo 4KB, 962,917 instrs, 1.7s): Load 48% (2.5µs/call — clona lista `chars` por token), ArrayGet 18% (45µs/call — clona la lista en cada acceso), `__str_subcadena_chars`+`sub_from_chars` 18.6% (~417µs/call — `a.clone()` de TODO el array en cada slice). Escalamiento 23.7x bytes → ~506x tiempo = O(n²) (23.7²=562)
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
- **Pendientes**: COMPILA-FALLA `test_texto_min`/`test_texto_std`/`jr_concurrencia` → `44_extension_methods`/`math` → Sprint 8 dogfooding stdlib + release v2.4.0 → bootstrapping doble.

---

## Bugs Conocidos y Fixes Recientes

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
| **Verificado**: fixpoint v4 CONFIRMADO — compiler_v4.nv → self (112,368 B) → self2 byte-IDENTICAL (0 diffs); self compila match.nv sin crash ("dos"); fuego 116/116 compilan, 61 CORRECTOS (+1), 0 fallos | — | — |

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
| cargo test | ~378 tests, 0 fallos | Todos los tests pasan |

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
  lumen-api/      → lib.rs
  lumen-plugin/   → lib.rs
docs/spec/        → grammar.ebnf, bytecode-format.md, error-codes.md, vm-spec.md
examples/         → *.nv (45 ejemplos funcionales)
stdlib/           → *.nv + stdlib/compiler/ (self-hosting)
scripts/          → PowerShell CI/CD, installers, git-hooks
```
