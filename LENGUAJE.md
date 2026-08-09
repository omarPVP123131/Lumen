# LÚMEN — Manual del Lenguaje

**v2.4.1 — La Biblia de LÚMEN**

> Un lenguaje de programación en español diseñado para aprender a programar sin fricción.  
> Simple, expresivo, y con esteroides de Rust por dentro.

---

## Índice

1. [Primeros Pasos](#1-primeros-pasos)
2. [Variables y Tipos](#2-variables-y-tipos)
3. [Operadores](#3-operadores)
4. [Texto (Strings)](#4-texto-strings)
5. [Listas (Arrays)](#5-listas-arrays)
6. [Diccionarios (Mapas)](#6-diccionarios-mapas)
7. [Control de Flujo](#7-control-de-flujo)
8. [Funciones](#8-funciones)
9. [Estructuras (Structs)](#9-estructuras-structs)
10. [Enums](#10-enums)
11. [Tuplas y Destructuring](#11-tuplas-y-destructuring)
12. [Genéricos](#12-genéricos)
13. [Resultados y Opciones](#13-resultados-y-opciones)
14. [Módulos e Imports](#14-módulos-e-imports)
15. [Tests y Afirmaciones](#15-tests-y-afirmaciones)
16. [Funciones Integradas (Builtins)](#16-funciones-integradas-builtins)
17. [Librería Estándar](#17-librería-estándar)
18. [Conversiones de Tipo](#18-conversiones-de-tipo)
19. [Concatenación Mixta](#19-concatenación-mixta)
20. [Indexación de Texto](#20-indexación-de-texto)
21. [Sobrecarga de Operadores](#21-sobrecarga-de-operadores)
22. [Guard Let](#22-guard-let)
23. [Impl Trait Return](#23-impl-trait-return)
24. [Ejemplos Completos](#24-ejemplos-completos)

---

## 1. Primeros Pasos

### Instalación

```bash
cargo install lumen
```

### Hola Mundo

Crea un archivo `hola.nv`:

```nv
imprimir("¡Hola, LÚMEN!");
```

Ejecuta:

```bash
lumen run hola.nv
```

### Modo Dual Inglés/Español

Todos los ejemplos en este manual usan `importar ingles;` en la primera línea. Esto habilita tanto español como inglés:

| Español | Inglés |
|---------|--------|
| `funcion` | `function` |
| `si` | `if` |
| `sino` | `else` |
| `mientras` | `while` |
| `para` | `for` |
| `retornar` | `return` |
| `verdadero` | `true` |
| `falso` | `false` |
| `imprimir` | `print` |
| `entero` | `integer` |
| `decimal` | `float` |
| `texto` | `string` |
| `booleano` | `boolean` |
| `lista` | `array` |
| `diccionario` | `dictionary` |
| `estructura` | `struct` |
| `importar` | `import` |

> **Regla**: Sin `importar ingles;`, solo funcionan las palabras en español. Con `importar ingles;`, ambas funcionan.

---

## 2. Variables y Tipos

### Tipos Primitivos

```nv
entero edad = 25;            // Número entero (i64)
decimal precio = 19.99;      // Número decimal (f64)  
texto nombre = "Ana";        // Cadena de texto
booleano activo = verdadero; // Verdadero o falso
```

### Declaración de Variables

```nv
// Con tipo explícito (recomendado)
entero x = 10;
texto s = "hola";

// Inferencia de tipo con :=
x := 42;
nombre := "LÚMEN";
```

### Tipos Compuestos

```nv
// Lista (array dinámico)
lista<entero> numeros = [1, 2, 3, 4, 5];
lista<texto> frutas = ["manzana", "pera", "uva"];

// Diccionario (mapa clave-valor)
diccionario<texto, entero> edades = { "Ana": 25, "Luis": 30 };

// Tupla
(entero, texto, booleano) tupla = (42, "hola", verdadero);

// Resultado (éxito o error)
Resultado<entero, texto> res = exito(42);

// Opcion (algo o nada)
Opcion<entero> talvez = algun(10);
Opcion<entero> nada = ninguno;
```

---

## 3. Operadores

### Aritméticos

```nv
entero suma = 5 + 3;        // 8
entero resta = 10 - 4;      // 6
entero producto = 3 * 7;    // 21
entero division = 7 / 2;    // 3 (¡división entera!)
entero modulo = 10 % 3;     // 1 (resto)
entero negativo = -(5);     // -5
```

> **Importante**: `entero / entero` siempre retorna entero (truncado hacia cero).  
> Para división decimal, usa operandos tipo `decimal`: `7.0 / 2.0 = 3.5`

### Comparación

```nv
booleano igual = a == b;     // Igualdad
booleano distinto = a != b;  // Desigualdad
booleano menor = a < b;      // Menor que
booleano mayor = a > b;      // Mayor que
booleano menor_igual = a <= b;
booleano mayor_igual = a >= b;
```

> Las comparaciones `<`, `>`, `<=`, `>=` solo funcionan con números.  
> Para texto usa `==` y `!=` para igualdad.

### Lógicos

```nv
booleano condicion = (x > 0) y (x < 10);   // Y lógico (and)
booleano alterna = (x < 0) o (x > 100);    // O lógico (or)
booleano negado = !activo;                  // Negación (not)
```

### Precedencia

De mayor a menor:
1. `!` (not), `-` (negación)
2. `*`, `/`, `%`
3. `+`, `-`
4. `<`, `>`, `<=`, `>=`
5. `==`, `!=`
6. `y` (and)
7. `o` (or)

Usa paréntesis para agrupar: `(a + b) * c`

---

## 4. Texto (Strings)

### Creación y Concatenación

```nv
texto saludo = "Hola";
texto nombre = "Mundo";
texto completo = saludo + " " + nombre;   // "Hola Mundo"

// Concatenación mixta (auto-conversión)
texto info = "Edad: " + 25;                // "Edad: 25"
texto precio_str = "Precio: $" + 19.99;    // "Precio: $19.99"
texto estado = "Activo: " + verdadero;      // "Activo: true"
```

### Interpolación

Las variables dentro de `{}` se sustituyen por su valor:

```nv
texto nombre = "Ana";
entero edad = 25;
imprimir("Me llamo {nombre} y tengo {edad} años");
// Salida: Me llamo Ana y tengo 25 años
```

### Longitud

```nv
texto s = "LÚMEN";
entero len = s.largo();       // 5
imprimir(largo(s));           // 5
```

### Indexación

Accede a caracteres individuales como `texto` de largo 1:

```nv
texto s = "hola";
texto primera = s[0];          // "h"
texto ultima = s[3];            // "a"

// Recorrer caracteres
entero i = 0;
mientras i < s.largo() {
    imprimir(s[i]);
    i = i + 1;
}
```

### Métodos de Texto (Builtins)

```nv
texto s = "  Hola Mundo  ";

// Mayúsculas / Minúsculas
__str_mayusculas(s);     // "  HOLA MUNDO  "
__str_minusculas(s);     // "  hola mundo  "

// Recortar espacios
__str_recortar(s);       // "Hola Mundo"

// Contiene subtexto
__str_contiene(s, "Mun");    // verdadero

// Dividir por delimitador
lista<texto> partes = __str_dividir("a,b,c", ",");  // ["a", "b", "c"]

// Dividir en caracteres (delimitador vacío)
lista<texto> chars = __str_dividir("hola", "");     // ["h", "o", "l", "a"]
```

---

## 5. Listas (Arrays)

### Creación y Acceso

```nv
lista<entero> nums = [10, 20, 30, 40];
entero primero = nums[0];        // 10
nums[2] = 35;                    // Modifica el tercer elemento
```

### Operaciones con Listas

```nv
lista<texto> frutas = ["manzana", "pera"];

// Agregar al final
frutas.agregar("uva");           // ["manzana", "pera", "uva"]

// Longitud
entero n = frutas.largo();       // 3
imprimir(largo(frutas));         // 3

// Recorrer con para-cada
para fruta en frutas {
    imprimir(fruta);
}
```

### Rangos

```nv
// Rango crea una lista de enteros
lista<entero> r = 0..5;           // [0, 1, 2, 3, 4]

para i en 0..10 {
    imprimir(i);                  // 0, 1, 2, ..., 9
}
```

### Listas Anidadas

```nv
lista<lista<entero>> matriz = [
    [1, 2, 3],
    [4, 5, 6],
    [7, 8, 9]
];
imprimir(matriz[1][2]);           // 6
```

---

## 6. Diccionarios (Mapas)

```nv
// Creación
diccionario<texto, entero> edades = {
    "Ana": 25,
    "Luis": 30,
    "Carlos": 22
};

// Acceso
imprimir(edades["Ana"]);          // 25

// Asignación (agregar o modificar)
edades["Diana"] = 28;             // Agrega nueva clave
edades["Ana"] = 26;               // Modifica existente

// Longitud
entero n = largo(edades);         // 4

// Recorrer claves y valores
// (usa __str_dividir para extraer claves si necesitas iterar)
```

---

## 7. Control de Flujo

### Si / Sino / Sino Si

```nv
entero nota = 85;

si nota >= 90 {
    imprimir("Excelente");
} sino si nota >= 70 {
    imprimir("Aprobado");
} sino {
    imprimir("Reprobado");
}
```

### Mientras (While)

```nv
entero contador = 0;
mientras contador < 5 {
    imprimir(contador);
    contador = contador + 1;
}
// Imprime: 0, 1, 2, 3, 4
```

### Para (For-Each)

```nv
lista<texto> nombres = ["Ana", "Luis", "Carlos"];
para nombre en nombres {
    imprimir(nombre);
}

// Con rango
para i en 0..3 {
    imprimir(i);     // 0, 1, 2
}
```

### Romper y Continuar

```nv
entero i = 0;
mientras i < 10 {
    si i == 5 { romper; }      // Sale del bucle
    si i % 2 == 0 {
        i = i + 1;
        continuar;              // Salta a la siguiente iteración
    }
    imprimir(i);
    i = i + 1;
}
// Imprime: 1, 3
```

### Elegir / Caso (Match)

```nv
entero dia = 3;

elegir dia {
    caso 1 { imprimir("Lunes"); }
    caso 2 { imprimir("Martes"); }
    caso 3 { imprimir("Miércoles"); }
    caso 4 { imprimir("Jueves"); }
    caso 5 { imprimir("Viernes"); }
    defecto { imprimir("Fin de semana"); }
}
```

---

## 8. Funciones

### Definición y Llamada

```nv
funcion entero suma(entero a, entero b) {
    retornar a + b;
}

entero resultado = suma(5, 3);     // 8
imprimir(resultado);
```

### Parámetros con Valor por Defecto

```nv
funcion texto saludar(texto nombre, texto saludo = "Hola") {
    retornar saludo + " " + nombre;
}

imprimir(saludar("Ana"));              // "Hola Ana"
imprimir(saludar("Ana", "Buenos días")); // "Buenos días Ana"
```

### Funciones sin Retorno (void)

```nv
funcion void mostrar_error(texto msg) {
    imprimir("ERROR: " + msg);
}
```

### Funciones Lambda / Anónimas

```nv
// Lambda con parámetros explícitos
(funcion(entero x) { retornar x * x; })(5);   // 25

// Lambda sin parámetros
funcion texto saludar() { retornar "hola"; }
```

### Funciones Anidadas (IIFE)

```nv
entero resultado = (funcion(entero a, entero b) {
    retornar a + b;
})(3, 7);
imprimir(resultado);  // 10
```

### Funciones como Valores

```nv
funcion entero doble(entero x) { retornar x * 2; }

// Pasar función como argumento
funcion entero aplicar(funcion(entero) entero f, entero x) {
    retornar f(x);
}

entero r = aplicar(doble, 5);   // 10
```

---

## 9. Estructuras (Structs)

### Definición

```nv
estructura Persona {
    nombre: texto,
    edad: entero,
    activo: booleano,
}
```

### Creación e Instancias

```nv
Persona p = Persona {
    nombre: "Ana García",
    edad: 30,
    activo: verdadero,
};

// Acceso a campos
imprimir(p.nombre);        // "Ana García"
imprimir(p.edad);          // 30

// Modificar campos
p.edad = 31;

// Copiar estructura (copia independiente)
Persona copia = p;
copia.edad = 25;
imprimir(p.edad);           // 31 (la original no cambia)
imprimir(copia.edad);       // 25
```

### Métodos con `impl`

```nv
estructura Contador {
    valor: entero,
}

impl Contador {
    funcion void incrementar(self) {
        self.valor = self.valor + 1;
    }

    funcion entero obtener(self) {
        retornar self.valor;
    }

    funcion void reiniciar(self) {
        self.valor = 0;
    }
}

Contador c = Contador { valor: 0 };
c.incrementar();
c.incrementar();
imprimir(c.obtener());      // 2
c.reiniciar();
imprimir(c.obtener());      // 0
```

> **Nota sobre `self`**: El primer parámetro `self` no necesita tipo explícito. El compilador lo resuelve automáticamente al tipo de la estructura.

---

## 10. Enums

### Enums Simples

```nv
enum Color { Rojo, Verde, Azul }

Color favorito = Color::Rojo;
```

### Enums con Datos

```nv
enum Forma {
    Circulo(decimal),          // Radio
    Rectangulo(decimal, decimal),  // Ancho, Alto
    Punto,
}

Forma f1 = Forma::Circulo(5.0);
Forma f2 = Forma::Rectangulo(3.0, 4.0);
Forma f3 = Forma::Punto;

// Pattern matching con elegir
elegir f1 {
    caso Circulo(r) { imprimir("Círculo de radio " + r); }
    caso Rectangulo(a, h) { imprimir("Rectángulo " + a + "x" + h); }
    caso Punto { imprimir("Un punto"); }
}
```

---

## 11. Tuplas y Destructuring

### Tuplas

```nv
(entero, texto, booleano) t = (42, "hola", verdadero);

// Acceso por índice (0-based)
imprimir(t.0);    // 42
imprimir(t.1);    // "hola"
imprimir(t.2);    // verdadero
```

### Destructuring

```nv
(entero, texto) par = (10, "diez");

// Desempaquetar en variables
(entero num, texto palabra) = par;
imprimir(num);      // 10
imprimir(palabra);  // "diez"

// Ignorar valores con _
(entero x, _) = par;
imprimir(x);        // 10
```

---

## 12. Genéricos

### Funciones Genéricas

```nv
// Función identidad genérica
funcion T identidad<T>(T valor) {
    retornar valor;
}

entero a = identidad<entero>(42);
texto b = identidad<texto>("hola");

// Inferencia de tipo
entero c = identidad(10);            // El tipo se infiere
```

### Primer Elemento de Lista Genérica

```nv
funcion T primero<T>(lista<T> items) {
    retornar items[0];
}

lista<entero> nums = [5, 10, 15];
entero p = primero<entero>(nums);     // 5
```

---

## 13. Resultados y Opciones

### Resultado<T, E>

Representa éxito (`exito(valor)`) o error (`error(mensaje)`).

```nv
// Crear resultados
Resultado<entero, texto> ok = exito(42);
Resultado<entero, texto> err = error("Algo salió mal");

// Pattern matching
funcion texto describir(Resultado<entero, texto> res) {
    elegir res {
        caso exito(v) { retornar "Éxito: " + v; }
        caso error(e) { retornar "Error: " + e; }
    }
}
```

### Opcion<T>

Representa un valor que puede o no existir.

```nv
// Crear opciones
Opcion<entero> presente = algun(100);
Opcion<entero> ausente = ninguno;

// Pattern matching
funcion texto mostrar(Opcion<entero> opt) {
    elegir opt {
        caso algun(v) { retornar "Valor: " + v; }
        caso ninguno { retornar "Sin valor"; }
    }
}
```

---

## 14. Módulos e Imports

### Importar un Módulo

```nv
importar "matematicas.nv";

entero r = matematicas_suma(3, 5);
```

> Las funciones del módulo se importan con el prefijo `{nombre_archivo}_`.  
> Ejemplo: `matematicas.nv` → `matematicas_suma()`, `matematicas_resta()`

### Importar con Alias

```nv
importar "matematicas.nv" como math;

entero r = math_suma(3, 5);
```

### Crear un Módulo

```nv
// archivo: matematicas.nv
funcion entero suma(entero a, entero b) {
    retornar a + b;
}

funcion entero resta(entero a, entero b) {
    retornar a - b;
}
```

### Importar varios módulos

```nv
importar "matematicas.nv";
importar "texto.nv";
importar ingles;        // Habilita palabras clave en inglés

entero r = matematicas_suma(1, 2);
entero len = texto_longitud("hola");
```

### Imports Circulares

LÚMEN detecta imports circulares y reporta:

```
E063 Import circular detectado
  --> modulo_a.nv:1:1
  Ayuda: Revisa las dependencias entre módulos
```

---

## 15. Tests y Afirmaciones

### Función `afirmar`

```nv
afirmar(1 + 1 == 2);
afirmar("hola".largo() == 4);
afirmar(verdadero);
```

> Si la condición es falsa, `afirmar` produce un error en tiempo de ejecución.

### Ejecutar Tests

```bash
lumen test archivo.nv
```

LÚMEN busca funciones cuyo nombre empieza con `test_` y las ejecuta:

```nv
funcion texto test_suma() {
    afirmar(2 + 2 == 4);
    afirmar(0 + 0 == 0);
    retornar "ok";
}

funcion texto test_multiplicacion() {
    afirmar(3 * 3 == 9);
    retornar "ok";
}
```

---

## 16. Funciones Integradas (Builtins)

### Entrada/Salida

| Función | Descripción | Ejemplo |
|---------|-------------|---------|
| `imprimir(x)` | Imprime un valor | `imprimir("hola")` |
| `print(x)` | Alias en inglés | `print("hello")` |
| `leer()` | Lee entrada (placeholder) | `leer()` |
| `read()` | Alias en inglés | `read()` |

### Colecciones

| Función | Descripción | Ejemplo |
|---------|-------------|---------|
| `largo(x)` | Longitud de lista/texto/diccionario | `largo([1,2,3])` → `3` |
| `len(x)` | Alias inglés | `len("hola")` → `4` |
| `agregar(lista, item)` | Agrega a lista | `agregar(arr, 5)` |
| `push(lista, item)` | Alias inglés | `push(arr, 5)` |

### Texto

| Función | Descripción | Ejemplo |
|---------|-------------|---------|
| `__str_longitud(s)` | Longitud de texto | `__str_longitud("hola")` → `4` |
| `__str_mayusculas(s)` | Convertir a mayúsculas | `__str_mayusculas("hola")` → `"HOLA"` |
| `__str_minusculas(s)` | Convertir a minúsculas | `__str_minusculas("HOLA")` → `"hola"` |
| `__str_recortar(s)` | Eliminar espacios | `__str_recortar(" hola ")` → `"hola"` |
| `__str_contiene(s, sub)` | Verificar contiene | `__str_contiene("hola", "ol")` → `verdadero` |
| `__str_dividir(s, delim)` | Dividir por delimitador | `__str_dividir("a,b", ",")` → `["a","b"]` |

### Listas

| Función | Descripción | Ejemplo |
|---------|-------------|---------|
| `__lista_invertir(lista)` | Invierte orden | `__lista_invertir([1,2,3])` → `[3,2,1]` |
| `__lista_ordenar(lista)` | Ordena numéricamente | `__lista_ordenar([3,1,2])` → `[1,2,3]` |

### Archivos

| Función | Descripción | Retorna |
|---------|-------------|---------|
| `__leer_archivo(path)` | Lee archivo | `Resultado<texto, texto>` |
| `__escribir_archivo(path, contenido)` | Escribe archivo | `Resultado<booleano, texto>` |
| `__existe_archivo(path)` | Verifica existencia | `booleano` |

### Tiempo

| Función | Descripción | Ejemplo |
|---------|-------------|---------|
| `__tiempo_ahora()` | Timestamp actual (segundos) | `__tiempo_ahora()` |

---

## 17. Librería Estándar

LÚMEN incluye módulos en `stdlib/`:

| Módulo | Archivo | Funciones |
|--------|---------|-----------|
| Matemáticas | `matematicas.nv` | `suma`, `resta`, `multiplica`, `divide` |
| Texto | `texto.nv` | `longitud`, `mayusculas`, `minusculas`, `recortar`, `dividir`, `contiene` |
| Colección | `coleccion.nv` | `largo`, `agregar`, `invertir`, `ordenar` |
| Fecha | `fecha.nv` | Operaciones con fechas |
| Archivos | `archivos.nv` | `leer`, `escribir`, `existe` |

### Uso de la Stdlib

```nv
importar "texto.nv";

texto s = "  Hola  ";
entero len = texto_longitud(s);          // 7
texto trim = texto_recortar(s);          // "Hola"
```

> Las funciones de la stdlib son wrappers de los builtins `__str_*`.  
> Puedes usar los builtins directamente o importar la stdlib para nombres más limpios.

---

## 18. Conversiones de Tipo

### `a_texto(x)` — Cualquier tipo a texto

```nv
texto s1 = a_texto(42);             // "42"
texto s2 = a_texto(3.14);           // "3.14"
texto s3 = a_texto(verdadero);      // "true"
texto s4 = a_texto("ya es texto");  // "ya es texto"
```

### `a_entero(s)` — Texto a entero (con Resultado)

```nv
Resultado<entero, texto> r1 = a_entero("42");

elegir r1 {
    caso exito(n) { imprimir("Éxito: " + n); }
    caso error(e) { imprimir("Error: " + e); }
}

Resultado<entero, texto> r2 = a_entero("no es número");
// r2 es error
```

### `a_decimal(s)` — Texto a decimal (con Resultado)

```nv
Resultado<decimal, texto> r1 = a_decimal("3.14");

elegir r1 {
    caso exito(n) { imprimir("Éxito: " + n); }
    caso error(e) { imprimir("Error: " + e); }
}
```

### Aliases en Inglés

| Español | Inglés |
|---------|--------|
| `a_texto(x)` | `to_texto(x)` |
| `a_entero(s)` | `to_int(s)` |
| `a_decimal(s)` | `to_float(s)` |

---

## 19. Concatenación Mixta

Cuando usas `+` con `texto` y otro tipo, LÚMEN convierte automáticamente:

```nv
texto info = "Puntuación: " + 95;           // "Puntuación: 95"
texto precio = "$" + 19.99;                  // "$19.99"
texto estado = "Estado: " + verdadero;       // "Estado: true"
texto inv = 42 + " es la respuesta";         // "42 es la respuesta"
```

> Esto funciona con entero, decimal y booleano.  
> No necesitas `a_texto()` explícito para concatenar.

---

## 20. Indexación de Texto

Accede a caracteres individuales con `texto[i]`:

```nv
texto s = "LÚMEN";

texto primera = s[0];      // "L"
texto segunda = s[1];      // "Ú"
texto ultima = s[4];       // "N"

// Cada carácter es texto de largo 1
imprimir(s[0].largo());    // 1
```

### Recorrer Caracteres

```nv
texto s = "abc";
entero i = 0;
mientras i < s.largo() {
    imprimir(s[i]);
    i = i + 1;
}
// Imprime: a, b, c
```

### Comparar Caracteres

```nv
texto s = "hola";
si s[0] == "h" {
    imprimir("Empieza con h");
}

// Verificar si es mayúscula
texto c = s[0];
si __str_mayusculas(c) == c {
    imprimir("Es mayúscula");
}
```

---

## 21. Ejemplos Completos

### Calculadora Simple

```nv
importar ingles;

funcion entero suma(entero a, entero b) { retornar a + b; }
funcion entero resta(entero a, entero b) { retornar a - b; }
funcion entero multiplica(entero a, entero b) { retornar a * b; }
funcion entero divide(entero a, entero b) { retornar a / b; }

funcion texto main() {
    imprimir(suma(10, 5));
    imprimir(resta(10, 5));
    imprimir(multiplica(3, 7));
    imprimir(divide(15, 3));
    retornar "ok";
}
```

### Fibonacci

```nv
importar ingles;

funcion entero fib(entero n) {
    si n <= 1 { retornar n; }
    retornar fib(n - 1) + fib(n - 2);
}

funcion texto main() {
    para i en 0..10 {
        imprimir(fib(i));
    }
    retornar "ok";
}
```

### FizzBuzz

```nv
importar ingles;

funcion texto main() {
    para i en 1..21 {
        si i % 15 == 0 { imprimir("FizzBuzz"); }
        sino si i % 3 == 0 { imprimir("Fizz"); }
        sino si i % 5 == 0 { imprimir("Buzz"); }
        sino { imprimir(i); }
    }
    retornar "ok";
}
```

### Gestor de Tareas (ToDo)

```nv
importar ingles;

estructura Tarea {
    titulo: texto,
    completada: booleano,
}

funcion void agregar_tarea(lista<Tarea> tareas, texto titulo) {
    Tarea t = Tarea { titulo: titulo, completada: falso };
    tareas.agregar(t);
}

funcion void completar_tarea(lista<Tarea> tareas, entero indice) {
    Tarea t = tareas[indice];
    t.completada = verdadero;
    tareas[indice] = t;
}

funcion void listar(lista<Tarea> tareas) {
    entero i = 0;
    mientras i < tareas.largo() {
        Tarea t = tareas[i];
        texto estado = "";
        si t.completada { estado = "[X]"; }
        sino { estado = "[ ]"; }
        imprimir(estado + " " + t.titulo);
        i = i + 1;
    }
}

funcion texto main() {
    lista<Tarea> mis_tareas = [];
    agregar_tarea(mis_tareas, "Comprar leche");
    agregar_tarea(mis_tareas, "Llamar al doctor");
    completar_tarea(mis_tareas, 0);
    listar(mis_tareas);
    retornar "ok";
}
```

### Contador de Palabras

```nv
importar ingles;

funcion entero contar_palabras(texto s) {
    lista<texto> partes = __str_dividir(s, " ");
    entero count = 0;
    para p en partes {
        si p != "" { count = count + 1; }
    }
    retornar count;
}

funcion texto main() {
    texto frase = "Hola mundo desde LÚMEN";
    imprimir(contar_palabras(frase));     // 4
    retornar "ok";
}
```

---

## Apéndice: Mensajes de Error

LÚMEN muestra errores con formato rico que incluye:

```
  E031 No puedes asignar un valor de tipo 'Texto' a una variable de tipo 'Entero'
  --> archivo.nv:3:12
   |
 2 | entero x = 0;
 3 | entero y = "hola";
   |            ^^^^^^
   |
   Ayuda: Usa un valor de tipo 'Entero' en lugar de 'Texto'
```

- **Código de error**: Identificador único (`E031`, `E035`, etc.)
- **Flecha** `-->`: Ubicación exacta (archivo:línea:columna)
- **Preview**: La línea con el error y contexto (línea anterior y posterior en gris)
- **Subrayado** `^^^^`: Marca visual del código problemático
- **Ayuda**: Sugerencia concreta para resolver el error

### Códigos de Error Comunes

| Código | Significado |
|--------|------------|
| E001 | Carácter inesperado |
| E020 | Expresión inesperada |
| E031 | Tipo incompatible en asignación |
| E033 | Variable no declarada |
| E035 | Operador requiere tipos compatibles |
| E040 | Número incorrecto de argumentos |
| E041 | Tipo de argumento incorrecto |
| E043 | Índice inválido |
| E044 | Indexación solo para listas/diccionarios/texto |
| E059 | Campo de struct no encontrado |
| E061 | Faltan campos en inicialización de struct |
| E063 | Import circular detectado |

---

## Referencia Rápida

### Palabras Clave

```
funcion  si    sino    mientras  para    retornar
entero   decimal  texto  booleano  void    verdadero  falso
lista    diccionario  estructura  impl   enum
elegir   caso   defecto  romper  continuar
importar  como  ingles  test  afirmar
Resultado  Opcion  exito  error  algun  ninguno
en       self   intentar  atrapar
```

### Operadores

```
+  -  *  /  %      Aritméticos
== != <  >  <= >=   Comparación
y  o  !             Lógicos (and, or, not)
.                   Acceso a campo
::                  Acceso a variante de enum
..                  Rango
```

### Jerarquía de Tipos

```
entero → decimal   (entero es asignable a decimal)
Lista<T>           (acepta Lista<void> como inicializador vacío)
TypeVar            (genéricos sin restricción)
```

---

*LÚMEN v2.4.1 — Creado con amor por la comunidad. Agosto 2026.*
