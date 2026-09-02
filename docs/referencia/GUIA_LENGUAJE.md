# LÚMEN — Guía Completa del Lenguaje (v3.94.24)

> Referencia práctica y verificada de toda la sintaxis de LÚMEN, con ejemplos
> mínimos que compilan y corren en **VM y en binario nativo** (`lumen run` y
> `lumen build --native`). Cada ejemplo fue validado con el arnés de paridad
> VM ⇄ AOT-C de `scripts/regresion_qa.py` (A–P) y con la suite de benchmarks
> de todas las áreas (`benchmarks/run_bench_all.py`).

---

## 1. Estructura de un programa

```lumen
// Comentario de línea. LÚMEN usa `//` (no hay comentarios /* */).

funcion entero principal() {
    imprimir("hola mundo");
    retornar 0;
}
```

- El punto de entrada es la función **`principal`** (también acepta `main`).
- Las declaraciones terminan implícitamente por línea; se puede usar `;`
  explícito al final.
- `imprimir(...)` acepta varios argumentos y los concatena en una línea.
- `a_texto(x)` convierte cualquier valor a `texto`.

---

## 2. Variables y tipos

```lumen
entero    n = 28;             // entero i64 con signo
decimal   pi = 3.14159;       // flotante f64
texto     s = "Hola";         // cadena UTF-8 inmutable
booleano  b = verdadero;      // verdadero / falso
lista<entero> xs = [1, 2, 3]; // lista dinámica tipada

// Inferencia de tipos con `sea`:
sea x = 42;        // entero
sea y = 3.14;      // decimal
sea z = "texto";   // texto
sea f = |q| q * 2; // closure

// Reasignación (el tipo no cambia):
x = x + 1;
```

### Tipos opcionales y resultados

```lumen
opcion<entero> a = algun(10);      // opción con valor
opcion<entero> b = ninguno;        // opción vacía

si sea algun(v) = a {
    imprimir("tiene valor: ", a_texto(v));
}

resultado<entero, texto> ok = exito(42);
resultado<entero, texto> err = error("falló");
```

### `cualquiera` y `numero`

`cualquiera` es el tipo dinámico; `numero` normaliza a `decimal`. Mezclar un
`cualquiera` con un entero produce `decimal` (asignarlo a `entero` es un error
E031). Usa `(x) como entero` para truncar.

```lumen
funcion cualquiera identidad(cualquiera x) { retornar x; }
entero c = (identidad(42)) como entero;   // ✓
```

---

## 3. Funciones

```lumen
funcion entero duplicar(entero n) {
    retornar n * 2;
}

// Parámetros: se aceptan los dos órdenes.
funcion entero suma(entero a, entero b) { retornar a + b; }      // `Tipo nombre`
funcion entero resta(a: entero, b: entero) { retornar a - b; }    // `nombre: Tipo`

// Parámetros por defecto:
funcion entero repetir(entero n, entero veces = 2) { retornar n * veces; }

// `retornar` sin valor en funciones `vacio`:
funcion vacio saludar(texto nombre) {
    imprimir("hola ", nombre);
}
```

---

## 4. Estructuras (structs) y métodos `impl`

```lumen
// (a nivel superior del archivo, no dentro de una función)
estructura Punto {
    x: entero,      // campos: SIEMPRE nombre: Tipo
    y: entero
}

// Construcción:
Punto p = Punto { x: 3, y: 4 };

// Acceso y mutación de campos:
p.x = 10;
imprimir("x=", a_texto(p.x), " y=", a_texto(p.y));

// Métodos inherentes con `impl` (receptor `este` sin tipo = por valor):
impl Punto {
    funcion entero sumar(este) {
        retornar este.x + este.y;
    }
    // método que MUTA la instancia: usa `prestado mut este`
    // (un `este` simple es por valor: los cambios NO persisten).
    funcion vacio mover(prestado mut este, entero dx, entero dy) {
        este.x = este.x + dx;
        este.y = este.y + dy;
    }
}

imprimir(a_texto(p.sumar()));   // 7
```

### Structs genéricos

```lumen
estructura Caja<T> { valor: T }

Caja<entero> c = Caja { valor: 42 };
imprimir(a_texto(c.valor));
```

### Structs anidados y listas de structs

```lumen
estructura Item { valor: entero }
lista<Item> items = [Item { valor: 1 }, Item { valor: 2 }];
imprimir(a_texto(items[0].valor));   // 1
```

---

## 5. Enums (tipos suma) y `elegir`

```lumen
enum Color { Rojo, Verde, Azul }

Color c = Color::Rojo;
imprimir(c);                       // Rojo

// Variantes con datos:
enum Op { Suma(entero, entero), Nada }

Op o = Op::Suma(2, 3);

elegir (o) {
    caso Op::Suma(a, b): imprimir("suma = ", a_texto(a + b));
    caso Op::Nada:      imprimir("nada");
}
```

También existe `elegir` con `defecto`:

```lumen
elegir (c) {
    caso Color::Rojo:  imprimir("rojo");
    caso Color::Verde: imprimir("verde");
    defecto:           imprimir("otro");
}
```

---

## 6. Lambdas y closures

```lumen
// Sintaxis con pipes (parámetros sin tipo inferido como `numero`):
sea doble = |x| x * 2;

// Con tipo explícito:
sea inc = |n: entero| { retornar n + 1; };

// Sintaxis `funcion`:
sea triple = funcion (entero n) { retornar n * 3; };

// Llamada:
imprimir(a_texto(doble(21)), a_texto(inc(41)), a_texto(triple(7)));
```

### Captura de variables (por snapshot)

Al crear un closure, las variables externas que referencia se **capturan por
valor** (snapshot). Mutar la variable dentro del closure **no** afecta al
definidor, y cada closure conserva su propio estado:

```lumen
entero contador = 0;
sea acumular = |n: entero| { contador = contador + n; retornar contador; };
entero a = acumular(5);   // 5
entero b = acumular(7);   // 12
imprimir(a_texto(contador));  // 0 (no cambió)
```

> La sintaxis de flecha `=>` **no** es LÚMEN (solo aparece dentro de strings
> JavaScript de `__js_eval`). La forma `|| { ... }` colisiona con el `||`
> lógico. Usa `funcion () { ... }` para lambdas sin parámetros.

---

## 7. Control de flujo

```lumen
// if / else
si (x > 0) {
    imprimir("positivo");
} sino {
    imprimir("no positivo");
}

// while
entero i = 0;
mientras (i < 5) {
    imprimir(a_texto(i));
    i = i + 1;
}

// for clásico
para (entero j = 0; j < 5; j = j + 1) {
    imprimir(a_texto(j));
}

// for-each
para n en [10, 20, 30] {
    imprimir(a_texto(n));
}

// `continuar` y `romper` funcionan en TODOS los bucles (incluido para … en)
para n en [1, 2, 3, 4, 5] {
    si (n == 3) { continuar; }
    si (n == 5) { romper; }
    imprimir(a_texto(n));
}

// try / catch
intentar {
    entero d = 1 / 0;         // lanza en runtime
} atrapar (e) {
    imprimir("error: ", e);
}
```

### Ternario

```lumen
entero mayor = (x > 3) ? x : 0;
imprimir("valor:", a_texto(x > 3 ? x : 0));
```

---

## 8. Operadores

```lumen
// Aritméticos: + - * / %  (el % es módulo)
// Comparación: == != < <= > >=
// Lógicos: && || !  (y, o, no en inglés)
// Bitwise: & | ^ ~ << >>
entero mascara = (0xFF & 0x0F) | (1 << 4) ^ 0x01;
entero invertido = ~8;    // -9
// `|>` pipe para encadenar llamadas (el resultado se pasa como 1.er argumento):
funcion lista<entero> duplicar_todo(lista<entero> xs) {
    lista<entero> out = [];
    para n en xs { out.agregar(n * 2); }
    retornar out;
}
lista<entero> r = [1, 2, 3] |> duplicar_todo();   // [2, 4, 6]
```

### Casts (`como`)

```lumen
entero  a = (3.9) como entero;     // 3 (trunca hacia cero)
decimal d = (5) como decimal;      // 5.0
booleano b = (1) como booleano;    // verdadero
texto  s = (42) como texto;        // "42"
```

---

## 9. Listas, rangos y slicing

```lumen
lista<entero> xs = [10, 20, 30, 40, 50];

// Acceso por índice (0-based):
imprimir(a_texto(xs[2]));          // 30

// Rangos: `inicio..fin` (fin exclusivo) y `inicio..=fin` (inclusivo):
lista<entero> a = xs[1..4];        // [20, 30, 40]
lista<entero> b = xs[1..=4];       // [20, 30, 40, 50]

// Slicing de texto:
texto s = "abcdef";
imprimir(s[1..4]);                 // "bcd"
imprimir(s[1..=4]);                // "bcde"

// Un rango también puede vivir en una variable:
sea r = 1..3;
imprimir(a_texto(xs[r].largo()));  // 2
imprimir(s[r]);                    // "bc"

// Métodos útiles:
imprimir(a_texto(xs.largo()));     // 5
xs.agregar(60);                    // agrega al final
```

### Comprensiones de listas

```lumen
lista<entero> dobles = [x * 2 para x en [1, 2, 3, 4]];
lista<entero> pares  = [n para n en 1..=10 si n % 2 == 0];
```

---

## 10. Mapas (diccionarios)

```lumen
sea d = __map_nuevo();              // mapa vacío

// Insertar (el mapa es por VALOR: hay que reasignar):
d = __map_poner(d, "clave", 100);
d = __map_poner(d, "otra", 200);

// Leer:
imprimir(a_texto(__map_obtener(d, "clave")));   // 100
imprimir(a_texto(__map_longitud(d)));           // 2

// Recorrer un mapa insertando en bucle:
entero i = 0;
mientras (i < 1000) {
    d = __map_poner(d, i, i * 2);
    i = i + 1;
}
```

> **Nota de rendimiento (documentada)**: el backend C implementa el mapa por
> valor con copia en cada inserción — O(n) por inserción y O(n²) de memoria
> para mapas grandes (sin recuento de referencias). La VM usa un mapa
> persistente (O(log n)) y maneja 30 000+ claves. Para mapas masivos usa la VM.

---

## 11. Texto y Unicode

```lumen
texto s = "héllo wörld 🚀 café";   // UTF-8

// Indexación y `largo()` operan sobre CODEPOINTS, no bytes:
imprimir(a_texto(s.largo()));      // 18
imprimir(s[0]);                    // "h"
imprimir(s[12]);                   // "🚀" (un emoji = 1 codepoint)

// Concatenación con +:
texto t = s + "!";

// Interpolación:
texto reporte = f"valor: {42} y {3.14}";
```

> `estructura`, `enum` e `impl` se declaran en el **ámbito superior** del
> archivo (no dentro de funciones).

---

## 12. Propiedad y préstamo (`dueno` / `prestado`)

```lumen
estructura Recurso { id: entero }

// `dueno T`: titularidad lineal (move semantics); se auto-desreferencia:
funcion vacio consumir(dueno Recurso r) {
    imprimir(a_texto(r.id));        // ✓ auto-desreferencia
}

// `prestado mut`: write-back real al llamador:
funcion vacio incrementar(prestado mut entero n) {
    n = n + 1;
}

funcion entero principal() {
    consumir(Recurso { id: 7 });

    entero x = 5;
    incrementar(x);                 // x == 6
    retornar 0;
}
```

---

## 13. Interoperabilidad y stdlib

```lumen
importar "matematicas.nv";     // importa un módulo de la stdlib

// FFI: eval de JavaScript (para bridges web), ensamblador, C y Rust:
__js_eval("console.log('hola');");
```

La stdlib (`stdlib/`) incluye: `matematicas.nv`, `concurrencia.nv`,
`crypto.nv`, `redis.nv`, `postgres.nv`, `dataframe.nv`, `tensor.nv`/`nn.nv`,
`gguf.nv` (LLMs), `ia.nv`, `vector_db.nv`, `motor_grafico.nv`, `gpu.nv`,
`ui_reactiva.nv`, `nexus.nv`, `actor.nv`, `plugins.nv`, `scheduler.nv`,
`autograd.nv`, `matriz_simd.nv`/`simd.nv`, entre otros.

---

## 14. Ejecutar y compilar

```bash
lumen run programa.nv            # interpreta con la VM
lumen check programa.nv          # valida sintaxis y tipos
lumen build programa.nv          # compila a bytecode .nvc
lumen build --native programa.nv # compila a binario nativo (C -O3)
```

---

*Documentación verificada con la suite de regresión A–P (20/20), los ejemplos
(396/396) y el benchmark de todas las áreas en v3.94.24.*
