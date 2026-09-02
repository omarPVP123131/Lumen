# LÚMEN — Lenguaje de Programación Nativo Bilingüe de Ultra-Alto Rendimiento

[![CI](https://github.com/omarPVP123131/Lumen/actions/workflows/ci.yml/badge.svg)](https://github.com/omarPVP123131/Lumen/actions)
[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-3.94.22-orange)
![Tests](https://img.shields.io/badge/tests-956%20passing-brightgreen)
![JIT](https://img.shields.io/badge/JIT-Tier--2%20%2B%20Tier--R%20activo-blueviolet)
![Bench](https://img.shields.io/badge/bench-JIT%20267ms%20(5.8x)-blue)
![Headless](https://img.shields.io/badge/headless-LUMEN__HEADLESS-lightgrey)
![Fases](https://img.shields.io/badge/fases-0--220%20+%20self--hosting-blueviolet)

**Autor principal:** **Omar Palomares Velasco** — [@omarPVP123131](https://github.com/omarPVP123131)

> **El primer lenguaje de programación moderno de sistemas y aplicaciones con el español y el inglés como ciudadanos de primera clase.**
> Pipeline completo: Lexer → Parser → Sema (Borrow Checker & Comptime) → IR (Neuro-Optimizador) → Bytecode → **VM + JIT Cranelift (Tier-1 / Tier-2 / Tier-R)** → AOT (C99 / Cranelift / LLVM / Stage-3 Autónomo).

---

## 🚀 Inicio Rápido

```bash
# 1. Crear un proyecto estructurado con plantilla
lumen new mi_proyecto --template web      # Plantillas: web | ia | game | default

# 2. Entrar y ejecutar en desarrollo
cd mi_proyecto
lumen run src/main.nv

# 3. Comprobar tipos y seguridad en todo el proyecto de una sola vez
lumen check .

# 4. Generar binario nativo independiente Zero-Dependencies
lumen bundle src/main.nv -o mi_app
./mi_app
```

---

## 💡 Características Principales de LÚMEN v3.94.22

### 1. Paridad Bilingüe 100% Nativa (Español / English)
```lumen
// En Español:
funcion entero calcular_fibonacci(entero n) {
    si n <= 1 { retornar n; }
    retornar calcular_fibonacci(n - 1) + calcular_fibonacci(n - 2);
}

// En Inglés (con 'importar ingles;'):
importar ingles;
function integer calculate_fibonacci(integer n) {
    if n <= 1 { return n; }
    return calculate_fibonacci(n - 1) + calculate_fibonacci(n - 2);
}
```

### 2. Modelos de Memoria Flexibles & Zero-GC
* **64-bit NaN-Boxing (`NanVal`)**: Valores compactos de 8 bytes por celda de memoria.
* **Borrow Checker Estático Opcional**: Tipos afines `prestado T`, `prestado mut T` y `dueno T` para latencia predecible sin pausas de Garbage Collection.
* **Asignador por Regiones (Arena)**: `RegionArena` con liberación en $O(1)$.
* **Runtime Autorregenerativo (*Self-Healing*)**: Captura excepciones imprevistas en producción y aplica *hot-patches* en caliente sin tirar el servidor ni perder sesiones.

### 3. Inteligencia Artificial & RAG Nativos
* **Autograd N-Dimensional (`tensor.nv`)**: Diferenciación automática con grafos de computación dinámicos y backward pass.
* **Inferencia INT8 Cuantizada (`ia.nv`)**: Matmul W8A16, Rotary Position Embeddings (RoPE), KV-Cache y muestreo Nucleus Top-P.
* **Base de Datos Vectorial RAG (`vector_db.nv`)**: Búsqueda por similitud coseno e índice HNSW para agentes de IA.

### 4. Microservicios Cloud & Bases de Datos Wire Protocol
* **Framework Web Nexus (`nexus.nv`)**: Estilo FastAPI / Axum con generación automática de especificaciones OpenAPI 3.0 y Swagger UI en `/docs`.
* **Driver PostgreSQL Wire 3.0 (`postgres.nv`)**: Cliente nativo en LÚMEN puro sin dependencias de `libpq` en C.
* **Driver Redis RESP3 (`redis.nv`)**: Cliente nativo con pipelines asíncronos en lote.

### 5. Motor de Videojuegos 2D/3D & Shaders GPU
* **Motor Gráfico (`motor_grafico.nv`)**: Cámaras 3D LookAt, Sprite Batcher GPU (1,000 sprites en 1 solo Draw Call), colisiones AABB/SAT y Raycasting 3D.
* **Shaders GPU (`gpu.nv`)**: Emisión directa de WebGPU WGSL, binarios SPIR-V (Vulkan/Metal) y NVIDIA CUDA PTX.
* **Big Data DataFrames (`dataframe.nv`)**: Columnas vectorizadas, `GroupBy` y filtros masivos estilo Polars/Arrow.

### 6. Compiladores y Multi-Arquitectura
* **Compilación Cruzada (`--target`)**: Servidores Linux x86_64, Apple Silicon ARM64 (M1/M2/M3/M4), Raspberry Pi, Windows `.exe` y RISC-V.
* **Stage-3 Bootstrap Autónomo (`asm_emitter.nv`)**: Generación directa de ejecutables ELF64 y PE32+ sin depender de GCC, Clang ni Rust con verificación **Fixed-Point Determinista**.

---

## 🛠️ Herramientas de Desarrollo (DX)

* **`lumen run <archivo.nv>`**: Ejecución instantánea con JIT hot tiering.
* **`lumen build --native <archivo.nv>`**: Compilación a código máquina (-O3).
* **`lumen bundle <archivo.nv> -o <app>`**: Empaquetado binario autónomo sin dependencias.
* **`lumen check .`**: Análisis semántico recursivo de todo el proyecto.
* **`lumen repl`**: REPL interactivo con comandos `:doc`, `:bench`, `:mem`, `:clear`.
* **`lumen ai <explain|fix|test|chat>`**: Asistente IA integrado en terminal.
* **`lumen doctor` & `lumen monitor`**: Diagnóstico de hardware, SIMD y telemetría TUI.
* **`lumen serve`**: Playground Web interactivo con WebGPU y Time-Travel Debugging.
* **Servidor LSP Pro**: Semantic Highlighting, Inlay Hints y Code Actions para VS Code, Neovim y JetBrains.

## 📊 Estado del Proyecto

| Área | Estado |
|------|--------|
| **Lenguaje Core (Fases 0-60)** | ✅ Completado |
| **Lenguaje Avanzado (Fases 61-70)** | ✅ Completado (OR patterns, rangos `..`/`..=`, if-let, string patterns) |
| **Herramientas & DX (Fases 71-95)** | ✅ Completado (LSP, debugger, fmt, lint, REPL, pkg, AOT, CLI) |
| **Stdlib Extendida (Fases 96-110)** | ✅ Completado |
| **Runtime & Sistema (Fases 111-130)** | ✅ Completado |
| **Concurrencia & Async (Fases 131-150)** | ✅ Completado |
| **GUI, TUI & Juegos (Fases 151-170)** | ✅ Completado |
| **Portabilidad (Fases 171-185)** | ✅ Completado (WASM, Docker, CI/CD, crate API) |
| **Self-Hosting (Compilador + VM en LÚMEN)** | ✅ Bootstrapping doble certificado — fixpoint byte-idéntico |
| **AI/ML (Fases 186-200)** | 🔜 Próximo hito |

**Verificación v3.94.22 — Producción Real:** **956 pruebas en verde** (636 e2e + 11 production + resto workspace), **695 vm tests** (636 e2e + 11 production + 48 unit), 396/396 en `lumen check`, 396 ejemplos `run` OK con `CI=1`, clippy sin avisos (`cargo clippy --all -- -D warnings`), **8 benches criterion** (`cargo bench -p lumen-bench`) y cuatro fuzzers diferenciales sin divergencias. `CHUNK_VERSION 7` con defaults persistidos (`FuncMeta.defaults`) y modo headless centralizado (`stdlib/graficos.nv:es_headless()` via `LUMEN_HEADLESS`/`CI`). AOT Industrial: C/Cranelift/LLVM completos con memoria nativa (`long long` sin tags, `_lw_*` handles). Ver checklist completo en [docs/desarrollo/produccion.md](docs/desarrollo/produccion.md).

### ⚡ Rendimiento JIT — rondas v3.5.31 → v3.5.37 (ago 2026)

El VM incorpora un **JIT Cranelift de tres niveles** activo por defecto
(ver [docs/arquitectura/jit.md](docs/arquitectura/jit.md)): Tier-R (recursión
auto-nativa en registros), Tier-2 (bucles con aritmética/arrays/textos nativos
sobre la arena de slots) y Tier-1 (delegación por shims). Medición
min-of-15, release:

| Tarea | JIT ON | Intérprete (`LUMEN_JIT=0`) | Ganancia |
|---|---|---|---|
| sum | 12.3 ms | 1138.4 ms | 92× |
| fib | 3.9 ms | 100.4 ms | 26× (~2× el C) |
| primes | 4.5 ms | 34.5 ms | 7.7× |
| strings | 165.7 ms | 177.2 ms | 1.07× |
| arrays | 58.5 ms | 91.5 ms | 1.56× |
| **TOTAL** | **~245 ms** | **1541.9 ms** | **6.3×** |

Evolución del total: 590 → 383 → 343.5 → 275.7 → 267.1 → **~245 ms**. Las rondas
también cazaron **4 bugs reales** (constant folder IR, folder de optimización,
puntero `flat` obsoleto en Tier-2, indexado sin guard en Load/Store nativos) —
todos arreglados y cubiertos por la batería de paridad ON/OFF.

### Producción Real (v3.94.22) — Checklist y Comandos

> Fixes escalables ya aplicados: `builder last_significant()` + `label_counter` global (fix fallthrough `Variable 'a'/'n'`), `vm FuncMeta.defaults` persistidos `CHUNK_VERSION 7` + `bind_args` unificado, `stdlib/graficos.nv es_headless()` centralizado.

```bash
# Suite completa (workspace 956)
cargo test --workspace

# E2E + regresión (636 e2e incluye fallthrough/matematicas/defaults/lambda/refs/comptime)
cargo test -p lumen-vm --test e2e

# Producción: aceptación + performance + integración (11 tests)
cargo test --test production

# Bench formal 8 benches (lexer, parser, pipeline, vm_fib_20 + 4 prod)
cargo bench -p lumen-bench
cargo bench -p lumen-bench -- --quick   # smoke CI

# Headless (sin display/SDL) — CI y local
LUMEN_HEADLESS=1 CI=1 cargo test --workspace
LUMEN_HEADLESS=1 CI=1 cargo run --bin lumen -- check examples
LUMEN_HEADLESS=1 cargo test --test production -- --nocapture

# Verificación local completa
$env:LUMEN_HEADLESS="1"; $env:CI="1"; cargo test --workspace; cargo bench -p lumen-bench -- --quick; .\target\debug\lumen.exe run examples\graficos_canvas_demo.nv
# Esperado: "Headless/CI detectado — demo omitida" o "init_fail_ok" sin Variable 'a'
```

**CI `headless-check`:** job Linux con `env: LUMEN_HEADLESS=1 CI=1` ejecuta `cargo test --workspace`, `lumen check examples`, `cargo test --test production`, `cargo bench -- --quick`. Ver `.github/workflows/ci.yml` y `docs/produccion.md`.

---

## 📚 Documentación Oficial

La documentación está organizada por carpetas — índice completo en [docs/README.md](docs/README.md):

* **Guías** ([docs/guias/](docs/guias/)) — [Libro Oficial](docs/guias/LIBRO_OFICIAL_LUMEN.md), [Guía Rápida UX](docs/guias/GUIA_RAPIDA_UX.md), [Currículum 7 días](docs/guias/CURRICULUM_7_DIAS.md).
* **Referencia** ([docs/referencia/](docs/referencia/)) — [Manual del Lenguaje](docs/referencia/LENGUAJE.md), [Especificación Formal](docs/referencia/ESPECIFICACION_FORMAL_LUMEN.md), [Herramientas](docs/referencia/HERRAMIENTAS.md), [CLI](docs/referencia/cli.md).
* **Arquitectura** ([docs/arquitectura/](docs/arquitectura/)) — [Pipeline del compilador](docs/arquitectura/architecture.md), [JIT (Tier-1/2/R)](docs/arquitectura/jit.md).
* **Desarrollo** ([docs/desarrollo/](docs/desarrollo/)) — [Roadmap](docs/desarrollo/roadmap.md), [Producción](docs/desarrollo/produccion.md), [Contribución](docs/desarrollo/contributing.md), [Self-hosting](docs/desarrollo/self-hosting.md).
* **Especificaciones** ([docs/spec/](docs/spec/)) — [bytecode .nvc](docs/spec/bytecode-format.md), [VM](docs/spec/vm-spec.md), [errores](docs/spec/error-codes.md).
* **Informes** ([docs/informes/](docs/informes/)) — benchmarks, tests, auditoría y fixpoint.
* **[CHANGELOG.md](CHANGELOG.md)** e **[info.md](info.md)** — historial completo y compendio técnico integral.

---

*LÚMEN v3.94.22 — © 2026 LÚMEN Core Team & Comunidad. **Autor principal: Omar Palomares Velasco** — [@omarPVP123131](https://github.com/omarPVP123131). 956 tests, 396 ejemplos, AOT C/Cranelift/LLVM completos, JIT Tier-1/Tier-2/Tier-R activo.*

