# PLAN: LÚMEN — Independencia Total (Self-Hosting como C/Rust)

**Objetivo:** LÚMEN se autocompila sin depender de Rust. El compilador, la VM y el runtime están escritos en LÚMEN. Bootstrap ocurre UNA sola vez con Rust. A partir de ahí, LÚMEN vive por sí mismo.

---

## Estado Actual — 4 Agosto 2026 (Sprint 7 🟢 — VM en LÚMEN funcional)

### Progreso

| Componente | Estado | Notas |
|-----------|--------|-------|
| **Sprint 3: Bootstrap rápido** | ✅ 533ms | `__compile_nv` + builtins string eficientes + ArrayGet optimizado |
| **Sprint 4: Self-hosting Total (mapas O(1))** | ✅ | `Value::Map` → `im::HashMap` con `Hash`/`Eq` manual |
| **Sprint 5: Pipeline puro LÚMEN** | ✅ | `lexer.nv`→`parser.nv`→`codegen.nv`→`__codegen_a_nvc` sin `__compile_nv` |
| **Sprint 6: Imports + Gramática** | ✅ | 6.1 imports ✅ · 6.2 gramática ✅ · 6.3 lexer CRLF ✅ · 6.4 enum/elegir/sea reales ✅ · 6.5 cortocircuito `&&`/`\|\|` ✅ · fixpoint v4 ✅ |
| **Sprint 7: VM en LÚMEN (`vm.nv`)** | 🟢 funcional | Ejecuta `demo_completo.nvc` **89/89 líneas 0 diffs en ~0.9s** (era ~120s); corutinas reales byte-IDENTICAL; batería test_vm.ps1 **39/40** (solo `stress_fecha` flaky timing) |
| **Optimización fixpoint** | ✅ **861s → 20.1s (43x)** | COW con `Arc` en Value (vm.rs) — clonado O(1) de strings/arrays grandes; fixpoint v4 byte-IDENTICAL |
| **Bootstrapping doble** (vm.nv compilada por LÚMEN y auto-ejecutándose) | ⏳ | Próximo hito — 0 dependencias de Rust |
| **Optimización fixpoint** | ✅ | Fixpoint v4 re-verificado en **5s** (112,368 B byte-IDENTICAL, 4 Ago) |
| **Sprint 8: Dogfooding + release v2.4.0** | 🟢 en curso | **fuego: 116/116 compilan · 75/116 CORRECTOS (+14)** con la cadena 100% LÚMEN (**opcion.nv + resultado.nv ahora OK+CORRECTO** vía Option/Result reales; 38 no-corregidos son gaps: traits, closures, FFI/GUI, tuplas, timing) — fixpoint v4 **113,857 B byte-IDENTICAL** |

### Arquitectura (actualizada)

```
┌──────────────────────────────────────────────────────────────┐
│     COMPILADOR LÚMEN EN LÚMEN (stdlib/compiler/)              │
│                                                              │
│  compiler_v4.nv  ✅  — autocontenido (lexer+parser+codegen+   │
│                        main concatenados, ~100 KB)            │
│  • Pipeline: leer .nv → lexer → parser → codegen → .nvc      │
│  • Fixpoint confirmado: self recompila su source IDÉNTICO     │
│  • Compila los 116 ejemplos de examples/ (fuego.ps1)          │
│                                                              │
│  VM EN LÚMEN (vm.nv)  🟢 funcional  — ejecutador de .nvc      │
│  • Dispatch opcodes 0-46, builtins bin() vía natives boxeados │
│  • Bands boxed: arrays < -1e9, strings [1e9,2e9), mapas       │
│    [2e9,3e9), struct 3e9, resultado 4e9, opcion 5e9, tupla    │
│    6e9, enum 7e9, fn 8e9, bool 9e9                            │
│  • demo_completo 0 diffs ~0.9s · corutinas reales (yield/     │
│    resume/ret, intercambio st/sp/pc) · tareas, JSON, crypto,  │
│    fs, env, tiempo — todo delegando a natives boxeados        │
│                                                              │
│  Bootstrapping doble  ⏳  — compiler_v4 compila vm.nv →       │
│  vm.nvc; la VM LÚMEN ejecuta vm.nvc (ejecutando .nvc)         │
└──────────────────────────────────────────────────────────────┘
```

### Bugs críticos arreglados en el pipeline puro (31 Julio 2026)

| Bug | Efecto | Fix |
|-----|--------|-----|
| `Jmp`/`JmpIf` con target directo | Loop infinito (VM lee `nums[idx]`) | Serializar target en tabla `nums` + arg = índice |
| If sin `JMP` tras then-body | Ejecutaba else tras then | Emitir JMP + backpatch (`_cg_patch`) |
| `chars[i]` no parseado | Indexación ignorada | Postfix `[expr]` en `_parse_pr` + `Index` en codegen + `OP_ARRAY_GET` |
| `intentar`→Nop en `cg_to_vm` | `Exito(Str)` quedaba en stack | `40 => 40` (TryUnwrap) |
| Print multi-arg → 1 solo Print | Output invertido/parcial | Un Print por argumento en orden |
| Break/Continue ignorados | Tokens duplicados en lexer | `loop_stack` con backpatches (breaks→fin, conts→loop_start) |
| `numero r;` sin inicializador | Stack underflow | PushInt 0 por defecto |
| Escapes de string no procesados | `"\""` rompía tokenización | `\n \t \r \" \\` convertidos en el lexer puro |
| `void`/`diccionario` no-keywords | Funciones corruptas (name="void") | Añadidas al mapa `kw` del lexer |
| Forward declaration `funcion X(...);` | Se tragaba la siguiente función | `;` tras firma → nodo `Vacio` (ignorar) |

### Tiempos de autocompilación (31 Julio 2026)

| Run | Pipeline | Tiempo | Resultado |
|-----|----------|--------|-----------|
| v4 inicial (im::HashMap, pre-fixes) | Rust build | 173s | 2,780 bytes rotos |
| v4 con \r fix + funcs | Rust build | 130s | 47 funcs, fallaba runtime |
| v4 con pipeline completo | Rust build | 193s | 54,712 bytes ✓ |
| **self** (fixpoint run 1) | LÚMEN puro | 203s | 54,712 bytes ✓ |
| **self** (fixpoint run 2) | LÚMEN puro | 197s | 54,712 bytes ✓ IDÉNTICO |

---

## Sprint 6 — Prueba de Fuego + Imports + Gramática (31 Julio - 2 Agosto 2026) ✅ COMPLETADO

### Harness

- **`fuego.ps1`**: compila los 115 ejemplos de `examples/` con `compiler_v4.nvc` (pipeline puro) y ejecuta cada `.nvc` generado comparándolo contra el pipeline Rust (`lumen run`), con timeout de 8s por ejecución
- **`target.txt`**: driver parametrizado en el main de `compiler_v4.nv` (2 líneas: entrada `.nv` / salida `.nvc`); main recorta `\r` (CRLF) y reporta `FALLO: <error>` con propagación real vía TryUnwrap en función (`ejecutar_pipeline`)
- **Bug del driver descubierto**: `\r` final en target.txt → nombre de archivo inválido en Windows (0x0D prohibido) → `fs::write` fallaba en silencio (TryUnwrap en top-level sin frame no propaga)

### Resultados (115 ejemplos)

| Métrica | Cantidad | Detalle |
|---------|----------|---------|
| **Compilan** (pipeline puro) | **115/115** | El parser puro no se cae con NINGUNA sintaxis (tolerante, sin sema) |
| **Correctos** (nvc == rust) | **29/115** | if/while/funciones/variables/print/break/continue/debug_parser/TUI mini |
| **Incompatibles** | **84/115** | Error en nvc o output distinto |
| **Timeouts** | 2 | `debug_parser3` (loop infinito), `gui_ventana` (GUI nativa) |

### Mapa de gaps — gramática faltante en el pipeline puro

| # | Gap | Ejemplos afectados (muestra) | Error típico en nvc |
|---|-----|------------------------------|---------------------|
| 1 | **`importar` (módulos)** | ~50: test_math_*, test_import, test_stdlib*, test_texto*, testing, tui_*, graficos_* | `Función 'matematicas_abs' no definida` |
| 2 | **`sea` (if-let/while-let)** | ~20: jr_fecha, sr_fecha, real_logger, test_sistema_*, test_ffi_*, test_red_*, tui_jr, sprint1_* | `Variable 'sea' no definida` |
| 3 | **`const`** | audio_demo, graficos_avanzado, graficos_canvas_demo, tilemap_demo | `Variable 'const' no definida` |
| 4 | **`para` (for/foreach)** | foreach, demo_completo, tui_test_min16-19, stress_test | `Variable 'j'/'r'/'total' no definida` |
| 5 | **`estructura` + `.campo` + `T { }`** | structs, charts_demo, tui_temas_demo | `Variable 'estructura' no definida` / `Función 'graficos_charts_Serie' no definida` |
| 6 | **`enum`** | enums | `Variable 'enum' no definida` |
| 7 | **`opcion` / `resultado`** | opcion, resultado | `Variable 'opcion'/'resultado' no definida` |
| 8 | **`elegir` (match)** | match | `Función 'elegir' no definida` |
| 9 | **Closures `\|x\|`** | lambda | `Variable 'x' no definida` |
| 10 | **Params default `b = 10`** | params_default | `Variable 'b' no definida` |
| 11 | **Genéricos `<T>`** | genericos, 43_tipos_asociados, 44_extension_methods (`rasgo`) | `Variable 'identidad' no definida` |
| 12 | **Destructuring `_`** | destructuring | `Variable '_' no definida` |
| 13 | **Tuplas `(a, b)`** | tuplas | nvc imprime solo `42` (paréntesis no parseado como tupla) |
| 14 | **Arrays anidados `arr[i][j]`** | arrays | `ArrayGet requires array or string` |
| 15 | **TryUnwrap top-level silencioso** | math, tui_test_min, utils | nvc vacío sin error visible |

### Sprint 6.1 — `importar` en parser puro ✅ (verificado)

- Resolver de módulos en `parser.nv` (~857-1094): `_imp_parse`, `_imp_prefijo`, `_imp_es_func`, `_imp_funcs`, `_imp_prefijar`, `_imp_leer`, `_imp_en_stack`, `_imp_resolver_rec`
- Fusión de ASTs de módulos importados + prefijo `modulo_` en nombres de funciones
- Nueva entrada `parser_parsear_con_base(tk, base)`: recibe el directorio base para imports relativos (computado en el main de `generar_v4.ps1`)
- ✅ Verificado: ejemplos con `importar` compilan y ejecutan con el pipeline puro

### Sprint 6.2 — Gramática: ediciones del usuario (21:00-21:02) 🟡 parcial

| Archivo | Cambios |
|---------|---------|
| `parser.nv` (1,259 líneas, 50,110 B) | `sea`/`let` → skip-noop (línea 400) · `const` → VarDecl (427) · **StructInit `T { f: v }`** (181-217) · **acceso `.campo` → Index+Texto** (261-270) · `estructura`/`struct` → skip hasta `}` + nodo `Vacio` (785) · `enum` → skip + nodo `Vacio` (802) · `para (init;cond;paso)` clásico · VarDecl tipado con `_st_tp`/`_st_tp_skip` (405) · resolver `_imp_*` · `ast_a_texto`/`ast_to_text` (1123) |
| `codegen.nv` (625 líneas, 23,049 B) | **StructInit → `__map_nuevo()` + `__map_poner()`** (200-217) · `error(x)` → ResultErr 39 (185-189) · `Intenta` → TryUnwrap 40 (241-246) · `Lista` → OP_ARRAY_NEW (248-259) · Binary 1-31 · Index → OP_ARRAY_GET |
| `lexer.nv` (166 líneas) | kw-map inline en `lexer_tokenizar`; tipos de token: 1=Ident, 2=Numero, 3=String, 4=Oper, 5=Punt, 6=Kw, 99=EOF |
| `generar_v4.ps1` (2,323 B) | **Driver nuevo**: prints `ENTRADA/SALIDA/Source/Tokens/AST/Instrs/OK` · propagación de errores top-level vía `__tipo_de(fin)` (**gap 15 cubierto**) · base-dir computado para imports relativos |
| `compiler_v4.nv` | Regenerado **90,356 B** (21:02) — concatena lexer+parser+codegen+driver nuevo |
| `vm.rs` | +3,675 B (21:00) — soporte runtime del pipeline puro |

**Diseño clave — structs = mapas:** `T { f: v }` → `__map_nuevo()` + `__map_poner()`; `.campo` → `Index("campo")` → OP_ARRAY_GET sobre el mapa. Consistente con `Value::Map` del runtime; base para `estructura` real más adelante.

### Sprint 6.3 — Lexer: normalización CRLF + `sub_from_chars` (1 Ago 2026) 🟡 en curso

Diagnóstico y corrección en el lexer self-hosted para **tokens truncados** (afectaba `retornar`, `items`, `numero`). Notas del usuario en `docs/progress-2026-08-01.md`.

| Ítem | Detalle |
|------|---------|
| Normalización `s2` (lexer.nv:34-35) | `__str_reemplazar(s, "\r\n", "\n")` + quitar `\r` sueltos — normalización consistente antes de tokenizar |
| Helper `sub_from_chars(cs, st, en)` (lexer.nv:216-224) | Reemplaza `__str_subcadena` en el lexer — evita corrupciones byte/char (subcadena indexa por bytes UTF-8, `largo()` por chars; con acentos en comentarios → índices divergían → tokens rotos) |
| Instrumentación temporal | `LEX-LOOP` debug (ln==7, lexer.nv:44) · `token_a_texto` (196) · main de `compiler_v4.nv`: dump primeros 80 tokens + `ast_to_text(ast, 0)` completo |
| ⚠️ **Slowdown** | `sub_from_chars` es **O(n²)** (`out = out + cs[j]` char a char en VM interpretada) vs `__str_subcadena` nativo → `test_stdlib_mini` (importa `coleccion.nv`) **cuelga >10 min**; `test_arr` sigue CORRECTO (archivo chico). Fix recomendado: `__str_subcadena` sobre la lista `chars` con índices de chars (nativo) o `__str_concat_list` |
| ⚠️ `.nvc` stale | `compiler_v4.nvc` (2:25 PM) era más viejo que `lexer.nv` (5:15 PM) — el binario no tenía el fix; **reconstruido 5:33 PM (75,643 B)** con el `compiler_v4.nv` del usuario (5:27 PM) |
| ⚠️ Regen pierde el main | El main de `compiler_v4.nv` (dump tokens + AST) NO está en `generar_v4.ps1` — regenerar desde el script borra la instrumentación del usuario |
| Pendientes del usuario | Validar que `retornar`/`items`/`numero` tokenizan bien → revertir heurísticas/dumps temporales → tests de tokenización CR/LF/LF-only |

### Estado actual ⚠️ (rescate completado 21:52 + 1 Ago)

| Ítem | Estado |
|------|--------|
| `parser.nv` | ✅ **Corregido**: faltaba `}` de cierre del bloque `si (_st_eof(st))` en `_parse_stmt` (línea 397) |
| `lexer.nv` | ✅ **Corregido**: la kw-map reescrita había perdido `en`/`in` → `foreach` roto (`Variable 'en' no definida`); añadidos |
| `codegen.nv` | ✅ **Corregido**: StructInit pasaba literales `"__map_nuevo"`/`"__map_poner"` a param `numero` de `_cg_emit_call` (E041) → roundtrip `__map_obtener` |
| `compiler_v4.nv` / `.nvc` | ✅ `compiler_v4.nv` del usuario **89,108 B** (5:27 PM, con main instrumentado) / `.nvc` reconstruido **75,643 B** (1 Ago 5:33 PM) — `lumen check` limpio |
| mini_fuego (5 ejemplos) | `test_arr` ✅ CORRECTO · `foreach` ✅ CORRECTO · `test_stdlib_mini` 🔥 **cuelga** (>10 min, lexer O(n²)) · `stress_test`/`demo_completo` 🟡 incompatibles (bugs abiertos) |
| Fixpoint v4 | Pendiente de re-verificar tras regenerar |
| `fuego.ps1` | Pendiente (objetivo: subir de 29/115 correctos) |
| Bugs abiertos | `items`/`total` no definida en for-each de funciones importadas (`test_stdlib_mini`, `stress_test`); sospechas: (1) desugar de `para numero item en` con tipo intermedio, (2) `_imp_prefijar` renombrando params. `demo_completo`: `Variable 'T' no definida` — call-site genérico `id<entero>(42)` (Ident `T` compilado como argumento → LOAD T); el fix de retorno `T` en `_parse_decl` no cubre la llamada |

### Sprint 6.4 — Builtins nativos + BUG RAÍZ stress_test (1 Ago, 18:00-18:40) ✅

**MISTERIO ORIG_FULL resuelto**: el `.nvc` de 5:33 PM (75,643 B, contenía `ORIG_FULL`) fue generado desde un `compiler_v4.nv` del usuario que ya no existe. Los fuentes actuales (lexer.nv 5:15 PM) llaman `__str_reemplazar` como builtin → E031 + fallo runtime.

**Builtins nativos añadidos en Rust** (fix del hang de `test_stdlib_mini`):

| Builtin | Alias | Efecto | Archivos |
|---------|-------|--------|----------|
| `__str_reemplazar` | `__str_replace` | `s.replace(pat, rep)` | vm.rs ~345 y ~3088 (dispatch ×2), sema.rs ~2430 (→Texto), **builder.rs ~992 (lista de calls)** |
| `__str_subcadena_chars` | `__str_slice_chars` | lista de chars → String (join, clamps) | Ídem |

> ⚠️ **CRÍTICO — builder.rs**: los builtins de llamada (`Expr::Call` con callee Ident) deben estar en el `matches!` gigante (~691-1001). Si falta, el IR emite `Load` + `CallValue @0` → runtime `Variable '__str_reemplazar' no definida`. El typing en sema.rs (→ Texto) NO es suficiente.
>
> ⚠️ **`en` es keyword**: el parámetro `en` de `sub_from_chars` causaba E011/E020 → renombrado a `fin` (lexer.nv, cambio permanente).

**BUG RAÍZ de stress_test — FIXEADO en parser.nv (~142)**:

El check de "genéricos en llamada `foo<T>(...)`" scaneaba desde el `<` hasta el **primer** `>` sin límites:
```
mientras (i < largo(arr))   ← el `<` (menor-que) dispara el scan
```
...y el scan se comía tokens 91→184 hasta el `>` de un `lista<entero>` posterior. El Ident resultante quedaba en pos 185 (`nums`), el body del `mientras` tragaba el resto del archivo (modulo/Punto/top-level perdidos) y el AST malformado producía JmpIf con targets rotos y `__main__` vacío.

**Fix aplicado** (parser.nv:142-157):
1. El scan **aborta** si encuentra `(` `)` `{` `}` `;` (nunca dentro de `<...>` de tipos).
2. Solo consume los genéricos si **después del `>` viene `(`** (verdadera llamada genérica).

```lumen
numero st_pre = st;
si (_st_ch(st, 4, "<")) {
    numero st_scan = st;
    mientras (!_st_eof(st_scan) && !_st_ch(st_scan, 4, ">")) {
        si (_st_ch(st_scan, 5, "(") || _st_ch(st_scan, 5, ")") ||
            _st_ch(st_scan, 5, "{") || _st_ch(st_scan, 5, "}") ||
            _st_ch(st_scan, 5, ";")) { romper; }
        st_scan = _st_adv(st_scan);
    }
    numero st_despues_gt = _st_adv(st_scan);
    si (_st_ch(st_scan, 4, ">") && _st_ch(st_despues_gt, 5, "(")) {
        st = st_despues_gt;
    } sino {
        st = st_pre;
    }
}
```

**Estado actual (post-fix, 1 Ago ~18:30)**:

| Ejemplo | Estado |
|---------|--------|
| `test_arr` | ✅ CORRECTO |
| `foreach` | ✅ CORRECTO |
| `test_stdlib_mini` | ✅ CORRECTO (`coleccion_contar: 2`) — hang muerto con el builtin |
| `stress_test` | 🟡 compila el archivo COMPLETO ahora; único fallo: `Variable 'Punto' no definida` = gap de `estructura` sin codegen real (siguiente feature) |
| `demo_completo` | 🟡 **bug `T` MUERTO** (fix de genéricos lo cubría); avanza hasta la sección 7 STRUCTS y falla con `Variable 'j' no definida` (campo de struct — mismo gap de estructuras) |

**Estado de los archivos**:

| Archivo | Estado |
|---------|--------|
| `stdlib/compiler/compiler_v4.nv` | ✅ regenerado **86,062 B** desde fuentes (check ✓), instrumentación de debug (TOK/PROG/BLK/WHILE/BIN/PR/CUERPO) revertida |
| `stdlib/compiler/compiler_v4.nvc` | ✅ rebuild **73,120 B** |
| `compiler_v4.nv` (raíz) | ✅ restaurado 86,062 B (el ps1 con CWD incorrecto lo había pisado con 1,918 B) |
| `stdlib/compiler/target.txt` | ✅ restaurado a `stdlib/coleccion.nv`/`stdlib/compile.nvc` |
| `cargo build --release` | ✅ 3× (11-16 s); el `lumen.exe` release actual incluye los builtins |

> ⚠️ **generar_v4.ps1**: rutas relativas — correr SIEMPRE con CWD `stdlib/compiler`; desde la raíz falla silenciosamente y pisa `compiler_v4.nv` raíz con basura.
>
> ⚠️ **mini_fuego.ps1**: correr desde la raíz; los ejemplos van como parámetro (`mini_fuego.ps1 test_arr foreach`); el bucle del script corta si un ejemplo tarda demasiado (correr de a 1-2).

**Siguiente**: ✅ COMPLETADO — `estructura`/`enum` reales vía desugar (6.4: enum/elegir/sea REALES — nodo `EnumInit`, `defecto:` con cadenas de `sino` reconstruidas desde el final por la persistencia de `im::HashMap`), cortocircuito `&&`/`||` real con JmpIf (6.5), **fixpoint v4 CONFIRMADO** (112,368 B byte-IDÉNTICO en self/self2) y **fuego.ps1: 116/116 compilan, 61 CORRECTOS, 0 fallos**. Detalles en AGENTS.md (Fixes 1-2 Agosto 2026).

---

## Fases del Plan (Sprint 6-8)

### 🟢 Sprint 6 — Imports + Gramática completa (compiler_v5 modular)

| Tarea | Estado | Notas |
|-------|--------|-------|
| `importar` en parser puro: resolver módulos, fusionar ASTs, prefijo `modulo_` | ✅ | Verificado: `parser_parsear_con_base` + resolver `_imp_*` |
| Keywords: `const`, `para`, `sea` (VarDecl real), `estructura`/`enum` (skip tolerante + StructInit `T {}` → mapas), `elegir`/`defecto:` reales | ✅ | 6.4: enum/elegir/sea REALES — EnumInit, `defecto:`, cadenas de `sino` (im::HashMap persistente) |
| Closures `\|x\|`, params default, genéricos `<T>`, destructuring `_`, tuplas `(...)` | ✅ | Closures IIFE (`funcion(){}(args)` hoisted → `__lambda_N`), params default inlineados en call-site, genéricos, `como` cast — todos REALES (commits 6d88fca, ee35e2d) |
| Arrays anidados, ArraySet `arr[i] = x` | ✅ | Verificado: `test_arr` y `foreach` CORRECTO (op 28/29/30) |
| TryUnwrap top-level con error visible | ✅ | Cubierto por driver nuevo (`__tipo_de(fin)` en main de generar_v4.ps1) |
| Cortocircuito `&&`/`\|\|` en codegen puro | ✅ | Helper `_cg_and_or` con JmpIf/Jmp reales (fixpoint v4 regresionado por And eager) |
| Traits `rasgo`/`impl`/`este` | ✅ | `impl Trait para Tipo` → métodos mangled `Tipo_Trait_metodo` + resolución `n.metodo()` por tipo de var (commit 9328fec) |
| Fixpoint v4 + `fuego.ps1` | ✅ | Fixpoint: self/self2 byte-IDÉNTICOS (SHA-256 90048DC9…) · fuego: **116/116 compilan, 108 CORRECTOS, 0 fallos** |

### 🟢 Sprint 7 — VM en LÚMEN + optimización

| Tarea | Estado | Notas |
|-------|--------|-------|
| `vm.nv` — ejecutador de .nvc en LÚMEN puro | ✅ | Dispatch 0-46, builtins vía natives boxeados (JSON, tarea, coro, crypto, fs, env, tiempo, tipo_de), bandas boxed, **demo_completo 0 diffs ~0.9s**, **corutinas reales** (reanudar/ceder/ret con intercambio st/sp/pc), `fmain` acepta `__main__`/`main`/`principal`, handlers tiempo/hilo/mutex/calendario, batería **39/40** (solo `stress_fecha` flaky por timing) |
| Optimización VM LÚMEN (~200s → <10s) | ✅ | `a_entero` O(n)→O(1) (demo 120s→0.9s), guards de banda [3e9,9e9) para ints reales grandes, fix `__map_poner` persistente (cadenas `sino` reconstruidas desde el final) |
| **Optimización VM Rust (causa del O(n²) del fixpoint)** | ✅ | **COW con `Arc`** en `Value::Str(Arc<str>)`/`Value::Array(Arc<Vec<Value>>)`: clonar Values grandes = O(1) (antes Load/ArrayGet/`__str_subcadena_chars` clonaban listas enteras → O(n²)). **Fixpoint v4: 861s → 20.1s (43x)**, self_out2 byte-IDENTICAL (112,368 B) |
| Bootstrapping doble (compiler_v4 compila vm.nv; vm.nvc corre en VM LÚMEN) | ⏳ | 0 dependencias de Rust — hito final |
| Fixpoint doble (compilador + VM) | ⏳ | |

### 🟢 Sprint 8 — Dogfooding completo (release v2.4.0)

| Tarea | Estado | Notas |
|-------|--------|-------|
| Compilar stdlib completo con compiler_v5 | ✅ | matematicas, texto, coleccion, fecha, json, csv, red, tui, graficos — stdlib completo compila con el pipeline puro (fuego 116/116 compilan) |
| Ejecutar 115 ejemplos con cadena 100% LÚMEN | ✅ | **fuego.ps1: 116/116 compilan · 108 CORRECTOS · 4 INCOMPATIBLES · 4 TIMEOUT · 0 fallos** (restantes: SDL/negativos por diseño/no-deterministas/GUI/FFI) |
| Benchmarks vs Rust | ✅ | `scripts/benchmark_vs_rust.ps1`: compile x5.4, run x231 (intérprete-en-intérprete; mediana x2-6) |
| Docs + AGENTS v2.4.0 + release | ⏳ | Docs sincronizadas (6 Ago 2026); falta tag/release oficial |

---

## Fases del Plan

### 🟢 Sprint 1-3: Lexer + Parser + Codegen Básicos

| Tarea | Estado | Notas |
|-------|--------|-------|
| `compiler.nv` — Lexer tokeniza `.nv` | ✅ | `__str_ord`, `__str_chr`, 4 tipos de token (Ident, Num, Str, Op) |
| `compiler.nv` — Parser básico | ✅ | VarDecl + Call. `pos+5` skip fijo |
| `compiler.nv` — Codegen básico | ✅ | PUSH, STORE, PRINT, HALT |
| `__str_chr` builtin | ✅ | vm.rs: `call_core_builtin()` + `CallValue` dispatch |
| Fix token values (`a_texto`→`__str_chr`) | ✅ | compiler.nv, lumen_compiler.nv, lumen_mini.nv |
| Fix `pos+6→pos+5` (off-by-one) | ✅ | compiler.nv, lumen_compiler.nv, lumen_mini.nv |

### 🟢 Sprint 4: Self-hosting Rápido (Completado 30 Julio 2026)

| Tarea | Estado | Notas |
|-------|--------|-------|
| Builtin `__compile_nv` (pipeline Rust nativo) | ✅ | Lex→Parse→Sema→IR→Codegen en un solo builtin |
| Builtins string eficientes | ✅ | `__str_subcadena`, `__str_concat_list`, `__str_starts_with`, `__str_to_chars` |
| ArrayGet optimizado (VM) | ✅ | `chars().collect()` → `chars().nth()`, sin alloc extra |
| compiler_v2.nv reescrito | ✅ | Usa `__compile_nv` en vez de imports LÚMEN |
| self_compile.nv | ✅ | Bootstraps compiler_v2_self.nvc en 533ms |
| **Self-compilación funcional** | ✅ | `533ms` — bootstrap práctico |

### ✅ Sprint 5: Self-hosting Total (Completado 31 Julio 2026)

| Tarea | Estado | Notas |
|-------|--------|-------|
| Pipeline puro completo (`lexer.nv`→`parser.nv`→`codegen.nv`→`__codegen_a_nvc`) | ✅ | Sin `__compile_nv` |
| `compiler_v4.nv` autocontenido (concatenación sin imports) | ✅ | 55,308 bytes; `generar_v4.ps1` |
| Bugs de saltos: JmpIf target→nums, If sin JMP, loop_stack break/continue | ✅ | Ver tabla de bugs críticos |
| Soporte genéricos `lista<texto>` en parser puro | ✅ | `_st_tp_skip` |
| Escapes de string en lexer puro (`\n \t \r \" \\`) | ✅ | Constructor de strings con conversión |
| Forward declarations (`funcion X(...);`) ignoradas | ✅ | Nodo `Vacio` |
| `void`/`diccionario` como keywords | ✅ | Mapa `kw` del lexer |
| **Fixpoint: self recompila su propio source con output idéntico** | ✅ | 52,160 → 54,712 bytes, 3 runs consecutivos |
| **Hito: LÚMEN compila LÚMEN sin Rust** | ✅ | `compiler_v4_self.nvc` (203s) |

### ✅ Sprint 1: Pipeline Library (Completado Julio 2026)

| Paso | Estado |
|------|--------|
| Fix sema: E035 string comparación (`<`/`>`/`<=`/`>=`) | ✅ |
| Fix sema: `__map_contiene` retorna `Booleano` | ✅ |
| Fix VM: Lt/Le/Gt/Ge soporta `Value::Str` | ✅ |
| Fix loader: Import con `/` busca extensión `.nv` | ✅ |
| Fix lexer.nv: `_lx_emit` no mutaba `tk` (paso por valor) | ✅ |
| Fix lexer.nv: usaba `tipo`/`valor` vs parser espera `t`/`v` | ✅ |
| Fix lexer.nv: EOF incluido en `cnt` | ✅ |
| Fix parser.nv: `pos > cnt` → `pos >= cnt` | ✅ |
| Fix codegen.nv: field names `left/right`→`izq/der`, `val`→`expr` | ✅ |
| Fix codegen.nv: faltaban handlers If/While/Block/Programa | ✅ |
| Fix codegen.nv: arguemento de Call no generaba código | ✅ |
| Fix codegen.nv: variable `sino` es keyword | ✅ |
| Fix multi-char operators en `lumen_mini.nv` y `lumen_compiler.nv` | ✅ |
| Pipeline `lexer_tokenizar→parser_parsear→codegen_generar` | ✅ |

**Hito:** `lumen check` pasa sin errores en todos los archivos de `stdlib/compiler/`

### ✅ Sprint 2: Codec `.nvc` + Self-compilación (Completado 30 Julio 2026)

| Paso | Estado |
|------|--------|
| Builtin `__codegen_a_nvc` — convierte mapa codegen → bytes `.nvc` | ✅ |
| Builtin `__file_write_binary` — escribe bytes a archivo | ✅ |
| Builtin `__num_a_f64_bytes` — número → 8 bytes f64 LE | ✅ |
| Registro en loader.rs como builtin | ✅ |
| Type-checking en sema.rs (return `Lista<Entero>`) | ✅ |
| Registro en IR builder para emitir `Call` opcode | ✅ |
| Pipeline `lexer_tokenizar→parser_parsear→codegen_generar→__codegen_a_nvc` | ✅ |
| Escribir `test_output.nvc` desde LÚMEN | ✅ |
| **Ejecutar `.nvc` generado por LÚMEN produce output correcto** | ✅ |
| **Hito:** `lumen test_output.nvc` → `42` | ✅ |

**Logro clave:** Por primera vez, LÚMEN genera bytecode `.nvc` válido desde código LÚMEN, que la VM Rust ejecuta correctamente. El pipeline completo de autocompilación está verificado.

### ✅ Sprint 3: Self-compilación completa (Completado 30 Julio 2026)

| Paso | Estado |
|------|--------|
| `compiler_v2.nv` — lee .nv de archivo, pipeline completo | ✅ |
| `resolver_imports` — importa inline AST recursivamente | ✅ |
| Compilar `compiler_v2.nv` con Rust → `.nvc` | ✅ |
| Verificar: `lumen compiler_v2.nvc` compila `ejemplo.nv` → `42` | ✅ |
| `compiler_v2.nvc` resuelve imports de archivos .nv | ✅ |
| **Hito:** LÚMEN compila LÚMEN (sin Rust) | ⏳ pendiente (muy lento, requiere optimización VM) |

**Nota:** La autocompilación del compilador completo (compiler_v2.nv) funciona pero es demasiado lenta para ejecución práctica (~minutos). Cada operación de string en la VM LÚMEN es O(n) char-by-char. Se necesita optimización del runtime para bootstrap real.

---

## Descubrimientos Clave

### 1. Token values rotos
`a_texto(cp)` produce strings como `"65"` en vez de `"A"`. Identificadores como `"entero"` se convertían en `"101110116101114111"`. Fix: `__str_chr(cp)` para convertir code point → carácter.

### 2. Off-by-one en parser
VarDecl consume 5 tokens (`entero x = 42 ;`), pero `pos = pos + 6` saltaba 6 posiciones, saltándose el siguiente statement. Estaba latente porque `a_texto` producía valores incorrectos y nunca entraba al bloque VarDecl.

### 3. ~~Bug conocido: `||` en `si` + `sino`~~ ✅ FIXED
El lexer LÚMEN no manejaba operadores multi-carácter (`||`, `&&`, `==`, `!=`, `<=`, `>=`), emitiéndolos como dos tokens separados. El parser buscaba `"||"` como un solo token, por lo que nunca coincidía.

**Fix (Julio 2026):** Se añadió detección de operadores multi-carácter en `lexer.nv` y `compiler.nv` antes del fallback a caracter simple.

### 4. ~~Bug conocido: No short-circuit `&&`~~ ✅ FIXED
El IR builder ya implementa short-circuit para `&&` y `||` vía JmpIf/Jmp desde Fase 19 (constant folding + DCE).

### 5. ~~Bug conocido: Struct parsing `E052`~~ ✅ FIXED
`parse_while` y `parse_if` ahora establecen `no_struct_init = true` antes de parsear la condición, evitando que `{` sea interpretado como inicio de inicialización de struct.

### 6. ~~Bug conocido: Bucle infinito en `continuar` dentro de bucles anidados~~ ✅ NO REPRODUCIDO
El IR builder usa un stack de `LoopLabels` que maneja correctamente `continuar` en bucles anidados. No se encontró bug en el código Rust. Se añadió test e2e (`test_nested_continue`) que verifica bucles anidados con `continuar`.

---

## Principio de Bootstrap

```
┌─────────────────────────────────────────────────────────────┐
│  ETAPA 1: Bootstrap inicial (una sola vez, luego se descarta)│
│                                                              │
│  stdlib/compiler/compiler.nv                                 │
│         ↓                                                     │
│  Compilador Rust (EXISTENTE) → compiler.nvc                  │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│  ETAPA 2: Self-compilación                                   │
│                                                              │
│  compiler.nvc → VM Rust → compila su propio fuente          │
│                        → compiler_v2.nvc                    │
│                                                              │
│  ¿compiler.nvc == compiler_v2.nvc? → ✅ Confirmación        │
│                                                              │
├─────────────────────────────────────────────────────────────┤
│  ETAPA 3: Independencia                                      │
│                                                              │
│  vm_core.nv → compila con Rust (última vez) → vm.exe        │
│  vm.exe ejecuta compiler.nvc → compila vm_core.nv           │
│                               → vm_v2.exe                   │
│                                                              │
│  A PARTIR DE AQUÍ: 0 dependencias de Rust                    │
└─────────────────────────────────────────────────────────────┘
```

---

## Lo que NO se implementa en LÚMEN

| Componente | Motivo | Alternativa |
|-----------|--------|-------------|
| **FFI** (`libloading`) | Necesita OS-level linking | Plugin `.dll` externo |
| **Crypto** (BCrypt) | API Windows específica | Plugin `.dll` |
| **SDL2 / GUI** | Depende de DLLs externas | Plugin `.dll` |
| **Threads** | Necesita `std::thread` | Plugin `.dll` |
| **AOT (Cranelift)** | Demasiado complejo para LÚMEN | Se mantiene en Rust como backend externo |

---

> **"If you want to build a ship, don't drum up people to collect wood. Make them long for the sea."**
> — Antoine de Saint-Exupéry
