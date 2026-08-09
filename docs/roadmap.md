# 🛣️ Roadmap Oficial de LÚMEN (v1.0.0 → v3.0.0)

> **Visión:** El mejor lenguaje de programación educativo bilingüe — rápido, seguro, expresivo, con la mejor DX del mercado.

---

## 📊 Estado de Progreso General

```
Lenguaje Core       [████████████████████████████████████████████] 100% (Fases 0-60)
Lenguaje Avanzado   [████████████████████████████████████████████] 100% (Fases 61-70)
Herramientas & DX   [████████████████████████████████████████████] 100% (Fases 71-95)
Stdlib Extendida    [████████████████████████████████████████████] 100% (Fases 96-110)
Runtime & Sistema   [████████████████████████████████████████████] 100% (Fases 111-130)
Concurrencia & Async[████████████████████████████████████████████] 100% (Fases 131-150)
GUI, TUI & Juegos   [████████████████████████████████████████████] 100% (Fases 151-170)
Portabilidad        [████████████████████████████████████████████] 100% (Fases 171-185)
AI/ML & Data        [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]   0% (Fases 186-200)
Producción & Cloud  [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]   0% (Fases 201-220)
```

---

## ✅ 1. Lenguaje Core (Fases 0-60) — 100%

### 🏗️ 1.1 Cimientos e Infraestructura (Fases 0-20)

| Fase | Nombre | Descripción y Logros Clave |
| :---: | :--- | :--- |
| **0-15** | **Infraestructura Base** | Lexer, parser, sema, IR, bytecode, VM, CLI, arrays, control de flujo avanzado. |
| **16** | **Funciones Avanzadas** | Parámetros default (`funcion foo(a, b = 10)`), Lambdas IIFE, Closures asignables. |
| **17** | **Estructuras/Objetos** | Declaración (`estructura`), inicialización, acceso y asignación de campos. |
| **18** | **Sistema de Módulos** | Directiva `importar`, ModuleLoader, resolución de rutas y prefijado de nombres. |
| **19** | **Optimizaciones** | Constant folding, Dead Code Elimination, Shared Constant Pools, Function Index Cache. |
| **20** | **Release v1.0.0** | SemVer, especificaciones, documentación oficial y README completo. |

### 🛠️ 1.2 Features del Lenguaje (Fases 21-35)

| Fase | Nombre | Descripción y Logros Clave |
| :---: | :--- | :--- |
| **21** | **Bucle For-Each** | Sintaxis `para x en lista`. Desugaring directo en IR a bucles `mientras`. |
| **22** | **Resultado<T,E>** | Tipo algebraico con `exito(valor)` y `error(mensaje)`. Sugar `intentar`. |
| **23** | **Opcion<T>** | Tipos opcionales con `algun(valor)` y `ninguno`. |
| **24** | **Enums/Tipos Suma** | Definición de `enum` bilingüe con soporte para acceso mediante `::`. |
| **25** | **Tuplas** | Tipo compuesto anónimo `(tipo, tipo)` y acceso estático `.0`, `.1`. |
| **26** | **Destructuring** | Desestructuración en declaraciones y asignaciones. Comodín `_`. |
| **27** | **Genéricos Básicos** | Parametrización `<T>` en funciones y structs con *type erasure*. |
| **28-29** | **Stdlib Inicial** | Módulos `matematicas`, `texto`, `coleccion`, y `fecha`. |
| **30** | **E/S de Archivos** | Builtins `leer_archivo`, `escribir_archivo`, `existe_archivo`. |
| **31** | **Stack Traces** | `CallFrame` en VM y visualización en tiempo real. |
| **32** | **Modo Dual Inglés** | Pre-scan para keywords en inglés con `importar ingles;`. |
| **33** | **Errores Vistosos** | Subrayado exacto con caret (`^^^^`) y colores ANSI. |
| **34** | **Fuzzing Integrado** | 3 targets de `cargo-fuzz` para lexer, parser y decoder. |
| **35** | **Calidad de Vida** | Operador `%`, `sino si`, `y`/`o` lógicos, paréntesis opcionales. |

### 🧪 1.3 Herramientas Base (Fases 36-41)

| Fase | Nombre | Descripción y Logros Clave |
| :---: | :--- | :--- |
| **36** | **Property Testing** | Round-trips con `proptest` para opcodes del codegen y tokens del lexer. |
| **37** | **lumen fmt** | Formateador automático inteligente (`crates/lumen-fmt`). |
| **38** | **lumen repl** | REPL interactivo con persistencia de variables (`crates/lumen-repl`). |
| **39** | **lumen test** | Framework de pruebas con bloques `test` y sentencia `afirmar`. |
| **40** | **Manifiesto Proyecto** | `lumen new` y gestión con `lumen.toml`. |
| **41** | **CI/CD & Releases** | Criterion benchmarks, CI/CD en push/PR y GitHub releases. |

### 🔬 1.4 Sintaxis Moderna (Fases 42-60)

| Fase | Concepto | Detalle de Implementación |
| :---: | :--- | :--- |
| **42** | **Inferencia de Tipos** | El compilador deduce el tipo (`x = 42` → entero). |
| **43** | **Métodos en Structs** | Bloques `impl Struct` que enlazan funciones a tipos de datos. |
| **44** | **Diccionarios** | Tipos llave-valor nativos (`diccionario<K,V>`). |
| **45** | **String Interpolation** | Desugaring de `"hola {nombre}"` a concatenación. |
| **46** | **Rangos Nativos** | Operadores `..` y `..=` en iteradores. |
| **47** | **Constantes** | Keyword `const` para expresiones en tiempo de compilación. |
| **48** | **String Indexing** | Acceso por índice `s[i]` a caracteres de texto. |
| **49** | **Conversiones** | `a_texto`, `a_entero`, `a_decimal` como métodos nativos. |
| **50** | **División Entera** | `entero / entero → entero` con truncación automática. |
| **51** | **Concatenación Mixta** | `"x" + 42` — coerción automática a texto. |
| **52** | **Errores Multi-línea** | Preview enriquecido con múltiples líneas subrayadas. |
| **53** | **Operador Ternario** | Expresión condicional compacta (`cond ? a : b`). |
| **54** | **Etiquetas de Loops** | `romper etiqueta` y `continuar etiqueta` en loops anidados. |
| **55** | **Pattern Matching Pro** | Exhaustividad, guardas, OR patterns, rangos y strings. |
| **56** | **Genéricos con Bounds** | Restricciones en firmas genéricas (`<T: Numerico>`). |
| **57** | **Matrices 2D** | `lista<lista<T>>` + stdlib `matrices.nv` con operaciones básicas. |
| **58** | **Enums Avanzados** | Variantes con datos `Variant(entero)` — pattern matching sobre payload. |
| **59** | **Closures Pro** | Captura por valor y referencia. Closures movibles y reutilizables. |
| **60** | **Async/Await** | Sintaxis `async funcion` / `esperar`. Sema + IR bases. |

---

## ✅ 2. Lenguaje Avanzado (Fases 61-70) — 100%

| # | Feature | Sintaxis | Estado |
|---|---------|----------|--------|
| 61 | **OR Patterns** | `caso Rojo \| Verde:` | ✅ |
| 62 | **If-let / While-let** | `si sea Algun(x) = opt { }` | ✅ |
| 63 | **Range Patterns** | `caso 0..10:` — comparación de rango | ✅ |
| 64 | **String Patterns** | `caso "hola":` — igualdad de string | ✅ |
| 65 | **Guard Let** | `sea x = expr sino { romper }` | ✅ |
| 66 | **Operator Overloading** | `impl Suma para MiTipo` | ✅ |
| 67 | **Extension Methods** | `impl MiRasgo para TipoExterno` | ✅ |
| 68 | **Associated Types** | `tipo Item;` en traits | ✅ |
| 69 | **Where Clauses** | `<T> donde T: Comparable` | ⏭️ Saltado |
| 70 | **Impl Trait return** | `-> impl Mostrable` | ✅ |

---

## ✅ 3. Herramientas & DX (Fases 71-95) — 100%

| # | Herramienta | Descripción | Estado |
|---|-------------|-------------|--------|
| 71 | **LSP Server** | Diagnósticos en vivo, parseo y sema al vuelo en editores | ✅ |
| 72 | **LSP: Completion** | Autocompletado de palabras clave y símbolos | ✅ |
| 73 | **LSP: Go-to-def** | Navegación a definiciones de símbolos | ✅ |
| 74 | **LSP: Hover** | Información de tipos y documentación al pasar el mouse | ✅ |
| 75 | **lumen doc** | Generación de HTML estático desde comentarios `///` | ✅ |
| 76 | **Debugger** | Breakpoints, step/continue, inspección de variables en runtime | ✅ |
| 77 | **lumen fmt avanzado** | `.lumen-fmt.toml` para indentación y reglas de formato | ✅ |
| 78 | **lumen lint** | Análisis de código muerto y complejidad ciclomática | ✅ |
| 79 | **REPL Pro** | Historial persistente, multilínea, resaltado, autocompletado | ✅ |
| 80 | **Package Manager** | `lumen install`, registry central, lock file, dependencias | ✅ |
| 81 | **Build Incremental** | Caché de compilación incremental para módulos sin cambios | ✅ |
| 82 | **Hot Reload** | Recarga automática de módulos en desarrollo con `lumen serve` | ✅ |
| 83 | **Playground Web** | Editor online con ejecución en navegador vía WASM | ✅ |
| 84 | **Benchmarks** | Suite criterion para pipeline completo (bench, parse, pipeline, VM) | ✅ |
| 85 | **Plugins API** | Sistema de plugins para fases del compilador (hooks pre/post) | ✅ |
| 86 | **AOT: C transpiler** | Transpilación a C + compilación con gcc/clang -O3 | ✅ |
| 87 | **AOT: Cranelift** | Backend nativo directo vía Cranelift JIT/AOT | ✅ |
| 88 | **AOT: LTO + optimización** | LTO, dead code stripping, inlining agresivo (`opt_level=speed_and_size`) | ✅ |
| 89 | **Cross-compilation** | Compilar para Linux/macOS/Windows/ARM desde cualquier SO | ✅ |
| 94 | **Single binary** | `lumen` como binario único (run, build, check, fmt, repl, doc, lsp, install) | ✅ |
| 95 | **Installer** | Script de instalación unificado (`curl \| sh` para Unix, `irm \| pwsh` para Windows) | ✅ |

**Nota:** Fases 90-93 (WASM, WASI, JS interop, Self-hosting) movidas a bloque de portabilidad.

---

## 📚 4. Librería Estándar Extendida (Fases 96-110)

Colecciones avanzadas, texto, I/O y redes. Todo implementado vía builtins de VM + stdlib `.nv`.

| # | Módulo | Feature | Descripción | Estado |
|---|--------|---------|-------------|--------|
| 96 | `coleccion` | **HashMap<K,V>** | Mapa hash con `__map_new`, `__map_get`, `__map_set`, `__map_len`, `__map_keys`, `__map_contains` | ✅ |
| 97 | `coleccion` | **HashSet<T>** | Conjunto sobre HashMap con `__set_new`, `__set_add`, `__set_has`, `__set_union`, `__set_inter`, `__set_diff` | ✅ |
| 98 | `coleccion` | **VecDeque<T>** | Cola doble con `__deque_new`, `__deque_push_front`, `__deque_push_back`, `__deque_pop_front`, `__deque_pop_back` | ✅ |
| 99 | `coleccion` | **BinaryHeap<T>** | Max-heap con `__heap_new`, `__heap_push`, `__heap_pop`, `__heap_peek`, `__heap_len` | ✅ |
| 100 | `coleccion` | **LinkedList<T>** | Lista doblemente enlazada con `__linked_new`, `__linked_push_front`, `__linked_push_back`, `__linked_pop_front`, `__linked_pop_back` | ✅ |
| 101 | `texto` | **Regex** | Expresiones regulares: `Regex::nuevo`, `es_coincide`, `capturar`, `reemplazar`, `dividir_regex` | ✅ |
| 102 | `texto` | **Unicode** | Normalización (NFC/NFD/NFKC/NFKD), categorías Unicode, case folding | ✅ |
| 103 | `texto` | **Format** | Formateo avanzado: padding, alineación, precisión decimal, notación científica | ✅ |
| 104 | `texto` | **Encoding** | Codificación UTF-8/16/32, Latin-1, detección automática de encoding | ✅ |
| 105 | `io` | **Buffered** | Lector/escritor con buffer, `LineReader`, lectura línea por línea | ✅ |
| 106 | `io` | **Streaming** | Streaming de archivos grandes con chunk reading, progreso, cancelación | ✅ |
| 107 | `io` | **SerialPort** | Comunicación por puerto serie (RS-232): baud rate, parity, stop bits | ✅ |
| 108 | `red` | **TCP** | Sockets TCP: `TcpListener`, `TcpStream`, `conectar`, `escuchar`, `aceptar` | ✅ |
| 109 | `red` | **HTTP** | Cliente HTTP/1.1: GET, POST, headers, status codes, body streaming | ✅ |
| 110 | `red` | **HTTP Servidor** | Servidor HTTP básico con routing, middleware, JSON responses | ✅ |

---

## ⚙️ 5. Runtime & Sistema (Fases 111-130)

| # | Módulo | Feature | Descripción | Estado |
|---|--------|---------|-------------|--------|
| 111 | `json` | **Parser** | Parseo de JSON desde texto a tipos nativos LÚMEN | ✅ vía serde_json |
| 112 | `json` | **Serializer** | Serialización de tipos LÚMEN a JSON con indentación | ✅ vía serde_json |
| 113 | `csv` | **Reader** | Lector CSV con delimitador, quoting, headers | ✅ vía __ffi_* |
| 114 | `csv` | **Writer** | Escritor CSV con configuración de formato | ✅ vía __ffi_* |
| 115 | `sqlite` | **Driver** | Binding nativo a SQLite: `abrir`, `ejecutar`, `consultar`, `transaccion` | ✅ vía __ffi_* a sqlite3.dll |
| 116 | `sqlite` | **ORM** | Mapeo objeto-relacional mínimo | 📋 |
| 117 | `sistema` | **Procesos** | Lanzar procesos hijo | ✅ vía __ffi_* |
| 118 | `sistema` | **Env** | Variables de entorno | ✅ vía __env_listar + __ffi_* |
| 119 | `sistema` | **Path** | Manipulación de rutas | ✅ vía __ffi_* |
| 120 | `sistema` | **Temp** | Archivos temporales | ✅ vía __ffi_* |
| 121 | `fecha` | **Timezone** | Zonas horarias IANA | ✅ vía __timezone_info |
| 122 | `fecha` | **Duracion** | Duración precisa | ✅ vía __duration_new/secs |
| 123 | `fecha` | **Format** | Formateo/parseo de fechas con patrones | ✅ vía __tiempo_formatear/parsear |
| 124 | `fecha` | **Calendario** | Calendarios no gregorianos | ✅ vía __calendar_hijri/persian |
| 125 | `crypto` | **Hash** | SHA-256, SHA-512, HMAC | ✅ vía BCrypt CNG |
| 126 | `crypto` | **AES** | Cifrado simétrico AES-128/256 | ✅ vía BCrypt AES-GCM |
| 127 | `crypto` | **JWT** | Creación y verificación de JWT | ✅ vía sha256 + base64url |
| 128 | `testing` | **Assert** | Macros de aserción | ✅ vía testing.nv (pure LÚMEN) |
| 129 | `testing` | **Mock** | Sistema de mocks | ✅ vía testing.nv (pure LÚMEN) |
| 130 | `testing` | **Coverage** | Cobertura de código | ✅ vía testing.nv (pure LÚMEN) |

---

## ⚡ 6. Concurrencia & Async (Fases 131-150)

| # | Feature | Descripción | Estado |
|---|---------|-------------|--------|
| 131 | **Thread::spawn** | `hilo::lanzar(|| expr)`, `hilo::dormir(ms)` | ✅ builtins + stdlib concurrencia.nv |
| 132 | **Thread::Join** | `hijo.esperar()` con resultado | ✅ builtins |
| 133 | **Sync::Mutex** | `Mutex<T>`, `bloquear`, `try_bloquear` | ✅ builtins + stdlib |
| 134 | **Sync::RwLock** | Múltiples lectores, un escritor | ✅ builtins + stdlib |
| 135 | **Sync::Arc** | Conteo de referencias atómico | ✅ builtins + stdlib |
| 136 | **Sync::Channel** | `canal::nuevo()`, `enviar`, `recibir` | ✅ builtins + stdlib |
| 137 | **Async::Runtime** | Task spawn/await via thread pool | ✅ vía __tarea_lanzar/esperar |
| 138 | **Async::Stream** | `Stream<T>`, `map`, `filter`, `colectar` | ✅ builtins + stdlib |
| 139 | **Async::File** | `leer_async`, `escribir_async` | ✅ vía __leer/escribir_archivo_async |
| 140 | **Async::TCP** | `TcpListener::aceptar_async` | ✅ vía __tcp_connect_async |
| 141 | **Async::Timer** | `Timer::despues(ms)`, `Timer::intervalo(ms)` | ✅ vía __timer_delay |
| 142 | **Async::Select** | `seleccionar!(fut1, fut2, fut3)` | ✅ builtins + stdlib |
| 143 | **Par::Iterator** | `par_iter`, `map_par`, `filter_par` | ✅ builtins + stdlib |
| 144 | **Par::Join** | `par::unir(f1(), f2())` | ✅ builtins + stdlib |
| 145 | **Act::Actor** | `Actor::nuevo`, `enviar`, `manejar_mensaje` | ✅ builtins + stdlib |
| 146 | **Act::Supervisor** | Reinicio automático en fallo | ✅ builtins + stdlib |
| 147 | **Act::Cluster** | Actores remotos vía TCP | ✅ builtins + stdlib |
| 148 | **Coro::Generator** | `generador { producir expr }` | ✅ builtins + stdlib |
| 149 | **Coro::AsyncGen** | `async generador { }` | 📋 |
| 150 | **Coro::Structured** | Ámbitos con cancelación | ✅ builtins (scope_handles) |

---

## 🎨 7. GUI, TUI & Juegos (Fases 151-170)

| # | Bloque | Features | Estado |
|---|--------|----------|--------|
| 151 | TUI | Terminal raw, limpiar, cursor | ✅ vía __ffi_* + tui_core.nv |
| 152 | TUI | Ventanas, bordes, título, redimensionar | ✅ vía tui.nv |
| 153 | TUI | Entrada de texto, teclas navegación, clipboard | ✅ vía tui.nv |
| 154 | TUI | Tablas con sort, scroll, columnas redimensionables | ✅ vía tui.nv |
| 155 | TUI | Menús desplegables, contexto, barras de herramientas | ✅ vía tui.nv |
| 156 | TUI | Layout engine: Flex, Grid, Stack, Padding, Alignment | ✅ vía tui.nv |
| 157 | GFX | Canvas 2D: píxeles, líneas, rectángulos, círculos, texto | ✅ vía graficos.nv (SDL2 FFI) |
| 158 | GFX | Sprites animados, hojas de sprites | ✅ vía graficos_avanzado.nv |
| 159 | GFX | Game loop: fixed timestep, delta time | ✅ vía graficos.nv |
| 160 | GFX | Input: teclado, mouse, gamepad, eventos | ✅ vía graficos.nv |
| 161 | GFX | Audio WAV/OGG: cargar, reproducir, pausar | ✅ vía graficos_avanzado.nv (SDL2_mixer) |
| 162 | GFX | Sistema de partículas | ✅ vía graficos_avanzado.nv (LÚMEN puro) |
| 163 | GFX | Tilemap: mapas de tiles 2D | ✅ vía graficos_tilemap.nv (cámara, colisiones) |
| 164 | GUI | Widgets: Botón, Etiqueta, CampoTexto | ✅ vía gui.nv (Win32 FFI) |
| 165 | GUI | Ventanas nativas | ✅ vía __gui_ventana/window |
| 166 | GUI | Canvas 2D en ventana | ✅ vía graficos_canvas.nv (círculos, líneas, triángulos, gradientes, texto) |
| 167 | GUI | Eventos: click, teclado, focus, drag | ✅ vía gui.nv |
| 168 | GUI | Temas y estilos CSS-like | ✅ vía tui_temas.nv (Catppuccin, claro, oscuro, alto contraste) |
| 169 | GUI | TreeView, drag-drop, multi-selección | 📋 |
| 170 | GUI | Charts: barras, líneas, pastel, dispersión | ✅ vía graficos_charts.nv |

---

## 📦 8. Portabilidad & Cross-Compilation (Fases 171-185)

| # | Feature | Descripción | Estado |
|---|---------|-------------|--------|
| 171 | **WASM backend** | Compilar a WebAssembly para ejecución en navegadores | ✅ v1.7.0 — VM refactorizada, crate lumen-wasm |
| 172 | **WASM: WASI** | Ejecutar en servidores/serverless vía WASI | ✅ build + CI + test |
| 173 | **WASM: JS interop** | Llamar funciones JS desde LÚMEN y viceversa | ✅ 17 bridge functions |
| 174 | **Self-hosting** | El compilador de LÚMEN escrito en LÚMEN | ✅ Pipeline LÚMEN→.nvc→ejecuta; **Sprint 5-8 (31 Jul-8 Ago): fixpoint puro confirmado** — `compiler_v4_self.nvc` byte-IDÉNTICO (SHA-256 3DA624D6…), 5s; **VM en LÚMEN (`vm.nv`) + fixpoint 861s→20.1s (43x COW Arc)**; **Stream/Async/Par/Actor/Generator delegados a natives**; **fuego: 112/117 ejemplos CORRECTOS**; **Bootstrapping doble (compilador + VM) CONFIRMADO** — SHA-256 `3DA624D6...` (150,684 B byte-idénticos) |
| 175 | **Docker Image** | `lumen:latest`, multi-stage, slim | ✅ Dockerfile |
| 176 | **Docker Compose** | Servicios lumen + lumen-repl | ✅ docker-compose.yml |
| 177 | **GitHub Action** | CI build/test/clippy/fmt/coverage | ✅ .github/workflows/ |
| 178 | **Testing::Bench** | Suite criterion en lumen-bench | ✅ |
| 179 | **Testing::Fuzz** | Fuzzing integrado para funciones | ✅ CI job con cargo-fuzz |
| 180 | **Testing::Mutation** | Mutar código y verificar tests | ✅ |
| 181 | **Obs::Log** | Logging: niveles, archivos rotativos | ✅ logging.nv + rotación + buffer |
| 182 | **Obs::Tracing** | Trazado distribuido | ✅ |
| 183 | **Obs::Metrics** | Contadores, histogramas | ✅ stdlib/metrics.nv |
| 184 | **Obs::Profiler** | CPU/memoria | ✅ |
| 185 | **Compiler API** | Usar LÚMEN como biblioteca | ✅ crate lumen-api |

---

## 🤖 9. AI/ML & Data Science (Fases 186-200)

| # | Bloque | Features | Estado |
|---|--------|----------|--------|
| 186 | Tensor | Tensor N-dimensional CPU/GPU, forma, reshape | 📋 |
| 187 | Tensor | Operaciones: suma, multiplicación, convolución, pooling | 📋 |
| 188 | Tensor | Diferenciación automática, backpropagation | 📋 |
| 189 | NN | Dense, Conv2D, RNN/LSTM/GRU | 📋 |
| 190 | NN | Optimizadores: SGD, Adam, AdamW, RMSprop | 📋 |
| 191 | NN | Trainer: batching, epochs, validación, early stopping | 📋 |
| 192 | ML | DataFrame: columnas tipadas, groupby, join, pivot | 📋 |
| 193 | ML | Carga/guarda de CSV, Parquet, Arrow | 📋 |
| 194 | ML | Preprocesamiento: normalize, standardize, one-hot | 📋 |
| 195 | ML | Regresión lineal, logística, polinomial | 📋 |
| 196 | ML | Clustering: K-Means, DBSCAN, hierarchical | 📋 |
| 197 | ML | PCA, t-SNE, reducción de dimensionalidad | 📋 |
| 198 | ML | Random Forest, Gradient Boosting | 📋 |
| 199 | ML | Métricas: accuracy, precision, recall, F1, MSE, R² | 📋 |
| 200 | ML | Serving de modelos: API REST, versionado | 📋 |

---

## ☁️ 10. Producción & Cloud (Fases 201-220)

| # | Bloque | Features | Estado |
|---|--------|----------|--------|
| 201 | Cloud | AWS S3: subir, descargar, listar, presigned_url | 📋 |
| 202 | Cloud | AWS DynamoDB: consultar, insertar, actualizar | 📋 |
| 203 | Cloud | AWS Lambda: empaquetado automático, deploy | 📋 |
| 204 | Cloud | GCP Storage: buckets, objetos, ACLs | 📋 |
| 205 | Cloud | Azure Blob: contenedores, blobs, leases | 📋 |
| 206 | Cloud | K8s Operator: deploy, scale, health checks | 📋 |
| 207 | Cloud | CI/CD Templates: GitHub Actions, GitLab CI | 📋 |
| 208 | Sec | OpenSSL/LibreSSL: cifrado asimétrico, X.509 | 📋 |
| 209 | Sec | OAuth 2.0: authorization code, refresh token | 📋 |
| 210 | Sec | JWT completo: emisión, verificación, claims | 📋 |
| 211 | Sec | Gestión de secretos: cifrado en reposo, rotación | 📋 |
| 212 | Docs | Sitio web `lumen-lang.org` con docs y playground | 📋 |
| 213 | Docs | Tutorial interactivo paso a paso | 📋 |
| 214 | Docs | Libro oficial "El Lenguaje LÚMEN" (ES/EN) | 📋 |
| 215 | Com | Foro/Comunidad, discusiones, ejemplos | 📋 |
| 216 | Com | Extensión VS Code: highlighting, snippets, debugging | 📋 |
| 217 | Com | Plugin JetBrains: resaltado, navegación | 📋 |
| 218 | Com | Playground online `play.lumen-lang.org` | 📋 |
| 219 | Com | Registro público de paquetes | 📋 |
| 220 | **v3.0.0 Release** | Documentación, sitio web, comunidad, release final | 📋 |

---

## 🎯 Hitos de Versión

| Versión | Alcance | Fases | Estado |
|---------|---------|-------|--------|
| **v1.0** | Compilador e infraestructura base | 0-20 | ✅ |
| **v1.2** | Features del lenguaje + stdlib inicial | 21-41 | ✅ |
| **v1.5** | Sintaxis moderna + genéricos + traits | 42-60 | ✅ |
| **v1.6** | Lenguaje avanzado + LSP + Herramientas DX completas | 61-95 | ✅ |
| **v1.7** | Stdlib extendida + WASM playground | 96-110 | ✅ |
| **v1.8** | FFI system + Crypto builtins | 111-127 | ✅ |
| **v1.9** | Concurrencia completa + Async runtime | 128-150 | ✅ |
| **v2.0** | GUI, TUI, Juegos funcionales | 151-170 | ✅ |
| **v2.5** | Portabilidad + WASI + Self-hosting | 171-185 | ✅ Pipeline .nvc completado |
| **v3.0** | AI/ML + Cloud + Producción | 186-220 | 📋 |

---

## 🔍 Estado Actual (Julio 2026)

### ✅ Todo esto funciona
- ✅ Lenguaje completo (0-70): variables, funciones, genéricos, enums, structs, traits, pattern matching
- ✅ Herramientas (71-95): LSP, fmt, repl, doc, debug, lint, test, build incremental, hot reload, package manager, AOT, CI/CD
- ✅ Stdlib collections: map, set, deque, heap, linked list
- ✅ Texto: string ops, regex, unicode, encoding, padding, tipo_de/typeof
- ✅ File I/O: leer/escribir/existe, buffer, streaming, listdir
- ✅ JSON: parse/serialize (vía serde_json)
- ✅ TCP: connect/listen/accept
- ✅ HTTP: get/post
- ✅ **FFI**: __ffi_cargar/load, __ffi_llamar/call, __ffi_asignar/alloc, __ffi_liberar/free, __ffi_escribir/write, __ffi_leer/read, __ffi_peek/poke
- ✅ **Crypto**: SHA-256, SHA-512, JWT encode/decode
- ✅ **Concurrencia**: hilos, mutex, canales, rwlock, arc, actores, supervisores, cluster, scope, streams, generadores, par, dormir, seleccionar
- ✅ **GUI**: ventanas nativas Win32, botones, inputs, checkboxes, etc.
- ✅ **TUI**: 24 componentes (ventanas, tablas, menús, editor, calendar, etc.)
- ✅ **GFX**: SDL2 canvas, rectángulos, texturas, input teclado
- ✅ **Corrutinas**: crear, ceder, reanudar
- ✅ **Fecha**: ahora, formatear ISO 8601, parsear, diferencia
- ✅ **Testing**: afirmar_verdadero/igual/distinto, mocks, coverage, mutation testing (pure LÚMEN)
- ✅ **Math**: abs, max, min, pow, sqrt, sin, cos (pure LÚMEN)
- ✅ **WASM**: compila a wasm32-unknown-unknown

### 📋 Lo que falta
- **Self-hosting completo (Fase 174)**: Pipeline LÚMEN→LÚMEN→.nvc→ejecuta ✅ (Sprint 2). Bootstrap ✅ (Sprint 3: `__compile_nv`, 533ms). HashMap O(1) ✅ (Sprint 4). **Self-hosting puro ✅ (Sprint 5, 31 Jul: fixpoint 54,712 B).** **Sprint 6 ✅ gramática completa (enum/elegir/sea/traits/closures/params-default).** **Sprint 7 ✅ VM en LÚMEN (`vm.nv`) + fixpoint 861s→20.1s (43x, COW Arc).** **Sprint 8 ✅ dogfooding: fuego 117/117 compilan · 112 CORRECTOS (8 Ago).** **Bootstrapping doble CONFIRMADO ✅ (8 Ago: SHA-256 3DA624D6..., 150,684 B byte-idénticos).** Release v2.4.1 pendiente.
- **SQLite ORM (Fase 116)**: mapeo objeto-relacional mínimo sobre SQLite.
- **AsyncGen (Fase 149)**: `async generador { }` — generadores asíncronos.
- **TreeView (Fase 169)**: widget TreeView con drag-drop y multi-selección.
- **AI/ML (Fases 186-200)**: tensores, redes neuronales, data science.
- **Producción & Cloud (Fases 201-220)**: AWS, GCP, Azure, K8s, docs, comunidad, extensiones VS Code/JetBrains.

---

> **LÚMEN** es el lenguaje de programación educativo bilingüe más completo en español/inglés.
> Diseñado para enseñar, prototipar y construir software real con una DX excepcional.
