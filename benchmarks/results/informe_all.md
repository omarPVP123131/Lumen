# Benchmark LÚMEN — todas las áreas del lenguaje (v3.94.24)

Mejor de 3 ejecuciones (segundos de pared). Mismo algoritmo en cada lenguaje.

| Tarea | Área | lumen-vm | lumen-aotc | C (-O3) |
|---|:---|---:|---:|---:|
| **fib** | recursión | 0.013s | 0.005s | 0.001s |
| **sum** | bucles | 0.020s | 0.001s | 0.001s |
| **primes** | primos | 0.018s | 0.002s | 0.001s |
| **strings** | strings | 1.364s | 0.012s | 0.010s |
| **arrays** | arrays | 0.309s | 0.002s | 0.002s |
| **structs** | structs | 2.239s | 0.090s | 0.001s |
| **enums** | enums | 1.163s | 0.064s | 0.001s |
| **maps** | mapas | 0.022s | 0.087s | 0.001s |
| **closures** | closures | 0.869s | 0.029s | 0.001s |
| **unicode** | unicode | 0.472s | 0.014s | 0.005s |

## Notas
- `maps` usa 1 000 claves: el backend C de mapas es de valor (O(n) por inserción, sin recolección), por lo que tamaños grandes solo son viables en la VM (mapa persistente ImMap). Ver AUDITORIA_V39424.md, ítem 10.
- `lumen-vm` corre con el binario de depuración; `lumen-aotc` compila a C -O3.
