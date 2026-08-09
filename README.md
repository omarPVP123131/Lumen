# LÚMEN — Lenguaje de Programación Nativo en Español

[![CI](https://github.com/omarPVP123131/Lumen/actions/workflows/ci.yml/badge.svg)](https://github.com/omarPVP123131/Lumen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-2.4.1-orange)
![Tests](https://img.shields.io/badge/tests-378%20passing-brightgreen)
![Fases](https://img.shields.io/badge/fases-0--185%20completadas%2Bblueviolet)

> **El primer lenguaje de programación moderno con el español como ciudadano de primera clase.**
> Pipeline completo escrito en Rust: Lexer → Parser → Sema → IR → Optimizador → Bytecode → VM.
> Compila a WASM. Corre en navegador, terminal y Docker.

---

## 🚀 Inicio Rápido

### Con Docker
```bash
docker compose up lumen
```

### Playground Web
```bash
cd crates/lumen-wasm
python serve.py
# Abrir http://localhost:8080/web/index.html
```

### Compilar desde fuente
```bash
git clone https://github.com/omarPVP123131/Lumen.git
cd Lumen
cargo build --release
./target/release/lumen run examples/demo_completo.nv
```

---
## 💡 El Lenguaje en un Vistazo

### Sintaxis Dual ES/EN

| Español | English |
|---------|---------|
| `funcion entero suma(entero a, entero b)` | `function integer sum(integer a, integer b)` |
| `imprimir("hola")` | `print("hello")` |
| `si x > 5 { } sino { }` | `if x > 5 { } else { }` |
| `mientras cond { }` | `while cond { }` |
| `para x en lista { }` | `for x in list { }` |

### Variables y Tipos

```nv
entero a = 42;
decimal pi = 3.14159;
texto saludo = "Hola LÚMEN";
booleano activo = verdadero;
lista<entero> nums = [1, 2, 3];
diccionario<texto, entero> edades;
```

### Funciones, Genéricos, Traits

```nv
funcion T identidad<T>(T valor) { retornar valor; }
imprimir(identidad<entero>(42));

rasgo Mostrable { funcion texto mostrar(este); }
impl Mostrable para entero {
    funcion texto mostrar(este) { retornar "Entero: " + a_texto(este); }
}
```

### Enums + Pattern Matching

```nv
enum Color { Rojo, Verde, Azul }
funcion texto describir(Color c) {
    elegir (c) {
        caso Color::Rojo: retornar "rojo";
        caso Color::Verde: retornar "verde";
        caso Color::Azul: retornar "azul";
    }
}
```

### Async / Tasks

```nv
funcion entero trabajo() { retornar 42; }
texto tid = __tarea_lanzar("trabajo");
entero res = __tarea_esperar(tid);
imprimir(res);
```

### JS Interop (WASM)

```nv
__js_call("console_log", "Hola desde LÚMEN!");
texto titulo = __js_eval("document.title");
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
| `lumen lsp` | Servidor LSP (diagnostics, completion, hover, go-to-def) |
| `lumen install <paquete>` | Instala paquetes del registry |

---

## 🌐 Playground WebAssembly

```bash
cd crates/lumen-wasm
wasm-pack build --target web --out-dir pkg
python serve.py
# http://localhost:8080/web/index.html
```

El playground incluye:
- Editor con 19 ejemplos interactivos
- Consola JS para interop LÚMEN ↔ JavaScript
- Ejecución en tiempo real
- Temas oscuro Catppuccin

---

## 🐳 Docker

```bash
docker build -t lumen:latest .
docker run -it lumen:latest run examples/demo_completo.nv
docker compose up  # Incluye lumen + lumen-repl
```

---

## 📊 Estado del Proyecto (v2.4.0)

```
Lenguaje Core         ████████████████████ 100% (0-60)
Lenguaje Avanzado     ████████████████████ 100% (61-70)
Herramientas & DX     ████████████████████ 100% (71-95)
Stdlib Extendida      ████████████████████ 100% (96-110)
Runtime & Sistema     ████████████████████ 100% (111-130)
Concurrencia & Async  ████████████████████ 100% (131-150)
GUI, TUI & Juegos     ████████████████████ 100% (151-170)
Portabilidad          ████████████████████ 100% (171-185)
Self-hosting          ████████████████████ 100% (compilador LÚMEN en LÚMEN, fixpoint)
AI/ML & Cloud         ░░░░░░░░░░░░░░░░░░░░   0% (186-220)
```

- ✅ **~378 tests** pasando, ~0 warnings (375/375 cargo test)
- ✅ **116 ejemplos** `.nv` — 108/108 CORRECTOS en la cadena 100% LÚMEN (fuego.ps1)
- ✅ **15 crates**: lexer, parser, sema, ir, codegen, vm, cli, fmt, repl, project, lsp, doc, aot, pkg, wasm
- ✅ **Self-hosting total**: compiler_v4.nv se compila a sí mismo (fixpoint byte-idéntico, 5s) + VM en LÚMEN (vm.nv)
- ✅ **Docker** multi-stage + docker-compose
- ✅ **WASM** playground con JS interop integrado
- ✅ **Sintaxis dual ES/EN** en todo el stdlib

---

## 📚 Documentación

| Documento | Descripción |
|-----------|-------------|
| [LENGUAJE.md](LENGUAJE.md) | Manual completo del lenguaje |
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
Skills de desarrollo en `.opencode/agents/lumen-engineer.md` y `.opencode/agents/lumen-tester.md`.

---

MIT License — © 2026 Omar Palomares Velasco
