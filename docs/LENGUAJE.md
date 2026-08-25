# LÚMEN — Manual Completo del Lenguaje

**Versión Oficial v3.2.0 — Especificación Integral de la Gramática y Ecosistema**

> LÚMEN es un lenguaje de programación nativo bilingüe (Español / Inglés) de ultra-alto rendimiento, tipado estático estricto y modelos de memoria adaptativos (64-bit NaN-Boxing y Borrow Checker Zero-GC).

---

## 📑 Tabla de Contenidos

1. [Sintaxis Bilingüe y Filosofía](#1-sintaxis-bilingüe-y-filosofía)
2. [Tipos de Datos Primitivos y Azúcar Opcional (T?)](#2-tipos-de-datos-primitivos-y-azúcar-opcional-t)
3. [Seguridad de Memoria y Borrow Checker (prestado / dueno)](#3-seguridad-de-memoria-y-borrow-checker-prestado--dueno)
4. [Metaprogramación en Tiempo de Compilación (comptime)](#4-metaprogramación-en-tiempo-de-compilación-comptime)
5. [Operadores, Bitwise y Operador Pipe (|>)](#5-operadores-bitwise-y-operador-pipe-)
6. [Cadenas de Texto e Interpolación f"..."](#6-cadenas-de-texto-e-interpolación-f)
7. [Listas, Slicing y Comprensiones Funcionales](#7-listas-slicing-y-comprensiones-funcionales)
8. [Consultas Integradas de Datos (LINQ / SQL Style)](#8-consultas-integradas-de-datos-linq--sql-style)
9. [Control de Flujo y Pattern Matching Exhaustivo](#9-control-de-flujo-y-pattern-matching-exhaustivo)
10. [Funciones, Lambdas y Closures](#10-funciones-lambdas-y-closures)
11. [Estructuras (Structs) y Métodos inherentes `impl`](#11-estructuras-structs-y-métodos-inherentes-impl)
12. [Enums y Tagged Unions con Datos](#12-enums-y-tagged-unions-con-datos)
13. [Interoperabilidad Políglota (Ensamblador / C / Rust)](#13-interoperabilidad-políglota-ensamblador--c--rust)
14. [Biblioteca Estándar Especializada](#14-biblioteca-estándar-especializada)

---

## 1. Sintaxis Bilingüe y Filosofía

LÚMEN cuenta con paridad de palabras clave al 100%. Agregar `importar ingles;` permite usar términos en inglés y español de forma intercambiable:

```lumen
// Español:
funcion entero duplicar(entero n) {
    retornar n * 2;
}

// English:
importar ingles;
function integer duplicate(integer n) {
    return n * 2;
}
```

---

## 2. Tipos de Datos Primitivos y Azúcar Opcional (T?)

```lumen
entero edad = 28;             // Entero de 64 bits con signo (i64)
decimal pi = 3.14159265;       // Flotante IEEE 754 de 64 bits (f64)
texto saludo = "Hola LÚMEN";   // Cadena UTF-8 inmutable
booleano activo = verdadero;   // Booleano (verdadero / falso)
lista<entero> nums = [1, 2, 3];// Lista dinámica tipada

// Azúcar sintáctico para tipos opcionales (T?):
texto? correo = algun("contacto@lumen.org");
entero? telefono = ninguno;
```

---

## 3. Seguridad de Memoria y Borrow Checker (`prestado` / `dueno`)

Para sistemas de misión crítica donde se requiere latencia predecible sin pausas de Garbage Collector:
* **`prestado T`**: Referencia inmutable sin copia.
* **`prestado mut T`**: Referencia mutable exclusiva (regla XOR de aliasing).
* **`dueno T`**: Propiedad lineal con transferencia única de titularidad (*move semantics*).

```lumen
funcion entero procesar_buffer(prestado texto mensaje, dueno lista<entero> datos) {
    imprimir("Lectura sin copia: ", mensaje);
    retornar largo(datos);
}
```

---

## 4. Metaprogramación en Tiempo de Compilación (`comptime`)

Evalúa expresiones y genera constantes durante la fase de compilación con cero sobrecarga en runtime:

```lumen
entero tamano_tabla = en_tiempo_compilacion { (1024 * 1024) / 16 + 42 };
imprimir("Constante precomputada: ", tamano_tabla); // 65578
```

---

## 5. Operadores, Bitwise y Operador Pipe (`|>`)

```lumen
// Operador Pipe para encadenamiento fluido:
lista<entero> resultado = [1, 2, 3, 4, 5] 
    |> pipe_filtrar_pares() 
    |> pipe_sumar_todos();

// Operadores a nivel de bits:
entero mascara = (0xFF & 0x0F) | (1 << 4) ^ 0x01;
```

---

## 6. Cadenas de Texto e Interpolación `f"..."`

```lumen
texto usuario = "Ana";
decimal saldo = 1450.50;
texto reporte = f"Cliente: {usuario} | Saldo Disponible: ${saldo} USD";
```

---

## 7. Listas, Slicing y Comprensiones Funcionales

```lumen
lista<entero> valores = [10, 20, 30, 40, 50];

// Comprensión de listas:
lista<entero> dobles_pares = [x * 2 para x en valores si x % 2 == 0];

// Slicing de rangos:
lista<entero> sublista = valores[1..4]; // [20, 30, 40]
```

---

## 8. Consultas Integradas de Datos (LINQ / SQL Style)

```lumen
lista<entero> primos = consultar x en valores 
                       donde x > 15 
                       ordenar_por x descendente 
                       seleccionar x * 3;
```

---

## 9. Control de Flujo y Pattern Matching Exhaustivo

```lumen
elegir (edad) {
    caso 0..17: imprimir("Menor de edad");
    caso 18..64: imprimir("Adulto");
    caso 65..120: imprimir("Adulto mayor");
    defecto: imprimir("Edad fuera de rango");
}
```

---

## 10. Estructuras (Structs) y Métodos `impl`

```lumen
estructura Vector2D {
    x: decimal,
    y: decimal
}

impl Vector2D {
    funcion decimal magnitud(este) {
        retornar matematicas_raiz((este.x * este.x) + (este.y * este.y));
    }
}

Vector2D v = Vector2D { x: 3.0, y: 4.0 };
imprimir("Magnitud: ", v.magnitud()); // 5.0
```

---

## 11. Interoperabilidad Políglota (Ensamblador / C / Rust)

```lumen
funcion vacio hardware_directo() {
    // Ensamblador inline x86_64:
    ensamblador {
        "mov rax, 1\nxor rbx, rbx\nnop"
    }

    // Código C99 inline:
    bloque_c {
        "int estado_c = 200;\n// C runtime"
    }

    // Código Rust inline:
    bloque_rust {
        "let _seguro = true;\n// Rust runtime"
    }
}
```

---

## 12. Biblioteca Estándar Especializada (`stdlib/`)

| Módulo | Descripción |
| :--- | :--- |
| `ia.nv` | Cuantización INT8 (W8A16), Rotary Position Embeddings (RoPE), KV-Cache, Top-P |
| `vector_db.nv` | Base de datos vectorial con índice HNSW y similitud coseno RAG |
| `tensor.nv` & `nn.nv` | Autograd N-dimensional dinámico, MLP, Multi-Head Attention, Transformers |
| `nexus.nv` | Framework Web estilo FastAPI / Axum con generación OpenAPI 3.0 y Swagger |
| `postgres.nv` | Cliente PostgreSQL Wire Protocol 3.0 binario nativo en LÚMEN puro |
| `redis.nv` | Cliente Redis RESP3 con canalizaciones asíncronas en lote |
| `motor_grafico.nv` | Cámaras 3D LookAt, Sprite Batcher GPU (1 Draw Call), física SAT/AABB y Raycast |
| `ui_reactiva.nv` | UI Declarativa Reactiva con Virtual DOM y hooks de estado |
| `gpu.nv` | Shaders WebGPU WGSL, binarios SPIR-V (Vulkan/Metal) y NVIDIA CUDA PTX |
| `dataframe.nv` | Big Data DataFrames vectorizados estilo Polars/Arrow con GroupBy y CSV/JSON |
| `tracing_jit.nv` | Compilador Tracing JIT Tier-3 con On-Stack Replacement (OSR) en caliente |
| `plugins.nv` | Sistema de plugins dinámicos `.so` / `.dll` con recarga en caliente sin downtime |
| `self_healing.nv` | Runtime autorregenerativo con hot-patching ante excepciones imprevistas |
| `crypto.nv` | Claves asimétricas Ed25519, SHA-256, SHA-512, AES, JWT |
| `actor.nv` | Modelo de Actores Erlang/OTP con buzones y árboles de supervisión |

---

*LÚMEN v3.2.0 — Documentación Oficial Sincronizada.*

> **Producción Real v3.1.4 (21 Ago 2026):** fixes escalables `last_significant()` + `label_counter` global, `CHUNK_VERSION 7` con `FuncMeta.defaults` persistidos + `bind_args` unificado, `stdlib/graficos.nv:es_headless()` (`LUMEN_HEADLESS`/`CI`), bench 8 (`cargo bench -p lumen-bench`), 616 e2e + 9 production = 673 vm tests (917 workspace), CI `headless-check`. Ver [docs/produccion.md](produccion.md).

---

## 15. Producción Real v3.2.0 — Fixes Escalables, Bench y Headless

> Checklist único: [docs/produccion.md](produccion.md)

**Fixes escalables (21 Ago 2026):**
- **Fallthrough `Variable 'a'/'n'`:** `crates/lumen-ir/src/builder.rs` `last_significant()` ignora `Label/Nop/Phi` para decidir terminador + `label_counter` global (evita colisión `Label(0)` en `codegen` global que rompía `matematicas.nv`).
- **Defaults persistidos `CHUNK_VERSION 7`:** `ir::Func.defaults` → `codegen::FuncMeta.defaults` (`Int/Float/Str/Bool`) serializado en `Bytecode` v7 (decode v6+7). `VM bind_args` usa `DefaultValue` en `Call`/`CallValue`/`run_function`.
- **Headless centralizado:** `stdlib/graficos.nv:es_headless()` con `getenv("CI"/"LUMEN_HEADLESS")` vía `__ffi` (`msvcrt`/`libc`/`libSystem`) → `iniciar()`/`ventana()` retornan `false/0` sin `SDL_Init`.

**Verificación producción:**
```bash
cargo test --workspace                          # 917 (616 e2e + 9 production)
cargo bench -p lumen-bench                      # 8 benches (4 prod nuevos)
cargo bench -p lumen-bench -- --quick           # smoke CI
LUMEN_HEADLESS=1 CI=1 cargo test --workspace
LUMEN_HEADLESS=1 CI=1 cargo run --bin lumen -- check examples
```

---

## 21. Álgebra Lineal 2D & Aceleración Vectorial SIMD (`matriz_simd.nv` y `simd.nv`)

LÚMEN cuenta con primitivas vectoriales nativas alineadas con instrucciones de hardware **AVX2 / AVX-512** (x86_64) y **ARM NEON**:

```lumen
importar "matriz_simd.nv";
importar "simd.nv";

// 1. Vectores SIMD de 256 bits (8 floats en paralelo)
simd_SimdF32x8 a = simd_f32x8_nuevo(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0);
simd_SimdF32x8 b = simd_f32x8_nuevo(10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0);
simd_SimdF32x8 suma = simd_sumar_f32x8(a, b); // 1 ciclo CPU

// 2. Matrices 2D con Tiled GEMM para Caché L1/L2
matriz_simd_Matriz2D M1 = matriz_simd_matriz_crear(4, 4, 1.0);
matriz_simd_Matriz2D M2 = matriz_simd_matriz_identidad(4);
matriz_simd_Matriz2D M3 = matriz_simd_matriz_multiplicar_simd(M1, M2, 4);
```

---

## 22. Inferencia de Modelos de Lenguaje Locales (`gguf.nv`)

Parser binario GGUF v3 para cargar pesos de modelos LLM (*Llama-3*, *Phi-3*, *Mistral*) cuantizados en `Q4_K_M` y `Q8_0` sin dependencias externas:

```lumen
importar "gguf.nv";

gguf_GgufModelo modelo = gguf_cargar_modelo("modelos/phi-3-mini.gguf", 0.7, 0.9);
gguf_GgufSesionChat chat = gguf_crear_sesion(modelo, "Eres un asistente de programación.");
texto respuesta = gguf_generar_respuesta(chat, "¿Cómo funciona el autograd?");
imprimir(respuesta);
```

---

## 23. Concurrencia Masiva M:N & Work-Stealing (`scheduler.nv`)

```lumen
importar "scheduler.nv";

scheduler_SchedulerPool pool = scheduler_crear_pool(8);
pool = scheduler_spawn(pool, "Calculo_Fisica", "10000_particulas");
pool = scheduler_ejecutar_todos(pool);

scheduler_SchedulerCanal canal = scheduler_canal_crear(10);
canal = scheduler_canal_enviar(canal, "MSG_01: Evento procesado");
texto msg = scheduler_canal_mirar(canal);
```

---

## 24. Diferenciación Automática & Grafos Dinámicos (`autograd.nv`)

```lumen
importar "autograd.nv";

autograd_TensorAutograd w = autograd_crear([0.5, -0.2], verdadero);
autograd_TensorAutograd x = autograd_crear([1.0, 2.0], falso);
autograd_TensorAutograd y_target = autograd_crear([2.0, 4.0], falso);

autograd_OptimizadorAdamW opt = autograd_crear_adamw(0.01, 0.001);

autograd_TensorAutograd pred = autograd_multiplicar(w, x);
autograd_TensorAutograd loss = autograd_mse_loss(pred, y_target);

w = autograd_cero_grad(w);
w = autograd_backward(loss, w);
w = autograd_paso_adamw(opt, w);
```

