> **Estado 30 Ago 2026 — v3.5.7 + rondas JIT v3.5.31→v3.5.37 ✅:**
> JIT Tier-1/Tier-2/Tier-R activo por defecto (TOTAL 267.1 ms, 5.8×), VM con
> pools de scope e invalidación selectiva de caché, pre-commit verde
> (fmt/clippy -D warnings/tests/check 396-396), fixpoint byte-idéntico.
> Siguientes candidatos medibles: aritmética de pila nativa para Sum+1 (loops
> con llamadas a builtins), mejoras de `a_texto`/`imprimir` en strings, y
> ampliar el Tier-R (Sum, profundidad).
>
# Siguientes Pasos — Roadmap LÚMEN v3.5.7 Producción Real → v3.1

**Estado actual (21 Ago 2026) — v3.5.7 Producción Real ✅:** Fases 0-185 completas + Self-hosting puro (Sprint 5-6 fixpoint) + **Sprint 7: VM en LÚMEN COMPLETADO** + **Sprint 8: dogfooding 112 CORRECTOS** + **Bootstrapping doble SHA-256 `3DA624D6...`** + **v2.4.6 (14 Ago)** + **v3.5.7 (20 Ago)** 167 bugs + **v3.5.7 Producción Real (21 Ago 2026):** fixes escalables `last_significant()` + `label_counter` global, `CHUNK_VERSION 7` con `FuncMeta.defaults` + `bind_args` unificado, `stdlib/graficos.nv:es_headless()` centralizado, **bench 8** (`cargo bench -p lumen-bench` — 4 prod: fallthrough, defaults, matematicas, headless), **636 e2e + 9 production = 695 vm tests (956 workspace)**, CI `headless-check` con `LUMEN_HEADLESS=1 CI=1`, `lumen check` 396/396. Ver [docs/produccion.md](produccion.md). Pendientes: **AI/ML (Fases 186-200)** + **Playground L2/L3** + **v3.1** (ver `docs/plan-v3.1.md` y `docs/produccion.md` §6).

---

## ✅ Completado — Producción Real v3.5.7 (21 Ago 2026)

| Área | Estado | Detalle |
|------|--------|---------|
| **Fixes escalables builder/VM** | ✅ | `last_significant()` + `label_counter` global (fallthrough `Variable 'a'/'n'`), `CHUNK_VERSION 7` + `FuncMeta.defaults` + `bind_args` unificado |
| **Headless centralizado** | ✅ | `stdlib/graficos.nv:es_headless()` (`LUMEN_HEADLESS`/`CI` vía `__ffi`) + CI `headless-check` (`LUMEN_HEADLESS=1 CI=1`) |
| **Suite + bench formal** | ✅ | 636 e2e (4 regresión) + 9 production = 695 vm tests, 956 workspace; bench 8 (`cargo bench -p lumen-bench`) |
| **Docs producción** | ✅ | `docs/produccion.md` checklist único, `VERSION` 3.1.4, `CHANGELOG` v3.5.7 producción |

Ver [docs/produccion.md](produccion.md) para reproducir local y CI.

## 🟢 Prioridad Alta — Fácil + Alto Impacto (v3.1)

## 🟡 Prioridad Media — Moderado + Alto Impacto (v3.1)

| Área | Complejidad | Impacto | Razón |
|------|-------------|---------|-------|
| **v3.1 defaults no literales + label_map per-function** | 🟢 Baja | 🔥 Alto | `FuncMeta` defaults `None` → thunk/Expr serializado, `label_map` per-function (ver `docs/produccion.md` §6) |
| **Fixes y pulido** | 🟢 Baja | 🔥 Alto | Estabilidad general, edge cases + `lumen fmt`/`check` en pre-commit |
| **Bootstrapping doble + release v2.4.6/v3.5.7** | 🟢 Baja | 🟡 Alto | ✅ COMPLETADO: vm.nv + fixpoint + tags v2.4.6/v3.5.7 producción |

## 🔵 Prioridad Baja — Muy Complejo + Nicho

| Área | Complejidad | Impacto | Razón |
|------|-------------|---------|-------|
| **Cloud (Fases 201-220)** | 🔴 Muy Alta | 🟡 Medio | AWS, GCP, Azure, K8s. Depende de ecosistema maduro. |
| **Extensiones VS Code/JetBrains** | 🟡 Media | 🟡 Medio | Atraería usuarios pero requiere mantenimiento constante. |

---

## 🎯 Recomendación

### ✅ Completado: **Self-hosting Rápido (30 Julio 2026)**
- `__compile_nv` builtin ejecuta pipeline Rust nativo (lex→parse→sema→ir→codegen)
- `compiler_v2.nv` reescrito: usa `__compile_nv` en vez del pipeline LÚMEN-in-LÚMEN lento
- Builtins string eficientes: `__str_subcadena`, `__str_concat_list`, `__str_starts_with`, `__str_to_chars`
- ArrayGet optimizado: `chars().collect()` → `chars().nth()`
- **Autocompilación: 533ms** (de >5min a 533ms)

### ✅ Completado: **Self-hosting Total (30 Julio 2026)**
- `Value::Map(Vec<...>)` → `HashMap<Value, Value>` con `Hash`/`Eq` manual
- `__map_get`/`__map_set`/`__map_contains`: O(n) → O(1)
- Sets union/inter/diff: O(n²) → O(n)
- `codegen_to_nvc`: lookups directos O(1)
- El parser LÚMEN-in-LÚMEN ahora tiene mapas O(1)

### ✅ Completado: **Self-hosting Puro (31 Julio 2026)**
- `compiler_v4.nv` autocontenido (55,308 bytes): lexer+parser+codegen+main sin imports
- Pipeline puro: lexer.nv → parser.nv → codegen.nv → `__codegen_a_nvc` → .nvc
- **Fixpoint confirmado**: `compiler_v4_self.nvc` (54,712 bytes, 49 funciones) se recompila a sí mismo con resultado idéntico en 3 runs (193s/203s/197s)
- 11 bugs críticos arreglados (saltos, TryUnwrap, print multi-arg, break/continue, escapes, keywords, forward decls)
- LÚMEN ya no depende de `__compile_nv` para compilarse

### 🟢 Completado: **Sprint 6-8 — LÚMEN autosuficiente (dogfooding total, 31 Jul - 8 Ago 2026)**

- **6.1 `importar` en el pipeline puro** ✅ — resolver `_imp_*` + `parser_parsear_con_base`, self-import detectado (canonicalize en loader.rs)
- **6.2-6.4 Gramática real** ✅ — `sea`/`const` (VarDecl), StructInit `T {}` → mapas, `.campo` → Index+Texto, `elegir`/`defecto:`/`caso`, enum `Nombre::Miembro(args)`, `algun`/`ninguno`/`exito`/`error`, cortocircuito `&&`/`||`, closures IIFE, params default, traits `rasgo`/`impl`/`este`, cast `como`
- **6.3 Runtime** ✅ — arrays anidados, ArraySet `arr[i] = x`, TryUnwrap top-level con `__tipo_de(fin)`, floats con `.`, lexer CRLF/UTF-8 seguro (`__str_subcadena_chars` nativo)
- **FIXPOINT v4 CONFIRMADO** ✅ — self/self2 byte-IDENTICAL (SHA-256 90048DC9… → 3DA624D6…), 5s
- **Sprint 7 — VM en LÚMEN** ✅ — `vm.nv` (dispatch 0-46, corutinas reales, handlers JSON/tarea/coro/crypto/fs/env/tiempo/hilo/mutex/calendario), demo 120s → 0.9s, batería 39/40
- **Sprint 8 — Dogfooding** ✅ — stdlib completo compila; **fuego.ps1: 396/396 compilan · 112 CORRECTOS · 1 INCOMPATIBLE · 4 TIMEOUT · 0 fallos**; benchmarks: compile x5.4, run x231 (mediana x2-6)
- **Paridad VM LÚMEN-Rust (8 Ago 2026)** ✅ — `__map_obtener` con boxeo por tipo real + lookup dual (keys boxed guest vs strings JSON host); handlers `__existe_archivo`/`__leer_archivo`/`__escribir_archivo` → **test_json_avanzado, test_sistema_directo, test_sistema_avanzado byte-IDÉNTICOS** · batería 39/40 (solo `stress_fecha` flaky timing) · `vm_self.nvc` regenerado (111,318 B) · **Stream/Async/Par/Actor/Generator 100% delegados** · `sprint1_concurrencia` 100% paridad
- **Bootstrapping doble CONFIRMADO** ✅ — SHA-256 `3DA624D6AD32E359D3714F7CD936563CE1A60ED633590CB580D695F24C7E282A` (150,684 B byte-idénticos en self/self2)
- **Fases 61-63 reales en el pipeline Rust (12 Ago)** ✅ — OR patterns (`|` en arms ya no es `BitOr`), **rangos `..`/`..=` end-to-end** (lexer tokens + `Expr::Range` + desugar IR + sema E044) como patrones de `elegir` y como expresión-lista; fix del self-loop de JmpIf en el match con `NotEqual; JmpIf(body)`; fix `tcp_listener` cfg para builds sin features; **3 ejemplos nuevos** (`examples/fase61_or_patterns.nv`, `fase63_range_patterns.nv`, `fase64_string_patterns.nv`) byte-idénticos en VM y backend C; lexer 27 / parser 45 / sema 56 tests · cargo test 0 FAILED
- **Rangos `..`/`..=` en el SELF-HOSTED (13 Ago)** ✅ — lexer.nv branch rango, parser.nv nodo `Range`, codegen.nv desugar lista + intercepto `==` con rango (short-circuit `_cg_and_or`), `OP_ARRAY_PUSH=32` y `32 => 32` en el encoder nativo; **FIXPOINT NUEVO SHA-256 `5D153BC631812524B3DD078380B6E9285A68E284FCB6E23D3DC97ADFA12076C5`** (139,732 B) — reemplaza a `3DA624D6…`
- **If-let REAL + payloads en elegir (13 Ago)** ✅ — `bind_pattern_vars` en sema (IfLet/GuardLet/Match arms), opcodes 52/53 `MatchType`/`MatchPayload` (IR/bytecode/VM/backend C/encoder nvc), `caso algun(x)` bindea x; **8 ejemplos de fase nuevos** (fase62 ×2, fase66 ×2, fase68 ×2, fase70 ×2) — todos OK en VM y backend C; **limpieza de warnings** (sema/aot/wasm); cargo test 0 FAILED

### Después: **AI/ML (Fases 186-200) + Docs**
- AI/ML + Data Science — feature más solicitada para un lenguaje moderno
- Tensores, redes neuronales, DataFrames desde stdlib
- Binding a librerías C (llama.cpp, ONNX)
- Docs + Comunidad — sitio web, tutorial interactivo, playground web

### Después: **Producción & Cloud (Fases 201-220)**
- AWS Lambda runtime, WASM plugins, extensión VS Code, empaquetado nativo

