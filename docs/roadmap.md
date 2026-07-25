# 🛣️ Roadmap Oficial de LÚMEN (v1.2.0 - v2.0.0)

Este documento detalla la evolución completa del lenguaje de programación educativo bilingüe **LÚMEN**, desde sus cimientos hasta su madurez y distribución.

---

## 📊 Estado de Progreso General
```
[████████████████████████████████████████████] Fases 0-54 (100% Completadas)
[████████░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░] Fases 55-60 (En Desarrollo)
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
| **28-29** | **Stdlib Inicial** | Módulos `matematicas`, `texto`, `coleccion`, y `fecha` con builtins VM. | Funcionalidades matemáticas, texto, colecciones, archivos y fechas. |
| **30** | **E/S de Archivos** | Funciones builtins `__leer_archivo`, `__escribir_archivo`, `__existe_archivo`. | Manipulación del sistema de archivos con Resultado<T,E>. |
| **31** | **Stack Traces** | `CallFrame` en VM y visualización en tiempo real de la traza de llamadas. | Depuración rápida en caso de errores en tiempo de ejecución. |
| **32** | **Modo Dual Inglés** | `importar ingles;` habilita keywords en inglés. El loader lo skipea, lexer/parser ya lo soportan. | Permite programar usando keywords bilingües en el mismo motor. |
| **33** | **Errores Vistosos** | Subrayado exacto con caret (`^^^^`), colores ANSI, preview multi-línea, conteo de errores. | Experiencia de diagnóstico excelente y sugerencias de ayuda claras. |
| **34** | **Fuzzing Integrado** | 3 targets de `cargo-fuzz`: fuzz_lexer, fuzz_parser, fuzz_decoder. | Garantiza inmunidad del compilador ante entradas corruptas. |
| **35** | **Calidad de Vida** | Operador módulo `%`, encadenamiento `sino si`, `y`/`o` lógicos, `const`, `?:`. | Sintaxis fluida, ergonómica y retrocompatible. |

### 🧪 Bloque 4: Herramientas de Desarrollo (Fases 36-41)
| Fase | Nombre | Descripción y Logros Clave | Impacto en el Lenguaje |
| :---: | :--- | :--- | :--- |
| **36** | **Property Testing** | Dependencia `proptest` configurada en `lumen-codegen`. | Base para tests de invariants (expansión en Fase 49-50). |
| **37** | **lumen fmt** | Formateador automático. Crate `lumen-fmt` en desarrollo. | Stencil estético uniforme para archivos `.nv`. |
| **38** | **lumen repl** | REPL interactivo. Crate `lumen-repl` en desarrollo. | Experimentación instantánea de sintaxis en terminal. |
| **39** | **lumen test** | Framework de pruebas con `afirmar`. | Ejecución nativa de suites de pruebas. |
| **40** | **Manifiesto Proyecto** | `lumen new`, `lumen.toml`. Crate `lumen-project` en desarrollo. | Scaffolding automatizado. |
| **41** | **CI/CD & Releases** | GitHub Actions CI + release multiplataforma (Windows, Linux, macOS). | Pipeline robusto y binarios listos. |

---

## 🏗️ Roadmap — Características en Desarrollo (Fases 42-60)

### 🛠️ Bloque 1: Lenguaje & Sintaxis (42-60)
| Fase | Concepto | Detalle de Implementación | Estado |
| :---: | :--- | :--- | :---: |
| **42** | **Inferencia de Tipos** | `x = 42` deduce tipo automáticamente. | ✅ Completado |
| **43** | **Métodos en Structs** | `impl Struct` con `self` shorthand. | ✅ Completado |
| **44** | **Maps/Diccionarios** | `diccionario<K,V>` nativo con opcodes dedicados. | ✅ Completado |
| **45** | **String Interpolation** | `"Hola {nombre}"` desugarea a concatenación. | ✅ Completado |
| **46** | **Rangos Nativos** | `0..5` y `0..=5` para iteración. | ✅ Completado |
| **47** | **Constantes** | `const entero MAX = 100;` inmutable. | ✅ Completado |
| **48** | **String Indexing** | `s[i]` retorna carácter como texto. | ✅ Completado |
| **49** | **Conversiones** | `a_texto()`, `a_entero()`, `a_decimal()` con Resultado. | ✅ Completado |
| **50** | **División Entera** | `entero / entero` trunca (Rust/C). | ✅ Completado |
| **51** | **Concatenación Mixta** | `texto + numero/bool` auto-convierte. | ✅ Completado |
| **52** | **Mejores Errores** | Preview multi-línea, conteo, ANSI, ModuleError estructurado. | ✅ Completado |
| **53** | **Operador Ternario** | `cond ? si : no` — expresión condicional compacta. | ✅ Completado |
| **54** | **Loop Labels** | `romper etiqueta` / `continuar etiqueta`. | ✅ Completado |
| **55** | **Pattern Matching** | Match exhaustivo con guardas (`caso x si x > 0`). | 🔄 En desarrollo |
| **56** | **Genéricos Pro** | Bounds `<T: Comparable>` con traits. | 🔄 En desarrollo |
| **57** | **Matrices 2D** | Sintaxis `[[1,2],[3,4]]` nativa. | 🔄 En desarrollo |
| **58** | **Enums Avanzados** | Variantes con datos (`Variant(entero)`). | ✅ Completado |
| **59** | **Closures Pro** | Captura por valor/referencia. | 🔄 En desarrollo |
| **60** | **Async/Await** | `async funcion` + `esperar` con event loop. | 📋 Planificado |

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
