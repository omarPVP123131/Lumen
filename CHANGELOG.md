# Changelog

## v1.6.0 — Julio 2026

### Added
- **Fase 65: Guard Let** — `sea patron = expr sino { romper/retornar/continuar }`. Desugaring en IR builder a JmpIf/Jmp.
  - Parseo de `sea` con guard-let (separado de variable declaration).
- **Fase 66: Operator Overloading** — `impl Suma for Punto` con método `fn sumar(self, otro)`.
  - `Expr::Binary` ahora tiene `resolved_method: Option<String>`.
  - Sema: `resolve_operator_overloads()` post‑analysis con `HashMap<String, TypeInfo>`.
  - IR builder: emite `Call` en lugar de `Binary` cuando hay overload resuelto.
  - Traits `Suma`, `Resta`, `Multiplica`, `Divide` en impl para structs.
- **Fase 70: Impl Trait return** — `funcion impl Rasgo foo() { retornar expr }`.
  - `Type::ImplTrait(String)` en AST.
  - Parseo de `impl Rasgo` en `parse_type()`.
  - Sema: mapea a `TypeInfo::TypeVar` (tipo opaco resuelto en llamada).

### Changed
- Version bump to 1.6.0.
- Trait impl functions reciben `"self"` como primer parámetro automáticamente.

### Fixed
- Pre-existing destructure e2e tests still failing (2 tests).
- Lexer test `test_pipe_token` expects `[Pipe]` but gets `[Pipe, Eof]`.

## v1.2.0 — Julio 2026

### Added
- **Stdlib (matematicas, texto, coleccion, fecha)** — Módulos estándar nativos en español.
- **E/S de Archivos** — Builtins `leer_archivo`, `escribir_archivo`, `existe_archivo`.
- **Stack Traces** — Pila de llamadas completa en errores en tiempo de ejecución.
- **Mensajes de Error Mejorados** — Subrayado exacto con caret (`^^^^`) y colores ANSI en terminal.

### Fixed
- Corrección de advertencias de Clippy (`approx_constant`, `question_mark`, `unneeded_wildcard_pattern`, `unnecessary_sort_by`) en integración continua.

## v1.1.0 — Julio 2026

### Added
- **Fase 21: For-Each Loop** — Sintaxis `para x en expr` / `for x in expr`.
  - 31 tests. Ejemplo `examples/foreach.nv`.
- **Fase 22: Opcion/Optional Type** — `opcion<T>` con `algun(valor)` y `ninguno`.
  - 15 tests. Opcodes 41 y 42. Ejemplo `examples/opcion.nv`.
- **Fase 23: Enums/Tipos Suma** — `enum Nombre { Variante, Variante(tipo) }`.
  - Namespaced access `Nombre::Variante`. Opcode `EnumCtor(43)`.
  - 20 tests.
- **Fase 24: Tuplas** — `(tipo, tipo)` y acceso `expr.0`, `expr.1`.
  - Opcodes `TupleNew(44)`, `TupleAccess(45)`. 4 tests.
- **Fase 25: Destructuring** — `entero x, texto y = expr`, wildcard `_`.
  - 14 tests.
- **Fase 26: Genéricos Básicos** — `<T, U>` en funciones y structs.
  - 17 tests.
- README mejorado con ejemplos de todas las características.
- Roadmap actualizado marcando fases 23-26 como completadas.

### Changed
- Workspace version a 1.1.0.
- CI ahora corre en branches `master` y `main`.
- MSRV actualizado a 1.82.

## v1.0.0 — Julio 2026

Release inicial de LÚMEN. Lenguaje de programación educativo en español con 21 fases completadas.
