# Benchmark LÚMEN vs C / C++ / Rust / Python (v3.5.29)

Mismo algoritmo en cada lenguaje. `Tiempo` = segundos de pared (mejor medición única);
`RSS` = memoria pico del proceso en MB.

| Tarea | lumen-vm | lumen-aotc | lumen-cranelift | c | cpp | rust | python |
|---|---|---|---|---|---|---|---|
| **fib** | 0.007s / nanMB | 0.005s / nanMB | 0.002s / nanMB | 0.001s / nanMB | 0.009s / nanMB | 0.003s / nanMB | 0.046s / nanMB |
| **sum** | 0.013s / nanMB | 0.002s / nanMB | 0.007s / nanMB | 0.001s / nanMB | 0.002s / nanMB | 0.001s / nanMB | 0.626s / nanMB |
| **primes** | 0.006s / nanMB | 0.002s / nanMB | 0.003s / nanMB | 0.002s / nanMB | 0.002s / nanMB | 0.002s / nanMB | 0.025s / nanMB |
| **strings** | 0.169s / nanMB | 0.015s / nanMB | 0.003s / nanMB | 0.012s / nanMB | 0.008s / nanMB | 0.014s / nanMB | 0.062s / nanMB |
| **arrays** | 0.060s / nanMB | 0.003s / nanMB | 0.003s / nanMB | 0.002s / nanMB | 0.003s / nanMB | 0.002s / nanMB | 0.040s / nanMB |

## Bench-5 oficial (regresión) — v3.5.41 post-fix

Protocolo: mejor medición única por tarea de `lumen run` (5 tareas, JIT apagado = default del binario).

| Estado | fib | sum | primes | strings | arrays | TOTAL |
|---|---|---|---|---|---|---|
| Certificado (v3.5.40) | — | — | — | — | — | **244 ms** (línea base ~245 ms) |
| Post-fix (v3.5.41) | 4.2–5.4 | 12.4–14.2 | 4.8–6.0 | 161.5–172.9 | 58.9–61.9 | **241.9–244.2 ms** (3 tandas) ✅ sin regresión |

JIT ON post-fix: TOTAL 251.1 ms (misma banda; las tareas no llegan a compilarse antes de terminar).
Re-verificación tras reconstrucción (3ª ronda, mismo fix): tandas de 241.9 / 244.2 / 242.8 ms y A/B intercalado con C en el mismo host (vm 248.9 ms vs C 16.5 ms, ratio 15.1× — C también bajó 8% respecto a la certificación: deriva de host, no regresión).
