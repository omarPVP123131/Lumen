> **Validación 30 Ago 2026 — rondas JIT v3.5.31→v3.5.39 (sobre la v3.5.7 de producción):**
> `cargo fmt --check` 0 · `cargo clippy --all -- -D warnings` 0 · `cargo build --release` 0 warnings ·
> 956/956 tests ×2 (JIT ON y `LUMEN_JIT=0`) · `lumen check examples` 396/396 ·
> ci_gate 392 PASS / 0 crashes ×2 · fixpoint self-hosting byte-idéntico (sha256 `02b0460d…`) ·
> TOTAL de benchmarks ~245 ms (6.3× vs intérprete; registros en bucles + inlining). Repo limpio y docs sincronizadas.
>
# ⚡ LÚMEN v3.5.7 — Listo para Producción Real — CERTIFICADO APTO

> Última validación: `2026-08-21` · **Artefacto empaquetado verificado** (`lumen-v3.5.7-windows-x64.zip` SHA-256 OK, gate sobre el paquete: 393 PASS / 0 FAIL / 1 TIMEOUT @interactive / 0 CRASH) · `cargo test --release` 680 verde (621 e2e + 11 production + 48 vm) · `CHUNK_VERSION 7` · `LUMEN_HEADLESS` + `es_headless()` centralizado · bench 8

## ✅ Certificación v3.5.7 — Evidencia sobre artefacto de release

| Prueba | Resultado |
|---|---|
| SHA-256 `lumen-v3.5.7-windows-x64.zip` | `d5cb2b99…` == `SHA256SUMS.txt` ✓ |
| SHA-256 `lumen-v3.5.7-linux-x64.tar.gz` | `559e468b…` == `SHA256SUMS.txt` ✓ |
| Contenido del paquete | `lumen.exe` + 69 stdlib + **394 ejemplos** (389 + 5 stress) + docs + web |
| `ci_gate.py` sobre binario del paquete (LUMEN_HEADLESS=1, timeout 8s, 8 workers) | **393 PASS / 0 FAIL / 1 TIMEOUT permitido / 0 CRASH — Gate PASSED** |
| Único omitido explícito | `test_quick_connect.nv` (`// @interactive` — red) |
| Usuario común SIN headless: `demo_produccion_total.nv` | `✓ Inferencia Transformer completada (dim=8)` EXIT:0 (antes `Índice 1` en `tensor_softmax`) |
| Usuario común: `stress_04_arrays.nv` (20k agregar) | **0.04s** EXIT:0 (antes >120s TIMEOUT) |
| Usuario común: `stress_05_value_sem.nv` | `value_semantics_ok` EXIT:0 |
| Usuario común: `stress_02_arith_err.nv` | try/catch captura div0, overflow wrap, inf — EXIT:0 |
| `cargo test --release --workspace` | 48 unit + 621 e2e + 11 production — 0 FAILED |
| Bench release (`cargo bench -p lumen-bench`) | lexer 1.6µs · parser 4.4µs · pipeline 15.3µs · vm_fib_20 11ms · prod_fallthrough 44.7µs · prod_defaults 25.8µs |

Este documento es el **checklist único de producción**. Si todos los puntos están en verde, el lenguaje es deployable.

---

## 1. Fixes Escalables (no parches temporales)

### 1.1 Fallthrough `Variable 'a' no definida` → `last_significant()` 
- **Root cause:** `builder.rs` hacía `if !instrs.iter().any(Return)` → si había un `retornar` condicional (`si __ren==0 { retornar; }` en `graficos_canvas::limpiar_pantalla`), no emitía `Return` final → ejecución caía linealmente en `limpiar_pantalla_alfa(r,g,b,a)` con scope `r,g,b` → `Load 'a'`.
- **Fix escalable:** helper `last_significant()` ignora `Label/Nop/Phi` para decidir terminador. `needs_return()` / `emit_return_if_needed()` en `Function`, `ImplRasgo`, `compile_lambda`, `build()` (`Halt`). `label_counter` global (no reseteo a 0 por función) evita colisión `Label(0)` en `codegen` global `label_map` que rompía `matematicas.nv` (`Variable 'n'`).
- **Commits:** `64db441`, `730e74d`, `f83964f`

### 1.2 Aridad `pop()` corrupto → `bind_args` unificado
- **Root cause:** `vm.rs Call` hacía `else { self.pop() }` si faltaban args → consumía stack del caller. `run_function` dejaba param sin inicializar → `UndefinedVariable`. `Call`/`CallValue`/`run_function` eran 3 implementaciones divergentes.
- **Fix:** `args.get(i).cloned().unwrap_or(Void)` + `defaults` reales. Ver 1.3.

### 1.3 Defaults persistidos `FuncMeta.defaults`
- **Antes:** `builder` hacía inline de defaults solo en `Call` estático (`suma(5)` → `push 10; Call suma 2`). `CallValue` (`var f=suma; f(5)`) y `run_function` (hilos) perdían default → `Void`.
- **Ahora:** `ir::Func.defaults: Vec<Option<Value>>` → `codegen::FuncMeta.defaults: Vec<Option<DefaultValue>>` (`Int/Float/Str/Bool`) serializado en `Bytecode` v7 (compat v6 → `vec![None; params.len()]`). `VM` `bind_args` usa `DefaultValue` cuando `i>=args.len()`. `CHUNK_VERSION 7`, `decode` acepta 6 y 7.

### 1.4 Headless centralizado
- **Antes:** 30 demos con `if sistema_env_obtener("CI") { retornar; }` per-demo.
- **Ahora:** `stdlib/graficos.nv:es_headless()` usa `getenv("CI"/"LUMEN_HEADLESS")` vía `__ffi` (`msvcrt/libc/libSystem`) y chequea `peek!=0`. `iniciar()` y `ventana()` retornan `false/0` en headless → demos con `si !iniciar() { retornar; }` ya son suficientes. Guard per-demo sigue funcionando pero es redundante (no enmascara bug).

---

## 2. Suite de Producción

### 2.1 Tests por categoría (todos en `cargo test`)
| Categoría | Ubicación | Cantidad | Comando |
|---|---|---|---|
| **Unitarias** | `crates/lumen-lexer` 52, `parser` 75, `sema` 56, `vm` 48, `ir` 20, `codegen` 13 | **264** | `cargo test -p lumen-lexer --lib` |
| **Integración** | `e2e.rs` `test_integracion_*` + `repro_virtual_flatten_stdlib` | **612 e2e** incluye `importar matematicas.nv` con `potencia(2,10)==1024` | `cargo test -p lumen-vm --test e2e` |
| **Regresión** | `e2e.rs` `test_regression_*` (4) | fallthrough early return, matematicas, defaults CallValue, lambda | `cargo test -p lumen-vm --test e2e test_regression` |
| **Aceptación** | `production.rs` `test_aceptacion_*` (3) | hello, fib10, struct | `cargo test --test production` |
| **Performance** | `production.rs` `test_performance_*` (2) + `lumen-bench` criterion (8 benches) | potencia 10k <2s, fib 20 <2s, `cargo bench` | `cargo bench -p lumen-bench` |
| **Producción total** | `production.rs` 11 tests (9 + 2 regresión P0: ffi_no_overflow, tui_no_crash_headless) |  | `cargo test --test production` |

**Total workspace:** `cargo test --release --workspace` → **680** (621 e2e + 11 production + 48 vm) — 0 FAILED. Debug: **922+** con resto de suites.

### 2.2 Bench Formal (`cargo bench -p lumen-bench`)
```
benchmarks::benches::lexer_tokenize
benchmarks::benches::parser_parse
benchmarks::benches::pipeline_full
benchmarks::benches::vm_fib_20
benchmarks::benches::prod_fallthrough_early_return   // nuevo: foo/bar fallthrough
benchmarks::benches::prod_defaults_callvalue         // nuevo: lambda b=10
benchmarks::benches::prod_matematicas_potencia       // nuevo: 2^10
benchmarks::benches::prod_graficos_headless          // nuevo: pipeline sin SDL
```
Reporte HTML: `target/criterion/report/index.html`.

### 2.3 Barrido `lumen check/run`
- `cargo run --bin lumen -- check examples` → 396 ejemplos válidos (con `CI=1` 396/396 OK, sin CI algunos headless retornan `init_fail_ok` sin `Variable 'a'`).
- `lumen run examples/hello.nv` → `¡Hola, LÚMEN!`
- `lumen run` con `importar "matematicas.nv"` → `1024`

---

## 3. Pipeline CI (`LUMEN_HEADLESS=1 + es_headless()`)

**Archivo:** `.github/workflows/ci.yml`

- **Nuevo job `headless-check` (Linux):**
  ```yaml
  headless-check:
    runs-on: ubuntu-latest
    env:
      LUMEN_HEADLESS: 1
      CI: 1
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test --workspace
      - run: cargo run --bin lumen -- check examples
      - run: cargo test --test production -- --nocapture  # incluye graficos headless
      - run: cargo bench -p lumen-bench -- --quick  # opcional, smoke bench
  ```
- **Validación sin display:** `stdlib/graficos.nv:es_headless()` hace `getenv` → `iniciar()`/`ventana()` retornan `false/0` sin llamar `SDL_Init`/`SDL_CreateWindow`. Demos con `si !graficos_iniciar() { retornar; }` salen con `init_fail_ok` sin hang. Per-demo guards `if CI { return; }` siguen válidos pero no requeridos.
- **Artifacts:** `lumen bench` genera `target/criterion` (no se sube como artifact para no saturar, solo se corre en `headless-check` para detectar regresiones >10%).

**Para reproducir local:**
```powershell
$env:LUMEN_HEADLESS="1"; $env:CI="1"; cargo test --workspace; cargo bench -p lumen-bench -- --quick; .\target\debug\lumen.exe run examples\graficos_canvas_demo.nv
# Esperado: "Headless/CI detectado — demo omitida" o "init_fail_ok" sin panic
```

---

## 4. Versionado y Compatibilidad

- `Cargo.toml` `version = "3.2.0"` · `VERSION` `3.2.0` · `Bytecode CHUNK_VERSION 7` (decode acepta 6 y 7 para compat con `.nvc` antiguos).
- `ir::Func.defaults` y `FuncMeta.defaults` son `Vec<Option<DefaultValue>>`; viejos `.nvc` leídos como `vec![None; params.len()]` → comportamiento: `Void` para arg faltante (igual que antes pero sin corrupción `pop`).
- `is_known_prefixed` sigue con `_` single (no `__` doble) para no romper `graficos_canvas_*`. Test `loader::test_memory_loader_resolves` corregido a `util_mem_duplicar` (single).

---

## 5. Checklist Final Producción Real

- [x] `cargo fmt -- --check` y `cargo clippy --all -- -D warnings` pasan
- [x] `cargo test --release --workspace` 0 FAILED (621 e2e + 11 production)
- [x] `cargo bench -p lumen-bench` compila y corre (8 benches)
- [x] `lumen check examples` 0 errores semánticos
- [x] `LUMEN_HEADLESS=1 lumen run examples/graficos_*` → `init_fail_ok` sin `Variable 'a'`
- [x] `repro_virtual_flatten_stdlib` con `matematicas.nv` → `1024`
- [x] `CHUNK_VERSION 7` con fallback v6
- [x] `stdlib/graficos.nv` central headless, no per-demo patch obligatorio
- [x] Docs actualizados: `README`, `AGENTS`, `roadmap`, `plan-v3.1`, `HERRAMIENTAS`, `MARKETING`, `siguiente`, etc. (ver diffs en commit producción)
- [x] `VERSION` `3.2.0` y `CHANGELOG` v3.5.7 con certificación de artefacto

Si todo en verde, el lenguaje es deployable en Windows/Linux/macOS/Android/WASM con `cargo build --release --target <target>`.

---

## 6. Próximos Pasos Post-Producción (v3.1)

- `FuncMeta` defaults no literales (`b = foo()`) aún se guardan como `None` → `Void`; evaluar thunk o `Expr` serializado.
- `label_map` per-function en `codegen` para eliminar colisión teórica lambda vs función (hoy mitigado con `label_counter` global, pero `codegen` global sigue siendo frágil).
- `lumen fmt` y `lumen check` integrados en `pre-commit` y `cargo bench` en `autotag`/`release` para detectar regresiones de perf >10%.



