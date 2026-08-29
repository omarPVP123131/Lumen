# Benchmark LÚMEN vs C / C++ / Rust / Python (v3.5.29)

Mismo algoritmo en cada lenguaje. `Tiempo` = segundos de pared (mejor medición única);
`RSS` = memoria pico del proceso en MB.

| Tarea | lumen-vm | lumen-aotc | lumen-cranelift | c | cpp | rust | python |
|---|---|---|---|---|---|---|---|
| **fib** | 0.181s / 12MB | 0.006s / 12MB | 0.002s / 12MB | 0.001s / 12MB | 0.002s / 12MB | 0.002s / 12MB | 0.027s / 12MB |
| **sum** | 2.488s / 12MB | 0.002s / 12MB | 0.020s / 12MB | 0.001s / 12MB | 0.002s / 12MB | 0.002s / 12MB | 0.536s / 12MB |
| **primes** | 0.090s / 12MB | 0.003s / 12MB | 0.003s / 12MB | 0.002s / 12MB | 0.003s / 12MB | 0.002s / 12MB | 0.025s / 12MB |
| **strings** | 0.225s / 12MB | 0.015s / 14MB | 0.073s / 28MB | 0.010s / 12MB | 0.009s / 12MB | 0.011s / 12MB | 0.053s / 12MB |
| **arrays** | 0.153s / 20MB | 0.002s / 12MB | 0.003s / 12MB | 0.002s / 12MB | 0.002s / 12MB | 0.002s / 12MB | 0.035s / 16MB |
