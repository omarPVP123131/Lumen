# 🚀 Guía Rápida de Experiencia de Usuario (DX / UX) — LÚMEN v3.0.0
### *The Ultimate LÚMEN Developer Experience & Cheat Sheet*

Bienvenido a **LÚMEN**, el lenguaje de programación nativo bilingüe de ultra-alto rendimiento. Esta guía resume todo lo que necesitas para dominar el ecosistema en minutos.

---

## ⚡ 1. Inicio Rápido en 30 Segundos

```bash
# 1. Crear un nuevo proyecto estructurado con plantilla
lumen new mi_proyecto --template web      # Plantillas: web | ia | game | default

# 2. Entrar al proyecto y ejecutar
cd mi_proyecto
lumen run src/main.nv

# 3. Comprobar tipos y memoria en todo el proyecto de una sola vez
lumen check .

# 4. Compilar a binario nativo independiente súper rápido
lumen bundle src/main.nv -o mi_app
./mi_app
```

---

## 💻 2. Comandos Esenciales de la CLI Inteligente

| Comando | Descripción | Ejemplo de Uso |
| :--- | :--- | :--- |
| `lumen run` | Ejecuta código fuente o bytecode en memoria | `lumen run app.nv` |
| `lumen build` | Compila a bytecode portátil (`.nvc`) o AOT nativo | `lumen build --native app.nv` |
| `lumen bundle` | Genera un binario nativo **Zero-Dependencies** | `lumen bundle app.nv -o binario` |
| `lumen check` | Verificación estática recursiva de tipos y memoria | `lumen check .` |
| `lumen repl` | REPL interactivo con comandos `:doc`, `:bench`, `:mem` | `lumen repl` |
| `lumen ai` | Asistente de IA (análisis, fixes, tests unitarios) | `lumen ai explain app.nv` |
| `lumen doctor` | Diagnóstico profundo de hardware, SIMD y compiladores | `lumen doctor` |
| `lumen monitor` | Panel TUI de telemetría y estado de memoria | `lumen monitor` |
| `lumen config` | Gestor de configuración y perfiles de compilación | `lumen config profile hpc` |
| `lumen serve` | Inicia el Playground Web interactivo con WebGPU | `lumen serve --port 8080` |
| `lumen lsp` | Servidor LSP Pro con Semantic Tokens e Inlay Hints | `lumen lsp` |

---

## 🧠 3. Modelos de Memoria y Perfiles de Optimización

LÚMEN te permite elegir el modelo de memoria ideal según tu caso de uso:

```bash
# Perfil RELEASE (Producción estándar con 64-bit NaN-Boxing + C99 -O3):
lumen build --profile release app.nv

# Perfil HPC (Supercómputo con Borrow Checker Zero-GC + AVX-512 SIMD):
lumen build --profile hpc app.nv

# Perfil MCU (Bare-metal embebido sin sistema operativo <32 KB):
lumen build --profile mcu sensor.nv

# Perfil CLOUD (Microservicios con Runtime Self-Healing y Hot-Patching):
lumen run --profile cloud servidor.nv
```

---

## 📚 4. Módulos Clave de la Biblioteca Estándar (`stdlib/`)

```lumen
// 1. Inteligencia Artificial & RAG
importar "ia.nv";             // Cuantización INT8 W8A16, RoPE, KV-Cache, Top-P
importar "vector_db.nv";      // Base de datos vectorial con índice HNSW y similitud coseno
importar "tensor.nv";         // Autograd dinámico N-dimensional y capas neuronales

// 2. Cloud, Web & Bases de Datos
importar "nexus.nv";          // Framework Web estilo FastAPI / Axum con OpenAPI 3.0
importar "postgres.nv";       // Cliente PostgreSQL Wire Protocol 3.0 nativo
importar "redis.nv";          // Cliente Redis RESP3 con pipelines asíncronos

// 3. Gráficos, Juegos & UI
importar "motor_grafico.nv";  // Cámaras 3D LookAt, Sprite Batcher GPU y física SAT/AABB
importar "ui_reactiva.nv";    // UI Declarativa Reactiva con Virtual DOM y Hooks
importar "gpu.nv";            // Shaders WebGPU WGSL, SPIR-V Vulkan y CUDA PTX

// 4. Concurrencia & Resiliencia
importar "actor.nv";          // Modelo de Actores Erlang/OTP con supervisión
importar "self_healing.nv";   // Runtime autorregenerativo con hot-patching
importar "concurrencia.nv";   // Scheduler de Fibras con Work-Stealing y canales Lock-Free
```

---

## ⌨️ 5. Cheat Sheet del REPL Interactivo (`lumen repl`)

Dentro del REPL interactivo puedes usar:
* `:doc <simbolo>` ➔ Muestra la documentación del tipo o función.
* `:bench <codigo>` ➔ Mide el tiempo de ejecución en microsegundos (µs).
* `:mem` ➔ Inspecciona el modelo de memoria activo (NaN-Boxing / Arena).
* `:clear` / `:limpiar` ➔ Reinicia las variables acumuladas.
* `:help` ➔ Muestra el menú de ayuda.
* `salir` / `exit` ➔ Termina la sesión.

---

*LÚMEN v3.0.0 — Diseñado con la mejor Experiencia de Usuario (DX) del mercado.*
