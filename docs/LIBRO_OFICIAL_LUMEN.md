# 📖 Aprende a Programar con LÚMEN: De Principiante a Ingeniero de Software
### *Learn to Code with LÚMEN: From Zero to Software Engineer*

**Versión Oficial v3.5.7 — Guía Completa de Computación, IA y Sistemas**

---

## 🧭 Mapa de la Ruta de Aprendizaje

```
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │ NIVEL 1: Fundamentos, Variables, Tipos y Sintaxis Bilingüe                  │
 ├─────────────────────────────────────────────────────────────────────────────┤
 │ NIVEL 2: Control de Flujo, Comprensiones, LINQ y Operador Pipe (|>)         │
 ├─────────────────────────────────────────────────────────────────────────────┤
 │ NIVEL 3: Estructuras de Datos, Métodos `impl`, Trait Bounds y Try-Catch     │
 ├─────────────────────────────────────────────────────────────────────────────┤
 │ NIVEL 4: Concurrencia M:N, Fibras, Canales Lock-Free, WebSockets y OpenAPI  │
 ├─────────────────────────────────────────────────────────────────────────────┤
 │ NIVEL 5: Inteligencia Artificial, Tensores, Autograd Dinámico y Transformers│
 ├─────────────────────────────────────────────────────────────────────────────┤
 │ NIVEL 6: Compiladores, LLVM IR, AOT Embebido (<32 KB) y Self-Hosting Total  │
 └─────────────────────────────────────────────────────────────────────────────┘
```

---

# Nivel 1: Fundamentos y Tipos de Datos

LÚMEN está diseñado para que cualquier persona pueda aprender a programar en su propio idioma materno sin sacrificar rendimiento nativo.

### 1.1 Hola Mundo Bilingüe
```lumen
// En Español:
imprimir("¡Hola, Mundo desde LÚMEN!");

// En Inglés (con 'importar ingles;'):
importar ingles;
print("Hello, World from LÚMEN!");
```

### 1.2 Tipos Primitivos y Variables
```lumen
entero edad = 28;             // Entero de 64 bits con signo (i64)
decimal pi = 3.14159265;       // Decimal IEEE 754 de 64 bits (f64)
texto saludo = "Hola";        // Cadena UTF-8 inmutable
booleano activo = verdadero;  // Booleano (verdadero / falso)

// Azúcar sintáctico para tipos opcionales (T?):
texto? correo = algun("usuario@lumen.org");
entero? telefono = ninguno;

// Interpolación moderna con f"..."
texto perfil = f"Usuario: {saludo} | Edad: {edad} | Próximo año: {edad + 1}";
```

---

# Nivel 2: Algoritmos, Comprensiones, Consultas LINQ y Pipelines

### 2.1 Operador Pipe (`|>`)
Encadena transformaciones funcionales de izquierda a derecha con cero sobrecarga:
```lumen
funcion entero duplicar(entero x) { retornar x * 2; }
funcion entero sumar_diez(entero x) { retornar x + 10; }

// 10 |> duplicar() |> sumar_diez() evalúa a 30
entero resultado = 10 |> duplicar() |> sumar_diez();
```

### 2.2 Comprensiones de Listas con Filtros y Rangos
```lumen
lista<entero> base = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

// Comprensión con condición:
lista<entero> pares_cuadrados = [x * x para x en base si x % 2 == 0];
// [4, 16, 36, 64, 100]

// Comprensión con rangos inclusivos:
lista<entero> decenas = [n * 10 para n en 1..=5];
// [10, 20, 30, 40, 50]
```

### 2.3 Consultas Declarativas de Datos (LINQ / SQL Style)
```lumen
lista<entero> transacciones = [120, 45, 890, 15, 340, 75, 1200, 50];

lista<entero> aprobadas = consultar t en transacciones
                          donde t >= 100
                          seleccionar t * 2;
// Resultado: [240, 1780, 680, 2400]
```

---

# Nivel 3: Estructuras, Métodos Inherentes, Pattern Matching y Try-Catch

### 3.1 Estructuras con Métodos `impl`
```lumen
estructura Punto {
    x: entero,
    y: entero
}

impl Punto {
    funcion entero distancia_al_origen(este) {
        retornar este.x * este.x + este.y * este.y;
    }
}

Punto p = Punto { x: 3, y: 4 };
imprimir("Distancia: ", p.distancia_al_origen()); // 25
```

### 3.2 Pattern Matching Estructural en Objetos
```lumen
elegir (p) {
    caso Punto { x: 0, y: 0 }: imprimir("En el origen (0, 0)");
    caso Punto { x: 0, y: val_y }: imprimir(f"Sobre el eje Y en y = {val_y}");
    caso Punto { x: val_x, y: 0 }: imprimir(f"Sobre el eje X en x = {val_x}");
    defecto: imprimir("Punto general");
}
```

### 3.3 Manejo Estructurado de Errores con `intentar { ... } atrapar (e)`
```lumen
intentar {
    imprimir("Ejecutando operación segura...");
    entero res = 100 / 0;
} atrapar (error_msg) {
    imprimir("Error capturado y contenido: ", error_msg);
}
```

### 3.4 Gestión Determinista de Recursos con `posponer` (RAII)
```lumen
funcion void procesar_datos() {
    posponer {
        imprimir("Limpieza y cierre garantizado de archivos (LIFO).");
    }
    imprimir("Procesando...");
}
```

---

# Nivel 4: Concurrencia M:N, Fibras, WebSockets y Microservicios

### 4.1 Fibras M:N con Planificador de Robo de Trabajo (Work-Stealing)
```lumen
importar "concurrencia.nv";

// Planificador M:N en espacio de usuario
concurrencia_PlanificadorWorkStealing scheduler = concurrencia_planificador_crear(4);
scheduler.programar_tarea(0, "tarea_computo_ia");
entero completadas = scheduler.robar_y_ejecutar(0);
```

### 4.2 Canales Lock-Free Basados en Ring Buffers
```lumen
concurrencia_CanalLockFree canal = concurrencia_canal_lockfree_crear(512);
canal = canal.enviar("Evento_Telemetria");
cualquiera dato = canal.recibir();
```

### 4.3 Servidor HTTP REST, WebSockets y OpenAPI 3.0 Automático
```lumen
importar "servidor.nv";

servidor_ServidorWeb app = servidor_crear(8080);
app = app.ruta_get("/api/saludo", "handle_saludo");
app = app.ruta_ws("/ws/chat", "handle_ws");
app = app.habilitar_swagger("/docs", "API LÚMEN", "2.4.4");

app.iniciar();
```

---

# Nivel 5: Inteligencia Artificial, Tensores y Transformers

### 5.1 Diferenciación Automática Dinámica (Autograd)
```lumen
importar "tensor.nv";

// Grafo dinámico: L = x * w + b
tensor_GrafoAutograd grafo = tensor_autograd_nuevo();
grafo = grafo.variable(3.0); // Nodo 0 (x)
grafo = grafo.variable(4.0); // Nodo 1 (w)
grafo = grafo.variable(5.0); // Nodo 2 (b)
grafo = grafo.multiplicar(0, 1); // Nodo 3 (x * w = 12.0)
grafo = grafo.sumar(3, 2);       // Nodo 4 (L = 17.0)

// Backward pass automático (regla de la cadena):
grafo = grafo.backward(4);
imprimir("dL/dx (w): ", grafo.gradiente(0)); // 4.0
imprimir("dL/dw (x): ", grafo.gradiente(1)); // 3.0
```

### 5.2 Multi-Head Self-Attention y Bloque Transformer
```lumen
importar "nn.nv";

nn_BloqueTransformer transformer = nn_transformer_crear(8, 2);
lista<decimal> embeddings = [0.1, 0.4, 0.8, -0.2, 0.5, 0.9, -0.4, 0.2];
lista<decimal> salida_atencion = transformer.procesar(embeddings);
imprimir("Representación Transformer: ", salida_atencion);
```

### 5.3 Álgebra Lineal y Criptografía Ed25519
```lumen
importar "matrices.nv";
importar "crypto.nv";

// Determinante 2x2 e inversa:
lista<lista<decimal>> m = [[4.0, 7.0], [2.0, 6.0]];
decimal det = matrices_determinante_2x2(m); // 10.0

// Criptografía asimétrica:
crypto_ParClavesEd25519 par = crypto_generar_par_claves("seed_segura");
texto firma = crypto_firmar_mensaje("Transaccion_Aprobada", par.clave_privada);
```

### 5.4 Base de Datos Vectorial Nativa & Búsqueda RAG (vector_db.nv)
```lumen
importar "vector_db.nv";

vector_db_BaseVectores db = vector_db_crear(3, "docs_ia");
db = vector_db_insertar(db, "doc_1", "LÚMEN y compiladores AOT", [0.9, 0.1, 0.2], "tech");

lista<vector_db_ResultadoBusqueda> ranking = vector_db_buscar(db, [0.85, 0.2, 0.1], 1, "coseno");
imprimir("Top 1 RAG: ", ranking[0].contenido, " Similitud: ", ranking[0].similitud);
```

### 5.5 Inferencia de Modelos de Lenguaje, Cuantización INT8 y RoPE (ia.nv)
```lumen
importar "ia.nv";

// Cuantización de pesos a INT8 (W8A16):
ia_TensorCuantizadoInt8 w_int8 = ia_cuantizar_int8([[0.9, -0.4], [0.2, 0.7]]);
lista<decimal> salida = ia_matmul_cuantizado(w_int8, [1.0, 0.5]);

// RoPE (Rotary Position Embeddings) y Muestreo Nucleus Top-P:
lista<decimal> q_rot = ia_aplicar_rope([1.0, 0.0, 0.5, 0.5], 1, 4);
lista<decimal> probs = ia_softmax([2.5, 1.2, 0.3, 4.8], 0.8);
entero token_id = ia_muestrear_top_p(probs, 0.9);
```

---

# Nivel 6: Arquitectura de Compiladores, AOT y Self-Hosting

### 6.1 Emisión Directa de LLVM IR Industrial
```bash
lumen build --aot llvm mi_programa.nv
```
* Emite código `@func()` con bloques básicos, instrucciones vectoriales y optimizaciones Polly / LTO.

### 6.2 Target Embebido Bare-Metal para Microcontroladores (<32 KB)
```bash
lumen build --embedded sensor_iot.nv
```
* Binarios C99 autónomos sin llamadas al sistema operativo listos para flashear en **ESP32 y Raspberry Pi Pico**.

### 6.3 Empaquetado Binario Standalone Zero-Dependencies (lumen bundle)
```bash
lumen bundle mi_programa.nv -o aplicacion_distribuible
```
* Genera un único binario ejecutable autocontenido listo para distribuir sin requerir LÚMEN ni dependencias externas.

### 6.4 Asistente Inteligente de CLI Integrado (lumen ai)
```bash
lumen ai explain mi_programa.nv   # Análisis estático y complejidad
lumen ai fix mi_programa.nv       # Diagnóstico y corrección sugerida
lumen ai test mi_programa.nv      # Generador automático de tests unitarios
lumen ai chat "Cómo usar RAG?"    # Consultas interactivas de arquitectura
```

### 6.5 Bootstrap 100% Self-Hosted en Puro LÚMEN
```bash
lumen bootstrap mi_programa.nv
```
* Compilación autónoma con el compilador escrito en LÚMEN puro (`stdlib/compiler/compiler_v4.nv`) con cero dependencias externas.

---

# Nivel 7: Tipos Afines, Borrow Checker Zero-GC y Seguridad de Memoria

LÚMEN combina la simplicidad sintáctica de alto nivel con un **Borrow Checker en tiempo de compilación** opcional para sistemas de ultra-baja latencia donde no se permite ninguna pausa de Garbage Collection.

### 7.1 Referencias Inmutables (`prestado`) y Propiedad Única (`dueno`)
```lumen
funcion entero procesar_datos(prestado texto buffer, dueno lista<entero> paquete) {
    imprimir("Acceso sin copia: ", buffer);
    imprimir("Tamaño del buffer: ", largo(paquete));
    retornar largo(paquete);
}

texto buffer = "Paquete de red TCP/IP";
lista<entero> payload = [0xFF, 0x01, 0x02];
procesar_datos(buffer, payload);
```

### 7.2 Metaprogramación Comptime (`en_tiempo_compilacion` / `comptime`)
Ejecuta funciones y cálculos matemáticos durante la compilación, insertando constantes directas en el binario sin sobrecarga en runtime:
```lumen
entero tamano_tabla = en_tiempo_compilacion { (1024 * 1024) / 16 + 42 };
imprimir("Constante precomputada en compilación: ", tamano_tabla); // 65578
```

---

# Nivel 8: Programación Políglota Unificada (Alto + Bajo Nivel)

LÚMEN permite combinar en el mismo archivo código de altísimo nivel (IA, RAG, WebSockets) con instrucciones de hardware de bajo nivel en ensamblador x86_64, C y Rust:

```lumen
// 1. Bloque de Ensamblador Nativo Inline:
ensamblador {
    "mov rax, 1\nxor rbx, rbx\nnop"
}

// 2. Bloque C99 Directo:
bloque_c {
    "int codigo_estado = 200;\n// C runtime"
}

// 3. Bloque Rust Directo:
bloque_rust {
    "let _seguro = true;\n// Rust runtime"
}
```

---

# Nivel 9: Shaders GPU Directos (SPIR-V, CUDA PTX, WebGPU WGSL)

El módulo `gpu.nv` compila y genera kernels de GPU listos para ejecución masiva en tarjetas gráficas:
* **WebGPU WGSL**: Shaders de multiplicación de matrices distribuidas (`gpu_generar_wgsl_matmul`).
* **Binario SPIR-V (Vulkan/Metal)**: Emisión directa de bytecode con opcodes estándar (`gpu_generar_spirv_shader`).
* **NVIDIA CUDA PTX**: Generación de ensamblador de arquitectura paralela PTX (`gpu_generar_ptx_kernel`).

```lumen
importar "gpu.nv";

texto wgsl = gpu_generar_wgsl_matmul(64, 64, 16);
lista<entero> spirv = gpu_generar_spirv_shader("matmul", 16);
texto ptx = gpu_generar_ptx_kernel("lumen_fma", 16, 16);
```

---

# Nivel 10: Emisor de Binarios ELF64/PE Autónomo (Stage 3)

El módulo `stdlib/compiler/asm_emitter.nv` es capaz de emitir binarios ejecutables **ELF64 (Linux)**, **PE32+ (Windows .exe)** y **Mach-O (macOS)** directamente desde LÚMEN puro sin invocar GCC, Clang ni ningún compilador anfitrión:

```lumen
importar "asm_emitter.nv";

asm_emitter_EmisorAsmX86 emisor = asm_emitter_asm_crear(4194304);
emisor.push_rbp();
emisor.mov_rax_inmediato(42);
emisor.emit_exit(0);

lista<entero> binario_elf = emisor.generar_elf64_binario();
imprimir("Bytes ELF64 generados: ", largo(binario_elf));
```

---

# Nivel 11: Compilación Cruzada Multi-Arquitectura Industrial

LÚMEN permite generar ejecutables nativos para cualquier sistema y procesador en un solo comando:

```bash
# Linux Servidores (x86_64 ELF64):
lumen build --native --target x86_64-linux-gnu app.nv

# Apple Silicon (ARM64 M1/M2/M3/M4 macOS Mach-O):
lumen build --native --target aarch64-apple-darwin app.nv

# Dispositivos ARM64 (Raspberry Pi 4/5 & AWS Graviton):
lumen build --native --target aarch64-linux-gnu app.nv

# Windows (x64 PE32+ .exe directo):
lumen build --native --target x86_64-pc-windows-msvc app.nv

# Hardware Abierto & Microcontroladores (RISC-V 64-bit):
lumen build --native --target riscv64-unknown-elf app.nv
```

---

# Nivel 12: Motor de Videojuegos 2D/3D & Gráficos Nativos (motor_grafico.nv)

El módulo `stdlib/motor_grafico.nv` proporciona un motor completo para simulaciones interactivas, renderizado acelerado por GPU y motores de física:

### 12.1 Álgebra Lineal 3D y Cámaras (LookAt Matrix)
```lumen
importar "motor_grafico.nv";

// Vectores 3D y Producto Cruz:
motor_grafico_Vector3D v1 = motor_grafico_vec3(1.0, 0.0, 0.0);
motor_grafico_Vector3D v2 = motor_grafico_vec3(0.0, 1.0, 0.0);
motor_grafico_Vector3D cruz = motor_grafico_vec3_cruz(v1, v2); // [0, 0, 1]

// Cámara 3D y Matriz de Proyección en Perspectiva:
motor_grafico_Vector3D pos = motor_grafico_vec3(0.0, 5.0, -10.0);
motor_grafico_Vector3D objetivo = motor_grafico_vec3(0.0, 0.0, 0.0);
motor_grafico_Camara3D cam = motor_grafico_camara3d_crear(pos, objetivo, 60.0, 1.777);
motor_grafico_Matriz4x4 vista = motor_grafico_camara3d_matriz_vista(cam);
```

### 12.2 Sprite Batcher Automático GPU (1 Solo Draw Call)
```lumen
motor_grafico_SpriteBatcher batch = motor_grafico_batcher_crear(5000);
para (entero i = 0; i < 1000; i = i + 1) {
    batch = motor_grafico_batcher_dibujar(batch, 1, i * 2.0, 50.0, 32.0, 32.0);
}
batch = motor_grafico_batcher_flush(batch); // Vuelca 1000 sprites en 1 solo Draw Call GPU
```

### 12.3 Detección de Colisiones AABB, SAT & Raycasting 3D
```lumen
// Colisiones AABB (Cajas 2D):
motor_grafico_CajaAABB jugador = motor_grafico_CajaAABB { min_x: 10.0, min_y: 10.0, max_x: 30.0, max_y: 30.0 };
motor_grafico_CajaAABB enemigo = motor_grafico_CajaAABB { min_x: 25.0, min_y: 20.0, max_x: 45.0, max_y: 40.0 };
booleano choca = motor_grafico_colision_aabb(jugador, enemigo); // verdadero

// Raycast 3D contra Esfera:
motor_grafico_Rayo3D rayo = motor_grafico_Rayo3D {
    origen: motor_grafico_vec3(0.0, 0.0, -10.0),
    direccion: motor_grafico_vec3(0.0, 0.0, 1.0)
};
motor_grafico_Esfera3D esfera = motor_grafico_Esfera3D {
    centro: motor_grafico_vec3(0.0, 0.0, 0.0),
    radio: 2.0
};
decimal distancia_impacto = motor_grafico_raycast_impacto_esfera(rayo, esfera); // 8.0
```

---

# Nivel 13: Compilador Neuro-Simbólico & Superoptimización SIMD (neuro_opt.nv)

El optimizador neuro-simbólico de LÚMEN analiza secuencias de bytecode y representación intermedia (IR) descubriendo patrones de reducción de coste computacional:
* **Strength Reduction**: Transforma multiplicaciones enteras por potencias de 2 en desplazamientos a nivel de bit (`x * 8 ➔ x << 3`).
* **Fusión de Instrucciones FMA**: Detecta `(a * b) + c` y genera instrucciones fusionadas vectoriales con un solo redondeo de punto flotante.
* **Eliminación de Cargas Redundantes**: Elimina accesos sucesivos a memoria (`Store(x), Load(x)`) manteniendo el valor en registros de hardware.

```lumen
importar "neuro_opt.nv";

neuro_opt_AnalizadorNeuroSimbilico opt = neuro_opt_crear();
imprimir("Reglas activas: ", largo(opt.reglas_activas));
imprimir("Instrucciones fusionadas: ", opt.instrucciones_eliminadas);
imprimir("Aceleración SIMD estimada: ", opt.aceleracion_simd_estimada, "x vs -O3");
```

---

# Nivel 14: Runtime Autorregenerativo & Hot-Patching en Caliente (self_healing.nv)

Para servicios en producción de misión crítica donde el tiempo de inactividad (*downtime*) es inadmisible:
* **Interceptación de Fallas**: Captura divisiones por cero imprevistas, datos corruptos o desbordamientos en la capa VM / JIT.
* **Hot-Patching en Caliente**: Reemplaza el bytecode de la función averiada por un fallback seguro y re-ejecuta la transacción sin tirar el servidor ni perder la sesión del usuario.

```lumen
importar "self_healing.nv";

self_healing_MotorSelfHealing motor = self_healing_iniciar();

// Registrar parche de auto-reparación en la tabla de despacho dinámico:
motor = self_healing_registrar_parche(
    motor, 
    "procesar_pago_stripe", 
    "procesar_pago_stripe_v2_sanitizado", 
    "Auto-reparación ante payload corrupto"
);

// Transacción con auto-recuperación transparente:
motor = self_healing_ejecutar_transaccion(motor, "procesar_pago_stripe", verdadero);
imprimir("Respuesta protegida: ", motor.ultimo_resultado);
imprimir("Estado de resiliencia: ", self_healing_obtener_metricas(motor));
```

---

# Nivel 15: Compilador Tracing JIT Tier-3 & On-Stack Replacement (tracing_jit.nv)

Para algoritmos intensivos que requieren el máximo rendimiento de ejecución en caliente:
* **Detección de Bucles Calientes**: Monitorea los encabezados de bucles e identifica iteraciones frecuentes.
* **Compilación de Trazas a RAM**: Compila la secuencia lineal de instrucciones directamente a código máquina en memoria.
* **On-Stack Replacement (OSR)**: Transfiere la ejecución interpretada al código nativo compilado a velocidades de **12.5x a 50x**.

```lumen
importar "tracing_jit.nv";

tracing_jit_CompiladorTracingJIT jit = tracing_jit_crear(20);
para (entero i = 0; i < 25; i = i + 1) {
    jit = tracing_jit_registrar_iteracion(jit, 101);
}
texto resultado_osr = tracing_jit_ejecutar_osr(jit, 101);
imprimir(resultado_osr);
```

---

# Nivel 16: Sistema de Plugins Dinámicos en Caliente (plugins.nv)

Permite extender aplicaciones en producción cargando y recargando bibliotecas compartidas (`.so` / `.dll` / `.dylib`) sin reiniciar el proceso:

```lumen
importar "plugins.nv";

plugins_GestorPlugins gestor = plugins_gestor_crear();
gestor = plugins_plugin_cargar(gestor, "compresor_zstd", "lib/zstd.so");
imprimir(plugins_plugin_ejecutar(gestor, "compresor_zstd", "transform", "Datos"));

// Recarga en caliente con nueva versión (0 downtime):
gestor = plugins_plugin_recargar_caliente(gestor, "compresor_zstd");
```

---

# Nivel 17: Motor de Big Data & DataFrames Vectorizados (dataframe.nv)

Análisis y procesamiento de datos tabulares a escala masiva con operaciones vectorizadas:

```lumen
importar "dataframe.nv";

dataframe_DataFrame df = dataframe_df_nuevo(["pais", "edad", "salario_usd"]);
df = dataframe_df_agregar_fila(df, ["Mexico", 28.0, 4500.0]);
df = dataframe_df_agregar_fila(df, ["España", 34.0, 5200.0]);
df = dataframe_df_agregar_fila(df, ["Mexico", 42.0, 6800.0]);

// Filtrado Vectorizado:
dataframe_DataFrame altos = dataframe_df_filtrar_mayor_que(df, "salario_usd", 4600.0);

// Agrupación GroupBy:
dataframe_DataFrame agrupado = dataframe_df_agrupar_por_promedio(df, "pais", "salario_usd");
imprimir(dataframe_df_a_csv(df));
```

---

*LÚMEN v3.5.7 — © 2026 LÚMEN Core Team & Comunidad.*

> **Producción Real (21 Ago 2026):** ver [docs/produccion.md](produccion.md) — 956 tests, bench 8, `es_headless()`, `CHUNK_VERSION 7`.

---

# CAPÍTULO 17.5: Producción Real v3.5.7 — De Código a Deploy (21 Ago 2026)

En este capítulo llevamos **LÚMEN a producción real** con fixes escalables (no parches por demo), suite formal y CI headless.

### 17.5.1 Fixes Escalables Llevados a Producción
- **Builder `last_significant()` + `label_counter` global:** evita fallthrough `Variable 'a'` (función con `si __ren==0 { retornar; }` sin `Return` final caía en `limpiar_pantalla_alfa`) y `Variable 'n'` en `matematicas.nv` (colisión `Label(0)` en `label_map` global).
- **VM `FuncMeta.defaults` persistidos `CHUNK_VERSION 7`:** `ir::Func.defaults` → `codegen::FuncMeta.defaults` (`Int/Float/Str/Bool`) serializados en `Bytecode` v7 (`decode` acepta 6 y 7). `bind_args` unificado para `Call`/`CallValue`/`run_function` (hilos) — `CallValue` (`var f=suma; f(5)`) ya no pierde defaults.
- **Headless centralizado `stdlib/graficos.nv:es_headless()`:** `getenv("CI"/"LUMEN_HEADLESS")` vía `__ffi` (`msvcrt`/`libc`/`libSystem`), `peek!=0` → `iniciar()`/`ventana()` retornan `false/0` sin `SDL_Init`. Demos con `si !iniciar() { retornar; }` salen con `init_fail_ok`/`Headless/CI detectado — demo omitida`.

### 17.5.2 Suite y Bench Formal (8 benches)
```bash
cargo test --workspace                          # 956 (636 e2e + 9 production, 695 vm tests)
cargo test -p lumen-vm --test e2e               # 636 e2e (4 regresión: fallthrough, matematicas, defaults, lambda)
cargo test --test production                    # 9 production (aceptación 3 + performance 2 + integración)
cargo bench -p lumen-bench                      # 8: lexer_tokenize, parser_parse, pipeline_full, vm_fib_20 + prod_fallthrough, prod_defaults, prod_matematicas, prod_graficos_headless
cargo bench -p lumen-bench -- --quick           # smoke CI — reporte target/criterion/report/index.html
```

### 17.5.3 CI `headless-check` y Verificación Local
```yaml
# .github/workflows/ci.yml — job headless-check (Linux)
headless-check:
  runs-on: ubuntu-latest
  env: { LUMEN_HEADLESS: 1, CI: 1 }
  steps:
    - run: cargo test --workspace
    - run: cargo run --bin lumen -- check examples
    - run: cargo test --test production -- --nocapture
    - run: cargo bench -p lumen-bench -- --quick
```
```powershell
# Repro local (PowerShell)
$env:LUMEN_HEADLESS="1"; $env:CI="1"; cargo test --workspace; cargo bench -p lumen-bench -- --quick; .\target\debug\lumen.exe run examples\graficos_canvas_demo.nv
# Esperado: Headless/CI detectado — demo omitida sin Variable 'a'
```

### 17.5.4 Checklist Deploy
Ver [docs/produccion.md](produccion.md) §5 — `cargo fmt`, `clippy`, `cargo test`, `cargo bench`, `lumen check`, `LUMEN_HEADLESS=1` headless, `CHUNK_VERSION 7`, `VERSION` 3.1.4. Con todo en verde: `cargo build --release --target <target>` deployable en Windows/Linux/macOS/Android/WASM.

---

# CAPÍTULO 18: Concurrencia Masiva M:N, Autograd & Alto Rendimiento SIMD (v2.4.6)

En este capítulo exploramos las tecnologías más avanzadas introducidas en LÚMEN v2.4.6:

## 18.1 Álgebra Lineal 2D & Tiled GEMM (`matriz_simd.nv`)
Multiplicación matricial $N 	imes N$ de alto rendimiento optimizada para la jerarquía de memoria caché L1/L2 con paralelismo vectorial 4-way / 8-way FMA (*Fused Multiply-Add*):

```lumen
importar "matriz_simd.nv";

matriz_simd_Matriz2D A = matriz_simd_matriz_crear(4, 4, 1.0);
matriz_simd_Matriz2D B = matriz_simd_matriz_identidad(4);
matriz_simd_Matriz2D C = matriz_simd_matriz_multiplicar_simd(A, B, 4);
matriz_simd_Matriz2D C_relu = matriz_simd_matriz_relu(C);
```

## 18.2 Diferenciación Automática & Optimizadores IA (`autograd.nv`)
Entrenamiento de redes neuronales con propagación automática de gradientes y optimizador AdamW:

```lumen
importar "autograd.nv";

autograd_TensorAutograd pesos = autograd_crear([0.5, -0.2, 0.8, 0.1], verdadero);
autograd_TensorAutograd entradas = autograd_crear([1.0, 2.0, 3.0, 4.0], falso);
autograd_TensorAutograd objetivos = autograd_crear([2.0, 4.0, 6.0, 8.0], falso);

autograd_OptimizadorAdamW opt = autograd_crear_adamw(0.05, 0.01);

autograd_TensorAutograd pred = autograd_multiplicar(pesos, entradas);
autograd_TensorAutograd perdida = autograd_mse_loss(pred, objetivos);

pesos = autograd_cero_grad(pesos);
pesos = autograd_backward(perdida, pesos);
pesos = autograd_paso_adamw(opt, pesos);
```

## 18.3 Scheduler de Concurrencia M:N & Work-Stealing (`scheduler.nv`)
Orquestación de micro-tareas concurrentes (*Green Threads*) con balanceo de carga automático y canales MPSC:

```lumen
importar "scheduler.nv";

scheduler_SchedulerPool pool = scheduler_crear_pool(8);
pool = scheduler_spawn(pool, "Calculo_Fisica", "data_1");
pool = scheduler_ejecutar_todos(pool);
```

## 18.4 Unikernel Bare-Metal x86_64 (`baremetal.nv`)
Arranque directo sobre el procesador y memoria física en <2 ms:

```lumen
importar "baremetal.nv";

baremetal_UnikernelConfig os = baremetal_arrancar_unikernel("LUMEN-OS");
imprimir(baremetal_resumen(os));
```

---

# CAPÍTULO 19: Inferencia Local de LLMs Cuantizados con GGUF v3 (`stdlib/gguf.nv`)

Ejecuta modelos de lenguaje de gran escala (como **LLaMA-3, Phi-3, Mistral, Gemma**) 100% offline sin depender de Python ni librerías externas:

```lumen
importar "gguf.nv";

// 1. Cargar archivo de pesos GGUF v3 (Q4_K_M)
gguf_GgufModelo modelo = gguf_cargar_modelo("modelos/llama3-8b.Q4_K_M.gguf", 0.7, 0.9);

// 2. Iniciar sesión interactiva con KV-Cache
gguf_GgufSesionChat sesion = gguf_crear_sesion(modelo, "Eres un asistente experto en LÚMEN.");

// 3. Generar respuesta con muestreo Top-P
texto respuesta = gguf_generar_respuesta(sesion, "¿Cómo calculo el factorial en LÚMEN?");
imprimir(respuesta);
```

---

# CAPÍTULO 20: Nexus Cloud Mesh & Microservicios RPC sobre HTTP/3 (`stdlib/nexus.nv`)

Construye mallas de microservicios distribuidos con comunicación RPC binaria tipada y protocolos QUIC / UDP:

```lumen
importar "nexus.nv";

// Iniciar nodo de la malla cloud
nexus_NexusNodoMesh nodo = nexus_iniciar_malla("nodo_us_east", "10.0.1.5", 9000, "HTTP3_QUIC");
nodo = nexus_registrar_servicio_mesh(nodo, "ServicioUsuariosRPC");

// Llamada a procedimiento remoto (Zero-Copy)
texto respuesta = nexus_invocar_rpc_mesh(nodo, "ServicioUsuariosRPC", "obtener_perfil", "{\"id\": 1}");
imprimir(respuesta);
```

---

# CAPÍTULO 21: GUI Nativa de Escritorio Direct2D / Win32 (`stdlib/ui_reactiva.nv`)

Crea aplicaciones de escritorio con aceleración gráfica por hardware a 144 FPS y un consumo de memoria inferior a 1.5 MB de RAM (sin la sobrecarga de Electron):

```lumen
importar "ui_reactiva.nv";

// 1. Hook de estado reactivo
ui_reactiva_EstadoReactivo contador = ui_reactiva_ui_estado_crear("0");

// 2. Ventana acelerada por Direct2D
ui_reactiva_VentanaNativaDirect2D win = ui_reactiva_ui_crear_ventana_direct2d("LÚMEN Studio Desktop", 1024, 768);
win = ui_reactiva_ui_agregar_componente_direct2d(win, ui_reactiva_ui_texto("Panel de Control"));
win = ui_reactiva_ui_agregar_componente_direct2d(win, ui_reactiva_ui_boton("Sumar (+1)", "btn_sumar"));

// 3. Reconciliación Virtual DOM
contador = ui_reactiva_ui_estado_actualizar(contador, "1");
imprimir("Valor del contador: ", contador.valor);
```

---

# CAPÍTULO 22: Audio Espacial 3D y Procesamiento Digital de Señales (`stdlib/audio_dsp.nv`)

Genera ondas en tiempo real, aplica filtros digitales y posiciona sonidos en el espacio tridimensional binaural:

```lumen
importar "audio_dsp.nv";

audio_dsp_BufferAudioMono onda = audio_dsp_oscilador_seno(440.0, 1.0, 44100);
audio_dsp_BufferAudioMono filtrado = audio_dsp_filtro_lowpass(onda, 0.25);
audio_dsp_BufferAudioEstereo 3d = audio_dsp_posicionar_3d(filtrado, 0.0, 0.0, 0.0, 8.0, 0.0, 6.0);

imprimir("Atenuación calculada: ", 3d.volumen_atenuado);
imprimir("Paneo estéreo (L/R) : ", 3d.balance_paneo);
```

---

*LÚMEN v3.5.7 — © 2026 LÚMEN Core Team & Comunidad.*

