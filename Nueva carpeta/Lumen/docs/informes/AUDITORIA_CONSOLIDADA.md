# LÚMEN — Reporte Consolidado de Auditoría, Re-verificación y Benchmarks

**Versiones cubiertas:** v3.2.0 (auditoría inicial) → v3.5.7 (re-verificación + benchmarks)
**Periodo:** 2026-08-24 a 2026-08-30
**Autor del reporte:** auditoría manual asistida por IA (Claude), a solicitud de Omar Palomares Velasco
**Alcance:** este documento une tres sesiones de trabajo en un solo lugar: (1) la auditoría inicial de bugs sobre v3.2.0, (2) la re-verificación de esos mismos bugs sobre v3.5.7 tras el trabajo de corrección del equipo, y (3) una comparación de rendimiento entre los tres modos de ejecución del lenguaje (VM con JIT, AOT vía C/GCC, AOT vía Cranelift). Cierra con una hoja de ruta concreta de prácticas de ingeniería para reducir la tasa de regresiones hacia adelante.

**Cómo leer este documento:** las Partes 1 y 2 están pensadas para leerse en conjunto — la Parte 1 documenta el estado original de cada bug, la Parte 2 dice qué pasó con cada uno en la siguiente versión relevante. La Parte 3 es independiente y se enfoca en desempeño, no en corrección. La sección final es la única con recomendaciones prospectivas; todo lo anterior es evidencia y hallazgos.

---

## Parte 1 — Auditoría inicial (v3.2.0): bugs encontrados, corregidos y estado de features

**Build probado:** `lumen-v3.2.0-linux-x64-musl` (re-descargado 2026-08-24, mismo tag `v3.2.0`)
**Entorno:** Linux x86_64, `cc (Ubuntu 13.3.0-6ubuntu2~24.04.1)`
**Fecha:** 2026-08-24
**Metodología:** pruebas manuales de CLI, VM (`lumen run`), formateador (`lumen fmt`), checker estático (`lumen check`) y backend AOT nativo (`lumen build --native`), usando tanto ejemplos oficiales de `examples/` como programas propios diseñados para estresar bordes del lenguaje.

---

### Índice de bugs encontrados

| # | Bug | Severidad | Estado |
|---|---|---|---|
| 1 | `lumen fmt` borraba asignaciones a campos de struct | 🔴 Crítica | ✅ **Corregido** en el build re-descargado |
| 2 | `arreglo[i].campo = valor` falla en runtime en ambos backends | 🔴 Crítica | ❌ Abierto |
| 3 | `si sea` (if-let) no destructura enums de usuario con datos | 🔴 Crítica | ❌ Abierto |
| 4 | Structs auto-referenciales vía `opcion<TipoPropio>` no se pueden construir | 🔴 Crítica | ❌ Abierto |
| 5 | Funciones que dependen de imports transitivos no son resolubles desde el import raíz | 🟠 Alta | ❌ Abierto |
| 6 | Ningún mecanismo de mutación por función funciona (paso por valor, `este`, ni `prestado mut`) — afecta tanto structs como listas | 🔴 Crítica | ❌ Abierto |
| 7 | `en_tiempo_compilacion` (comptime) no evalúa en tiempo de compilación — se ejecuta en runtime como cualquier bloque normal | 🟠 Alta | ❌ Abierto |

---

### Bug #1 — `fmt` borraba asignaciones a campos de struct (CORREGIDO)

**Encontrado en la primera descarga de v3.2.0. Confirmado corregido en la re-descarga del mismo tag.**

Cualquier `obj.campo = expr;` sobre un struct ya inicializado desaparecía del archivo tras `lumen fmt`, sin error ni warning.

```lumen
estructura Cuenta { saldo: entero }
funcion entero principal() {
    Cuenta cuenta = Cuenta { saldo: 1000 };
    cuenta.saldo = 500;   // esta línea desaparecía tras `lumen fmt`
    retornar 0;
}
```

**Estado actual:** re-probado con el mismo caso mínimo y con un programa completo (constructor + fibonacci recursivo + suma de arreglo) en el build re-descargado. La línea sobrevive el formateo y el resultado (`1250`) se mantiene correcto antes y después de `fmt`, tanto en VM como en AOT nativo. **Sin evidencia de regresión.**

---

### Bug #2 — Mutación de campo dentro de arreglo indexado falla en ambos backends

**Patrón roto:** `arreglo[i].campo = valor;` donde `arreglo` es `lista<StructT>`.

```lumen
estructura Persona { nombre: texto, edad: entero }
funcion entero principal() {
    lista<Persona> gente = [
        Persona { nombre: "Ana", edad: 30 },
        Persona { nombre: "Beto", edad: 25 }
    ];
    gente[1].edad = 26;   // rompe aquí
    imprimir(a_texto(gente[1].edad));
    retornar 0;
}
```

- `lumen check`: ✓ dice que es válido (no detecta el problema).
- `lumen run`: ❌ `Error de tipo: StructGet requires struct value`
- `lumen build --native`: compila el binario sin error, pero al **ejecutarlo** falla con `Campo 'edad' no encontrado en struct` — mensaje distinto al de la VM, lo que sugiere que son dos implementaciones separadas con el mismo bug conceptual.
- La **lectura** del mismo patrón (`entero e = gente[1].edad;`) funciona perfecto en ambos backends — el bug es específico de usarlo como **lvalue** (destino de asignación).
- El bug es independiente del índice usado (falla igual en `gente[0]` y `gente[1]`).

**Hipótesis:** el resolver de lvalues contempla `variable.campo = valor` pero no la cadena `IndexAccess -> FieldAccess = valor`.

---

### Bug #3 — `si sea` (if-let) no soporta destructuring de enums de usuario con datos

El ejemplo oficial `examples/fase62_if_let.nv` demuestra `si sea algun(n) = opt { ...usar n... }` funcionando para el tipo builtin `opcion<T>`. El mismo patrón **no funciona** para un `enum` de usuario con variantes con datos — que es justo el ejemplo estrella de `examples/enums.nv`.

```lumen
enum Resultado { Exitoso(entero), Pendiente }
funcion entero principal() {
    Resultado r = Resultado::Exitoso(42);
    si sea Resultado::Exitoso(valor) = r {
        imprimir(a_texto(valor));   // 'valor' nunca se registra en el scope
    }
    retornar 0;
}
```

Probé las dos formas razonables del patrón, y **ambas fallan de forma distinta:**
- **Calificada** (`Resultado::Exitoso(valor)`): `check` rechaza con `E033: La variable 'valor' no está declarada`.
- **Sin calificar** (`Exitoso(valor)`): `check` la acepta como válida, pero `run` falla con `Error: Variable 'Exitoso' no definida` (lo interpreta como llamada a función suelta, no como patrón de variante).

**Hipótesis:** `opcion<T>` está implementado como caso especial hardcodeado de pattern-matching en el compilador, y ese mecanismo nunca se generalizó al sistema de `enum` de usuario, a pesar de ser sintácticamente análogo.

**Nota relacionada (no es bug, es ausencia de feature):** `elegir/caso` tampoco destructura variantes con datos, pero no hay evidencia en los ejemplos oficiales de que esté pensado para hacerlo (el único ejemplo, `match.nv`, solo matchea por valor literal contra un `entero`). Actualmente **ningún** mecanismo de control de flujo en LÚMEN permite destructurar con éxito una variante de enum de usuario con datos.

---

### Bug #4 — Structs auto-referenciales vía `opcion<TipoPropio>` no se pueden construir

El patrón estándar para listas ligadas y árboles (`campo: opcion<Self>`) está completamente roto, incluso en el caso base trivial:

```lumen
estructura Nodo {
    valor: entero,
    siguiente: opcion<Nodo>,
}
funcion entero principal() {
    Nodo n2 = Nodo { valor: 2, siguiente: ninguno };  // ya falla aquí
    retornar 0;
}
```

```
E031 El campo 'siguiente' espera un valor de tipo 'Opcion(Struct { name: "Nodo", fields: [] })',
     no 'Opcion(Struct { name: "Nodo", fields: [("valor", Entero), ("siguiente", ...)] })'
```

El propio mensaje delata el bug: el checker compara `Nodo` contra sí mismo, pero una copia tiene `fields: []` (vacía) y la otra los campos reales. Confirmado que **no depende de instanciar recursión real de datos** — falla incluso con el caso base (`ninguno`) y con una variable intermedia explícitamente tipada como `opcion<Nodo>`.

**Hipótesis:** problema clásico de "tying the knot" en registro de tipos recursivos — al procesar `estructura Nodo { ... opcion<Nodo> ... }`, el compilador registra un placeholder vacío de `Nodo` para resolver la referencia circular, y nunca lo actualiza con los campos reales tras terminar de parsear el struct completo.

**Impacto:** bloquea por completo listas ligadas, árboles binarios, grafos — cualquier estructura de datos recursiva vía `opcion<T>`.

---

### Bug #5 — Funciones que dependen de imports transitivos no son resolubles desde el import raíz

Si `a.nv` importa `b.nv` y define una función que internamente llama a algo de `b.nv`, esa función de `a.nv` **no puede usarse** desde un tercer archivo `main.nv` que solo importa `a.nv` — a pesar de que `main.nv` nunca toca `b.nv` directamente.

**`b.nv`:**
```lumen
funcion entero b_valor() {
    retornar 20;
}
```

**`a.nv`:**
```lumen
importar "b.nv";

funcion entero a_valor() {
    retornar 10;
}

funcion entero usar_b() {
    retornar b_valor() + 1;
}
```

**`main.nv`:**
```lumen
importar "a.nv";

funcion entero principal() {
    imprimir(a_texto(usar_b()));   // rompe aquí
    retornar 0;
}
```

```
E042 La función 'b_a_valor' no está definida
  --> main.nv:8:14
```

**Aislamiento del patrón:** no es que los imports no sean transitivos en general — una función de `a.nv` que **no** depende de `b.nv` (como `a_valor()`) sí es completamente visible y usable desde `main.nv` sin problema, incluso con el import interno de `b.nv` presente en `a.nv`. El bug es específico de funciones cuyo cuerpo depende de un import de segundo nivel.

**Detalles adicionales sospechosos:**
- El nombre de función buscado en el error, `b_a_valor`, **no existe en ningún archivo** del caso de prueba — parece una concatenación incorrecta de prefijos de módulo al aplanar la cadena de imports (probablemente `b_` + `a_valor` o similar).
- El número de línea reportado (`main.nv:8`) no corresponde al archivo real, que solo tiene 5 líneas — sugiere que el tracking de ubicación usa la línea de otro archivo (`a.nv` o `b.nv`) pero la etiqueta incorrectamente con el nombre del archivo raíz.
- El segundo error (`E035: Operador aritmético requiere números, no 'Void' y 'Entero'`) es ruido derivado del primero, no un problema adicional.

**Hipótesis:** el sistema de "namespacing" que califica funciones importadas (ej. `modulo_funcion`) concatena prefijos incorrectamente cuando la cadena de resolución pasa por más de un nivel de import, en vez de resolver el nombre final una sola vez contra la tabla de símbolos ya aplanada.

---

### Bug #6 — `este` en métodos de `impl Rasgo para Struct` no persiste mutaciones

Cuando un método definido en `impl Rasgo para MiStruct` recibe `este` y muta uno de sus campos (`este.campo = valor;`), el cambio **no se refleja** en la variable original tras la llamada. `este` se comporta como una copia local descartable en vez de una referencia al receptor.

```lumen
estructura Contador {
    valor: entero,
}

rasgo Incrementable {
    funcion vacio incrementar(este);
}

impl Incrementable para Contador {
    funcion vacio incrementar(este) {
        este.valor = este.valor + 1;
    }
}

funcion entero principal() {
    Contador c = Contador { valor: 0 };
    c.incrementar();
    c.incrementar();
    c.incrementar();
    imprimir("Valor: " + a_texto(c.obtener()));  // imprime "Valor: 0", debería ser 3
    retornar 0;
}
```

**Confirmado en ambos backends** (VM y AOT nativo `build --native`) con el mismo resultado incorrecto.

**Comparación directa que aísla el bug con precisión** — en el mismo programa, mutar el campo directamente sí funciona, pero mutar el mismo campo vía un método de `impl` no tiene ningún efecto:

```lumen
c.valor = 500;               // funciona: c.valor pasa a ser 500
c.incrementar();              // internamente hace este.valor = 999;
// tras esto, c.valor sigue siendo 500, no 999
```

Esto descarta cualquier duda: no es un problema del valor inicial, del tipo de dato, ni de cuántas veces se llama — es que **la asignación a un campo de `este` dentro de un método de `impl` nunca se propaga de vuelta al objeto original**, en ningún caso probado.

**Por qué es crítico:** invalida el patrón de "objetos con estado mutable a través de métodos", que es precisamente la capacidad que el propio ejemplo oficial `44_extension_methods.nv` (Extension Methods / traits) presenta como feature central del sistema de tipos de LÚMEN. Sin esto, cualquier abstracción OOP con estado (contadores, colecciones, builders, máquinas de estado) es inutilizable si se construye sobre `impl ... para Struct` con métodos que mutan.

**Hipótesis confirmada con una prueba adicional:** agregando un `imprimir(este.valor)` justo después de la mutación, dentro del propio método, se ve `1` (el cambio sí ocurrió). Pero justo después de que `principal()` recupera el control, `c.valor` vuelve a ser `0`. Esto confirma sin ambigüedad que `este` se pasa **por valor** (copia completa del struct) al invocar el método, la mutación ocurre correctamente sobre esa copia local, y la copia modificada simplemente se descarta al terminar el método en vez de escribirse de vuelta al struct original.

**No es específico de `impl`/traits — es un problema general de paso de structs a cualquier función.** Confirmé que el mismo patrón falla igual con una función libre (sin `impl` de por medio):

```lumen
funcion vacio incrementar_libre(Contador c) {
    c.valor = c.valor + 1;
}
// tras llamar incrementar_libre(c) dos veces, c.valor sigue en 0
```

**Y el mecanismo que debería resolver esto (`prestado mut`, el borrow-checker documentado en `ESPECIFICACION_FORMAL_LUMEN.md`) está roto de una forma distinta y aún más básica:** intentar usar `prestado mut Contador` como tipo de parámetro para poder mutar el struct del llamador falla con un error de tipos — el compilador no permite acceder a campos de un valor `prestado mut`, tratándolo como un wrapper opaco sin auto-desreferencia:

```lumen
funcion vacio incrementar_ref(prestado mut Contador c) {
    c.valor = c.valor + 1;   // falla aquí
}
```

```
E060 No puedes acceder a un campo de un valor de tipo
     'Prestado { inner: Struct { name: "Contador", fields: [("valor", Entero)] }, mutable: true }'
  --> mut_prestado.nv:6:15
  Ayuda: Solo los structs tienen campos
```

El mensaje de ayuda ("Solo los structs tienen campos") es, en sí mismo, la evidencia del bug: `Prestado { inner: Struct{...}, mutable: true }` **envuelve** exactamente un struct, pero el compilador no lo reconoce como tal para efectos de acceso a campos — falta la desreferenciación automática (`c.valor` debería resolverse como `(*c).valor` de forma transparente, tal como ocurre en Rust con `&mut T`).

**Conclusión combinada:** actualmente **no existe ninguna forma funcional en LÚMEN de mutar un struct a través de una llamada a función** — ni pasándolo por valor a una función libre, ni a través de `este` en un método de `impl`, ni usando el mecanismo explícito de borrow (`prestado mut`) diseñado para este propósito exacto. Los tres caminos posibles están rotos, cada uno de forma distinta.

**El problema no es exclusivo de structs — afecta también a `lista<T>` (y probablemente a cualquier tipo compuesto).** Repetí los mismos tres experimentos con listas:

```lumen
funcion vacio agregar_elemento(lista<entero> l, entero valor) {
    l.agregar(valor);
}
// tras agregar_elemento(numeros, 4), numeros.largo() sigue igual — el push se pierde
```
```lumen
funcion vacio duplicar_primero(lista<entero> l) {
    l[0] = l[0] * 2;
}
// tras duplicar_primero(numeros), numeros[0] no cambia — la mutación por índice se pierde igual
```

Y el mecanismo de borrow explícito falla con el mismo patrón exacto que con structs, solo que ahora sobre llamada a método en vez de acceso a campo:

```lumen
funcion vacio agregar_ref(prestado mut lista<entero> l, entero valor) {
    l.agregar(valor);   // falla aquí
}
```
```
E047 No puedes llamar 'agregar' en un valor de tipo 'Prestado { inner: Lista(Entero), mutable: true }'
  Ayuda: 'agregar' solo se puede llamar en listas
```

**Esto generaliza la causa raíz del problema:** `prestado`/`prestado mut` nunca se auto-desreferencia hacia el tipo interior (`inner`), sin importar si ese tipo interior es un struct (acceso a campo) o una lista (llamada a método). El "wrapper" `Prestado { inner: T, mutable: bool }` se trata de forma completamente opaca en el checker de tipos — nunca hay un paso que diga "si necesito acceder a un campo/método de T y tengo un `Prestado<T>`, debo desenvolver automáticamente". Esto sugiere que el borrow-checker está implementado a nivel de anotación de tipo (probablemente para el análisis de aliasing/ownership descrito en `ESPECIFICACION_FORMAL_LUMEN.md`), pero la fase de resolución de miembros (campos y métodos) nunca fue conectada a esa envoltura de tipo.

**Alcance final:** ni structs ni listas pueden mutarse a través de ninguna llamada a función (paso por valor, `este`, o `prestado mut`) en el estado actual de LÚMEN v3.2.0. Esto afecta la totalidad de los tipos compuestos con estado mutable del lenguaje.

**Mitigación disponible (workaround funcional confirmado):** retornar el struct/lista modificado y reasignarlo en el llamador **sí funciona correctamente** en ambos casos:
```lumen
funcion Contador incrementar(Contador c) {
    c.valor = c.valor + 1;
    retornar c;
}
// uso: c = incrementar(c);  -> funciona, c.valor se actualiza correctamente
```
```lumen
funcion lista<entero> agregar_y_retornar(lista<entero> l, entero valor) {
    l.agregar(valor);
    retornar l;
}
// uso: numeros = agregar_y_retornar(numeros, 4);  -> funciona, largo() se actualiza
```
Esto significa que el bug #6 no bloquea el lenguaje por completo — un estilo puramente funcional (retornar el valor nuevo en vez de mutar in-place) es una mitigación viable mientras no se corrija el paso por referencia. Vale la pena que quede documentado como workaround recomendado hasta que se arregle.

---

### Bug #7 — `en_tiempo_compilacion` (comptime) no evalúa en tiempo de compilación

El propio ejemplo oficial `demo_fronteras_totales.nv` presenta `en_tiempo_compilacion { ... }` bajo el comentario `// Zero Runtime Cost`, prometiendo evaluación durante la compilación (metaprogramación comptime, al estilo Zig). En la práctica, el bloque se ejecuta **en el mismo momento que el resto del programa**, intercalado con la ejecución normal — es decir, en runtime, no en tiempo de compilación.

**Prueba concluyente (orden de ejecución observable):**

```lumen
funcion entero con_efecto() {
    imprimir("EFECTO: esta funcion se llamo");
    retornar 99;
}

funcion entero principal() {
    imprimir("--- antes de declarar x ---");
    entero x = en_tiempo_compilacion { con_efecto() };
    imprimir("--- despues de declarar x, x = " + a_texto(x) + " ---");
    retornar 0;
}
```

Salida real:
```
--- antes de declarar x ---
EFECTO: esta funcion se llamo
--- despues de declarar x, x = 99 ---
```

Si `en_tiempo_compilacion` evaluara de verdad en tiempo de compilación, el mensaje `"EFECTO: ..."` debería aparecer *antes* de cualquier salida del programa (idealmente durante `lumen check`/`lumen build`, no durante `lumen run`), o el compilador debería insertar directamente la constante `99` sin ejecutar ningún side effect visible en runtime. En cambio, aparece exactamente en el punto del flujo de ejecución donde está escrito el bloque — indistinguible de un bloque `{ con_efecto() }` normal sin la palabra clave.

**Nota:** el bloque `en_tiempo_compilacion { ... }` solo acepta una única expresión (no una secuencia de statements con `;`); intentar poner múltiples statements produce errores de sintaxis (`E015`, `E012`, `E020`). Esto por sí solo no es un bug — es razonable que un bloque comptime simple solo evalúe una expresión — pero confirma que la limitación sintáctica no viene acompañada de la semántica de evaluación anticipada que el nombre promete.

**Impacto:** cualquier uso de `en_tiempo_compilacion` como optimización real (evitar cálculo repetido en runtime, folding de constantes complejas, validación en tiempo de compilación) no tiene ningún efecto — es puramente cosmético en el estado actual del compilador. El caso simple mostrado en `demo_fronteras_totales.nv` (`(1024 * 1024) / 16 + 42`) da el resultado numérico correcto, pero eso es indistinguible de evaluación en runtime para una expresión sin side effects — el bug solo se revela con un side effect observable como el de arriba.

**Sugerencia de investigación:** revisar si existe algún paso de folding/evaluación anticipada para el nodo `ComptimeBlock` en el pipeline de compilación, o si actualmente `en_tiempo_compilacion { expr }` se desazucara directamente a `{ expr }` sin ninguna fase adicional. Dado que el ejemplo con aritmética simple sí "funciona" (da el resultado correcto), es posible que el parser reconozca la sintaxis pero el resto del pipeline (checker, codegen) simplemente ignore la semántica especial y trate el bloque como código normal.

---

### Lo que funciona correctamente ✅

Confirmado con pruebas manuales en esta sesión, tanto en VM como (donde aplica) en AOT nativo:

- **Instalación y CLI base:** descarga del binario musl (0-deps), `lumen doctor`, `lumen --version`, `lumen check`, `lumen run`, `lumen fmt`, `lumen build --native` — todos operativos.
- **`fmt`:** ya no borra código (bug #1 corregido).
- **Structs simples:** creación, lectura y mutación de campos en variable directa (`obj.campo = valor`) — funciona correctamente, incluyendo tras pasar por `fmt`.
- **Structs anidados (no recursivos):** un struct como campo de otro struct distinto (`Rectangulo { esquina: Punto, ... }`) funciona correctamente, incluyendo mutación de campos anidados (`r.esquina.x = 99;`) y semántica de copia por valor al asignar entre variables.
- **Arreglos de structs — lectura:** acceso directo por índice y en bucle con índice dinámico funcionan bien. (La mutación de un campo de struct dentro de un arreglo es el bug #2.)
- **Listas — mutación dentro del mismo scope/función local:** `numeros.agregar(x)` y `numeros[i] = valor` funcionan correctamente cuando se hacen directamente en la función donde la lista fue declarada (confirmado en `examples/arrays.nv` y en varias pruebas propias). El problema (bug #6) es específico de pasar la lista **a otra función** e intentar mutarla ahí.
- **Genéricos:** funciones genéricas (`funcion T identidad<T>(T valor)`) y structs genéricos (`estructura Par<T, U>`) funcionan correctamente, confirmado con el ejemplo oficial `genericos.nv`.
- **`opcion<T>` (tipo builtin):** `algun(x)`, `ninguno`, comparación (`==`, `!=`), `si sea algun(n) = opt { }`, y uso dentro de `elegir/caso` — todo funciona correctamente.
- **`resultado<T,E>` y `intentar` (propagación de error tipo `?`):** `exito(x)`, `error(msg)`, y `intentar expr` para propagar errores funcionan correctamente, incluso usados dentro de expresiones aritméticas compuestas (`(intentar f(a)) + (intentar f(b))`).
- **Enums simples (sin destructuring):** declaración, construcción de variantes con y sin datos (`Color::Rojo`, `Resultado::Exitoso(100)`), e impresión — funcionan correctamente. El problema es específico de intentar destructurar la variante (bug #3).
- **`elegir/caso` (match C-style):** matching por valor literal contra `entero` funciona correctamente.
- **Recursión de funciones:** fibonacci recursivo probado extensamente, sin problemas, en VM y AOT nativo.
- **Corutinas:** el ejemplo oficial `corutinas_demo.nv` corre sin problemas, incluyendo múltiples corrutinas intercaladas con `yield`.
- **Backend AOT nativo (`build --native`, C -O3):** genera binarios funcionales y correctos para todo el código que no toca los bugs #2 o #4.
- **Sistema de módulos — import de un solo nivel:** `importar "archivo.nv";` seguido de uso directo de una función definida en ese archivo funciona correctamente, incluso si el archivo importado a su vez tiene sus propios imports internos (mientras no se use una función que dependa de ellos).
- **Rasgos (`rasgo`) y `impl Rasgo para Tipo` — declaración y despacho de métodos:** el ejemplo oficial `44_extension_methods.nv` (extension methods sobre `entero`/`texto`, sin mutación de estado) funciona perfectamente. El despacho de método (`n.a_formato()`) y la resolución del `impl` correcto según el tipo funcionan bien — el problema es específico de la persistencia de mutaciones sobre `este` (bug #6), no de la mecánica de traits en sí.
- **Closures / lambdas — captura y mutación de variables del entorno:** a diferencia de structs y listas pasadas a funciones (bug #6), las lambdas (`funcion(...) { ... }`) sí capturan variables externas correctamente **por referencia**, incluyendo mutación que persiste entre llamadas. Confirmado con un patrón de "contador vía closure" ejecutado varias veces dentro de un bucle — cada llamada ve el valor actualizado por la llamada anterior. Este es un mecanismo de captura distinto (probablemente celdas/upvalues) del que se usa para pasar parámetros normales a funciones, y funciona correctamente donde el otro no.
- **`en_tiempo_compilacion` (comptime) — sintaxis y aritmética simple:** el bloque se parsea correctamente y produce el resultado numérico correcto para expresiones aritméticas puras sin side effects. (La ejecución real en tiempo de compilación, que es la promesa central de la feature, es el bug #7.)

### Lo que no funciona / está roto ❌

- **`arreglo[i].campo = valor`** (bug #2) — rompe en VM y AOT nativo, no detectado por `check`.
- **`si sea Enum::Variante(x) = valor`** para enums de usuario con datos (bug #3) — no hay forma que funcione de punta a punta.
- **Structs auto-referenciales vía `opcion<TipoPropio>`** (bug #4) — imposible construir ni el caso base.
- **Funciones que dependen de imports transitivos** (bug #5) — no resolubles desde el archivo raíz, con mensajes de error que citan nombres de función inexistentes y números de línea incorrectos.
- **Mutación de `este` dentro de métodos de `impl Rasgo para Struct`** (bug #6) — se pasa por valor y la mutación se descarta al salir del método; lo mismo ocurre con funciones libres y con `prestado mut`; afecta también a `lista<T>` pasada a funciones. Invalida cualquier patrón de estado mutable encapsulado en una función.
- **`en_tiempo_compilacion` (comptime)** (bug #7) — no evalúa realmente en tiempo de compilación, se ejecuta en runtime intercalado con el resto del programa; feature de fachada en el estado actual.
- **Inconsistencia de sintaxis** entre orden de campos de struct (`nombre: Tipo`) y parámetros de función (`Tipo nombre`) — no es un bug funcional, pero genera fricción real y no está claramente documentada.
- **`--help` del CLI** no ejemplifica la convención real de parámetros ni aclara la ausencia de métodos inherentes (`impl` sin trait) en los ejemplos incluidos.

---

### Prioridad sugerida de corrección

1. **Bug #6 (ningún mecanismo de mutación por función funciona: ni valor, ni `este`, ni `prestado mut`, afectando structs y listas por igual)** — el más urgente de todos: bloquea cualquier código con abstracciones de estado mutable (contadores, colecciones, builders), y la causa raíz (falta de auto-desreferenciación de `Prestado<T>`) es transversal a todo el sistema de tipos compuestos, no solo a `impl`.
2. **Bug #4 (structs recursivos)** — bloquea una categoría entera de estructuras de datos, con causa raíz ya identificable desde el propio mensaje de error.
3. **Bug #2 (mutación en arreglos indexados)** — patrón de código muy común, y el hecho de que `check` no lo detecte es particularmente peligroso.
4. **Bug #3 (`si sea` con enums de usuario)** — rompe la promesa central de "tipos suma con pattern matching" que el lenguaje presenta como feature estrella.
5. **Bug #5 (imports transitivos)** — afecta cualquier proyecto real dividido en más de dos archivos con dependencias en cadena; los mensajes de error engañosos (nombre de función inexistente, línea incorrecta) agravan el problema.
6. **Bug #7 (comptime no evalúa en tiempo de compilación)** — menos urgente que los anteriores porque no bloquea código (todo "funciona" en runtime), pero es una feature vendida como pilar del lenguaje que actualmente no cumple su promesa; vale la pena decidir si se implementa de verdad o se retira temporalmente de la documentación hasta que lo esté, para no generar expectativas falsas.
7. Limpieza de consistencia sintáctica y documentación (no bloqueante, pero afecta la curva de aprendizaje).

---

## Parte 2 — Re-verificación en v3.5.7: qué se corrigió, qué sigue abierto, y 2 bugs nuevos

**Build probado:** `lumen-v3.5.7-linux-x64-musl`
**Build de referencia:** `lumen-v3.2.0-linux-x64-musl` (re-descarga del 2026-08-24)
**Fecha:** 2026-08-30
**Metodología:** se tomó cada uno de los 7 bugs documentados contra v3.2.0 y se re-ejecutó el mismo caso mínimo de reproducción contra v3.5.7, sin modificar el código de prueba salvo donde fue necesario para aislar un comportamiento nuevo.

---

### Resumen ejecutivo

De los 6 bugs que seguían abiertos al cierre de la sesión anterior, **4 quedaron completamente resueltos**, 1 tiene una mejora parcial real pero el problema de fondo persiste, y 1 no muestra cambios. Además, se descubrió **1 bug nuevo** al profundizar en la re-verificación del bug #4, y **1 regresión nueva** en el backend AOT nativo al re-verificar el bug #2.

| # | Bug | Estado en v3.2.0 | Estado en v3.5.7 |
|---|---|---|---|
| 1 | `fmt` borraba asignaciones a campos de struct | ✅ Corregido (ya en v3.2.0 re-descarga) | ✅ Sigue corregido |
| 2 | `arreglo[i].campo = valor` falla en runtime | ❌ Abierto | ✅ **Corregido en VM** / ❌ **Nueva regresión en AOT nativo** (ver Bug #8) |
| 3 | `si sea` no destructura enums de usuario con datos | ❌ Abierto | ✅ **Completamente corregido** |
| 4 | Structs auto-referenciales vía `opcion<Self>` no se pueden construir | ❌ Abierto | ✅ **Construcción corregida** / ❌ **Nuevo bug en destructuring** (ver Bug #9) |
| 5 | Funciones que dependen de imports transitivos no resolubles | ❌ Abierto | ⚠️ **Mejora parcial** — mensajes de error correctos, pero el bug funcional persiste |
| 6 | Ningún mecanismo de mutación por función funciona | ❌ Abierto | ✅ **Corregido vía `prestado mut`** (paso por valor sigue copiando, por diseño) |
| 7 | `en_tiempo_compilacion` no evalúa en tiempo de compilación | ❌ Abierto | ❌ **Sin cambios** |

---

### Bug #1 — `fmt` borraba asignaciones a campos de struct — ✅ Sigue corregido

No re-probado exhaustivamente en esta ronda (ya se había confirmado corregido en la re-descarga de v3.2.0); no hay indicios de regresión en el uso general de `fmt` durante esta sesión.

---

### Bug #2 — Mutación de campo en arreglo indexado — ✅ Corregido en VM, ❌ regresión en AOT nativo

```lumen
estructura Persona { nombre: texto, edad: entero }
funcion entero principal() {
    lista<Persona> gente = [
        Persona { nombre: "Ana", edad: 30 },
        Persona { nombre: "Beto", edad: 25 }
    ];
    gente[1].edad = 26;
    imprimir(a_texto(gente[1].edad));
    retornar 0;
}
```

- **`lumen run` (VM):** ✅ imprime `26` correctamente. **El bug original está resuelto.**
- **`lumen build --native`:** ❌ el mismo programa **ya no compila**. El C generado tiene errores de tipo:
```
bug2.c:2589:40: error: incompatible types when returning type 'Val' but 'long long int' was expected
  if (__builtin_expect(_err,0)) return _v_void();
```
Esto es un bug **nuevo** respecto a v3.2.0 (donde el mismo caso sí compilaba, aunque fallaba al ejecutarse con un mensaje distinto). Ver detalle en **Bug #8** más abajo.

---

### Bug #3 — `si sea` no destructura enums de usuario con datos — ✅ Completamente corregido

```lumen
enum Resultado { Exitoso(entero), Fallido(texto), Pendiente }
funcion vacio procesar(Resultado r) {
    si sea Resultado::Exitoso(valor) = r { imprimir("Exito con valor: " + a_texto(valor)); retornar; }
    si sea Resultado::Fallido(msg) = r { imprimir("Fallo: " + msg); retornar; }
    imprimir("Pendiente u otro");
}
funcion entero principal() {
    procesar(Resultado::Exitoso(42));
    procesar(Resultado::Fallido("timeout"));
    procesar(Resultado::Pendiente);
    retornar 0;
}
```

Salida en v3.5.7:
```
Exito con valor: 42
Fallo: timeout
Pendiente u otro
```

Las tres variantes (con dato entero, con dato texto, y sin datos) se destructuran y despachan correctamente. **Sin reservas — este bug está resuelto de raíz.**

---

### Bug #4 — Structs auto-referenciales — ✅ Construcción corregida, ❌ nuevo bug en destructuring (Bug #9)

**La parte original del bug (construcción) está resuelta:**
```lumen
estructura Nodo { valor: entero, siguiente: opcion<Nodo> }
funcion entero principal() {
    Nodo n2 = Nodo { valor: 2, siguiente: ninguno };
    Nodo n1 = Nodo { valor: 1, siguiente: algun(n2) };
    imprimir(a_texto(n1.valor));   // funciona: imprime 1
    retornar 0;
}
```
`check` y `run` ya no fallan al construir el struct recursivo — el bug de "tying the knot" documentado en la sesión anterior está corregido.

**Pero surge un problema nuevo al intentar leer el valor anidado vía destructuring**, ver **Bug #9**.

---

### Bug #5 — Imports transitivos — ⚠️ Mejora parcial, bug funcional persiste

```lumen
// b.nv
funcion entero b_valor() { retornar 20; }

// a.nv
importar "b.nv";
funcion entero a_valor() { retornar 10; }
funcion entero usar_b() { retornar b_valor() + 1; }

// main.nv
importar "a.nv";
funcion entero principal() { imprimir(a_texto(usar_b())); retornar 0; }
```

```
E042 La función 'usar_b' no está definida
  --> main.nv:4:22
```

**Lo que mejoró:** el nombre de función en el error ahora es el correcto (`usar_b`, no el nombre corrompido `b_a_valor` que se veía en v3.2.0), y el número de línea (`main.nv:4`) ahora corresponde realmente al archivo señalado. Ambos defectos de diagnóstico reportados anteriormente están corregidos.

**Lo que sigue roto:** `usar_b()` — una función de `a.nv` que depende de un import transitivo de `b.nv` — sigue sin poder resolverse desde `main.nv`. El bug funcional de fondo no tiene cambios; solo mejoró la calidad del mensaje de error.

---

### Bug #6 — Mutación de structs/listas por función — ✅ Corregido vía `prestado mut`

Se re-probaron las 4 variantes documentadas en la sesión anterior:

| Variante | v3.2.0 | v3.5.7 |
|---|---|---|
| Struct por valor a función libre | ❌ No persiste | ❌ No persiste (correcto por diseño — paso por valor debe copiar) |
| `este` sin anotar en método de `impl` | ❌ No persiste | ❌ No persiste (comportamiento consistente con "por valor" salvo que se pida referencia) |
| `prestado mut Contador` en función libre | ❌ Error de tipos (`Prestado` opaco) | ✅ **Funciona correctamente** |
| `prestado mut lista<entero>` en función libre | ❌ Error de tipos (`Prestado` opaco) | ✅ **Funciona correctamente** |
| `prestado mut este` en método de `impl` | No probado en v3.2.0 (bloqueado por el bug de `este` simple) | ✅ **Funciona correctamente** |

Ejemplo confirmado funcionando en v3.5.7:
```lumen
impl Incrementable para Contador {
    funcion vacio incrementar(prestado mut este) {
        este.valor = este.valor + 1;
    }
}
// tras 3 llamadas a c.incrementar(), c.obtener() devuelve 3 correctamente
```

**Este es el arreglo más significativo de la sesión.** El mecanismo de borrow-checker (`prestado`/`prestado mut`) ya no es una anotación decorativa — ahora permite mutación real tanto en funciones libres como en métodos de `impl`, para structs y para listas. El paso por valor simple (sin `prestado`) sigue sin mutar el original, pero eso es el comportamiento correcto y esperado de un paso por valor — ya no es un bug, es la contraparte correcta de tener ahora un mecanismo de referencia que sí funciona.

---

### Bug #7 — `en_tiempo_compilacion` no evalúa en tiempo de compilación — ❌ Sin cambios

Mismo caso de prueba, mismo resultado que en v3.2.0:
```lumen
funcion entero con_efecto() {
    imprimir("EFECTO: esta funcion se llamo");
    retornar 99;
}
funcion entero principal() {
    imprimir("--- antes de declarar x ---");
    entero x = en_tiempo_compilacion { con_efecto() };
    imprimir("--- despues de declarar x, x = " + a_texto(x) + " ---");
    retornar 0;
}
```
Salida (idéntica a v3.2.0):
```
--- antes de declarar x ---
EFECTO: esta funcion se llamo
--- despues de declarar x, x = 99 ---
```
El bloque `en_tiempo_compilacion` sigue ejecutándose en el mismo punto del flujo de runtime, no antes. Sin cambios respecto a la sesión anterior.

---

### Bug #8 (NUEVO) — Regresión: `build --native` ya no compila código con mutación de struct dentro de lista indexada

**Severidad:** 🟠 Alta (regresión — el mismo caso compilaba, aunque incorrectamente, en v3.2.0)

Al re-verificar el bug #2 (ya corregido en la VM), se descubrió que el backend AOT nativo ahora **falla en tiempo de compilación de C** para el mismo programa:

```bash
lumen build --native bug2.nv
```
```
bug2.c: In function '_f_principal':
bug2.c:2589:40: error: incompatible types when returning type 'Val' but 'long long int' was expected
 2589 |   if (__builtin_expect(_err,0)) return _v_void();
bug2.c:2596:40: error: incompatible types when returning type 'Val' but 'long long int' was expected
bug2.c:2600:40: error: incompatible types when returning type 'Val' but 'long long int' was expected
Error compilacion C (exit exit status: 1)
```

El patrón del error (`return _v_void();` en una función que se espera devuelva `long long int`) sugiere que el arreglo del bug #2 en el path del checker/VM introdujo una ruta de código (probablemente relacionada con el manejo de errores de `StructSet` sobre elementos de listas indexadas) cuya firma de tipo en el C generado no coincide con lo que el resto del codegen espera para esa función — un desajuste de tipos entre el "wrapper" genérico `Val` (usado para acomodar el nuevo camino de manejo de errores) y el tipo de retorno primitivo esperado.

**Impacto:** cualquier programa que use el patrón recién arreglado (`arreglo[i].campo = valor`) y que se intente compilar con `--native` fallará — es decir, el arreglo del bug #2 solo es utilizable hoy en la VM, no en producción vía AOT.

**Sugerencia:** revisar el codegen C para la asignación a campos de struct dentro de acceso indexado — específicamente la conversión entre el tipo `Val` (que parece envolver resultados que pueden fallar, ligado al sistema de manejo de errores) y el tipo de retorno nativo de la función generada.

---

### Bug #9 (NUEVO) — `si sea algun(x) = valor` infiere mal el tipo del binding cuando `T` en `opcion<T>` es un struct

**Severidad:** 🔴 Crítica (bloquea el caso de uso principal para el que se había arreglado el bug #4 — construir un struct recursivo ahora funciona, pero leerlo de vuelta no)

```lumen
estructura Punto { x: entero, y: entero }
funcion entero principal() {
    opcion<Punto> op = algun(Punto { x: 10, y: 20 });
    si sea algun(p) = op {
        imprimir(a_texto(p.x));
    }
    retornar 0;
}
```
```
E060 No puedes acceder a un campo de un valor de tipo 'Numero'
  --> bug_opcion_struct.nv:9:26
   si sea algun(p) = op {
       imprimir(a_texto(p.x));
                        ^^
  Ayuda: Solo los structs tienen campos
```

**Confirmado que no es específico de recursividad:** el mismo error ocurre con un struct simple no auto-referencial (`Punto`), así que el problema es general a `si sea algun(x) = valor` donde `valor: opcion<StructT>` para cualquier `StructT`, no solo en el contexto de listas ligadas.

**Hipótesis:** el binding `p` dentro de `si sea algun(p) = op` parece estar recibiendo un tipo hardcodeado (`Numero`, quizás un default genérico previo a la implementación completa de inferencia de tipos para el payload de `opcion<T>`) en vez de inferir correctamente `T` a partir del tipo declarado de `op` (`opcion<Punto>`). Es posible que este bug sea consecuencia colateral del trabajo reciente para arreglar el bug #3 (`si sea` con enums de usuario) — el nuevo mecanismo de inferencia de tipos para bindings de patrón podría no haberse generalizado correctamente al caso donde el payload es un struct en vez de un entero/texto.

**Impacto:** el arreglo del bug #4 (construcción de structs recursivos) es solo parcialmente utilizable — se puede **construir** una lista ligada, pero no se puede **recorrer** usando el mecanismo idiomático (`si sea algun(nodo) = actual.siguiente`), que es precisamente cómo se recorre este tipo de estructura en la práctica.

**Sugerencia:** revisar la misma fase de inferencia de tipos de bindings de patrón que se tocó para el bug #3, específicamente el caso donde el patrón es `algun(x)` (built-in `opcion<T>`) y `T` es un tipo struct definido por el usuario, no un primitivo.

---

### Balance general de esta ronda

**Lo bueno:** el equipo resolvió 3 de los 6 bugs abiertos de forma completa y sólida (#3, #4-construcción, #6), y mejoró significativamente el diagnóstico de un cuarto (#5) aunque el problema de fondo persista. El arreglo del bug #6 en particular es un cambio de fondo importante — el borrow checker pasó de ser una anotación sin efecto a un mecanismo funcional real.

**Lo que preocupa:** dos bugs nuevos aparecieron como efecto colateral directo de estos arreglos (#8 en AOT nativo, #9 en inferencia de tipos de patrones). Esto sugiere que el ritmo de corrección de bugs no está acompañado (todavía) de una batería de tests de regresión que cubra las combinaciones cruzadas de features relacionadas — cada arreglo puntual parece estar exponiendo una nueva grieta adyacente. Vale la pena, antes de seguir agregando arreglos puntuales, invertir en tests que crucen: "construcción + destructuring" para cada tipo genérico builtin, y "VM vs AOT" para cada patrón de mutación, ya que son exactamente los dos ejes donde aparecieron las regresiones de esta ronda.

---

### Bug #10 — RESUELTO en v3.5.41: las DECLARACIONES fusionadas se ejecutaban con semántica de ASIGNACIÓN (corrompían los locales del frame llamante en recursión)

**Síntoma:** en recursión, tras la primera vuelta del caso base, el callee leía valores corruptos de variables locales del *caller* (fallo SILENCIOSO — sin error de runtime). El resultado visible: árboles fractales que "explotaban" fuera del lienzo, coordenadas que divergían de la simulación línea a línea.

**Causa raíz (bisectada con trace instrumentado en la VM):** el pipeline Rust fusiona el patrón `Load a; Load b; Binary; StoreLocal d` en un super-opcode (`FusedBin`/`FusedBinK`) para evitar 3 dispatch. El patrón de fusión aceptaba TANTO `Store` (asignación) como `StoreLocal` (declaración), y el opcode resultante ejecutaba SIEMPRE la semántica de asignación: `do_store_by_idx` resuelve el binding más cercano y, con hit de la caché de nombres, escribe DIRECTO en ese slot — aunque pertenezca al scope de un frame ANCESTRO. En `entero x2 = x + dx;` dentro de una función recursiva, el `FusedBin` del frame hijo daba cache-hit sobre el slot `x2` del frame padre y lo clobberaba; todos los descendientes escribían el MISMO slot (el de la primera frame), y al volver del caso base el padre leía el `x2` del último descendiente. Por eso la divergencia aparecía siempre "en la primera línea tras el retorno del caso base".

**Reproducción real (byte-exacta, VM nativa + wasm):**

```
sea G = [];
funcion entero arbol(entero n, entero x, entero y, entero dx, entero dy) {
    si (n <= 0) { retornar 0; }
    entero x2 = x + dx;        // ← DECLARACIÓN fusionada → clobberaba al padre
    entero y2 = y + dy;
    G.agregar(x); G.agregar(y); G.agregar(x2); G.agregar(y2);
    entero r1 = arbol(n - 1, x2, y2, (dx - dy) / 2, (dx + dy) / 2);
    entero r2 = arbol(n - 1, x2, y2, (dx + dy) / 2, (dy - dx) / 2);
    retornar r1 + r2 + 1;
}
```

Antes del fix: la línea 13 del trazado ya divergía de la simulación (segunda llamada del nodo a profundidad 10 recibía `x2=343` en vez de `338`). Con JIT y sin JIT (bug compartido por el helper `lj_fused_bin*`).

**Nota sobre la repro anterior:** la función `f(12,100,60)` con `c1=(a+b)/2; c2=(a-b)/2` daba 0 también en la simulación de referencia (la pareja (a,b) es una contracción que converge a (0,0)); el "esperado 1308160" era un fantasma — esa repro NO era un bug. El bug real necesita una DECLARACIÓN aritmética (`x = a ± b` con `+`, `-` o `*`, sin `Div`/`Mod` de por medio, que impedían la fusión) leída después de una llamada recursiva.

**Fix (v3.5.41):** dos opcodes nuevos que preservan la distinción semántica del lenguaje:
- `FusedBinK`/`FusedBin` (tags 5/6) — ASIGNACIÓN (`x = a op b`): sigue resolviendo el binding más cercano (necesario para `G = 9` dentro de una función, que asigna el global).
- `FusedBinKLocal`/`FusedBinLocal` (tags 12/13) — DECLARACIÓN (`entero x = a op b`): el destino se escribe SIEMPRE en el scope actual del frame (espejo exacto de `StoreLocal`, con invalidación selectiva de la caché de nombres al insertar).
- Cambios: patrón de fusión separado en codegen, encode/decode de bytecode, dispatch del intérprete (ambos sitios), helpers del JIT (`lj_fused_bink_local`/`lj_fused_bin_local`), análisis del Tier-2 (el destino local nunca se promueve a registro) y disasm.

**Verificación del fix:** arbolA (4095 segmentos, profundidad 12) byte-exacto contra simulación de referencia con JIT on/off y desde `.nvc`; paridad JIT↔intérprete 167/167 programas deterministas; 2 tests de regresión nuevos (`test_regresion_bug10_*`); 958/958 tests; clippy -D warnings y fmt limpios; suite de benchmarks 20/20 checksums sin regresión.

**Prioridad:** alta (era) — la recursión con locales es programación cotidiana y el fallo era silencioso. **Estado: RESUELTO.**

**Post-fix:** el ejemplo del árbol en el playground dejó de usar el workaround (lista global + bucle final) y volvió a la formulación natural — locales aritméticos `x2/y2` + dibujo por el puente JS durante la recursión, las dos variantes originalmente rotas. El paquete wasm del showcase se regeneró con el fix y el playground pasó la suite completa en navegador real (34/34, 0 errores de consola, 19146 píxeles — el mismo trazado byte-exacto de 4095 segmentos). El fixpoint self-hosting se re-verificó con el MISMO sha256 de la certificación (el compilador self-hosted no emite los opcodes nuevos), y la barra de producción completa pasó (hook pre-commit 4/4, paridad 3 backends 28/28, suite de benchmarks 20/20). Cierre de la ronda post-fix: `ci_gate.py` ×2 (JIT on/off) **PASS 392/389 ambas** (idéntico a la certificación) y **Bench-5 oficial de regresión TOTAL 241.9–244.2 ms en 3 tandas** vs los 244 ms certificados (mejor medición por tarea, 5 repeticiones, JIT apagado) — sin regresión de desempeño. Tras dos reseteos adicionales del sandbox, la barra entera se re-verificó sobre el build reconstruido: hook pre-commit 4/4 (958/958 tests), fixpoint byte-idéntico con el MISMO sha256 de la certificación, paridad 3 backends 28/28, bench-suite 20/20, ci_gate ×2 PASS 392/389 y navegador real 7/7 con 0 errores de consola.

---

## Parte 3 — Benchmarks de velocidad: VM (JIT) vs AOT-C (GCC) vs AOT-Cranelift

**Build probado:** `lumen-v3.5.7-linux-x64-musl`
**Entorno:** contenedor Linux x86_64, 1 core disponible, `cc (Ubuntu 13.3.0)`
**Fecha:** 2026-08-30
**Metodología:** cada medición se repitió 3 veces con `time` (bash), separando tiempo de **compilación** (cuando aplica) del tiempo de **ejecución** del binario/programa ya compilado. Números consistentes entre repeticiones (variación <5%), sin necesidad de más muestras.

---

### Caso 1 — Fibonacci recursivo, `fibonacci(32)` (~7M llamadas de función)

Este caso estresa despacho de funciones y recursión — carga de cómputo real, no trivialmente reducible a una fórmula cerrada por un optimizador.

| Backend | Tiempo de compilación | Tiempo de ejecución |
|---|---|---|
| **VM (`lumen run`)** | N/A (sin paso de compilación) | **0.020s** |
| **AOT-C (`build --native`, GCC -O3)** | 0.36s | 0.10s |
| **AOT-Cranelift (`build --aot rust`)** | 1.06s | **0.014s** |

**Hallazgos:**
- La VM tiene JIT tiering real y activo — confirmado con `LUMEN_JIT_LOG=1`, que muestra `🔥 Hot function detected: 'fibonacci' (50 llamadas) -> JIT Tier-1 activado` y `✅ recursión nativa (registros)`. No es un intérprete puro; promueve funciones calientes a código nativo en runtime.
- **Cranelift ejecuta más rápido que GCC -O3 en este caso** (0.014s vs 0.10s, ~7x) — resultado contraintuitivo, ya que Cranelift normalmente se posiciona como backend de compilación rápida a costa de calidad de código (su caso de uso típico es JIT, no AOT de producción). Aquí ocurre lo contrario: compila más lento (1.06s vs 0.36s) pero el binario resultante es notablemente más rápido.
- **La VM con JIT (0.020s) es más rápida que el binario AOT-C (0.10s)** para esta carga específica. Esto sugiere que el JIT Tier-1 de la VM se está especializando mejor para el patrón de recursión caliente que el código C genérico compilado por GCC sin ese conocimiento de perfil de ejecución.
- Orden de ejecución más rápida a más lenta: **Cranelift > VM-JIT > AOT-C**.

---

### Caso 2 — Bucle iterativo, suma de 100,000,000 enteros

Este caso reveló un matiz importante sobre cómo interpretar benchmarks de AOT.

| Backend | Tiempo de compilación | Tiempo de ejecución |
|---|---|---|
| **VM (`lumen run`)** | N/A | 10.11s |
| **AOT-C (`build --native`, GCC -O3)** | 0.28s | **0.001s** ⚠️ |
| **AOT-Cranelift (`build --aot rust`)** | 1.15s | 0.11s |

**Hallazgo importante — el número de AOT-C es engañoso, no un bug:**

El binario AOT-C ejecuta en `0.001s`, lo cual es físicamente imposible si estuviera sumando 100M enteros uno por uno. Verifiqué el `.c` intermedio (vía `LUMEN_KEEP_C=1`) y confirmé que LÚMEN genera un bucle `while` real y correcto — **no hay ninguna trampa del lado de LÚMEN.** Lo que ocurre es que **GCC -O3 reconoce el patrón de suma aritmética consecutiva y lo colapsa a la fórmula cerrada de Gauss** (`n(n-1)/2`) durante su propia optimización — una transformación de compilador completamente legítima y esperada de cualquier optimizador de producción maduro.

Cranelift **no realiza esta optimización** (ejecuta el bucle real, `0.11s`), lo cual explica por qué aquí parece "más lento" que GCC — en realidad está haciendo el trabajo real, mientras que GCC encontró un atajo matemático válido para este caso particular.

**Conclusión metodológica:** el Caso 1 (fibonacci) es el benchmark más honesto de los dos para comparar "velocidad de código generado" entre backends, porque la recursión no es trivialmente reducible a una fórmula cerrada. El Caso 2 mide, sin querer, la sofisticación del optimizador de GCC más que el rendimiento real de LÚMEN — buen recordatorio de que hay que diseñar cargas de benchmark que resistan folding de constantes al comparar compiladores.

La VM tarda notablemente más en este caso (10.11s) que en el Caso 1 relativo a su carga — sugiere que el JIT tiering se activa y beneficia más a funciones recursivas con llamadas repetidas que a bucles `mientras` de cuerpo simple; sería interesante para el equipo revisar si el detector de "función caliente" cubre bien bucles largos dentro de la función `principal`, no solo funciones invocadas muchas veces.

---

### Resumen de tiempos de compilación (ambos casos)

| Backend | Fibonacci | Bucle |
|---|---|---|
| GCC (`--native`) | 0.36s | 0.28s |
| Cranelift (`--aot rust`) | 1.06s | 1.15s |

Cranelift compiló consistentemente **~3-4x más lento** que GCC en ambos casos probados, en este entorno de una sola sesión de compilación (sin reutilización de caché de Cargo/rustc entre ejecuciones, lo cual podría explicar buena parte de la diferencia — Cranelift vive dentro del toolchain de Rust, y el overhead de arrancar `rustc`/`cargo` para invocar el backend probablemente domina el tiempo total más que la generación de código en sí). Valdría la pena que el equipo mida el tiempo de compilación "en caliente" (segunda compilación consecutiva del mismo proyecto, con caché de Cargo ya poblada) para separar el costo de arranque del toolchain del costo real de codegen — la comparación justa contra GCC probablemente sea mejor de lo que sugieren estos números en frío.

---

### Recomendaciones para benchmarking futuro

1. **Usar cargas que resistan constant-folding** (recursión, acceso a memoria dependiente de datos de entrada, aleatoriedad) para medir codegen real — el Caso 2 de este reporte es un buen ejemplo de por qué un bucle simple con datos estáticos no sirve para esto.
2. **Separar tiempo de compilación en frío vs. en caliente** para Cranelift específicamente, dado que corre sobre el toolchain de Rust/Cargo, cuyo tiempo de arranque puede dominar mediciones de un solo build.
3. **Investigar por qué el JIT de la VM no parece tierizar bucles largos tan agresivamente como funciones recursivas calientes** — si el objetivo es que la VM sea competitiva sin necesidad de AOT, cerrar esa brecha en bucles sería tan importante como el tiering de funciones.
4. Si Cranelift resulta consistentemente más rápido en ejecución que GCC -O3 para cargas típicas de LÚMEN (más allá de este único caso de fibonacci), podría valer la pena promoverlo como backend por defecto de `--native` en vez de requerir la flag explícita `--aot rust`, dado que aquí ejecutó ~7x más rápido a cambio de ~3x más tiempo de compilación — una relación probablemente favorable para la mayoría de casos de uso de producción, donde se compila una vez y se ejecuta muchas.

---

## Parte 4 — Hoja de ruta sugerida a partir de ahora

Esta sección resume, en un solo lugar, qué haría yo si estuviera en el lugar de Omar hoy, ordenado por lo que rendiría más rápido. No es una lista de "features nuevas" — es específicamente sobre **cómo dejar de reabrir bugs adyacentes cada vez que se corrige uno**, que fue el patrón más claro que se repitió entre v3.2.0 y v3.5.7 (3 bugs corregidos, 2 bugs nuevos aparecieron como efecto colateral directo).

### 1. Antes que nada: congelar features un ciclo y meterle un arnés de regresión al corpus existente

El proyecto ya tiene ~45+ archivos en `examples/` que cubren casi todas las features del lenguaje. Ese corpus **no se está usando como suite de regresión** — se usa como documentación de sintaxis (que es como yo lo usé para aprender a escribir código válido). Convertirlo en test suite es el cambio de mayor apalancamiento posible:

```bash
# Pseudocódigo de lo que se necesita, adaptable a lo que ya exista de `lumen test`
for f in examples/*.nv; do
    lumen check "$f"                    # debe pasar sin error
    lumen run "$f" > "golden/$f.out"    # capturar output esperado UNA VEZ, a mano, revisado
done
```

Después, en CI, en cada commit:
```bash
for f in examples/*.nv; do
    actual=$(lumen run "$f")
    expected=$(cat "golden/$f.out")
    diff <(echo "$actual") <(echo "$expected") || fail "$f regresó"
done
```

Esto sería suficiente para haber atrapado, automáticamente, **el bug #8 de esta sesión** (la regresión de `build --native` al arreglar el bug #2) — con solo correr el corpus existente contra los dos backends (VM y AOT) en cada build de CI y comparar que ambos den el mismo resultado.

### 2. Test de "roundtrip" específico para `fmt` (esto ya casi te muerde una vez)

El bug de `fmt` borrando código fue el más peligroso de toda la sesión porque era silencioso. La forma estándar de blindar un formateador (así es como lo hacen `rustfmt` y `gofmt`) es un invariante automático, no revisión manual:

```bash
for f in examples/*.nv; do
    ast_antes=$(lumen check --dump-ast "$f")     # si no existe --dump-ast, usar el AST interno en un modo debug
    lumen fmt "$f"
    ast_despues=$(lumen check --dump-ast "$f")
    [ "$ast_antes" = "$ast_despues" ] || fail "fmt cambió el AST de $f"
done
```

Si `check`/el compilador no expone un `--dump-ast` hoy, ese comando en sí mismo es una inversión que vale la pena — no solo sirve para este test, sino que hubiera acelerado bastante mi propio proceso de diagnóstico en varios de los bugs de esta sesión (en más de un caso tuve que inferir la causa raíz solo a partir del texto del mensaje de error).

### 3. Test cruzado "VM vs AOT-C vs AOT-Cranelift" para cada archivo del corpus

Dado que ya se encontraron discrepancias reales entre backends (mensajes de error distintos en el bug #2 original, y la regresión del bug #8 que solo afecta a AOT-C), un chequeo de "mismo input, mismo output, en los tres backends" atraparía por diseño toda una clase de bugs de codegen antes de que lleguen a un release:

```bash
for f in examples/*.nv; do
    out_vm=$(lumen run "$f")
    lumen build --native "$f" -o /tmp/bin_c && out_c=$(/tmp/bin_c)
    lumen build --aot rust "$f" -o /tmp/bin_cl && out_cl=$(/tmp/bin_cl)
    [ "$out_vm" = "$out_c" ] && [ "$out_vm" = "$out_cl" ] || fail "discrepancia de backends en $f"
done
```

### 4. Property-based testing / fuzzing dirigido a las combinaciones que ya demostraron ser frágiles

El propio CLI ya tiene `lumen fuzz` — vale la pena apuntarlo específicamente a las intersecciones de features que resultaron ser el patrón de bug más común en esta sesión completa:
- **Tipo genérico builtin (`opcion<T>`, `resultado<T,E>`) × tipo interior struct de usuario.** Tanto el bug #4 original como el bug #9 nuevo cayeron exactamente en esta intersección (`opcion<StructDeUsuario>`). Un fuzzer que genere structs aleatorios y los inserte como parámetro de tipo de `opcion`/`resultado`, combinado con construcción + destructuring + mutación, tiene alta probabilidad de encontrar la próxima variante de este mismo patrón antes que un usuario.
- **Mutación (`prestado mut`) × contenedor (`lista<T>` de structs, structs anidados, structs recursivos).** El bug #2 y el bug #6 comparten la raíz de "algo con estado mutable dentro de otra cosa" — vale la pena generar automáticamente combinaciones de anidación (struct dentro de lista, lista dentro de struct, struct dentro de struct) cruzadas con los tres mecanismos de mutación (valor, `este`, `prestado mut`).
- **Imports transitivos con profundidad variable (2, 3, 4+ niveles) y con/sin ciclos.** El bug #5 solo se probó a profundidad 2 en esta sesión; no hay garantía de que a profundidad 3 el comportamiento sea el mismo una vez que se arregle.

### 5. Separar explícitamente "sintaxis aceptada por `check`" de "sintaxis con semántica implementada"

Varios bugs de esta sesión (#2 original, #3 original en su forma sin calificar, #6) tenían la misma forma: `check` decía que el programa era válido, y luego fallaba en runtime. Antes de agregar la siguiente feature nueva, valdría la pena una auditoría corta y específica de: *¿hay algún camino sintáctico que `check` acepta pero para el que el intérprete/codegen no tiene una implementación real?* Esto probablemente ya se resolvió parcialmente con los arreglos de esta ronda, pero como práctica permanente, cualquier nueva construcción sintáctica debería tener su test de "check dice válido Y run/build hace lo que se espera" antes de mergear, no después.

### 6. Sobre el benchmarking y Cranelift, ya que fue el foco de esta ronda

- Vale la pena medir el tiempo de compilación de Cranelift **en caliente** (con caché de Cargo ya poblada) antes de sacar conclusiones definitivas sobre si es "lento para compilar" — el número en frío que medimos aquí (1.06s vs 0.36s de GCC) puede estar dominado por el arranque del toolchain de Rust más que por el codegen real.
- Si en más benchmarks (no solo fibonacci) Cranelift sigue generando código más rápido que GCC -O3, es una señal fuerte para promoverlo a backend por defecto de `--native`, con GCC como fallback o modo explícito para quien priorice tiempo de compilación sobre tiempo de ejecución (ej. iteración rápida en desarrollo).
- El hallazgo de que el JIT de la VM le gana a GCC -O3 en fibonacci pero es mucho más lento que ambos AOT en el bucle sugiere que **cerrar la brecha de tiering para bucles** podría ser una optimización de alto impacto — permitiría que más gente use la VM directamente sin necesitar compilar a nativo para código con bucles intensivos, que es probablemente más común que la recursión pesada en código real.

### Orden de prioridad sugerido, todo junto

1. **Convertir `examples/` en suite de regresión automática (punto 1)** — es lo más barato de implementar (el corpus ya existe) y lo que hubiera atrapado más bugs de esta sesión por sí solo.
2. **Test de roundtrip para `fmt` (punto 2)** — específico y barato, cierra la clase de bug más peligrosa (silenciosa) que ya se vio una vez.
3. **Test cruzado de backends (punto 3)** — habría atrapado la regresión de AOT-C de esta misma ronda.
4. **Separación check-vs-implementado como criterio de merge (punto 5)** — cambio de proceso, no de código; barato de adoptar ya mismo.
5. **Fuzzing dirigido a las intersecciones ya conocidas como frágiles (punto 4)** — más inversión, pero es lo que probablemente encuentre la "próxima ronda" de bugs antes que un usuario en producción.
6. **Investigación de Cranelift como backend por defecto (punto 6)** — no es corrección de bugs, es una oportunidad de producto una vez que la base sea más estable.

Si tuviera que resumirlo en una sola frase: el equipo ya sabe corregir bugs puntuales con velocidad y precisión — lo que falta es la red que evite que cada corrección abra una grieta nueva al lado, y esa red se construye con automatización barata sobre el corpus que ya existe, no con más código de producto.

---

## Validación del workflow de CI real (2026-08-30, ronda post-fix v3.5.41)

**Contexto:** el CI de GitHub (`.github/workflows/ci.yml`, 17 jobs) es la única puerta que no se puede ejecutar localmente en Windows/macOS. Para maximizar la probabilidad de que el primer push salga verde se hizo una **simulación local de los jobs Linux** (`~/verif/ci_sim.sh`, auto-reparadora ante resets del sandbox) y una auditoría del YAML paso a paso.

**Simulación local — 14/14 ✅:**
fmt · clippy -D warnings · tests 958/958 · check examples 396/396 · headless (tests + check) · production suite · cargo bench --quick · wasm-check (wasm32-unknown-unknown + wasm32-wasip1+wasi) · gate sobre artefacto empaquetado (PASS 392/389) · aot-smoke (VM = AOT-C = Cranelift, salidas idénticas) · bench-suite 20/20 · lógica fuzz-paridad.

**3 correcciones aplicadas al workflow (descubiertas por la auditoría):**
1. **gate Linux: workers 8 → 4.** Con 8 workers en máquinas de poca RAM, un ejemplo puede ser matado por el OOM-killer (exit 137 no está en la lista de crashes del gate → se cuenta como FAIL funcional). Reproducido localmente: `sr/matrices_sr.nv` fallaba bajo presión y pasa 3/3 en solitario. Con workers 4 el gate empaquetado dio PASS 392/389 consistente.
2. **bench-suite: `python` → `${{ matrix.py }}`** (`python3` en macOS): los runners de macOS no tienen el alias `python`.
3. **`scripts/fuzz_paridad.ps1`: binario nativo sin `.exe` en Linux.** `lumen build --native` produce el binario SIN extensión en Linux; el script solo buscaba `<nombre>.exe`, así que en el runner ubuntu TODOS los archivos contaban como FALLA-COMPILA (el job no comparaba nada). Ahora detecta ambas formas.

**Hallazgo cerrado (v3.5.42): 5 divergencias VM↔AOT pre-existentes en `fuzz/` — ARREGLADAS.**
`closure_multi.nv` (capturas compartidas entre instancias de closure), `gen_ref.nv` (write-back de `prestado mut` a celda sin inicializar → crash nativo «Indice 1 fuera de rango»), `pid_caps.nv`, `regex_braces.nv`, `regex_dollar.nv` (grupos regex 1-based y `$0` vacío). Prueba de pre-existencia (no eran regresiones del fix v3.5.41): binario construido desde **HEAD pristino** (worktree aparte, `1a019a5`) reproduce las MISMAS 5 divergencias, y la VM pristina vs la VM actual da salida idéntica en los 5. **Resolución v3.5.42**: salida nativa byte-idéntica a la VM en los 5 con el binario release; barrido completo de `fuzz/` (29 programas, VM vs `--native`) → **25 PAR / 0 DIF / 4 FALLA-COMPILA** (solo `lexer_bench*` y `parser_bench`, por `__map_*` sin soporte en AOT-C — deuda separada); 4 tests de regresión nuevos (C/Cranelift/LLVM/VM), 961/961.
