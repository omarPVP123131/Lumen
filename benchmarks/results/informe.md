# Benchmark LÚMEN vs C / C++ / Rust / Python (v3.5.29)

Mismo algoritmo en cada lenguaje. `Tiempo` = segundos de pared (mejor medición única);
`RSS` = memoria pico del proceso en MB.

| Tarea | lumen-vm | lumen-aotc | lumen-cranelift | c | cpp | rust | python |
|---|---|---|---|---|---|---|---|
| **fib** | 0.005s / nanMB | 0.005s / nanMB | 0.002s / nanMB | 0.002s / nanMB | 0.011s / nanMB | 0.002s / nanMB | 0.030s / nanMB |
| **sum** | 0.028s / nanMB | 0.002s / nanMB | 0.007s / nanMB | 0.001s / nanMB | 0.002s / nanMB | 0.001s / nanMB | 0.614s / nanMB |
| **primes** | 0.012s / nanMB | 0.003s / nanMB | 0.003s / nanMB | 0.002s / nanMB | 0.002s / nanMB | 0.002s / nanMB | 0.025s / nanMB |
| **strings** | 0.172s / nanMB | 0.022s / nanMB | 0.003s / nanMB | 0.011s / nanMB | 0.013s / nanMB | 0.017s / nanMB | 0.066s / nanMB |
| **arrays** | 0.061s / nanMB | 0.003s / nanMB | 0.003s / nanMB | 0.002s / nanMB | 0.003s / nanMB | 0.002s / nanMB | 0.047s / nanMB |

---

## Ronda v3.5.37 — estado actual (min-of-15, release, JIT default-ON)

| Tarea | JIT ON | JIT OFF (intérprete) | Ganancia | Salida |
|---|---|---|---|---|
| sum | 28.1 ms | 1138.4 ms | 41× | 49999995000000 ✓ |
| fib | 4.4 ms | 100.4 ms | 23× | 121393 ✓ |
| primes | 11.8 ms | 34.5 ms | 2.9× | 2262 ✓ |
| strings | 162.3 ms | 177.2 ms | 1.09× | 2888890 ✓ |
| arrays | 60.5 ms | 91.5 ms | 1.51× | 19999900000 ✓ |
| **TOTAL** | **267.1 ms** | **1541.9 ms** | **5.8×** | — |

Evolución: 590 → 383 → 343.5 → 275.7 → 272.6 → **267.1 ms**.
Arquitectura del JIT: [docs/arquitectura/jit.md](../docs/arquitectura/jit.md).

---

## Historial v3.5.31 → v3.5.37 (resumen; run_bench.py regenera este archivo — se re-anexa aquí)

- **v3.5.31/32**: Tier-2 de bucles (aritmética de pila nativa, Call/argc
  nativos, verifier estático, Jmp en cuerpo); arrays Tier-2; super-opcodes
  de 6 (BinCmpJmp/BinKCmpJmp/BinKKCmpJmp); strings Tier-2 (dyn_arith);
  probe fusionado lj_probe_int + lj_call_fast. 590 → 383 ms.
- **v3.5.33**: BUG del constant folder IR (MIN/-1 panic + % truncante).
  Gate dyn_written. 343.5 ms.
- **v3.5.34**: BUG del folder optimize.rs (delta NETO: `f(3)+1` perdía el
  argumento y el Add). Tier-R recursión nativa en registros (fib 74→4.4).
  has_refs en Ret. 275.7 ms.
- **v3.5.35**: BUG de soundness — flat obsoleto en Tier-2 (realocación por
  calls a usuario); refetch del flat. JmpIf nativo (contar_primos Tier-2).
  272.6 ms.
- **v3.5.36**: pools de buffers de scope (slot_pool/map_pool) + invalidación
  SELECTIVA de la caché de variables. fib OFF 107.7→100.4 ms. 268 ms.
- **v3.5.37**: análisis estático de tipos VTag (concat rápido lj_concat,
  Load/Store nativos por etiqueta, arrays con etiqueta de elementos) +
  guardas de `slots`. 267.1 ms.
