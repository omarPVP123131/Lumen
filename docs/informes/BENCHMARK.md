# LÚMEN v3.0.0 — Reporte de Benchmarks

**Fecha:** 21 Agosto 2026  
**Versión:** v3.0.0  
**Hardware:** Windows x86_64 (16 cores, GCC/MinGW + Clang)

---

## 1. Compilación

| Target | Tiempo | Notas |
|--------|--------|-------|
| `cargo build -p lumen-cli` (dev) | 7–14s | Incremental |
| `cargo build --release -p lumen-cli` | 62s | Optimizado |
| `cargo test --workspace` (debug) | 2.1s (e2e 172) + 0.5s unit | 414 tests, 0 FAILED |
| `cargo test --workspace --release` | 2.1s | Idem |

## 2. Verificación de Ejemplos

- `lumen check examples`: **396/396 VÁLIDO, 0 errores** (21 Ago 2026)
- `lumen run demo_completo.nv` (VM, debug): **~2.1s** (33 secciones + FIN)
- `lumen run demo_completo.nv` (VM, release esperado): **~0.9s** (medición previa Sprint 8)
- Headless/CI demos (charts, graficos_avanzado, tui_temas): **omitidas con `CI=1` o `LUMEN_HEADLESS=1`** — requieren display SDL2 / consola interactiva; marcadas como `INCOMPATIBLE por diseño` en `fuego.ps1`

## 3. AOT Dual Backend (bench_fib.nv: fib(26) + loop 100k)

Mediciones Sprint 8 (12 Ago, `stdlib/compiler/bench_fib.nv`, 8 runs calientes):

| Backend | Tiempo | Speedup vs VM |
|---------|--------|---------------|
| **VM (interprete)** | 856 ms | 1× |
| **C (GCC -O3)** | 22 ms | **38×** |
| **Cranelift (SSA)** | 5.6 ms | **152×** |

*Batería dual `aot_bateria_dual.ps1` (38 ejemplos, watchdog 25s):*
- C: **38 OK / 0 DIFF** (paridad total)
- Rust/Cranelift: **12 OK / 26 DIFF** (límite diseño: sin strings/colecciones)
- FAIL 0, SKIP 1 (math), HANG 0

*Build overhead AOT:*
- `gcc_link` ~183 ms
- `codegen_cranelift` 2 ms
- `compilar_a_ir` 0.8 ms

## 4. Self-Hosting

- `compiler_v4.nv` (132–150 KB) → `compiler_v4.nvc` → `self_out.nvc` (112–165 KB) → `self_out2.nvc` **byte-idéntico** (~5s)
- Fixpoint SHA-256 `3DA624D6...` (8 Ago) y `DF7676DE...` (14 Ago, OR patterns)
- Batería self-hosted `test_vm.ps1`: **39/40 OK** (solo `stress_fecha` flaky 0ms vs 17ms)

## 5. CI/CD (GitHub Actions)

| Job | Runner | Estado v3.0.0 |
|-----|--------|---------------|
| fmt / clippy -D warnings | ubuntu-latest | ✅ 0 warnings |
| linux-test / windows-test | ubuntu/windows | ✅ 414 tests |
| wasm-check | ubuntu | ✅ |
| build x86_64-unknown-linux-gnu | ubuntu | ✅ |
| build x86_64-unknown-linux-musl (cross) | ubuntu | ✅ |
| build aarch64-unknown-linux-gnu (cross) | ubuntu | ✅ fix `gui_ffi.rs:132` `.cast()` |
| build aarch64-linux-android (cross) | ubuntu | ✅ fix `gui_ffi.rs:132` `.cast()` |
| build aarch64-apple-darwin / x86_64-apple-darwin | macos-14 | ✅ allow_fail (colas) |
| fuzz (nightly, 5min) | ubuntu | ⏳ propuesto |

## 6. Próximos Benchmarks Propuestos

- [ ] `bench_fib` en release vs debug en CI (publicar en cada tag)
- [ ] `fuzz` diferencial 4 corpora (structs/listas, closures, rechazo, regex) — 5min nightly
- [ ] `cargo bench` (lumen-bench) con Criterion para lexer/parser/sema

---

*LÚMEN v3.0.0 — benchmarks reproducibles con `cargo build --release -p lumen-cli && lumen run examples/demo_completo.nv`.*

---

## Sección actualizada — rondas JIT v3.5.31→v3.5.37 (30 Ago 2026, Linux e2b, min-of-15, release)

| Tarea | JIT ON | Intérprete | C (gcc -O3) | Cranelift AOT |
|---|---|---|---|---|
| sum | 28.1 ms | 1138.4 ms | ~1 ms | ~7 ms |
| fib | 4.4 ms | 100.4 ms | ~2 ms | ~2.2 ms |
| primes | 11.8 ms | 34.5 ms | ~5 ms | ~5 ms |
| strings | 162.3 ms | 177.2 ms | ~10 ms | ~4 ms |
| arrays | 60.5 ms | 91.5 ms | ~2.7 ms | ~4.2 ms |
| **TOTAL** | **267.1 ms** | **1541.9 ms** | — | — |

Evolución del total JIT ON: 590 → 383 → 343.5 → 275.7 → **267.1 ms**.
fib JIT (4.4 ms) queda a ~2× del C; strings/arrays dominados por el formateo
de texto y la semántica de arrays (los AOT cranelift ya vencen al C en strings).
Detalle por ronda en `benchmarks/results/informe.md` y
[docs/arquitectura/jit.md](../arquitectura/jit.md).


---

## Ronda v3.5.38+v3.5.39 (2026-08-30): registros en bucles + inlining

Ventanas limpias (min-of-15/30; el host de esta sesión arranca procesos a
~8–16 ms vs ~1 ms de la sesión v3.5.37, ver nota en `benchmarks/results/informe.md`):

| Tarea | v3.5.37 ON | v3.5.39 ON | Intérprete | C (gcc -O3) | Δ ON |
|---|---|---|---|---|---|
| sum | 28.1 ms | **12.3 ms** | ~1140 ms | ~1 ms | **2.3×** |
| fib | 4.4 ms | **3.9 ms** | 100.4 ms | ~2 ms | 1.13× |
| primes | 11.8 ms | **4.5 ms** | 34.5 ms | ~1.7 ms | **2.6×** |
| strings | 162.3 ms | 165.7 ms | 177.2 ms | ~10.7 ms | = |
| arrays | 60.5 ms | 58.5 ms | 91.5 ms | ~2 ms | = |
| **TOTAL** | **267.1 ms** | **~245 ms** | 1541.9 ms | — | **-8%** |

Evolución: 590 → 383 → 343.5 → 275.7 → 272.6 → 268.0 → 267.1 → **~245 ms**
(6.3× vs intérprete).

Mecánica: v3.5.38 = promoción de slots calientes a SSA con block params
(bucles sin tocar el flat; bail seguro porque re-ejecuta el frame desde cero);
v3.5.39 = inlining de callees simples con pila neutra y dominancia de seeds.

Meta de la ronda (200–210 ms) no alcanzada por decisión de alcance: requiere el
trabajo profundo diferido (SSO de strings ~162→50, arrays especializados
~60→12). El JIT AOT cranelift ya vence al C en strings (3.5 ms) y empata en
arrays.
