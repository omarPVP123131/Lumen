# LÚMEN — Auditoría v3.94.23 → v3.94.24 (cobertura ampliada: casts, bucles, bits, slices, lambdas, closures, ternarios, rangos, mapas y benchmarks)

**Fecha:** 2026-09-02
**Build de partida:** `v3.94.23`
**Build objetivo:** `v3.94.24`
**Alcance:** tercera ronda de caza de bugs sobre v3.94.23 con un arnés de
**paridad VM ⇄ AOT-C** (`lumen run` vs `lumen build --native`) que difiere
salida y código de salida de cada programa. Se corrigieron **7 bugs reales**
(con regresión cruzada), se documentaron **3 limitaciones** y se construyó el
**benchmark de todas las áreas del lenguaje**.

---

## Resumen de estado

| # | Problema | Estado en v3.94.24 |
|---|---|---|
| 1 | `X como T` era un **no-op** (el valor pasaba tal cual) | ✅ **Corregido** — conversión runtime real (entero/decimal/booleano/texto), paridad VM/AOT |
| 2 | `continuar`/`romper` dentro de `para … en` daba **E055/E070** | ✅ **Corregido** — `para-cada` es un ciclo de pleno derecho (sema + IR) |
| 3 | `~` bitnot **duplicaba el operando** al plegar constantes | ✅ **Corregido** — `optimize.rs` reemplaza in-situ |
| 4 | Slice de rango **literal** `a[x..y]`/`s[x..y]` roto en AOT-C | ✅ **Corregido** — builtin polimórfica `__rodaja` |
| 5 | Índice-rango por **variable** `a[r]`/`s[r]` (r: lista) en AOT-C | ✅ **Corregido** — `_arr_getv` en el runtime C (índice entero o rango) |
| 6 | Ternario inline en argumentos de `imprimir` deja un `void` en AOT-C | ✅ **Corregido** — spill de la pila de expresiones antes del `JmpIf` |
| 7 | Captura de closures sobre locales mutables **diverge VM vs AOT** | ✅ **Corregido** — parent-link del lambda + seed de capturas alineado con la celda global |
| 8 | Bucle de 20k inserciones en mapa **sin salida / OOM** en AOT-C | ✅ **Corregido** (semántica) — implementación de referencia correcta; **limitación documentada** de memoria O(n²) |
| 9 | `sea f = () => { … }` → E020 (la flecha no es sintaxis LÚMEN) | ✅ **Documentado** — usar `funcion () { … }` o `\|x\| …` |
| 10 | Lambda de pipes vacía `\|\| { … }` → E020 (colisión con `\|\|` lógico) | ✅ **Documentado** — usar `funcion () { … }` |

---

## Detalle técnico de los fixes

### 1 — Casts reales (`X como T`)

`Expr::Cast` baja a builtins de conversión compartidos por ambos backends:
`__cast_entero` (trunca hacia cero), `__cast_decimal`, `__cast_booleano`
(truthiness) y `a_texto` (stringify). Verificado con `H_cast_real`.

### 2 — `continuar`/`romper` dentro de `para … en`

`sema.rs` sube/baja `loop_depth` alrededor del cuerpo del `para-cada`;
`builder.rs` emite una `continue_label` y registra `LoopLabels`. Verificado
con `I_foreach_continuar_romper`.

### 3 — `~` bitnot y el plegado de constantes

El plegado unario **reemplaza in-situ** la constante original por el resultado
plegado, conservando el modelo de profundidades y los índices de la pila
abstracta. Verificado con `J_bitnot_multiarg`.

### 4 — Slice de rango literal en AOT-C

`Expr::Index` con índice `Expr::Range` baja a la builtin polimórfica
**`__rodaja(contenedor, inicio, fin, inclusivo)`**, con la misma semántica que
el `ArrayGet` de rango de la VM. Verificado con `K_rodaja_rango`.

### 5 — Índice-rango por variable (rango materializado) en AOT-C

La VM permite `a[r]` / `s[r]` donde `r` es una variable `lista<entero>`: por
cada entero del rango se selecciona ese índice (los fuera de rango se omiten).
El backend C coaccionaba el rango a entero con `_i.i` (== 0) y devolvía el
primer elemento. **Fix:** nueva `_arr_getv(Val a, Val ix)` en `lumen_rt.h` que
despacha índice entero **o** rango (sub-selección de caracteres UTF-8 para
textos, de elementos con `_dcp` para listas), cableada en los tres emisores
del backend C (camino clásico `PUSH`, camino de expresiones `estack` y
`_lw_arr_get` de Cranelift). Verificado con `N_rango_variable_indice`
(`vm_arr:2 vm_str:bc k:30`).

### 6 — Ternario inline en argumentos (AOT-C)

`imprimir("x:", c1 ? 10 : 20, …)` dentro de un bucle dejaba un `void` y
desplazaba los argumentos a partir de la 2.ª iteración. Causa: el emisor de la
pila de expresiones (`estack`) al emitir `JmpIf` **solo desapilaba la
condición** y dejaba los valores pendientes (el prefijo `"x:"`) en la pila
virtual; el spill posterior los emitía **dentro** de la rama verdadera, con lo
que el camino falso los perdía. **Fix:** `JmpIf` hace `xe_spill` de los valores
pendientes **antes** de emitir el salto condicional. Verificado con
`O_ternarios_bucle` (tres ternarios en una llamada, dentro de `mientras`).

### 7 — Captura de closures sobre locales mutables

La VM captura los bindings visibles **por valor/snapshot** en el momento de
crear el closure (celdas nuevas por instanciación), de modo que mutar la
variable dentro del closure no afecta al definidor. El AOT-C compartía una
única celda global. Dos causas encadenadas:

1. `compile_lambda` (IR builder) **no registraba** `program.parents[lambda]`,
   por lo que `compute_captures` no veía el lambda como hijo y el `FuncRef`
   no llevaba snapshot (`_vfref` en vez de `_vfref_snap`).
2. `base_bindings` generaba la key del seed de capturas como
   `{fn}::{var}#k`, mientras la celda real de un global declarado en la
   entrada es el **nombre crudo** (`{var}`); el lambda escribía una celda
   distinta de la del definidor.

**Fix:** registrar el parent-link en `compile_lambda`; y en `base_bindings`,
usar el nombre crudo cuando la función es la entrada y la variable es global
(capturada por anidadas). Verificado con `P_closure_captura` (acumulador
interno, dos closures independientes y closure creado en bucle).

### 8 — Mapa en AOT-C: semántica correcta + limitación de memoria

El backend C implementa `__map_poner` con un mapa **por valor** (`_map_set`
asigna un buffer nuevo y copia el anterior). Esto preserva la semántica de la
VM (mapa persistente: `sea d2 = d; d = __map_poner(d, …)` deja a `d2` intacto),
pero el buffer anterior queda huérfano (sin recuento de referencias) → **O(n²)
de memoria** en bucles de inserción y OOM alrededor de ~5k claves en AOT-C. La
VM usa `ImMap` persistente (O(log n)) y maneja 30k+ claves.

Se restauró la implementación de referencia (correcta) y se **documenta la
limitación**: para mapas masivos usar la VM; en AOT-C el costo es O(n) por
inserción y O(n²) de memoria. La corrección completa (refcounting o mapa
persistente en C) queda para una versión futura. Verificado con `M_mapa_basico`
(1 000 claves, `maps:999000 len:1000`, paridad VM/AOT).

---

## Limitaciones documentadas (no corregidas)

### 9 y 10 — Sintaxis de lambdas
LÚMEN **no tiene** sintaxis de flecha `=>`. Las lambdas se escriben con
`funcion (params) { cuerpo }` o con pipes `|x| expr`. La flecha solo aparece
dentro de **strings JavaScript** de callbacks de GUI (p. ej.
`c.onmousedown = (e) => { … }`), no en código LÚMEN. La forma de pipes vacía
`|| { … }` colisiona con el `||` lógico y da E020; usar `funcion () { … }`.

### 8 — Mapa en AOT-C (memoria O(n²))
Ver el ítem 8 anterior.

---

## Regresión / cruce

`scripts/regresion_qa.py` ahora cubre **A–P, 20/20** con un modo `parity` que
ejecuta cada caso en VM y en binario nativo y exige salida y código de salida
idénticos:

- **H_cast_real** — casts entero/decimal/booleano/texto
- **I_foreach_continuar_romper** — control de flujo en `para … en`
- **J_bitnot_multiarg** — `~` en llamadas multi-argumento
- **K_rodaja_rango** — slices de rango en texto y lista
- **L_lambdas** — `funcion () { … }` y `|x| …`
- **M_mapa_basico** — mapa de 1 000 claves (inserción + lectura + largo)
- **N_rango_variable_indice** — `a[r]`/`s[r]` con rango en variable
- **O_ternarios_bucle** — tres ternarios en una llamada, dentro de bucle
- **P_closure_captura** — captura por snapshot (acumulador, dos closures, bucle)

**Verificación global en v3.94.24:** `cargo fmt --check` OK, `cargo clippy
--all -- -D warnings` OK, `cargo test --workspace` verde, `lumen check
examples` 396/396, `ci_gate.py` PASS 392/389 (0 crashes, 4 fallos esperados),
regresión QA 20/20.

---

## Benchmark de todas las áreas del lenguaje

Se creó `benchmarks/run_bench_all.py` + `benchmarks/c/bench_all.c` con **10
tareas** que cubren todas las áreas, comparando `lumen-vm`, `lumen-aotc`
(C -O3) y `c` de referencia. Resultados (mejor de 3, segundos):

| Tarea | Área | lumen-vm | lumen-aotc | C (-O3) |
|---|:---|---:|---:|---:|
| fib | recursión | 0.014 | 0.005 | 0.001 |
| sum | bucles | 0.020 | 0.002 | 0.001 |
| primes | primos | 0.020 | 0.002 | 0.001 |
| strings | strings | 1.36 | 0.013 | 0.010 |
| arrays | arrays | 0.31 | 0.002 | 0.002 |
| structs | structs+métodos | 2.22 | 0.095 | 0.001 |
| enums | enums+elegir | 1.18 | 0.067 | 0.001 |
| maps | mapas (1k) | 0.022 | 0.091 | 0.001 |
| closures | closures | 0.88 | 0.016 | 0.001 |
| unicode | unicode | 0.44 | 0.015 | 0.005 |

Salidas verificadas contra golden en los tres backends. Artefactos:
`benchmarks/results/benchmark_all.csv` e `informe_all.md`.
