# LÚMEN — Guía rápida de sintaxis y uso

Bienvenido a **LÚMEN**, un lenguaje educativo y una máquina virtual simple para aprender cómo funciona un compilador y una VM. Este README te da lo esencial para empezar rápido: sintaxis, ejemplos, cómo compilar/ejecutar y resolución de problemas comunes.

---

## Contenido

1. Introducción rápida
2. Características principales
3. Sintaxis (español e inglés)
4. Directivas de archivo
5. Ejemplos (rápidos)
6. CLI (ejecutar / compilar / check)
7. Mensajes de error y debugging básico
8. Cómo contribuir / siguiente pasos

---

## 1 — Introducción rápida

LÚMEN es un lenguaje *didáctico* que se compila a un bytecode ejecutado por una VM basada en pila. Está diseñado para experimentar con compiladores, bytecode y conceptos de ejecución: variables, expresiones, comparaciones, condicionales y bucles.

Por defecto el lenguaje está en **español**, pero puedes forzar **inglés** con una directiva en la primera línea (`import english`).

---

## 2 — Características principales

* Tipos simples: enteros (`i32`).
* Declaración de variables y asignaciones.
* Expresiones aritméticas con precedencia: `*` `/` > `+` `-`.
* Comparadores: `==`, `<`, `>`.
* Control de flujo: `si ... sino ...`, `mientras`.
* `imprimir` para salida (o `print` en inglés).
* Errores de sintaxis con posición (`[línea:col]`).
* Verificación semántica básica: detecta uso de variables antes de declarar.

---

## 3 — Sintaxis

### Keywords (internas)

* Español (por defecto): `numero`, `imprimir`, `si`, `sino`, `mientras`.
* Inglés: `number`, `print`, `if`, `else`, `while` (activa con `import english`).

> Nota: internamente las keywords se mapean a los mismos tokens, así que ambos idiomas generan el mismo bytecode.

### Declaración y asignación

```lumen
numero x = 10       # declarar x con 10
x = x + 1           # asignación (se permite también como VarDecl en el AST)
```

### Imprimir / salida

```lumen
imprimir x          # válido
imprimir(x + 2)     # también válido
```

En inglés (directiva `import english`):

```lumen
import english
number x = 5
print x
```

### Condicionales

```lumen
si (x < y) {
    imprimir x
} sino {
    imprimir y
}
```

### Bucles

```lumen
mientras (contador < 5) {
    imprimir(contador)
    contador = contador + 1
}
```

### Expresiones y operadores

* Aritmética: `+ - * /` (precedencia correcta, paréntesis soportados).
* Comparaciones: `==`, `<`, `>`.
* Unario: `-expr` (se implementa como `0 - expr`).

---

## 4 — Directivas de archivo

* `import english` — si la **primera línea no vacía** del archivo es esta directiva, el compilador tratará las keywords en inglés. La directiva se elimina antes del análisis léxico para evitar interferir con el código.
* `import spanish` — fuerza el idioma español (opcional si quieres dejarlo explícito).

---

## 5 — Ejemplos rápidos

### Factorial en español (ejemplo corto)

```lumen
numero n = 5
numero res = 1
mientras (n > 1) {
    res = res * n
    n = n - 1
}
imprimir(res)
```

### Factorial en inglés (directiva)

```lumen
import english
number n = 5
number res = 1
while (n > 1) {
    res = res * n
    n = n - 1
}
print(res)
```

---

## 6 — CLI: compilar y ejecutar

El proyecto incluye una CLI mínima (`lumen`) con estos comandos:

* `lumen run <file.lumen>` — compila y ejecuta el archivo.
* `lumen build <file.lumen>` — compila a bytecode y escribe `file.nvc`.
* `lumen check <file.lumen>` — verifica sintaxis y semántica (no ejecuta).

Ejemplos:

```bash
cargo build --release
# Ejecutar (release)
target/release/lumen run ejemplos/factorial.lumen
# Generar bytecode
target/release/lumen build ejemplos/factorial.lumen
```

También puedes usar `cargo run -- run ejemplos/factorial.lumen` durante desarrollo.

---

## 7 — Mensajes de error y debugging básico

* Errores de sintaxis incluyen la posición: `[línea:col]` para saber exactamente dónde está el problema.
* Error semántico común: "variable 'x' usada antes de declarar" → declarar la variable con `numero` / `number` antes.
* Si ves `Se esperaba '=' después del identificador, encontré: Identifier(english)` significa que el compilador leyó `import english` como código. Asegúrate de que `import english` sea la primera línea no vacía del archivo.

Si el lexer encuentra un carácter inesperado actualmente producirá `panic!`; en próximas versiones esto se convertirá en un error de compilación amigable.

---

## 8 — Cómo contribuir y siguientes pasos

Temas recomendados para mejorar el lenguaje (puedes trabajar en cualquiera de estos):

* Añadir comentarios `//` y `/* */` en el lexer.
* Soportar `;` como terminador opcional de sentencias.
* Añadir `--verbose` a la CLI para mostrar tokens / AST / bytecode.
* Agregar posiciones (span) en los nodos del AST para mensajes semánticos más precisos.
* Tests automatizados para todos los ejemplos y casos límite.

---

## Archivos relevantes (para mirar el código)

* `src/compiler/lexer.rs` — análisis léxico (tokens + posiciones).
* `src/compiler/parser.rs` — parser recursivo-descendente con precedencia.
* `src/compiler/codegen.rs` — genera bytecode para la VM.
* `src/vm.rs` — máquina virtual que ejecuta el bytecode.
* `src/cli/mod.rs` — interface de línea de comandos.

---

Si quieres, puedo:

* Añadir ejemplos adicionales (fibonacci, suma de array, condicionales anidados).
* Generar un archivo `examples/` con `.lumen` listos para probar.
* Añadir sección con preguntas frecuentes (FAQ).

¿Quieres que añada ejemplos concretos (por ejemplo: factorial, fibonacci, tablas) dentro del repo `examples/`?```
