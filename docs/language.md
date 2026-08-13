# Referencia del Lenguaje LÚMEN — v2.4.1

LÚMEN es un lenguaje de programación educativo con sintaxis en español y equivalentes
opcionales en inglés. Pipeline completo: Lexer → Parser → Sema → IR → Bytecode → VM.

---

## Tipos de Datos

| Tipo | Descripción | Ejemplo |
|------|-------------|---------|
| `entero` / `integer` | Entero 64 bits con signo | `42`, `-10` |
| `decimal` / `float` | Float 64 bits IEEE-754 | `3.14`, `-0.5` |
| `numero` / `number` | Alias de `decimal` | `3.14` |
| `texto` / `string` | Cadena UTF-8 | `"Hola"` |
| `booleano` / `boolean` | Booleano | `verdadero` / `falso` |
| `lista<T>` / `array<T>` | Lista dinámica de tipo `T` | `[1, 2, 3]` |
| `diccionario<K,V>` | Mapa llave-valor | `{"a": 1}` |
| `opcion<T>` | Valor opcional (null safety) | `algun(42)` / `ninguno` |
| `resultado<T,E>` | Éxito o error | `exito(42)` / `error("msg")` |
| `(T, U)` | Tupla heterogénea | `(42, "hola")` |
| `funcion(...) -> T` | Tipo función | `funcion(entero) -> entero` |
| `estructura { ... }` | Tipo estructura | `Persona { nombre: texto }` |
| `enum { ... }` | Tipo suma / enum | `Color { Rojo, Verde(entero) }` |

---

## Variables

```lumen
// Con tipo explícito
entero edad = 25;
texto nombre = "Ana";
booleano activo = verdadero;

// Con inferencia de tipos (recomendado)
edad = 25;
nombre = "Ana";
activo = verdadero;

// Constantes
const PI = 3.14159;
const MAX_USUARIOS = 1000;
```

---

## Operadores

| Categoría | Operadores |
|-----------|-----------|
| Aritméticos | `+`, `-`, `*`, `/`, `%` |
| Comparación | `==`, `!=`, `<`, `>`, `<=`, `>=` |
| Lógicos | `&&` / `y`, `\|\|` / `o`, `!` / `no` |
| Ternario | `condicion ? valor_si : valor_no` |
| Rangos | `inicio..fin`, `inicio..=fin` |

---

## Strings

```lumen
// Interpolación de strings
texto saludo = "Hola {nombre}, tienes {edad} años";

// Indexación
texto s = "Hola";
texto c = s[0]; // "H"

// Concatenación mixta
texto msg = "Resultado: " + 42; // "Resultado: 42"

// Conversiones
texto t = a_texto(42);        // "42"
entero n = a_entero("42");    // 42 (Resultado<entero,texto>)
```

---

## Control de Flujo

### si / sino (if/else)

```lumen
si edad >= 18 {
    imprimir("Mayor de edad");
} sino si edad >= 13 {
    imprimir("Adolescente");
} sino {
    imprimir("Menor");
}
```

### mientras (while)

```lumen
entero i = 0;
mientras i < 5 {
    imprimir(i);
    i = i + 1;
}
```

### para (for / for-each)

```lumen
// For-each sobre lista
para x en numeros {
    imprimir(x);
}

// Rango
para i en 0..10 {
    imprimir(i);
}

// Rango inclusivo
para i en 1..=5 {
    imprimir(i);
}

// For clásico
para (entero i = 0; i < 10; i = i + 1) {
    imprimir(i);
}
```

### elegir / match

```lumen
elegir valor {
    caso 1: imprimir("uno");
    caso 2 | 3: imprimir("dos o tres");   // OR patterns
    caso 4..10: imprimir("entre 4 y 9"); // Range patterns
    caso "hola": imprimir("saludo");      // String patterns
    defecto: imprimir("otro");
}
```

**OR patterns**: `caso A | B | C:` — el `|` dentro de un arm separa alternativas
(no es `BitOr`); si el sujeto coincide con cualquiera, se ejecuta el cuerpo.
Sirve con enums, enteros y strings.

**Range patterns**: `caso inicio..fin:` (exclusivo, `inicio <= x < fin`) y
`caso inicio..=fin:` (inclusivo, `inicio <= x <= fin`). Los límites deben ser
numéricos y el sujeto del `elegir` numérico también (E044/E056 si no).

**Rangos como expresión**: `inicio..fin` produce una lista de enteros
(`inicio..fin` excluye `fin`; `inicio..=fin` lo incluye). Útil para construir
series: `lista<entero> serie = 0..5;` → `[0, 1, 2, 3, 4]`. En el backend C y
la VM se desugara en un bucle con `ArrayNew`/`ArrayPush`; el backend Cranelift
no emite el cuerpo por límite de diseño (sin colecciones).

Guardias: `caso X si condicion:` — la guardia debe ser booleana (E034) y se
evalúa solo si el patrón coincide.

### Loop Labels (etiquetas)

```lumen
externo: mientras verdadero {
    mientras verdadero {
        romper externo;  // Sale del loop externo
    }
}
```

---

## Funciones

```lumen
// Declaración básica
funcion entero suma(entero a, entero b) {
    retornar a + b;
}

// Parámetros default
funcion entero suma(entero a, entero b = 10) {
    retornar a + b;
}
imprimir(suma(5));     // 15
imprimir(suma(5, 20)); // 25

// Genérica
funcion T identidad<T>(T valor) {
    retornar valor;
}

// Con bounds
funcion texto mostrar<T: Mostrable>(T valor) {
    retornar valor.mostrar();
}
```

---

## Lambdas / Closures

```lumen
// IIFE (Invocación Inmediata)
entero r = funcion(entero x) { retornar x * 2; }(5);

// Asignable
dup = funcion(entero x) { retornar x * 2; };
imprimir(dup(5)); // 10

// Closure (captura de entorno)
entero factor = 3;
multiplica = funcion(entero x) { retornar x * factor; };
imprimir(multiplica(7)); // 21
```

---

## Estructuras y Métodos

```lumen
estructura Rectangulo {
    ancho: decimal,
    alto: decimal
}

impl Rectangulo {
    funcion decimal area(Rectangulo self) {
        retornar self.ancho * self.alto;
    }
    funcion texto describir(Rectangulo self) {
        retornar "Rect {self.ancho}x{self.alto}";
    }
}

Rectangulo r = Rectangulo { ancho: 10.0, alto: 5.0 };
imprimir(r.area());     // 50.0
imprimir(r.describir()); // "Rect 10x5"
```

---

## Traits (Rasgos)

```lumen
rasgo Mostrable {
    funcion texto mostrar(este);
}

rasgo Comparable {
    funcion booleano es_mayor(este, otro: este);
}

impl Mostrable para Rectangulo {
    funcion texto mostrar(este) {
        retornar "Rect({este.ancho}, {este.alto})";
    }
}
```

---

## Tipos Asociados en Traits

```lumen
rasgo Contenedor {
    tipo Item;
    funcion Item obtener(este);
    funcion nada insertar(este, Item valor);
}

impl Contenedor para Caja {
    tipo Item = entero;
    funcion entero obtener(este) { retornar este.valor; }
    funcion nada insertar(este, entero valor) { este.valor = valor; }
}
```

---

## Enums y Pattern Matching

```lumen
enum Forma {
    Circulo(decimal),
    Rectangulo(decimal, decimal),
    Triangulo(decimal, decimal, decimal)
}

funcion decimal area(Forma f) {
    elegir f {
        caso Forma::Circulo(r)          { retornar 3.14159 * r * r; }
        caso Forma::Rectangulo(b, h)    { retornar b * h; }
        caso Forma::Triangulo(a, b, c)  { retornar (a + b + c) / 2.0; }
    }
}
```

---

## Tuplas y Destructuring

```lumen
// Declaración
(entero, texto) par = (42, "hola");
imprimir(par.0); // 42
imprimir(par.1); // "hola"

// Destructuring
entero x, texto etiqueta = (100, "Coord X");
imprimir("{etiqueta}: {x}");

// Wildcard
entero primero, _ = (1, 2);
```

---

## Opcion<T> — Null Safety

```lumen
opcion<entero> opt = algun(42);
opcion<entero> vacio = ninguno;

// Pattern matching
elegir opt {
    caso algun(valor): imprimir("Tengo: {valor}");
    caso ninguno: imprimir("Vacío");
}

// If-let
si sea algun(v) = opt {
    imprimir("Valor: {v}");
}

// Guard Let
sea algun(valor) = opt sino {
    imprimir("No hay valor");
    retornar;
}
imprimir("Obtenido: {valor}");
```

---

## Resultado<T,E> — Manejo de Errores

```lumen
resultado<entero, texto> r = exito(42);
resultado<entero, texto> e = error("falló");

elegir r {
    caso exito(v): imprimir("OK: {v}");
    caso error(msg): imprimir("Error: {msg}");
}

// Conversiones seguras
resultado<entero, texto> n = a_entero("123");
```

---

## Guard Let

```lumen
// Desestructuración con rama divergente si no hay match
sea x = calcular() sino {
    imprimir("Falló el cálculo");
    retornar;
}

sea algun(valor) = obtener_opcional() sino {
    continuar;
}
```

El bloque `sino` debe contener una instrucción divergente: `romper`, `retornar` o `continuar`.

---

## Sobrecarga de Operadores

```lumen
estructura Punto { x: entero, y: entero }

impl Suma para Punto {
    funcion Punto sumar(Punto self, Punto otro) {
        retornar Punto { x: self.x + otro.x, y: self.y + otro.y };
    }
}

Punto a = Punto { x: 1, y: 2 };
Punto b = Punto { x: 3, y: 4 };
Punto c = a + b; // Punto { x: 4, y: 6 }
```

Traits disponibles: `Suma`, `Resta`, `Multiplica`, `Divide`.

---

## Extension Methods

```lumen
rasgo Duplicable {
    funcion entero duplicar(este);
}

impl Duplicable para entero {
    funcion entero duplicar(este) {
        retornar este * 2;
    }
}

entero n = 21;
imprimir(n.duplicar()); // 42
```

Funciona para: `entero`, `texto`, `decimal`, `booleano`, `lista`, `opcion`, `resultado`, `tupla`.

---

## Impl Trait Return

```lumen
funcion impl Comparable crear_comparable() {
    retornar 42; // tipo concreto inferido en llamada
}
```

---

## Módulos e Imports

```lumen
importar "math.nv";
importar utils;
importar "datos.nv" como datos;
importar ingles;  // activa keywords en inglés
```

---

## Tests

```lumen
funcion nada test_suma() {
    afirmar(suma(2, 3) == 5);
    afirmar(suma(0, 0) == 0);
}
```

Ejecutar: `lumen test archivo.nv`

---

## Librería Estándar

| Módulo | Funciones clave |
|--------|----------------|
| `matematicas` | `raiz`, `potencia`, `abs`, `piso`, `techo`, `redondear`, `seno`, `coseno` |
| `texto` | `largo`, `mayusculas`, `minusculas`, `contiene`, `reemplazar`, `dividir`, `unir` |
| `coleccion` | `ordenar`, `filtrar`, `mapear`, `reducir`, `invertir`, `unico` |
| `fecha` | `ahora`, `formato`, `diferencia`, `agregar_dias` |
| `archivos` | `leer_archivo`, `escribir_archivo`, `existe_archivo`, `listar_directorio` |
| `matrices` | `crear_matriz`, `multiplicar`, `transponer`, `determinante` |

---

## Entrada/Salida

```lumen
imprimir("Hola Mundo");       // stdout
texto entrada = leer();       // stdin (una línea)
```
