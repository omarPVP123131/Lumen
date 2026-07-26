# 🛣️ Roadmap Oficial de LÚMEN (v1.0.0 → v3.0.0)

> **Visión:** El mejor lenguaje de programación educativo bilingüe — rápido, seguro, expresivo, con la mejor DX del mercado.

---

## 📊 Estado de Progreso General

```
Lenguaje Core      [████████████████████████████████████████████] 100% (Fases 0-60)
Lenguaje Avanzado  [████████████████████████████████████████████] 100% (Fases 61-70)
Herramientas       [██████████████████████████████████████████░░]  95% (Fases 71-85)
Distribución       [████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░]  40% (Fases 86-95)
Stdlib & Runtime   [██████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]  15% (Fases 96-140)
Concurrencia       [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]   0% (Fases 141-160)
GUI & Gráficos     [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]   0% (Fases 161-175)
AI/ML & Data       [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]   0% (Fases 176-195)
Producción & Cloud [░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░]   0% (Fases 196-230)
```

---

## ✅ 1. Lenguaje Core (Fases 0-60) — 100% completado

### 🏗️ 1.1 Cimientos e Infraestructura (Fases 0-20)

| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **0-15** | **Infraestructura Base** | Lexer, parser, sema, IR, bytecode, VM, CLI, arrays, control de flujo avanzado. | Construcción del compilador y VM stack-based básico. |
| **16** | **Funciones Avanzadas** | Parámetros default (`funcion foo(a, b = 10)`), Lambdas IIFE, Closures asignables. | Funciones de primera clase y clausuras completas en VM. |
| **17** | **Estructuras/Objetos** | Declaración (`estructura`), inicialización, acceso y asignación de campos. | Soporte inicial para tipos de datos compuestos y POO básica. |
| **18** | **Sistema de Módulos** | Directiva `importar`, ModuleLoader, resolución de rutas y prefijado de nombres. | Evita colisiones de nombres al importar bibliotecas. |
| **19** | **Optimizaciones** | Constant folding, Dead Code Elimination, Shared Constant Pools, Function Index Cache. | Reducción de tamaño del bytecode y aceleración de ejecución. |
| **20** | **Release v1.0.0** | SemVer, especificaciones, documentación oficial y README completo. | Hito de estabilidad del primer compilador de producción. |

### 🛠️ 1.2 Features del Lenguaje (Fases 21-35)

| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **21** | **Bucle For-Each** | Sintaxis `para x en lista`. Desugaring directo en IR a bucles `mientras`. | Sintaxis de iteración limpia y segura sobre colecciones. |
| **22** | **Resultado<T,E>** | Tipo algebraico con `exito(valor)` y `error(mensaje)`. Sugar `intentar`. | Manejo de errores idiomático y seguro sin panics arbitrarios. |
| **23** | **Opcion<T>** | Tipos opcionales con `algun(valor)` y `ninguno`. | Evita errores de puntero nulo (*null safety*). |
| **24** | **Enums/Tipos Suma** | Definición de `enum` bilingüe con soporte para acceso mediante `::`. | Modelado de dominio complejo con variantes seguras. |
| **25** | **Tuplas** | Tipo compuesto anónimo `(tipo, tipo)` y acceso estático `.0`, `.1`. | Agrupación rápida de tipos sin declarar estructuras. |
| **26** | **Destructuring** | Desestructuración en declaraciones y asignaciones. Comodín `_`. | Extracción rápida de valores de tuplas. |
| **27** | **Genéricos Básicos** | Parametrización `<T>` en funciones y structs con *type erasure*. | Reutilización de código con seguridad de tipos. |
| **28-29** | **Stdlib Inicial** | Módulos `matematicas`, `texto`, `coleccion`, y `fecha`. | Funcionalidades matemáticas básicas y manipulación de texto. |
| **30** | **E/S de Archivos** | Builtins `leer_archivo`, `escribir_archivo`, `existe_archivo`. | Manipulación del sistema de archivos integrado con la VM. |
| **31** | **Stack Traces** | `CallFrame` en VM y visualización en tiempo real. | Depuración rápida en errores en tiempo de ejecución. |
| **32** | **Modo Dual Inglés** | Pre-scan para keywords en inglés con `importar ingles;`. | Keywords bilingües en el mismo motor. |
| **33** | **Errores Vistosos** | Subrayado exacto con caret (`^^^^`) y colores ANSI. | Diagnóstico excelente y sugerencias de ayuda claras. |
| **34** | **Fuzzing Integrado** | 3 targets de `cargo-fuzz` para lexer, parser y decoder. | Inmunidad del compilador ante entradas corruptas. |
| **35** | **Calidad de Vida** | Operador `%`, `sino si`, `y`/`o` lógicos, paréntesis opcionales. | Sintaxis fluida, ergonómica y retrocompatible. |

### 🧪 1.3 Herramientas Base (Fases 36-41)

| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **36** | **Property Testing** | Round-trips con `proptest` para opcodes del codegen y tokens del lexer. | Garantiza invariants del compilador en miles de casos aleatorios. |
| **37** | **lumen fmt** | Formateador automático inteligente (`crates/lumen-fmt`). | Estilo uniforme y automático para cualquier archivo `.nv`. |
| **38** | **lumen repl** | REPL interactivo con persistencia de variables (`crates/lumen-repl`). | Experimentación instantánea de sintaxis en terminal. |
| **39** | **lumen test** | Framework de pruebas con bloques `test` y sentencia `afirmar`. | Ejecución nativa de suites (`lumen test <file>`). |
| **40** | **Manifiesto Proyecto** | `lumen new` y gestión con `lumen.toml`. | Scaffolding automatizado y rutas de librerías centralizadas. |
| **41** | **CI/CD & Releases** | Criterion benchmarks, CI/CD en push/PR y GitHub releases. | Binarios listos para Windows, Linux y macOS. |

### 🔬 1.4 Sintaxis Moderna (Fases 42-60)

| Fase | Concepto | Detalle de Implementación | Beneficio de Experiencia (DX) |
| :---: | :--- | :--- | :--- |
| **42** | **Inferencia de Tipos** | El compilador deduce el tipo (`x = 42` → entero). | Código limpio sin declarar tipos evidentes. |
| **43** | **Métodos en Structs** | Bloques `impl Struct` que enlazan funciones a tipos de datos. | Transición a POO idiomática estructurada. |
| **44** | **Diccionarios** | Tipos llave-valor nativos con opcodes `MapNew`, `MapGet`, `MapSet`. | Estructuras de datos dinámicas indispensables. |
| **45** | **String Interpolation** | Desugaring de `"hola {nombre}"` a concatenación. | Formateo de strings elegante y legible. |
| **46** | **Rangos Nativos** | Operadores `..` y `..=` en iteradores. | Bucles e índices más expresivos. |
| **47** | **Constantes** | Keyword `const` para expresiones en tiempo de compilación. | Inmutabilidad garantizada y optimizaciones avanzadas. |
| **48** | **String Indexing** | Acceso por índice `s[i]` a caracteres de texto. | Manipulación de texto directa sin helpers. |
| **49** | **Conversiones** | `a_texto`, `a_entero`, `a_decimal` como métodos nativos. | Conversión de tipos idiomática y segura. |
| **50** | **División Entera** | `entero / entero → entero` con truncación automática. | Comportamiento matemático correcto por defecto. |
| **51** | **Concatenación Mixta** | `"x" + 42` — coerción automática a texto. | Formateo rápido sin conversión explícita. |
| **52** | **Errores Multi-línea** | Preview enriquecido con múltiples líneas subrayadas. | Diagnósticos de compilación de nivel profesional. |
| **53** | **Operador Ternario** | Expresión condicional compacta (`cond ? a : b`). | Expresividad máxima en condicionales de una línea. |
| **54** | **Etiquetas de Loops** | `romper etiqueta` y `continuar etiqueta` en loops anidados. | Salida ordenada en algoritmos multidimensionales. |
| **55** | **Pattern Matching Pro** | Exhaustividad, guardas, OR patterns, rangos y strings. | Selección de flujo declarativa, compacta y robusta. |
| **56** | **Genéricos con Bounds** | Restricciones en firmas genéricas (`<T: Numerico>`). | Polimorfismo seguro y reutilización genérica. |
| **57** | **Matrices 2D** | `lista<lista<T>>` + stdlib `matrices.nv` con operaciones básicas. | Soporte para algoritmos matriciales y ciencia de datos. |
| **58** | **Enums Avanzados** | Variantes con datos `Variant(entero)` — pattern matching sobre payload. | Modelado de estados complejos con datos adjuntos. |
| **59** | **Closures Pro** | Captura por valor y referencia. Closures movibles y reutilizables. | Programación funcional madura con callbacks. |
| **60** | **Async/Await** | Sintaxis `async funcion` / `esperar`. Sema + IR bases. | Fundamento para I/O no bloqueante y concurrencia futura. |

---

## ✅ 2. Lenguaje Avanzado (Fases 61-70) — 100% completado

| # | Feature | Sintaxis | Estado |
|---|---------|----------|--------|
| 61 | **OR Patterns** | `caso Rojo \| Verde:` | ✅ |
| 62 | **If-let / While-let** | `si sea Algun(x) = opt { }` | ✅ |
| 63 | **Range Patterns** | `caso 0..10:` — comparación de rango | ✅ |
| 64 | **String Patterns** | `caso "hola":` — igualdad de string | ✅ |
| 65 | **Guard Let** | `sea x = expr sino { romper }` | ✅ |
| 66 | **Operator Overloading** | `impl Suma for MiTipo` | ✅ |
| 67 | **Extension Methods** | `impl MiRasgo for TipoExterno` | ✅ |
| 68 | **Associated Types** | `tipo Item;` en traits | ✅ |
| 69 | **Where Clauses** | `<T> donde T: Comparable` | ⏭️ Salta — ya soportado por `<T: Rasgo>` |
| 70 | **Impl Trait return** | `-> impl Mostrable` | ✅ |

---

## 🛠️ 3. Herramientas & DX (Fases 71-85)

| # | Herramienta | Descripción | Estado |
|---|-------------|-------------|--------|
| 71 | **LSP Server** | Diagnósticos en vivo en VS Code | ✅ |
| 72 | **LSP: Completion** | Autocompletado de símbolos | ✅ |
| 73 | **LSP: Go-to-def** | Navegación a definiciones | ✅ |
| 74 | **LSP: Hover** | Información de tipos al pasar el mouse | ✅ |
| 75 | **lumen doc** | Generación de HTML desde comentarios `///` | ✅ |
| 76 | **Debugger** | Breakpoints, step, continue, inspect de variables | ✅ |
| 77 | **lumen fmt avanzado** | `.lumen-fmt.toml` para configurar reglas de formato | ✅ |
| 78 | **lumen lint** | Análisis estático: código muerto, complejidad ciclomática | ✅ |
| 79 | **REPL Pro** | Historial persistente, multilínea, resaltado, autocompletado | ✅ |
| 80 | **Package Manager** | `lumen install`, registry central, lock file | ✅ |
| 81 | **Build Incremental** | Caché de módulos para builds más rápidos | ✅ |
| 82 | **Hot Reload** | Recarga automática de módulos en dev con `lumen serve` | ✅ |
| 83 | **Playground Web** | Editor online con ejecución en navegador | ✅ |
| 84 | **Benchmarks** | Suite de rendimiento automatizada (criterion) | 📋 |
| 85 | **Plugins API** | Extensibilidad del compilador vía plugins | 📋 |

---

## 📦 4. Distribución & Portabilidad (Fases 86-95)

| # | Feature | Descripción | Estado |
|---|---------|-------------|--------|
| 86 | **AOT: C transpiler** | Transpilación a C + compilación con gcc/clang -O3 | ✅ |
| 87 | **AOT: Cranelift** | Backend nativo directo (sin dependencia de gcc) | ✅ (base) |
| 88 | **AOT: LTO + optimización** | Link-time optimization, dead code stripping | 📋 |
| 89 | **WASM backend** | Compilar a WebAssembly para navegadores | 📋 |
| 90 | **WASM: WASI** | Ejecutar en servidores/serverless vía WASI | 📋 |
| 91 | **WASM: JS interop** | Llamar funciones JS desde LUMEN y viceversa | 📋 |
| 92 | **Cross-compilation** | Compilar para Linux/macOS/Windows desde cualquier SO | 📋 |
| 93 | **Self-hosting** | El compilador de LUMEN escrito en LUMEN | 📋 |
| 94 | **Single binary** | `lumen` como binario único con todos los subcomandos | 📋 |
| 95 | **Installer** | Script de instalación unificado (`curl \| sh`) | 📋 |

---

## 📚 5. Stdlib & Runtime (Fases 96-140)

### 5.1 Colecciones (96-105)
HashMap, HashSet, VecDeque, BinaryHeap, BTreeMap, BTreeSet, LinkedList, iteradores lazy, ordenamiento avanzado, slices.

### 5.2 Texto (106-110)
Regex, Unicode normalization, format avanzado, encoding (UTF-8/16/32), parsing de números.

### 5.3 I/O (111-115)
Buffered I/O, streaming, archivos grandes, pipes, serial ports.

### 5.4 Red (116-120)
TCP/UDP sockets, HTTP client, HTTP server, WebSocket, TLS/SSL.

### 5.5 JSON/Serde (121-125)
Serialización/deserialización automática, formatos binarios (MessagePack, Bincode), CSV, YAML.

### 5.6 Base de Datos (126-130)
SQLite driver nativo, PostgreSQL, MySQL, ORM simple, migraciones.

### 5.7 Sistema (131-135)
Procesos, variables de entorno, señales, archivos temporales, paths.

### 5.8 DateTime (136-140)
Zonas horarias, duraciones, formatting, parsing, calendarios.

---

## ⚡ 6. Concurrencia & Async (Fases 141-160)

| # | Feature | Estado |
|---|---------|--------|
| 141-145 | **Threads**: spawn, join, channels, mutex, atomics | 📋 |
| 146-150 | **Async runtime real**: event loop, futures, streams, tokio-like | 📋 |
| 151-155 | **Paralelismo**: parallel iterators, map-reduce, rayon-like | 📋 |
| 156-160 | **Actores**: actor model, supervisión, message passing | 📋 |

---

## 🎨 7. GUI & Gráficos (Fases 161-175)

| # | Feature | Estado |
|---|---------|--------|
| 161-165 | **TUI**: terminal UI con ventanas, menús, tablas | 📋 |
| 166-170 | **2D Gráficos**: canvas, sprites, game loop | 📋 |
| 171-175 | **GUI Nativa**: widgets, layout engine, eventos | 📋 |

---

## 🤖 8. AI/ML & Data Science (Fases 176-195)

| # | Feature | Estado |
|---|---------|--------|
| 176-180 | **Tensores**: operaciones matriciales, autodiff | 📋 |
| 181-185 | **Redes Neuronales**: capas, optimizadores, entrenamiento | 📋 |
| 186-190 | **DataFrames**: CSV, Parquet, operaciones tabulares | 📋 |
| 191-195 | **ML Pipelines**: preprocesamiento, entrenamiento, serving | 📋 |

---

## 🚀 9. Producción & DevOps (Fases 196-230)

| # | Feature | Estado |
|---|---------|--------|
| 196-200 | **Testing avanzado**: benchmarks, fuzz, mutation, property | 📋 |
| 201-205 | **Observabilidad**: logging, tracing, metrics, profiling | 📋 |
| 206-210 | **Cloud SDKs**: AWS, GCP, Azure | 📋 |
| 211-215 | **Docker & K8s**: imágenes oficiales, operadores | 📋 |
| 216-220 | **CI/CD templates**: GitHub Actions, GitLab CI | 📋 |
| 221-225 | **Seguridad**: crypto, hashing, JWT, OAuth | 📋 |
| 226-230 | **v3.0 Release**: Documentación final, sitio web, comunidad | 📋 |

---

## 🎯 Hitos de Versión

| Versión | Alcance | Fases | Estado |
|---------|---------|-------|--------|
| **v1.0** | Compilador e infraestructura base | 0-20 | ✅ Completado |
| **v1.2** | Features del lenguaje + stdlib inicial | 21-41 | ✅ Completado |
| **v1.5** | Sintaxis moderna + genéricos + traits | 42-60 | ✅ Completado |
| **v1.6** | Lenguaje avanzado + LSP + Herramientas DX | 61-83 | ✅ Completado |
| **v2.0** | Distribución completa (WASM, cross-compile) + Herramientas Pro | 84-95 | 🏗️ En progreso |
| **v2.5** | Stdlib madura + Concurrencia real | 96-160 | 📋 Planificado |
| **v3.0** | GUI + AI/ML + Cloud + Producción — Lenguaje completo | 161-230 | 📋 Planificado |

---

> **LÚMEN** es el lenguaje de programación educativo bilingüe más completo en español/inglés.
> Diseñado para enseñar, prototipar y construir software real con una DX excepcional.
