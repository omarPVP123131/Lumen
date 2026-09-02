# LÚMEN — Resolución de la Auditoría v3.94.22 → v3.94.23

**Fecha:** 2026-09-02
**Build de partida:** `lumen-v3.94.22-linux-x64-musl`
**Build objetivo:** `v3.94.23`
**Alcance:** cierre del reporte definitivo de auditoría v3.94.22 (ítems A–K), con
correcciones de código, documentación y una suite de regresión/cruce que fija
el comportamiento para evitar que estos bugs reaparezcan.

---

## Resumen de estado por ítem

| # | Problema | Estado en v3.94.23 |
|---|---|---|
| A | `dueno T` no se auto-desreferencia | ✅ **Corregido** (sema: campo, índice, método, operadores) |
| B | Hilos/mutex sin type-safety, fallo silencioso a `void` | ✅ **Corregido** (E042 en `check` para literales + error explícito en runtime) |
| C | `en_tiempo_compilacion` no evalúa en tiempo de compilación | ✅ **Corregido** (E090 si no es evaluable; puro sigue plegando) |
| D | Mutación en binding de `si sea` no persiste | ✅ **Documentado** (limitación intencional: bindings por valor) |
| E | Overflow de enteros con wraparound silencioso | ✅ **Documentado** + aritmética verificada (`suma_verificada`, etc.) |
| F | 3 módulos de stdlib no compilan | ✅ **Verificado** (69/69 compilan; la causa raíz ya se había corregido en v3.84.4) |
| G | Mensaje confuso `cualquiera`→`decimal` | ✅ **Corregido** (E031 explica la normalización) |
| H | Sintaxis campos vs. parámetros inconsistente | ✅ **Documentado** (ambos órdenes en params; campos siempre `nombre: Tipo`) |
| I | `--help` / docs sin concurrencia | ✅ **Documentado** (`docs/referencia/concurrencia.md` + CLI) |
| J | Cascadas de errores del parser | ✅ **Reforzado** (dedup por posición exacta + nota de agrupados) |
| K | Paralelismo real no verificado | ✅ **Documentado** (limitación de entorno de 1 core; procedimiento de medición) |

---

## Detalle técnico de los fixes

### A — Auto-desreferencia de `Dueno<T>`
`TypeInfo::Dueno` es un wrapper de titularidad a nivel de tipos (sin
representación en runtime). El checker ya atravesaba `Prestado` (fix v3.5.7)
pero no `Dueno`. Se extendió la auto-desreferencia en `sema.rs` para:
acceso a campo (`FieldAccess`), indexación (`Index`), llamadas a método
(`MethodCall`) y operadores binarios (`Binary`). Cubre struct simple,
struct genérico instanciado y lista. La destructuración de enums y las
llamadas a método ya funcionaban y se mantienen.

### B — Type-safety de la API de concurrencia
Dos capas:
1. **`check`**: `validate_reflection_targets` en `sema.rs` valida que los
   literales de texto pasados a `hilo_lanzar*`, `mutex_bloquear`,
   `rwlock_*`, `tarea_lanzar*`, `stream_*`, `par_*`, `generador_nuevo`,
   `scope_lanzar` nombren una función definida; si no, emite **E042** con el
   span del literal. Soporta los nombres con prefijo de módulo importado.
2. **Runtime**: `run_function_or_value` en `vm.rs` convierte `Err(...)` de
   `run_function` en `Value::Error("Función '...' no definida")` en vez de
   `void`. Aplicado a hilos, mutex, tareas, streams, `par_*` y generadores.

### C — `en_tiempo_compilacion` con semántica real
`ComptimeEvaluator` ya plegaba expresiones puras. El problema era la
degradación silenciosa: lo no evaluable quedaba como bloque runtime. Ahora
`find_unfolded_comptime` (en `lumen-ir/comptime.rs`) recorre el programa tras
el plegado y el CLI emite **E090** con el span del bloque para todo
`comptime`/`en_tiempo_compilacion` no evaluable.

### E — Overflow: decisión de diseño + variantes verificadas
- **Documentado**: `+ - * /` hacen wraparound (como C/Rust release).
- **Nuevos builtins**: `__suma_verificada`, `__resta_verificada`,
  `__multiplicacion_verificada`, `__division_verificada` (y alias
  `__checked_*`) en la VM, con `checked_*` de Rust; devuelven
  `exito(...)`/`error(...)`. Expuestos en `stdlib/matematicas.nv` como
  `suma_verificada`, `resta_verificada`, `multiplicacion_verificada`,
  `division_verificada` con tipo `resultado<entero, texto>`.

### G — Mensaje E031 con contexto
Cuando se asigna `Decimal` a `Entero`, el mensaje E031 ahora explica que
`cualquiera`/`numero` normaliza los números a `decimal` y sugiere la
conversión explícita `(x) como entero`.

### J — Cascadas del parser
`push_error` ahora deduplica por **posición exacta** (línea+columna) además
del dedup previo por (código, línea); `parse_with_report` expone el contador
de suprimidos y el CLI muestra "(N error(es) derivado(s) de la misma causa
raíz agrupados)".

---

## Suite de regresión y cruce

- **`scripts/regresion_qa.py`**: 11 casos que cubren A–G (10 programas + el
  gate de stdlib de 69 módulos). Se ejecuta en CI (`qa-regresion`).
- **Tests unitarios Rust**:
  - `sema.rs`: `dueno` campo/index/genérico, validación de targets de
    reflexión, mensaje de normalización `cualquiera`.
  - `comptime.rs`: pliegue puro y reporte de impuro.
- **CI**: nuevo job `qa-regresion` + gate de stdlib en el workflow.

## Verificación (antes de merge)

- `cargo test --workspace` — todo verde (956+ tests).
- `lumen check` de los 396 ejemplos.
- `scripts/ci_gate.py` — 0 crashes / 0 fails.
- `scripts/regresion_qa.py` — 11/11.
- `stdlib/*.nv` — 69/69 compilan limpio.
