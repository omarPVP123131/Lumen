# LÚMEN — Manual Completo del Lenguaje

**Versión Oficial v3.0 — Especificación Integral de la Gramática y Ecosistema**

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

### Semántica de paso de parámetros (importante)

LÚMEN pasa **todos** los parámetros **por valor** de forma predeterminada, sin
importar si son primitivos, `estructura` o `lista<T>`. Mutar un parámetro dentro
de una función **no** afecta a la variable del llamador:

```lumen
estructura Caja { valor: entero }

funcion vacio no_muta(Caja c) { c.valor = 999; }   // opera sobre una copia

Caja a = Caja { valor: 1 };
no_muta(a);
imprimir(a.valor);   // 1  — sin cambios
```

Para que una mutación sea visible fuera de la función hay que pedirlo de forma
explícita con `prestado mut`, que sí pasa por referencia:

```lumen
funcion vacio si_muta(prestado mut Caja c) { c.valor = 999; }

Caja b = Caja { valor: 1 };
si_muta(b);
imprimir(b.valor);   // 999

// Igual para listas: acumular sin tener que reasignar el retorno.
funcion vacio acumular(prestado mut lista<entero> camino, entero n) {
    si n <= 0 { retornar; }
    camino.agregar(n);
    acumular(camino, n - 1);   // funciona también en recursión
}

lista<entero> camino = [];
acumular(camino, 5);
imprimir(camino.largo());   // 5
```

`prestado` (sin `mut`) permite leer sin copiar pero prohíbe mutar: intentarlo
produce el error `E061`. La alternativa sin préstamos sigue siendo válida:
devolver el valor modificado y reasignarlo (`mi_lista = f(mi_lista);`).

---

### Ámbito de las variables

Cada bloque (`si`, `sino`, `mientras`, `para`, `para ... en` y los bloques
sueltos `{ ... }`) abre su propio ámbito. Una variable declarada dentro **sombrea**
a la de fuera mientras dura el bloque, y la exterior queda intacta al salir:

```lumen
entero x = 1;
si (1 > 0) {
    entero x = 2;   // variable nueva, sólo vive aquí dentro
    imprimir(x);    // 2
}
imprimir(x);        // 1 — la de fuera no se tocó
```

Ojo con la diferencia entre **declarar** y **asignar**. Sin tipo delante no hay
variable nueva: se muta la de fuera, que es justo lo que se quiere en un
acumulador.

```lumen
entero suma = 0;
para i en 1..4 { suma = suma + i; }   // asigna, no declara
imprimir(suma);                        // 6
```

Los parámetros y las variables de una función o de una lambda pertenecen a su
propio marco, así que nunca chocan con los de quien llama.

## 3.1. Conversiones de Tipo y Builtins Numéricos

Todas las conversiones siguen el prefijo **`a_<tipo>()`**. Los nombres de tipo
(`texto`, `entero`, `decimal`) **no** son funciones de conversión:

```lumen
texto  s = a_texto(42);        // 42  → "42"
entero n = a_entero("42");     // "42" → 42   (inversa de a_texto)
decimal d = a_decimal("3.5");  // "3.5" → 3.5

// a_entero trunca hacia cero y tolera espacios y signo:
a_entero("  -17 ");            // -17
a_entero("3.9");               // 3
```

Cuando la entrada puede ser inválida (parsers, entrada de usuario), usa las
variantes **`_seguro`**, que devuelven `resultado<T, texto>`:

```lumen
resultado<entero, texto> r = a_entero_seguro(token);
elegir (r) {
    caso exito(v): imprimir("número: ", v);
    caso error(e): imprimir("entrada inválida: ", e);
}

si es_numero(token) { /* ... */ }   // validación booleana previa
```

Builtins numéricos disponibles sin importar ningún módulo — `abs` preserva el
tipo del argumento (entero → entero, decimal → decimal):

| Función (ES / EN) | Descripción |
|---|---|
| `abs` / `absoluto` | Valor absoluto |
| `minimo` / `min`, `maximo` / `max` | Mínimo y máximo de dos valores |
| `raiz` / `sqrt` | Raíz cuadrada |
| `potencia` / `pow` | Potencia `base^exp` |
| `piso` / `floor`, `techo` / `ceil`, `redondear` / `round` | Redondeos |

Definir una función propia con uno de estos nombres es válido: la tuya tiene
prioridad sobre el builtin.

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

### Números: división, resto y desplazamientos

El operador `%` devuelve el **resto truncado**, con el signo del dividendo
(igual que C, Rust o Java; no es el módulo euclídeo de Python):

```lumen
imprimir(-7 % 3);   // -1   (no 2)
imprimir(7 % -3);   //  1
imprimir(-7 / 3);   // -2   (la división entera trunca hacia cero)
```

Con decimales, `%` es el resto en coma flotante: `5.5 % 2.0` es `1.5`.

Los desplazamientos `<<` y `>>` sólo aceptan enteros y el número de posiciones
se limita a `0..63`; `>>` es aritmético, así que conserva el signo.

### Decimales: lo que el lenguaje NO admite

- **Un decimal exige la parte fraccionaria escrita**: `1.0`, no `1.`, y `2.0`
  en vez de `2` donde se espere un `decimal`.
- **No hay notación científica.** `1.0e10` es un error de sintaxis (E012); hay
  que escribir el número completo. Tampoco se imprime nunca en esa notación: un
  decimal muy grande o muy pequeño sale siempre en decimal plano, de forma
  idéntica en el intérprete y en el binario compilado.
- `raiz(-1.0)` imprime `NaN`, y los infinitos, `inf` / `-inf`.

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

### Añadir elementos a una lista

`agregar` admite las dos sintaxis, y desde v3.0 son **equivalentes**: ambas
modifican la lista.

```lumen
lista<entero> l = [1, 2];
agregar(l, 3);   // forma función
l.agregar(4);    // forma método
imprimir(largo(l));   // 4
```

Recuerda que las listas se pasan **por valor**: si añades dentro de una función,
el llamador no lo verá salvo que el parámetro sea `prestado mut`.

```lumen
funcion vacio mete(prestado mut lista<entero> l) { agregar(l, 9); }
lista<entero> xs = [1];
mete(xs);
imprimir(largo(xs));   // 2
```

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

### Patrones de enumeración con datos

Un `caso` puede desestructurar los datos que lleva una variante ligándolos a
variables disponibles en el cuerpo del caso. Se admiten varios datos, literales
para filtrar, `_` para ignorar y patrones OR con `|`:

```lumen
enum Figura { Circulo(decimal), Rect(decimal, decimal), Punto }

funcion decimal area(Figura f) {
    elegir (f) {
        caso Figura::Circulo(r): retornar 3.14159 * r * r;  // liga r
        caso Figura::Rect(w, h): retornar w * h;            // liga w y h
        caso Figura::Punto: retornar 0.0;
    }
    retornar 0.0;
}

enum Msg { Codigo(entero) }

elegir (m) {
    caso Msg::Codigo(404): imprimir("no encontrado"); // literal: filtra
    caso Msg::Codigo(c):   imprimir("codigo: ", c);   // captura el resto
}
```

El número de variables capturadas debe coincidir con los datos de la variante;
si no, el compilador informa `E067` indicando cuántos se esperaban.

---

### El dato capturado conserva su tipo

En un `elegir`, la variable que captura el contenido de `algun`, `exito` o
`error` tiene el tipo real del valor, no sólo números. Puedes capturar structs,
listas o texto y usarlos con normalidad:

```lumen
estructura Usuario { nombre: texto, }

funcion opcion<Usuario> buscar() {
    retornar algun(Usuario { nombre: "Ana" });
}

elegir (buscar()) {
    caso algun(u): imprimir(u.nombre);   // acceso al campo: correcto
    caso ninguno: imprimir("nadie");
}
```

Lo mismo con `resultado<T, E>`: `exito(v)` liga `v` con `T` y `error(e)` liga
`e` con `E`.

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

*LÚMEN v3.0 — Documentación Oficial Sincronizada.*


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
