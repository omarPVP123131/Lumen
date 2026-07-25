# 🛣️ Roadmap Oficial de LÚMEN (v1.2.0 - v2.0.0)

Este documento detalla la evolución completa del lenguaje de programación educativo bilingüe **LÚMEN**, desde sus cimientos hasta su madurez y distribución.

---

## 📊 Estado de Progreso General
```
[████████████████████████████████████████████] Fases 0-54 (100% Completadas)
[████████████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░] Fases 55-60 (En Desarrollo/Planificadas)
[░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] Fases 61-80 (Planificadas)
```

---

## ✅ Fases Completadas (0-41)
Estas fases representan la construcción de la infraestructura base, el compilador, la máquina virtual, las características del lenguaje, y el ecosistema inicial de pruebas y distribución.

### 🏗️ Bloque 1: Cimientos e Infraestructura (Fases 0-15)
| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **0-15** | **Infraestructura Base** | Lexer, parser, sema, IR, bytecode, VM, CLI, arrays, control de flujo avanzado. | Construcción del compilador y VM stack-based básico. |
| **16** | **Funciones Avanzadas** | Parámetros default (`funcion foo(a, b = 10)`), Lambdas IIFE, Closures asignables. | Funciones de primera clase y clausuras completas en VM. |
| **17** | **Estructuras/Objetos** | Declaración (`estructura`), inicialización, acceso y asignación de campos. | Soporte inicial para tipos de datos compuestos y POO básica. |
| **18** | **Sistema de Módulos** | Directiva `importar`, ModuleLoader, resolución de rutas y prefijado de nombres. | Evita colisiones de nombres al importar bibliotecas. |
| **19** | **Optimizaciones** | Constant folding, Dead Code Elimination, Shared Constant Pools, Function Index Cache. | Reducción de tamaño del bytecode y aceleración de ejecución. |
| **20** | **Release v1.0.0** | SemVer, especificaciones, documentación oficial y README completo. | Hito de estabilidad del primer compilador de producción. |

### 🛠️ Bloque 2: Features del Lenguaje (Fases 21-27)
| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **21** | **Bucle For-Each** | Sintaxis `para x en lista`. Desugaring directo en IR a bucles `mientras`. | Sintaxis de iteración limpia y segura sobre colecciones. |
| **22** | **Resultado<T, E>** | Tipo algebraico de datos con `exito(valor)` y `error(mensaje)`. Sugar `intentar`. | Manejo de errores idiomático y seguro sin panics arbitrarios. |
| **23** | **Opcion<T>** | Tipos opcionales con `algun(valor)` y `ninguno`. | Evita errores de puntero nulo (*null safety*). |
| **24** | **Enums/Tipos Suma** | Definición de `enum` bilingüe con soporte para acceso mediante `::`. | Modelado de dominio complejo con variantes seguras. |
| **25** | **Tuplas** | Tipo compuesto anónimo `(tipo, tipo)` y acceso estático `.0`, `.1`. | Agrupación rápida de tipos sin declarar estructuras. |
| **26** | **Destructuring** | Desestructuración en declaraciones (`entero x, texto y = ...`) y asignaciones. | Extracción rápida de valores de tuplas con comodín `_`. |
| **27** | **Genéricos Básicos** | Parametrización `<T>` en firmas de funciones y structs con *type erasure*. | Reutilización de código con estricta seguridad de tipos. |

### 📚 Bloque 3: Stdlib, Errores y Calidad de Vida (Fases 28-35)
| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **28-29** | **Stdlib Inicial** | Módulos `matematicas`, `texto`, `coleccion`, y `fecha`. | Funcionalidades matemáticas básicas y manipulación de texto. |
| **30** | **E/S de Archivos** | Funciones builtins `leer_archivo`, `escribir_archivo`, `existe_archivo`. | Manipulación del sistema de archivos integrado con la VM. |
| **31** | **Stack Traces** | `CallFrame` en VM y visualización en tiempo real de la traza de llamadas. | Depuración rápida en caso de errores en tiempo de ejecución. |
| **32** | **Modo Dual Inglés** | Pre-scan para habilitar keywords en inglés con `importar ingles;`. | Permite programar usando keywords bilingües en el mismo motor. |
| **33** | **Errores Vistosos** | Subrayado exacto de errores con caret (`^^^^`) y colores ANSI enriquecidos. | Experiencia de diagnóstico excelente y sugerencias de ayuda claras. |
| **34** | **Fuzzing Integrado** | 3 targets de `cargo-fuzz` compilando para lexer, parser y decoder. | Garantiza la inmunidad del compilador ante entradas corruptas. |
| **35** | **Calidad de Vida** | Operador módulo `%`, encadenamiento `sino si`, `y`/`o` lógicos, paréntesis opcionales. | Sintaxis fluida, ergonómica y retrocompatible. |

### 🧪 Bloque 4: Herramientas de Desarrollo (Fases 36-41)
| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **36** | **Property Testing** | Round-trips con `proptest` para opcodes del codegen y tokens del lexer. | Garantiza invariants del compilador en miles de casos aleatorios. |
| **37** | **lumen fmt** | Formateador automático inteligente (`crates/lumen-fmt`). | Stencil estético uniforme y automático para cualquier archivo `.nv`. |
| **38** | **lumen repl** | REPL interactivo con persistencia de variables y declaraciones (`crates/lumen-repl`). | Experimentación instantánea de sintaxis directamente en terminal. |
| **39** | **lumen test** | Framework de pruebas integrado con bloques `test` y sentencia `afirmar`. | Ejecución nativa de suites de pruebas (`lumen test <file>`). |
| **40** | **Manifiesto Proyecto** | Inicializador de proyectos con `lumen new` y gestión con `lumen.toml`. | Scaffolding automatizado y rutas de librerías centralizadas. |
| **41** | **CI/CD & Releases** | Criterion benchmarks, CI/CD en push/PR y GitHub releases multiplataforma. | Pipeline robusto y binarios listos para Windows, Linux y macOS. |

---

## 🏗️ Roadmap Planificado (Fases 42-80)
Estas fases están diseñadas para consolidar a LÚMEN como un lenguaje maduro, potente y utilizable para desarrollo de scripts profesionales, aplicaciones web (WASM), herramientas de enseñanza y sistemas interactivos de depuración.

### 🛠️ Bloque 1: Lenguaje & Sintaxis (Features 42-60)
| Fase | Concepto | Detalle de Implementación | Beneficio de Experiencia (DX) |
| :---: | :--- | :--- | :--- |
| **42** | **Inferencia de Tipos** | El compilador deduce el tipo estático de la variable (`x = 42` → entero). | Código limpio, rápido de escribir, sin declarar tipos evidentes. |
| **43** | **Métodos en Structs** | Implementación de bloques `impl Struct` para enlazar funciones a tipos de datos. | Transición de POO básica a POO idiomática estructurada. |
| **44** | **Maps/Diccionarios** | Tipos llave-valor nativos en VM y codegen con opcodes `MapNew`, `MapGet`, etc. | Estructuras de datos dinámicas indispensables en desarrollo real. |
| **45** | **String Interpolation**| Desugaring automático en parser de `"hola {nombre}"` a concatenación. | Formateo de strings elegante y libre de ruidosas concatenaciones. |
| **46** | **Rangos Nativo** | Operadores `..` y `..=` para desugaring elegante en iteradores. | Bucles e índices mucho más expresivos y legibles. |
| **47** | **Constantes** | Keywords `const` para expresiones conocidas en tiempo de compilación. | Inmutabilidad garantizada y optimizaciones avanzadas del compilador. |
| **53** | **Operador Ternario** | Soporte para sintaxis de expresión condicional compacta (`cond ? a : b`). | Expresividad máxima para condicionales sencillos de una línea. |
| **54** | **Etiquetas de Loops** | Soporte para `break 'label` y `continue 'label` en loops anidados. | Facilita la salida ordenada de algoritmos y matrices multidimensionales. |
| **55** | **Pattern Matching** | Expansión del actual `match` a desempaquetado de enums, rangos y tuplas. | Selección de flujo declarativa, compacta y robusta. |
| **56** | **Genéricos Pro** | Restricciones de tipo (*bounds*) en firmas genéricas (`<T: Numerico>`). | Polimorfismo altamente seguro y reutilización de código genérico. |
| **57** | **Matrices** | Soporte nativo para arreglos bidimensionales (`matriz[x][y]`) con slicing. | Preparación para algoritmos científicos e ingeniería de datos. |
| **58** | **Enums Avanzados** | Soporte para variantes de Enums que contengan datos (`Variant(entero)`). | Expresión de estados complejos y semántica algebraica avanzada. |
| **59** | **Closures Pro** | Captura completa por valor/referencia de contextos exteriores en closures. | Programación funcional del más alto nivel con lambdas complejas. |
| **60** | **Async/Await** | Soporte para `async funcion` y expresión `esperar` con event loop en VM. | Permite I/O asíncrono no bloqueante nativo en LÚMEN. |

### 🚀 Bloque 2: Herramientas & Ecosistema (61-75)
| Fase | Herramienta | Detalle de Implementación | Impacto |
| :---: | :--- | :--- | :--- |
| **50** | **LSP Server** | Servidor basado en JSON-RPC que interactúa con editores (VS Code, etc.). | Errores en vivo, autocompletado y autodiagnósticos de semántica. |
| **51/52**| **Stdlib Expandida** | Builtins nativos para parsear/serializar JSON e interactuar con HTTP. | Capacidad para crear clientes API y scripts de automatización reales. |
| **61** | **Exports & Visibilidad** | Keywords `exportar` e `importar solo { ... }` para modularización. | Control estricto de visibilidad para desarrollo de librerías. |
| **62** | **lumen doc** | Generador de documentación estática (HTML) desde comentarios `///`. | Auto-documentación instantánea y estructurada para librerías. |
| **63** | **Build Incremental** | Caché del AST y tipos del sema para evitar re-compilaciones costosas. | Tiempos de compilación de milisegundos en proyectos grandes. |
| **67** | **Package Manager** | Utilidad `lumen install <repo>` para clonar dependencias remotas. | Creación de ecosistema compartido por la comunidad educativa. |
| **68** | **Backend WASM** | Generación de WebAssembly directo o runtime embebido en un motor JS. | Creación de playgounds web, juegos de navegador y ejecución en frontend. |
| **69** | **FFI (C Interop)** | Permite declarar `extern funcion` y linkear con librerías nativas `.dll` / `.so`. | Acceso a todo el ecosistema nativo de C/C++ y drivers. |
| **70** | **REPL Pro** | Historial persistente, atajos, multilínea y resaltado dinámico en CLI. | REPL de grado profesional para interactuar ágilmente en terminal. |
| **71** | **Debugger** | Breakpoints interactivos, paso a paso (`step`) e inspección de VM stack. | Herramienta didáctica suprema para inspeccionar cómo ejecuta la VM. |
| **72** | **Fmt Avanzado** | Archivo `.lumen-fmt.toml` para configurar reglas estéticas del código. | Personalización del formato de código según estándares de equipos. |
| **73** | **lumen lint** | Análisis estático para código muerto, complejidad o variables sin uso. | Garantía de código limpio, mantenible y libre de malas prácticas. |
| **75** | **Plugins** | API para inyectar pasos personalizados en el compilador o en el sema. | Extensibilidad ilimitada para investigación y educación avanzada. |

### ⚡ Bloque 3: Performance, Educación & Lanzamiento (76-80)
- **76**: **Gramática EBNF Inmutable**: Estabilización final de la sintaxis y gramática oficial.
- **77**: **Ecosistema Educativo**: Creación de un Playground web interactivo con tutoriales paso a paso basados en WASM.
- **79**: **VM Performance Tuning**: NaN-boxing, optimización de inlining en bytecode y JIT-like cache para hot-paths en ejecución.
- **80**: **LÚMEN v2.0 Release**: Lanzamiento final del lenguaje educativo bilingüe más completo y rápido.
