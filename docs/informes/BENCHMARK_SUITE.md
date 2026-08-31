# 📊 LÚMEN — Suite Variada de Benchmarks (sort · matmul · sieve · dict)

**Versión:** v3.5.40 (aditiva a la suite oficial de 5 benchmarks — NO la reemplaza)
**Fecha:** 2026-08-30
**Entorno de medición:** Linux x86_64, 2 cores, `target/release`, min-of-N en
ventanas limpias (carga del sistema verificada antes de medir).

---

## Por qué esta suite

La suite oficial (`fib/sum/primes/strings/arrays`) mide el núcleo de bucles y
recursión. Esta suite variada cubre **patrones de datos reales** que el
lenguaje necesita para posicionarse:

| Tarea | Patrón que estresa | Carga |
|---|---|---|
| `sort` | acceso por índice + escritura `a[i]=v` + control de flujo | quicksort 100k con shuffle LCG (x=42, a=1103515245, c=12345, m=2³¹) |
| `matmul` | matrices anidadas, triple bucle, aritmética acumulada | 48×48 entera: `A=(i*j)%5`, `B=(i+j)%7` |
| `sieve` | escritura por índice dispersa sobre array grande | criba de Eratóstenes hasta 1M |
| `dict` | mapa: inserción + re-escritura + lectura | 30 000 claves enteras |

**Goldens (checksums exactos, calculados con implementaciones de referencia):**

```
sort:333333333300000   matmul:520167   sieve:78498   dict:449985000
```

Los 5 lenguajes (C, C++, Rust, Python, Lúmen) implementan los **mismos
algoritmos** y deben imprimir exactamente estos valores — la comparación
cruzada es justa por construcción. `scripts/ci_bench_suite.py` verifica los
20 checksums y es parte de la CI multiplataforma (ubuntu/windows/macos).

---

## Resultados (min-of-N, release, 2 cores)

| Tarea | C (gcc -O2) | C++ (g++ -O2) | Rust (rustc -O) | Python 3 | Lúmen JIT | Lúmen intérprete | vs C (JIT) |
|---|---|---|---|---|---|---|---|
| sort 100k | 12.2 ms | 11.9 ms | 11.3 ms | 194 ms | 1149 ms | 1172 ms | 94× |
| matmul 48³ | 1.7 ms | 2.7 ms | 1.6 ms | 4.7 ms | 63 ms | 58 ms | 37× |
| sieve 1M | 3.3 ms | 3.9 ms | 3.9 ms | 62 ms | 1163 ms | 1201 ms | 352× |
| dict 30k | 1.5 ms | 3.6 ms | 3.4 ms | 4.1 ms | 241 ms | 246 ms | 161× |
| **TOTAL** | **~19 ms** | **~22 ms** | **~20 ms** | **265 ms** | **~2.6 s** | **~2.7 s** | **~140×** |

Cada celda es la mediana de 5–7 repeticiones con el binario ya compilado
(incluye el spawn de proceso, ~1–2 ms por medición — idéntico para todos).

**Lectura honesta de los números:**

1. **El bug que esta suite cazó vale más que cualquier ratio.** El prototipo
   original de `sieve(1M)` **no terminaba**: cada `criba[m]=0` clonaba el
   vector entero (`Arc::make_mut` con refcount 2 → O(n) por escritura,
   O(n²) total). El fix (opcode `ArraySetVar`, v3.5.40) lo bajó a ~1.2 s.
   Sin esta suite, el bug seguía vivo en producción.
2. **La frontera de optimización queda mapeada con precisión**: en las 4
   tareas el coste está dentro de los handlers de arrays/mapas (ArrayGet,
   ArraySetVar, `__map_*` cruzan la frontera VM en cada operación). El JIT
   Tier-2 ya compila estos bucles a código nativo, pero cada acceso a
   elemento paga una llamada al runtime. El siguiente salto (arrays de
   enteros nativos en Tier-2, como ya hace el backend AOT C con
   `arr_vars_by_name`) apunta directo a cerrar estos ratios.
3. **C es el listón correcto**: los ratios honestos vs C (37×–352×) son la
   métrica del roadmap de rendimiento; Python queda superado en 3 de 4
   tareas ya hoy con el intérprete, y el JIT se lo disputa en matmul/dict
   con margen claro de mejora.

---

## Metodología

- **Mismos algoritmos en los 5 lenguajes** (quicksort Hoare con pivote al
  inicio + pila explícita, matmul ingenuo, criba clásica, mapa con las
  mismas 3 fases). Sin `std::sort` ni `sort_unstable` escondidos.
- **Checksums independientes**: los goldens se calcularon en Python con
  implementaciones de referencia ANTES de escribir las versiones Lúmen; la
  suite CI los re-verifica en cada push.
- **min-of-N en ventanas limpias**: cada medición toma el mínimo de 5–7
  repeticiones; la carga del sistema se comprueba antes de medir.
- **Binarios en directorios temporales**: la suite CI nunca escribe
  binarios en `benchmarks/` (no contamina el árbol).

## Reproducción

```bash
# Todo de una vez (verifica checksums en los 5 lenguajes):
python3 scripts/ci_bench_suite.py

# Lúmen, tarea por tarea:
./target/release/lumen run benchmarks/lumen/sieve.nv   # sieve:78498
LUMEN_JIT_LOG=1 ./target/release/lumen run benchmarks/lumen/sieve.nv  # ver Tier-2

# C/C++/Rust:
gcc -O2 benchmarks/c/bench_suite.c -o /tmp/suite_c && /tmp/suite_c
g++ -O2 benchmarks/cpp/bench_suite.cpp -o /tmp/suite_cpp && /tmp/suite_cpp
rustc -O benchmarks/rust/bench_suite.rs -o /tmp/suite_rs && /tmp/suite_rs
python3 benchmarks/python/bench_suite.py
```

## Showcase web (wasm)

`crates/lumen-wasm/web/showcase.html` corre esta suite **en el navegador**
(VM pura en wasm32 — sin JIT, que requiere memoria ejecutable) y verifica
los checksums en vivo, con la tabla nativa al lado. Incluye un mini-editor
y el puente LÚMEN↔JS (`__js_llamar` → `window.__lumen_call`).

```bash
# Regenerar el paquete wasm (web/showcase-pkg/, gitignored):
cargo build -p lumen-wasm --target wasm32-unknown-unknown --release --features wasm
wasm-bindgen --target web --out-dir crates/lumen-wasm/web/showcase-pkg \
  target/wasm32-unknown-unknown/release/lumen_wasm.wasm
python3 -m http.server 8090 -d crates/lumen-wasm/web   # servir y abrir showcase.html
```

## Certificación v3.5.40 (árbol completo, 2026-08-30)

| Puerta | Resultado |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all -- -D warnings` | PASS (0 warnings) |
| `cargo test --workspace` | **956/956** |
| `lumen check examples` | **396/396** |
| `ci_gate.py` ×2 (JIT ON / OFF) | **PASS 392/389** ambas |
| Paridad VM vs AOT-C vs Cranelift | **28 OK / 0 divergen** |
| Fixpoint self-hosting | byte-idéntico (170985 B, sha256 `02b0460d…`) |
| Bench-5 oficial (regresión) | **TOTAL 244 ms** (línea base ~245 ms) |
| Suite variada (5 lenguajes) | **20/20 checksums** |
| Showcase wasm (Node, runtime real) | 4/4 benchmarks + editor + puente JS |
