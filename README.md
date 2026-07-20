# LÚMEN v1.0

[![CI](https://github.com/omarPVP123131/Lumen/actions/workflows/ci.yml/badge.svg)](https://github.com/omarPVP123131/Lumen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-1.0.0-orange)

**Lenguaje de programación educativo de alto rendimiento** con sintaxis en español. Compilador propio (lexer → parser → IR → bytecode → VM) escrito en Rust.

## Quick Start

```bash
git clone https://github.com/omarPVP123131/Lumen.git
cd Lumen
cargo build --release
./target/release/lumen run examples/hello.nv
```

## Características

- **Sintaxis en español** con equivalentes opcionales en inglés
- **Tipado estático**: `entero`, `decimal`, `texto`, `booleano`, `lista<T>`
- **Estructuras de control**: `si`/`sino`, `mientras`, `para`, `elegir`/`match`
- **Funciones**: parámetros default, lambdas IIFE, closures
- **Estructuras/Objetos**: declaración, inicialización, acceso a campos
- **Arrays dinámicos**: `lista<T>` con `agregar`/`largo`
- **Módulos**: `importar` con alias y detección circular
- **Sin dependencias externas** en runtime
- **213 tests**, 0 warnings

## Comandos

| Comando | Descripción |
|---------|-------------|
| `lumen run <file>` | Ejecutar .nv o .nvc |
| `lumen build <file>` | Compilar a .nvc |
| `lumen check <file>` | Verificar sintaxis + semántica |
| `lumen disasm <file>` | Desensamblar .nvc |
| `lumen run -L <dir> <file>` | Ejecutar con ruta de librerías |

## Ejemplo

```lumen
funcion entero factorial(entero n) {
    si (n <= 1) { retornar 1; }
    retornar n * factorial(n - 1);
}
imprimir(factorial(5));
```

## Documentación

- [Referencia del lenguaje](docs/language.md)
- [CLI](docs/cli.md)
- [Arquitectura](docs/architecture.md)
- [Roadmap](docs/roadmap.md)
- [Contribuir](docs/contributing.md)
- [Especificación bytecode](docs/spec/bytecode-format.md)
- [Especificación VM](docs/spec/vm-spec.md)

## Licencia

MIT — © 2026 Omar Palomares Velasco
