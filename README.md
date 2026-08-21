# LÚMEN — Lenguaje de Programación Nativo Bilingüe de Ultra-Alto Rendimiento

[![CI](https://github.com/omarPVP123131/Lumen/actions/workflows/ci.yml/badge.svg)](https://github.com/omarPVP123131/Lumen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-3.0.0-orange)
![Tests](https://img.shields.io/badge/tests-904%20passing-brightgreen)
![Fases](https://img.shields.io/badge/fases-0--185%20+%20self--hosting-blueviolet)

> **El primer lenguaje de programación moderno de sistemas y aplicaciones con el español y el inglés como ciudadanos de primera clase.**
> Pipeline completo: Lexer → Parser → Sema (Borrow Checker & Comptime) → IR (Neuro-Optimizador) → Bytecode JIT (Cranelift Tier-3 OSR) → AOT (C99 / LLVM / Stage-3 Autónomo).

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

## 💡 Características Principales de LÚMEN v3.0.0

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

**Verificación v3.0.0:** 904 pruebas en verde (Linux y Windows), 389/389 en `lumen check`, 389 ejemplos `run` OK con `CI=1`, clippy sin avisos y cuatro fuzzers diferenciales (structs/listas, closures, rechazo y regex) sin divergencias.

---

## 📚 Documentación Oficial

* **[Libro Oficial LÚMEN](docs/LIBRO_OFICIAL_LUMEN.md)** — De 0 a Ingeniero de Software.
* **[Guía Rápida & Cheat Sheet](docs/GUIA_RAPIDA_UX.md)** — Referencia rápida de comandos y sintaxis.
* **[Manual del Lenguaje](docs/LENGUAJE.md)** — Especificación técnica completa de la gramática.
* **[Guía de Herramientas](docs/HERRAMIENTAS.md)** — CLI, REPL, Debugger, LSP y AOT.
* **[Roadmap y Arquitectura](docs/roadmap.md)** — Plan de evolución del ecosistema.

---

*LÚMEN v3.0.0 — © 2026 LÚMEN Core Team & Comunidad.*
