# Referencia del Lenguaje LÚMEN

LÚMEN es un lenguaje de programación educativo con sintaxis en español y equivalentes opcionales en inglés.

## Tipos de Datos

| Tipo | Descripción | Ejemplo |
|------|-------------|---------|
| `entero` / `integer` | Entero 64 bits con signo | `42`, `-10` |
| `decimal` / `float` | Float 64 bits IEEE-754 | `3.14`, `-0.5` |
| `numero` / `number` | Alias de `decimal` | `3.14` |
| `texto` / `string` | Cadena UTF-8 | `"Hola"` |
| `booleano` / `boolean` | Booleano | `verdadero` / `cierto` / `true`, `falso` / `false` |
| `lista<T>` / `array<T>` | Lista dinámica de tipo `T` | `[1, 2, 3]` |
| `funcion(...) -> T` / `function(...) -> T` | Tipo función | `funcion(entero) -> entero` |
| `estructura { ... }` / `struct { ... }` | Tipo estructura | `Persona { nombre: texto }` |

## Variables

Declaración con tipo explícito:
```lumen
entero edad = 25;
texto nombre = "Ana";
booleano activo = verdadero;
```

Asignación:
```lumen
edad = 26;
nombre = "Luis";
```

## Operadores

Aritméticos: `+`, `-`, `*`, `/`

Comparación: `==`, `!=`, `<`, `>`, `<=`, `>=`

Lógicos: `&&` / `y` (and), `||` / `o` (or), `!` / `no` (not)

## Control de Flujo

### si / sino (if/else)
```lumen
si (edad >= 18) {
    imprimir("Mayor de edad");
} sino {
    imprimir("Menor de edad");
}
```

### mientras (while)
```lumen
entero i = 0;
mientras (i < 5) {
    imprimir(i);
    i = i + 1;
}
```

### para (for)
```lumen
para (entero i = 0; i < 10; i = i + 1) {
    imprimir(i);
}
```

### elegir (match)
```lumen
elegir (x) {
    caso 1: imprimir("uno");
    caso 2: imprimir("dos");
    defecto: imprimir("otro");
}
```

### romper / continuar (break/continue)
```lumen
mientras (verdadero) {
    si (i == 5) { romper; }
    si (i == 2) { i = i + 1; continuar; }
    imprimir(i);
    i = i + 1;
}
```

## Funciones

Declaración:
```lumen
funcion entero suma(entero a, entero b) {
    retornar a + b;
}
```

Parámetros default:
```lumen
funcion entero suma(entero a, entero b = 10) {
    retornar a + b;
}
imprimir(suma(5));    // 15
imprimir(suma(5, 20)); // 25
```

## Lambdas / Closures

IIFE (Invocación Inmediata):
```lumen
entero r = funcion(entero x) { retornar x * 2; }(5);
imprimir(r); // 10
```

Asignable:
```lumen
dup = funcion(entero x) { retornar x * 2; };
imprimir(dup(5)); // 10
```

## Estructuras

Declaración:
```lumen
estructura Persona {
    nombre: texto,
    edad: entero
}
```

Inicialización y acceso:
```lumen
Persona p = Persona { nombre: "Ana", edad: 30 };
imprimir(p.nombre);
p.edad = 31;
```

## Listas / Arrays

```lumen
lista<entero> nums = [1, 2, 3];
nums.agregar(4);
imprimir(nums.largo()); // 4
imprimir(nums[0]);      // 1
```

## Módulos

```lumen
importar "math.nv";
importar utils;
importar "datos.nv" como datos;
```

## Comentarios

```lumen
// Comentario de línea
/* Comentario de bloque */
```

## Entrada/Salida

```lumen
imprimir("Hola Mundo");   // print
texto entrada = leer();   // read input
```
