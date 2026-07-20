# Changelog

## v1.0.1 — Julio 2026

### Added
- **Fase 21: For-Each Loop** — Sintaxis `para x en expr` / `for x in expr` para iterar sobre listas.
  - Desugaring a while-loop en IR, reutiliza opcodes existentes.
  - 31 nuevos tests (19 unit + 12 e2e).
  - Ejemplo en `examples/foreach.nv`.
- **Fase 22: Opcion/Optional Type** — Tipo opcional/nullable seguro `opcion<T>`.
  - Variantes `algun(valor)` y `ninguno`.
  - Opcodes `OptionSome(41)` y `OptionNone(42)`.
  - 15 nuevos tests (5 sema + 10 e2e).
  - Ejemplo en `examples/opcion.nv`.

### Changed
- Workspace version actualizada a 1.0.1.
- CI ahora corre en branches `master` y `main`.
- Sema: match arms y comparaciones usan `can_assign` (soporta `ninguno` → `Opcion<T>`).

## v1.0.0 — Julio 2026

Release inicial de LÚMEN. Lenguaje de programación educativo en español con 21 fases completadas.
