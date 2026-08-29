# Benchmark LÚMEN vs C / C++ / Rust / Python (v3.5.29)

Mismo algoritmo en cada lenguaje. `Tiempo` = segundos de pared (mejor medición única);
`RSS` = memoria pico del proceso en MB.

| Tarea | lumen-vm | lumen-aotc | lumen-cranelift | c | cpp | rust | python |
|---|---|---|---|---|---|---|---|
| **fib** | 0.222s / nanMB | 0.052s / nanMB | 0.068s / nanMB | 0.008s / nanMB | 0.011s / nanMB | 0.009s / nanMB | 0.054s / nanMB |
| **sum** | 2.716s / nanMB | 0.038s / nanMB | 0.124s / nanMB | 0.007s / nanMB | 0.011s / nanMB | 0.008s / nanMB | 0.573s / nanMB |
| **primes** | 0.087s / nanMB | 0.037s / nanMB | 0.062s / nanMB | 0.007s / nanMB | 0.010s / nanMB | 0.008s / nanMB | 0.046s / nanMB |
| **strings** | 0.294s / nanMB | 0.045s / nanMB | 0.349s / nanMB | 0.015s / nanMB | 0.015s / nanMB | 0.019s / nanMB | 0.082s / nanMB |
| **arrays** | 0.151s / nanMB | 0.039s / nanMB | 0.138s / nanMB | 0.007s / nanMB | 0.010s / nanMB | 0.008s / nanMB | 0.064s / nanMB |
