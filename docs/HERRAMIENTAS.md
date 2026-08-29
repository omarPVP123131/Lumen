# LÚMEN — Guía Completa de Herramientas y Ecosistema (DX)

**v3.5.7 — Herramientas Oficiales de Desarrollo, Depuración y Despliegue**

---

## 📑 Tabla de Contenidos

1. [CLI de LÚMEN (Comandos y Banderas)](#1-cli-de-lúmen-comandos-y-banderas)
2. [Centro de Configuración y Perfiles (`lumen config`)](#2-centro-de-configuración-y-perfiles-lumen-config)
3. [Empaquetador Standalone Zero-Dependencies (`lumen bundle`)](#3-empaquetador-standalone-zero-dependencies-lumen-bundle)
4. [REPL Interactivo Pro (`lumen repl`)](#4-repl-interactivo-pro-lumen-repl)
5. [Time-Travel Debugger (`lumen debug`)](#5-time-travel-debugger-lumen-debug)
6. [Asistente Inteligente IA (`lumen ai`)](#6-asistente-inteligente-ia-lumen-ai)
7. [Diagnóstico y Telemetría (`lumen doctor` & `lumen monitor`)](#7-diagnóstico-y-telemetría-lumen-doctor--lumen-monitor)
8. [Gestor de Paquetes (`lumen-pkg`)](#8-gestor-de-paquetes-lumen-pkgs)
9. [Servidor LSP Pro para Editores (`lumen-lsp`)](#9-servidor-lsp-pro-para-editores-lumen-lsp)
10. [Playground Web con WebGPU (`lumen serve`)](#10-playground-web-con-webgpu-lumen-serve)

---

## 1. CLI de LÚMEN (Comandos y Banderas)

```bash
lumen <comando> [opciones] <archivo.nv>
```

### Comandos Principales
* `lumen run <archivo>`: Ejecuta código fuente o bytecode en memoria con JIT hot tiering.
* `lumen build --native <archivo>`: Compilación AOT nativa C99/GCC `-O3`.
* `lumen bundle <archivo> -o <app>`: Genera un binario nativo **Zero-Dependencies**.
* `lumen check .`: Verificación semántica recursiva de todo el proyecto.
* `lumen new <nombre> --template <ia|web|game|default>`: Scaffolding de proyectos.
* `lumen fmt <archivo>`: Formateador automático de código fuente.
* `lumen test <archivo>`: Ejecución de suites de pruebas unitarias.
* `lumen bench <archivo>`: Micro-benchmarking de rendimiento y throughput.

---

## 2. Centro de Configuración y Perfiles (`lumen config`)

```bash
# Conmutar perfiles predefinidos de optimización y memoria:
lumen config profile release   # -O3, Neuro-Optimizador activo, LTO
lumen config profile hpc       # AVX-512 SIMD, Zero-GC Borrow Checker, FMA Fused
lumen config profile mcu       # Bare-metal sin SO (<32 KB Freestanding)
lumen config profile cloud     # Nexus Web, Self-Healing Runtime, Postgres/Redis Wire
lumen config profile dev       # -O0, Time-Travel activo, JIT instantáneo

# Listar configuración activa:
lumen config list
```

---

## 3. Empaquetador Standalone Zero-Dependencies (`lumen bundle`)

Genera un único ejecutable binario autónomo que incluye el código compilado, las dependencias y la biblioteca estándar:

```bash
lumen bundle src/main.nv -o mi_servicio
./mi_servicio
```

---

## 4. REPL Interactivo Pro (`lumen repl`)

Entorno interactivo con evaluación en caliente, 64-bit NaN-Boxing y comandos especiales:
* `:doc <simbolo>`: Documentación interactiva de funciones y módulos.
* `:bench <código>`: Medición de latencia de ejecución en microsegundos (`µs`).
* `:mem`: Inspección del estado del modelo de memoria.
* `:clear`: Reinicia el ámbito de variables acumuladas.
* `:help`: Muestra la guía de comandos interactivos.

---

## 5. Time-Travel Debugger (`lumen debug`)

Depurador con capacidad de retroceder en el tiempo:
* `step` / `s`: Avanzar una instrucción de bytecode.
* `back` / `step-back`: **Retroceder una instrucción restaurando el estado exacto de la pila y registros**.
* `history` / `timeline`: Muestra la línea de tiempo de instantáneas (*snapshots*) de memoria.
* `continue` / `c`: Continuar ejecución hasta el siguiente breakpoint.

---

## 6. Asistente Inteligente IA (`lumen ai`)

* `lumen ai explain <archivo>`: Análisis estático de AST, funciones, estructuras y complejidad.
* `lumen ai fix <archivo>`: Detección y sugerencias de corrección de tipos y memoria.
* `lumen ai test <archivo>`: Generación automática de pruebas unitarias con aserciones.
* `lumen ai chat "<pregunta>"`: Asistente interactivo de arquitectura y sintaxis.

---

## 7. Diagnóstico y Telemetría (`lumen doctor` & `lumen monitor`)

* `lumen doctor`: Diagnóstico integral de compiladores C, extensiones vectoriales SIMD, modelos de memoria y módulos de la stdlib.
* `lumen monitor`: Panel TUI en tiempo real con métricas de memoria, JIT tiering y microservicios.

---

## 8. Gestor de Paquetes (`lumen-pkg`)

* `lumen add <paquete>` / `lumen install <target>`: Instala paquetes locales, archivos `.lmp`, carpetas, repositorios GitHub, crates de Rust (`cargo:<crate>`) y cabeceras C (`c:<header.h>`).
* `lumen publish [dir]`: Firma criptográfica SHA-256 y empaquetado para distribución.
* `lumen pack` / `lumen unpack`: Empaquetado y descompresión de archivos `.lmp`.

---

## 9. Servidor LSP Pro para Editores (`lumen-lsp`)

Soporte oficial para Visual Studio Code, Neovim y JetBrains:
* **Semantic Highlighting**: 11 categorías de tokens semánticos en tiempo real.
* **Inlay Hints**: Tipos deducidos sobre variables `sea`/`let`.
* **Signature Help**: Resaltado de parámetros activos en llamadas.
* **Code Actions**: QuickFixes automáticos y formateo integral.

---

## 10. Playground Web con WebGPU (`lumen serve`)

Inicia un servidor local en `http://localhost:8080` con:
* Depurador visual Time-Travel interactivo con barra de snapshots.
* Simulador de partículas WebGPU en tiempo real a 60 FPS.
* Presets interactivos listos para ejecutar de Nexus Web, PostgreSQL Wire 3.0, Redis RESP3, UI Reactiva, IA INT8 y Self-Hosting.

---

---

## 11. Producción Real v3.5.7 — Bench, Headless y Checklist

> Checklist único: [docs/produccion.md](produccion.md) — `VERSION` 3.1.4 · `CHUNK_VERSION 7` · 956 tests.

**Fixes escalables llevados a producción (21 Ago 2026):**
- `builder last_significant()` + `label_counter` global (fallthrough `Variable 'a'/'n'`)
- `vm FuncMeta.defaults` persistidos `CHUNK_VERSION 7` + `bind_args` unificado (`Call`/`CallValue`/`run_function`)
- `stdlib/graficos.nv:es_headless()` centralizado (`getenv CI/LUMEN_HEADLESS` vía `__ffi`)

**Comandos producción:**
```bash
cargo test --workspace                          # 956 (636 e2e + 9 production, 695 vm tests)
cargo test -p lumen-vm --test e2e               # 636 e2e (4 regresión: fallthrough, matematicas, defaults, lambda)
cargo test --test production                    # 9 production (aceptación 3 + performance 2 + integración)
cargo bench -p lumen-bench                      # 8 benches (lexer, parser, pipeline, vm_fib_20 + 4 prod)
cargo bench -p lumen-bench -- --quick           # smoke CI
LUMEN_HEADLESS=1 CI=1 cargo test --workspace
LUMEN_HEADLESS=1 CI=1 cargo run --bin lumen -- check examples
LUMEN_HEADLESS=1 cargo test --test production -- --nocapture
```

**CI `headless-check`:** job Linux `env: LUMEN_HEADLESS=1 CI=1` corre `cargo test --workspace`, `lumen check examples`, `cargo test --test production`, `cargo bench -- --quick` (ver `.github/workflows/ci.yml`).

**Bench formal 8** (`crates/lumen-bench/benches/benchmarks.rs`): `lexer_tokenize`, `parser_parse`, `pipeline_full`, `vm_fib_20`, `prod_fallthrough_early_return`, `prod_defaults_callvalue`, `prod_matematicas_potencia`, `prod_graficos_headless` — reporte `target/criterion/report/index.html`.

*LÚMEN v3.5.7 Producción Real — Documentación de Herramientas Sincronizada (956 tests, bench 8, headless `es_headless()`).*

