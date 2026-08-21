# Ejemplos de LÚMEN

Colección de 45 programas cortos escritos para ejercitar el lenguaje y cazar
bugs. Todos se ejecutan sin errores con el compilador corregido.

Para ejecutar cualquiera:

```bash
lumen run ejemplos/01_basico/a01_texto_basico.nv -L stdlib
```

Para ejecutarlos todos de golpe:

```bash
for f in ejemplos/*/*.nv; do echo "== $f"; lumen run "$f" -L stdlib; done
```

## Contenido

| Carpeta | Qué cubre |
|---|---|
| `01_basico/` | Textos (vacíos, escapes, unicode), concatenación, precedencia de operadores, comparaciones, lógicos y aritmética mixta entero/decimal. |
| `02_control/` | `romper`/`continuar` en `para-cada`, bucles anidados, retorno temprano dentro de un bucle, listas vacías y recursión. |
| `03_datos/` | Listas (indexación, anidadas), tuplas, mapas vía `__map_*`, structs dentro de listas y semántica de copia por valor. |
| `04_funciones/` | Parámetros por defecto, recursión mutua, retorno de structs y listas, lambdas (captura, IIFE, retorno temprano) y `prestado mut`. |
| `05_tipos/` | Enums simples y con datos, `elegir`/`caso`, `resultado<T,E>` y `opcion<T>`. |
| `06_regresiones/` | Un caso mínimo por cada bug encontrado en esta ronda (BUG-014 a BUG-020). |
| `07_avanzado/` | Rasgos (`rasgo` + `impl ... para`), enums con datos, `opcion<T>` y `resultado<T,E>`, closures (captura, anidamiento, mutación, lambdas en bucles), métodos con `prestado mut self`, `posponer` (defer), igualdad estructural, el patrón comodín `_` en `elegir` y diccionarios `__map_*`. |
| `08_practicos/` | Programas completos y pequeños: estadísticas de una lista, manipulación de texto, una agenda con búsqueda, FizzBuzz, ordenación por burbuja y rangos/comprensiones/pipeline. |

## Notas de sintaxis que conviene recordar

Varios de estos ejemplos nacieron de equivocaciones al escribir el lenguaje.
Las formas correctas son:

- Bucle sobre lista: `para n en lista { ... }` — **sin** declarar el tipo de `n`.
- Añadir a una lista: el método `lista.agregar(x)`, no `agregar(lista, x)`.
- Lambdas: `f = funcion(entero x) { retornar x; };` — no existe la flecha `=>`.
- Enumeraciones: la palabra clave es `enum`, no `enumeracion`.
- Opcionales: `algun(valor)` y `ninguno`.
- `elegir` requiere paréntesis: `elegir (x) { ... }`.
- No existe `var` ni `+=`. Para declarar sin tipo explícito se usa `sea`.
- Los mapas no tienen literal `{"a": 1}`; se usan los builtins `__map_*`.
- En un `rasgo`, el receptor del método se escribe `este`: `funcion texto describir(este);`.
- `posponer { ... }` ejecuta su bloque al salir de la función (LIFO si hay varios),
  también cuando se sale por un `retornar` temprano.
- `==` y `!=` comparan por **contenido**, de forma recursiva: listas, listas
  anidadas, structs, tuplas, enums con datos, `exito`/`error` y `algun`/`ninguno`.
- Rangos: `a..b` excluye el extremo y `a..=b` lo incluye. Hay comprensiones
  (`[n * n para n en 1..6 si cond]`) y operador pipeline (`5 |> triplicar`).
- En `elegir`, el caso por defecto se escribe `defecto:` o con el patrón comodín
  `caso _:`; también sirve como subpatrón para ignorar un campo
  (`Punto{x: 0, y: _}`). Los casos se prueban en orden, así que el comodín va al
  final. Sobre un enum, `caso _:` no cuenta para la exhaustividad: usa `defecto:`.
- Una lambda **devuelta** por otra lambda pierde sus capturas (`Variable 'n' no
  definida`): las closures no sobreviven al marco que las creó. Capturar y usar
  la lambda dentro de la misma función sí funciona.
- Los diccionarios no tienen literal ni tipo propio: se usan los builtins
  `__map_*` y son **persistentes**. `__map_poner` no muta, devuelve un mapa
  nuevo, así que hay que reasignar (`m = __map_poner(m, "k", v);`).
- `largo()` sobre texto cuenta **caracteres**, no bytes: `largo("áéíóú ñ")` es 7.
  Hay tres formas equivalentes: `largo(s)`, `s.largo()` y `__str_longitud(s)`.
- Los textos se indexan por carácter: `"Lumen"[0]` es `"L"` y `"áéí"[1]` es `"é"`.
- Los decimales sin parte fraccionaria se imprimen sin el `.0` (`12.0` sale como `12`);
  el valor interno sí conserva la precisión.
