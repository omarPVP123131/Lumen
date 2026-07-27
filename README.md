# LÚMEN — Lenguaje de Programación Nativo en Español

[![CI](https://github.com/omarPVP123131/Lumen/actions/workflows/ci.yml/badge.svg)](https://github.com/omarPVP123131/Lumen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-1.6.0-orange)
![Tests](https://img.shields.io/badge/tests-317%20passing-brightgreen)
![Fases](https://img.shields.io/badge/fases-0--95%20completadas-blueviolet)

> **El primer lenguaje de programación moderno con el español como ciudadano de primera clase.**
> Pipeline completo escrito en Rust: Lexer → Parser → Sema → IR → Optimizador → Bytecode → VM.

---

## 🚀 Inicio Rápido

### Instalación (binario)

1. Descarga el ejecutable en [Releases](https://github.com/omarPVP123131/Lumen/releases)
2. Agrégalo a tu `PATH` y ejecuta:

```bash
lumen run mi_programa.nv
```

### Compilar desde fuente

```bash
git clone https://github.com/omarPVP123131/Lumen.git
cd Lumen
cargo build --release
./target/release/lumen --help
```

---

## 💡 El Lenguaje en un Vistazo

### Hola Mundo

```nv
imprimir("¡Hola, LÚMEN!");
```

### Variables e Inferencia de Tipos

```nv
x = 42;             // entero (inferido)
nombre = "Ana";     // texto (inferido)
activo = verdadero; // booleano (inferido)
```

### Funciones, Genéricos y Traits

```nv
rasgo Mostrable {
    funcion texto mostrar(este);
}

funcion T identidad<T>(T valor) {
    retornar valor;
}

imprimir(identidad<entero>(42));
imprimir(identidad<texto>("LÚMEN"));
```

### Enums con Datos y Pattern Matching

```nv
enum Forma {
    Circulo(decimal),
    Rectangulo(decimal, decimal)
}

funcion decimal area(Forma f) {
    elegir f {
        caso Forma::Circulo(r)       { retornar 3.14159 * r * r; }
        caso Forma::Rectangulo(b, h) { retornar b * h; }
    }
}

Forma mi_forma = Forma::Circulo(5.0);
imprimir("Área: ", area(mi_forma)); // 78.53975
```

### Guard Let

```nv
sea Algun(valor) = obtener_config() sino {
    imprimir("Sin configuración, usando defaults");
    retornar;
}
imprimir("Config cargada: ", valor);
```

### Sobrecarga de Operadores

```nv
estructura Vector2D { x: decimal, y: decimal }

impl Suma para Vector2D {
    funcion Vector2D sumar(Vector2D self, Vector2D otro) {
        retornar Vector2D { x: self.x + otro.x, y: self.y + otro.y };
    }
}

Vector2D a = Vector2D { x: 1.0, y: 2.0 };
Vector2D b = Vector2D { x: 3.0, y: 4.0 };
Vector2D c = a + b; // { x: 4.0, y: 6.0 }
```

### Tipos Asociados en Traits

```nv
rasgo Contenedor {
    tipo Item;
    funcion Item obtener(este);
}

impl Contenedor para Caja {
    tipo Item = entero;
    funcion entero obtener(este) { retornar este.valor; }
}
```

### String Interpolation

```nv
texto saludo = "Hola {nombre}, tienes {edad} años.";
imprimir(saludo);
```

---

## 🛠️ Herramientas

| Comando | Descripción |
|---------|-------------|
| `lumen run <archivo>` | Ejecuta fuente `.nv` o bytecode `.nvc` |
| `lumen build <archivo>` | Compila a bytecode optimizado `.nvc` |
| `lumen check <archivo>` | Análisis léxico + semántico sin ejecutar |
| `lumen disasm <archivo>` | Desensambla bytecode a texto legible |
| `lumen fmt <archivo>` | Formatea código (soporta `.lumen-fmt.toml`) |
| `lumen repl` | REPL interactivo con historial y autocompletado |
| `lumen new <nombre>` | Crea proyecto con scaffolding y `lumen.toml` |
| `lumen test <archivo>` | Ejecuta funciones `test_*` |
| `lumen lint <archivo>` | Análisis estático: código muerto, complejidad |
| `lumen doc <archivo>` | Genera HTML desde comentarios `///` |
| `lumen debug <archivo>` | Depurador con breakpoints e inspección |
| `lumen serve` | Servidor de desarrollo con hot reload |
| `lumen lsp` | Servidor LSP para VS Code (diagnostics, completion, hover, go-to-def) |

---

## 📊 Estado del Proyecto (v1.6.0)

```
Lenguaje Core      (Fases 0-60)   ████████████████████ 100%
Lenguaje Avanzado  (Fases 61-70)  ████████████████████ 100%
Herramientas       (Fases 71-95)  ████████████████████ 100%
Distribución       (Fases 96-110) ██░░░░░░░░░░░░░░░░░░  20%
```

- ✅ **317 tests** pasando, ~0 warnings
- ✅ **45 ejemplos** `.nv` funcionales
- ✅ **14 crates**: lexer, parser, sema, ir, codegen, vm, cli, fmt, repl, project, lsp, doc, aot, pkg

---

## 📚 Documentación

| Documento | Descripción |
|-----------|-------------|
| [LENGUAJE.md](LENGUAJE.md) | Manual completo del lenguaje — la Biblia de LÚMEN |
| [HERRAMIENTAS.md](HERRAMIENTAS.md) | Guía de herramientas, CI/CD y flujo de trabajo |
| [docs/language.md](docs/language.md) | Referencia rápida de sintaxis |
| [docs/cli.md](docs/cli.md) | Referencia completa de comandos CLI |
| [docs/architecture.md](docs/architecture.md) | Arquitectura interna del compilador y VM |
| [docs/roadmap.md](docs/roadmap.md) | Roadmap completo v1.0 → v3.0 |
| [CHANGELOG.md](CHANGELOG.md) | Historial de versiones |
| [MARKETING.md](MARKETING.md) | Visión, posicionamiento y comparativas |

---

## ❤️ Contribuir

Abre un *Issue* o *Pull Request*. Consulta [CONTRIBUTING.md](docs/contributing.md) para la guía completa.

---

MIT License — © 2026 Omar Palomares Velasco
