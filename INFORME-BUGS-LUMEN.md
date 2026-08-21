# Informe final — Bugs de LÚMEN v2.4.6

**Rama:** `fix/bugs-v2.4.6` · **Commits:** `bb7ced3` + `0c3ff57` (+ BUG-014/015 sin commitear, a peticion) · **Repo:** `/home/user/lumen-src`

## Resumen

Los **8 bugs reportados** están resueltos. Además busqué activamente bugs nuevos y
encontré y parcheé **12 más** (BUG-009 a BUG-020), varios de corrupción silenciosa:
código que se ejecutaba mal **sin emitir ningún error**.

Para la última ronda escribí **45 ejemplos nuevos** (en `ejemplos/`, organizados por
tema) que ejercitan textos, control de flujo, estructuras de datos, funciones,
lambdas y tipos algebraicos. De ahí salieron 5 bugs más.

## Los 8 bugs del reporte

| # | Severidad | Estado | Nota |
|---|---|---|---|
| BUG-008 | 🔴 | ✅ Resuelto | **La premisa del reporte era incorrecta** (ver abajo) |
| BUG-007 | 🟠 | ✅ Resuelto | `a_entero`, `a_decimal`, `a_numero` + variantes `_seguro` y `es_numero` |
| BUG-003 | 🟠 | ✅ Resuelto | Destructuring de datos de enum en `elegir`/`caso`, incl. patrones OR y literales |
| BUG-006 | 🟡 | ✅ Resuelto | `resultado`/`opcion` pasan a *soft keywords*: son tipo sólo si les sigue `<` |
| BUG-002 | 🟡 | ✅ Resuelto | `texto(42)` → sugiere `a_texto(...)` en vez de un genérico "no definida" |
| BUG-001 | 🟡 | ✅ Resuelto | `abs`/`absoluto`, `minimo`/`maximo`, `raiz`, `potencia`, `piso`, `techo`, `redondear` |
| BUG-005 | 🟡 | ✅ Resuelto | `lumen test` ejecuta y cuenta las aserciones sueltas; sale con código 1 si fallan |
| BUG-004 | 🟢 | ✅ Resuelto | El REPL conserva las variables entre líneas por *pipe* |

### Corrección importante sobre BUG-008

El reporte decía que *"`struct` se pasa por referencia pero `lista<T>` por valor"*.
**Esa inconsistencia no existe**: verifiqué en dos backends que **todo se pasa por
valor**, structs incluidos. El síntoma real (mutaciones que se pierden) era cierto,
pero la causa era otra: `prestado mut` —el mecanismo de paso por referencia que
`LENGUAJE.md` ya documentaba— **no estaba implementado**. Ahora sí copia de vuelta al
llamador, incluso en recursión y a varios niveles de llamada. `prestado` sin `mut`
permite leer y rechaza mutar con un `E061` explicativo.

## Bugs nuevos encontrados y parcheados

- **BUG-010 — Retorno temprano truncaba el programa** (🔴 el más grave). Una función con
  `retornar` dentro de una rama no recibía su instrucción `Ret`: la ejecución seguía
  sobre el cuerpo de la función siguiente y **el resto del programa se perdía en
  silencio**. El emisor comprobaba si había *algún* `Return` en el cuerpo en vez de
  mirar la última instrucción.
- **BUG-011 — `lista[i].campo = v`** fallaba en runtime por orden de operandos en la pila.
- **BUG-012 — Los builtins nuevos ensombrecían funciones del usuario.** Lo detecté al
  correr la regresión: añadir `abs()` rompía cualquier programa con su propio `abs()`.
  Introducido por mi propio fix de BUG-001 y corregido antes de cerrar.
- **BUG-013 — Los builtins devolvían `void` al compilar a nativo.** Sólo estaban en la
  VM; con `lumen build --aot c` el binario imprimía `void` donde el intérprete daba el
  valor correcto. Implementados en el runtime en C con la misma semántica.
- **BUG-009 — `imprimir` con varios argumentos** emitía una línea por argumento en vez
  de una sola concatenada, lo que además rompía los mensajes de `stdlib/testing.nv`.
- **BUG-014 — `main` se ejecutaba dos veces.** Con código en el nivel superior y una
  llamada explícita a `main()`, el compilador añadía otra auto-invocación. Todo el
  cuerpo de `main` corría dos veces (efectos secundarios incluidos).
- **BUG-015 — `romper`/`continuar` rotos en `para ... en`.** Se rechazaban con
  `E070`/`E055` ("fuera de un bucle") pese a estar dentro de uno. Al corregir esa
  comprobación afloró un segundo fallo debajo: el generador de código las descartaba
  en silencio y el bucle seguía iterando igual. `mientras` y `para` clásico sí
  funcionaban, por eso había pasado inadvertido.
- **BUG-016 — Tipo inexistente daba un error confuso.** `Foo x = 5;` con `Foo` sin
  definir producía un `E031` que filtraba la representación interna del compilador
  (`Struct { name: "Foo", fields: [] }`). Ahora dice `E062 El tipo 'Foo' no está
  definido`.
- **BUG-017 — Las lambdas no podían capturar variables del entorno.** Cualquier
  closure que leyera una variable externa reventaba con `Variable '__cap_N_x' no
  definida`. El compilador registraba el renombrado del slot pero nunca emitía el
  código que lo rellenaba: los closures estaban rotos de raíz.
- **BUG-018 — Una función del usuario llamada `leer` se ignoraba en silencio.** El
  builtin de stdin la ensombrecía y devolvía `""`, así que `funcion entero leer()`
  imprimía vacío sin ningún aviso. Es la misma clase de fallo que BUG-012, pero en
  un builtin del núcleo que no había cubierto.
- **BUG-020 — `prestado mut self` no mutaba en métodos de `impl`.** La copia de
  vuelta sólo se emitía para funciones libres, así que `c.incrementar()` no tenía
  efecto sobre `c`. El caso equivalente con función libre sí funcionaba.

## Verificación

- **428 pruebas** unitarias y de regresión en verde (incluye una suite nueva de 41
  pruebas end-to-end, `crates/lumen-vm/tests/regresiones_v247.rs`).
- **45 ejemplos nuevos** en `ejemplos/`, todos ejecutándose sin errores.
- **144 ejemplos** de `examples/` comparados contra una compilación del tag `v2.4.6`
  hecha con la misma toolchain → **cero regresiones**. Las únicas diferencias son la
  corrección intencionada de `imprimir` (78 casos) y variables de entorno/FFI.
- **45 programas** de `test_agents/` sin errores.
- Paridad **VM ↔ AOT en C** verificada para todas las conversiones y builtins nuevos.
- `cargo fmt --all` y `cargo clippy --all-targets -- -D warnings` limpios.

## Documentación

`LENGUAJE.md`: nueva §3.1 de conversiones y builtins numéricos, semántica de paso de
parámetros y `prestado mut` en §3, y patrones de enumeración con datos en §9.
`CHANGELOG.md`: entrada `[2.4.7]` con los 13 bugs agrupados por categoría.

## Pendiente (no abordado)

- **Corrupción de heap en `tui_puro.nv` / `tui_jr.nv`** (`rc=134`, `free(): invalid next
  size`). **Ya está presente en v2.4.6**, no es una regresión; es un bug independiente
  que merece su propia investigación.
- Dos mejoras detectadas pero no implementadas por quedar fuera del alcance: métodos de
  instancia (`"hola".mayusculas()`, `[3,1,2].ordenar()` → `E050`) y los literales
  booleanos que se imprimen como `true`/`false` en vez de en español.
