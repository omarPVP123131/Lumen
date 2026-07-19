# Roadmap de LÚMEN

## v1.0.0 — Completado ✅

| Fase | Descripción | Estado |
|------|-------------|--------|
| 0 | Cimientos del proyecto (workspace, CI, tooling) | ✅ |
| 1 | Especificación formal (gramática EBNF, tipos, errores) | ✅ |
| 2 | Lexer (tokenización, errores recuperables) | ✅ |
| 3 | Parser y AST (recursive descent + Pratt, error recovery) | ✅ |
| 4 | Análisis semántico (type checking, scope) | ✅ |
| 5 | IR (three-address code, constant folding) | ✅ |
| 6 | Bytecode (.nvc format, codegen, disassembler) | ✅ |
| 7 | VM (stack-based, call frames, 37 opcodes) | ✅ |
| 8 | CLI completa (run, build, check, disasm) | ✅ |
| 9 | Split numérico (entero i64, decimal f64) | ✅ |
| 10 | Tipos compuestos (arrays/lista<T>) | ✅ |
| 11 | Strings (operaciones, concatenación) | ✅ |
| 12 | Booleanos (operaciones lógicas, cortocircuito) | ✅ |
| 13 | Control de flujo avanzado (break, continue, match) | ✅ |
| 14 | Parámetros default en funciones | ✅ |
| 15 | Lambdas IIFE | ✅ |
| 16 | Closures (lambdas asignables, función type) | ✅ |
| 17 | Estructuras/Objetos | ✅ |
| 18 | Módulos (importar, loader, circular detection) | ✅ |
| 19 | Optimizaciones (constant folding, DCE, shared pools) | ✅ |
| 20 | v1.0 Release (docs, README, versionado) | ✅ |

## v1.1 — Próximo

- [ ] Publicar en crates.io
- [ ] GitHub Release con binarios precompilados (Windows, Linux, macOS)
- [ ] Benchmarks públicos y documentados
- [ ] Modo debug paso a paso (step-through)

## v1.2 — Herramientas

- [ ] LSP server (autocompletado, errores en tiempo real)
- [ ] Extensión VS Code (syntax highlighting)
- [ ] `lumen fmt` (formateador automático)
- [ ] `lumen repl` (bucle interactivo)

## v2.0 — Educación y Ecosistema

- [ ] Playground web (WASM)
- [ ] Visualización del stack en tiempo de ejecución
- [ ] Material didáctico: tutoriales, ejercicios, banco de problemas
- [ ] Generación de diagramas de flujo desde código
- [ ] Gestor de paquetes simple

## v3.0 — Madurez

- [ ] Compilación separada de módulos
- [ ] Sistema de tipos avanzado (genéricos, traits)
- [ ] Optimizaciones agresivas (inlining, escape analysis)
- [ ] NaN-boxing para representación de valores compacta
- [ ] FFI (llamar C desde LÚMEN)
