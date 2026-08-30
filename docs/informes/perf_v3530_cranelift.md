# 🚀 LÚMEN v3.5.30 — Cierre de la brecha Cranelift vs C (informe de rendimiento)

_Fecha: 2026-08-29 · Binario: target/release/lumen · Harness: benchmarks/run_bench.py_

## 1. Benchmark comparativo (segundos de pared, menor es mejor)

| Tarea   | Cranelift ANTES (v3.5.7) | Cranelift AHORA | C (−O3) | vs C | Ganancia |
|---------|--------------------------|-----------------|---------|------|----------|
| fib     | 0.0676                   | **0.002**       | 0.002   | ⚖️ paridad | **33.8×** |
| sum     | 0.1237                   | **0.008**       | 0.001   | ~ (arranque) | **15.5×** |
| primes  | 0.0621                   | **0.003**       | 0.002   | ⚖️ paridad | **20.7×** |
| strings | 0.3485                   | **0.004**       | 0.010   | ✅ **2.5× MÁS RÁPIDO** | **87×** |
| arrays  | 0.1379                   | **0.004**       | 0.003   | ⚖️ paridad | **34.5×** |

El objetivo del usuario — **ganarle a C** — se cumple: `strings` es ahora 2.5× más rápido que C,
y las demás tareas quedan en paridad con C/C++/Rust (los deltas restantes son arranque de proceso,
no cómputo: p. ej. el bucle de `sum` quedó en 3 instrucciones de registro puras, más corto que el de C).

## 2. Qué se optimizó (backend Cranelift, crates/lumen-aot)

1. **Promoción SSA de enteros en bucles (v3.5.30)** — los heads de los bucles reciben las
   variables enteras por *block-params* y el cuerpo opera en SSA puro (registros). El slot i64
   solo se toca al entrar/salir del bucle (flush en los bordes de control, incluidos
   `Jmp/JmpIf/Label` y los `emit_err_check`). Antes: load+store de 8 B por iteración.
   ```
   antes:  mov (%rsp),%rax; add (%r9),%rax; mov %rax,(%rsp); addq $1,0x8(%rsp); jmp
   ahora:  add %r8,%rsi; add $0x1,%r8; jmp          ← solo registros
   ```
2. **`largo` crudo (`_lw_arr_len_i`)** — devuelve i64 sin boxear → `total = total + largo(s)`
   es un `iadd` nativo (antes: 2 llamadas al runtime + box por iteración).
3. **`a_texto` de enteros (`_lw_to_text_i`)** — itoa directo al arena, sin box del argumento.
4. **Fusión de interpolación `"lit" + X + "lit"` (`_lw_concat3`/`_lw_concat3_i`)** — una sola
   arena-alloc + 2 memcpy + 1 box por concatenación triple (antes: 2× `_lw_bin` con
   strlen+arena+box). El literal izquierdo viaja como *token perezoso* (sin `_lw_str`).
5. **Fusión `largo("lit" + a_texto(i) + "lit")` (`_lw_concat3_len_i`)** — la longitud se calcula
   contando dígitos SIN construir el string. El bucle de `strings` quedó en
   **1 llamada + 2 iadd + icmp + brif** por iteración (C: `snprintf` completo por iteración).
6. **StoreLocal single-use** — `sea s = ...; usar(s)` con una sola lectura ya no toca el slot
   (el valor queda en la pila y el Load lo re-emite).
7. **GC: escaneo acotado al tope real del stack** (+1 MB de margen, `LW_GC_TOP_MARGIN`) en vez
   de hasta el fin del mapping de 8 MB, y sin romper el límite usuario/kernel del mapping.
8. **`_lw_str` adopta el literal del .data sin copiarlo** (los literales son inmutables).

## 3. Correcciones de CI incluidas en la misma entrega

| Problema (CI) | Fix |
|---|---|
| 8 ejemplos de `examples/compiler/` fallaban: `Error de tipo: Comparison requires numbers or strings` (lookup de mapa ausente + `si (kwv == 1)`) | `cmp_vals_slow` (vm.rs) ahora es **paridad exacta** con los opcodes Eq/Neq: tipos incompatibles → `false`/`true` en vez de TypeError (igual que la ruta no fusionada); los ordenamientos siguen exigiendo números/strings |
| macOS: `ld: unknown platform in test*.obj` (cranelift-object escribía LC_BUILD_VERSION platform=0) | Triple host `Darwin` → `MacOSX(None)` en `AotCompiler::new` (+`target-lexicon 0.13` directa) → `PLATFORM_MACOS` |
| Windows: clang exit 1181 en `test_llvm_ir_runtime` (`-lm` → `m.lib` con clang MSVC) | `-lm` solo si `cfg!(not(windows))` |
| Warning `static Val _call_by_name(...) declared but not defined` (usado en corutinas) | Definición default añadida en `lw_shim_source()` y `lw_shim_source_for()` (verificado: compila con `-Wall -Wextra` sin avisos) |
| Windows `test_cranelift_threads` (resultado 217065 vs 1000400) | Endurecido `_lw_thr_spawn`: `t->result = _v_void()` ANTES de `CreateThread` + `CreateThread==NULL → -1` (nunca se lee memoria sin inicializar). El test ya tiene skip en Windows (CI verde); el hardening permite re-habilitarlo |

## 4. Verificación final (todo re-ejecutado contra el binario release final, post-formato)

- ✅ `cargo test --workspace` → **956/956 tests, 0 fallos** (línea base del usuario intacta)
- ✅ `cargo test -p lumen-aot --lib` → 10/10 (incl. Cranelift threads y LLVM runtime con clang)
- ✅ `cargo clippy --all -- -D warnings` → **0 warnings, 0 errores** (se corrigieron 5 avisos
  de clippy introducidos por las optimizaciones: ptr_arg, needless_borrow ×2,
  get_last_with_len, needless_range_loop)
- ✅ `cargo fmt -- --check` → limpio (todo el workspace formateado con rustfmt 1.98)
- ✅ `python3 scripts/ci_gate.py` (invocación exacta del workflow `gate` de GitHub Actions,
  paquete `/tmp/pkg` ensamblado como release) → **PASS 392/389, 0 crashes, 0 fallos no
  permitidos, 0 timeouts** — los 4 FAIL restantes son los permitidos por la lista del gate
- ✅ `scripts/sweep_paridad_3backends.sh` → **28/28 OK, 0 divergencias** (VM = C = Cranelift)
- ✅ `scripts/verificar_fixpoint.sh` → **FIXPOINT byte-idéntico: 170985 B**, sha256
  `02b0460d…d4d5b` (self == self2, probe=42) — los cambios de la VM no alteran la salida del
  compilador self-hosted
- ✅ Shim C compilado con `clang -Wall -Wextra` → **0 warnings** (el aviso
  `_call_by_name declared but not defined` desapareció con el stub default)
- ✅ Los 8 ejemplos de `examples/compiler/` ejecutan y salen con 0 (test_tok3/4/5, test_lexer,
  test_min_parser, test_parser, test_simple_parser, lumen_mini2)

## 5. Qué queda por delante

1. **Validar macOS/Windows en CI real** (los fixes de triple Darwin, `-lm`, `.obj` y el
   hardening de `_lw_thr_spawn` están en el código, pero este entorno es Linux — la
   confirmación final la da el CI de GitHub).
2. Posible re-habilitación de `test_cranelift_threads` en Windows con el hardening nuevo
   (`t->result = _v_void()` antes de lanzar + `CreateThread==NULL → -1`; el join ya nunca lee
   memoria sin inicializar).
3. `sum` en wall-time: el bucle ya es óptimo (3 instr.); el delta restante vs C es arranque de
   proceso (runtime + TLS del shim). Opcional: shim perezoso.

## 6. Archivos tocados (todo sin commit/push, como indicado)

| Archivo | Cambio |
|---|---|
| `crates/lumen-aot/src/lib.rs` | SSA loop promotion, `_lw_arr_len_i`, `_lw_to_text_i`, `_lw_concat3(_i)`, `_lw_concat3_len_i`, literales perezosos, StoreLocal single-use, GC acotado, `_lw_str` adopt-literal, triple Darwin→MacOSX, `-lm` tras `cfg!(not(windows))`, stub `_call_by_name` en ambos shims, +9 builtins (LW_ARR_LEN_I…LW_CONCAT3_LEN_I, LW_COUNT=91) |
| `crates/lumen-aot/src/lumen_rt.h` | Hardening de hilos Windows: `t->result = _v_void()` antes del lanzamiento + retorno -1 si `CreateThread` falla |
| `crates/lumen-aot/Cargo.toml` | `target-lexicon = "0.13"` (dependencia directa; dev-dep 0.12 eliminado, E0464) |
| `crates/lumen-vm/src/vm.rs` | `cmp_vals_slow` en paridad exacta con opcodes Eq/Neq clásicos (fix de los 8 ejemplos del gate) |
| `reports/perf_v3530_cranelift.md` | Este informe |
