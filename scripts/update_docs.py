# -*- coding: utf-8 -*-
"""Script para actualizar todos los .md del proyecto LÚMEN al estado v1.6.0"""

import os

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def write(path, content):
    full = os.path.join(ROOT, path)
    with open(full, 'w', encoding='utf-8') as f:
        f.write(content)
    print(f"OK: {path}")

# ==============================================================
# README.md — Cara pública del proyecto
# ==============================================================
write("README.md", """\
# LÚMEN — Lenguaje de Programación Nativo en Español

[![CI](https://github.com/omarPVP123131/Lumen/actions/workflows/ci.yml/badge.svg)](https://github.com/omarPVP123131/Lumen/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
![Version](https://img.shields.io/badge/version-1.6.0-orange)
![Tests](https://img.shields.io/badge/tests-307%20passing-brightgreen)
![Fases](https://img.shields.io/badge/fases-0--83%20completadas-blueviolet)

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
Herramientas       (Fases 71-83)  ███████████████████░  95%
Distribución       (Fases 86-95)  ████████░░░░░░░░░░░░  40%
```

- ✅ **307 tests** pasando, ~9 warnings
- ✅ **44 ejemplos** `.nv` funcionales
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
""")

# ==============================================================
# AGENTS.md — Diario de construcción (sincronizado con v1.6.0)
# ==============================================================
write("AGENTS.md", """\
# AGENTS.md — Diario de construcción de LÚMEN

**v1.6.0 — Release: Julio 2026**

---

## Testing (Actual)

| Crate | Tests | Tipo |
|-------|-------|------|
| lumen-lexer | 24 | unit |
| lumen-parser | 42 | unit |
| lumen-sema | 43 | unit |
| lumen-ir | 20 | unit + folding |
| lumen-codegen | 13 | unit |
| lumen-codegen | 5 | proptest |
| lumen-vm | 45 | unit |
| lumen-vm | 110 | e2e |
| lumen-fmt | 2 | unit |
| lumen-repl | 2 | unit |
| lumen-project | 1 | unit |
| **Total** | **~307** | |

**~9 warnings, ~307 tests passing. 44/44 ejemplos funcionando.**

---

## Fases completadas

### Fases 0-15: Infraestructura base ✅
Lexer, parser, sema, IR, bytecode, VM, CLI, arrays, control de flujo avanzado.

### Fase 16: Funciones avanzadas ✅
Parámetros default, Lambdas IIFE, Closures.

### Fase 17: Estructuras/Objetos ✅
`estructura`, inicialización, acceso, asignación de campos.

### Fase 18: Módulos ✅
`importar`, ModuleLoader, detección circular.

### Fase 19: Optimizaciones ✅
Constant folding, DCE, shared pools.

### Fase 20: v1.0 Release ✅

### Fases 21-27: Features del lenguaje ✅
For-Each, Resultado<T,E>, Opcion<T>, Enums, Tuplas, Destructuring, Genéricos.

### Fases 28-30: Stdlib + Archivos ✅
matematicas, texto, coleccion, fecha, archivos.

### Fase 31: Stack Traces ✅

### Fase 32-33: Mensajes de Error Mejorados ✅
Caret, ANSI, preview multi-línea, conteo de errores.

### Fase 34: Fuzzing ✅
3 targets cargo-fuzz (lexer, parser, decoder).

### Fase 35: Property-Based Testing ✅
Proptest en codegen.

### Fase 37: lumen fmt ✅
`lumen fmt` formatea código .nv. Crate `lumen-fmt`.

### Fase 38: lumen repl ✅
REPL interactivo. Crate `lumen-repl`.

### Fase 39: lumen test ✅
`lumen test` ejecuta funciones `test_*`.

### Fase 40: lumen.toml + lumen new ✅
`lumen new` scaffolding. Crate `lumen-project`.

### Fase 41: CI/CD + Releases ✅
GitHub Actions CI + release multiplataforma.

### Fases 42-57: Lenguaje & Sintaxis (Bloque 1) ✅
- 42: Inferencia de tipos (`x = 42`)
- 43: Métodos en structs (`impl Struct`)
- 44: Diccionarios (`diccionario<K,V>`)
- 45: String interpolation (`"Hola {nombre}"`)
- 46: Rangos (`0..5`, `0..=5`)
- 47: Constantes (`const`)
- 48: String indexing (`s[i]`)
- 49: Conversiones (`a_texto`, `a_entero`, `a_decimal`)
- 50: División entera (`entero/entero → entero`)
- 51: Concatenación mixta (`"x" + 42`)
- 52: Mejores errores (preview multi-línea)
- 53: Operador ternario (`?:`)
- 54: Loop labels (`romper etiqueta`)
- 55: Pattern matching exhaustivo + guardas
- 56: Genéricos con bounds (`<T: Rasgo>`)
- 57: Matrices 2D (`lista<lista<T>>` + stdlib matrices.nv)

### Fase 58: Enums Avanzados ✅
Variantes con datos (`Variant(entero)`).

### Fase 59: Closures Pro ✅
Captura por valor/referencia. Closures movibles.

### Fase 60: Async/Await ✅
Sintaxis `async funcion` / `esperar`. Sema + IR bases.

### Fase 65: Guard Let ✅
`sea patron = expr sino { romper/retornar/continuar }`. Desugaring en IR builder a JmpIf/Jmp.
`Stmt::GuardLet` en AST, `parse_guard_let()` en parser, sema + loader, IR builder.

### Fase 66: Operator Overloading ✅
Vía trait method convention (`impl Suma for Punto`). `Expr::Binary.resolved_method`.
Sema: `resolve_operator_overloads()` post-analysis walk con HashMap de inferencia de tipos.
IR builder: `resolved_method` → `Call` o `Binary` nativo.

### Fase 67: Extension Methods ✅
`impl Trait para TipoPrimitivo` (`impl Duplicable para entero`). `type_to_impl_name()` mapea
`entero`, `texto`, `decimal`, `booleano`, `lista`, `opcion`, `resultado`, `tupla`.

### Fase 68: Tipos Asociados en Traits ✅
`tipo Item;` en traits y `tipo Item = T;` en impl.
`AssociatedType` e `ImplAssociatedType` en AST, sema e IR.

### Fase 69: Where Clauses ⏭️
Saltado — `<T: Rasgo>` syntax ya soporta bounds.

### Fase 70: Impl Trait return ✅
`funcion impl Rasgo foo() { retornar expr; }`. `Type::ImplTrait(String)` en AST.
Parseo en `parse_type()`, mapea a `TypeInfo::TypeVar` en sema.

### Fases 71-74: LSP Server ✅
`lumen lsp` — Diagnósticos en vivo, Autocompletado, Go-to-definition, Hover de tipos.
Crate `lumen-lsp`. Protocolo JSON-RPC sobre stdin/stdout.

### Fase 75: lumen doc ✅
Generación de HTML desde comentarios `///`. Crate `lumen-doc`.

### Fase 76: Debugger ✅
Depurador interactivo con breakpoints, step, continue, inspect de variables.

### Fase 77: lumen fmt avanzado ✅
Soporte para `.lumen-fmt.toml` (`indent_spaces`, etc.). Crate `lumen-fmt`.

### Fase 78: lumen lint ✅
Análisis estático de código muerto y complejidad ciclomática.

### Fase 79: REPL Pro ✅
Historial persistente, multilínea, resaltado de sintaxis, autocompletado.

### Fase 80: Package Manager ✅
`lumen install`, registry central, lock file. Crate `lumen-pkg`.

### Fase 81: Build Incremental ✅
Caché de compilación incremental para builds más rápidos.

### Fase 82: Hot Reload ✅
Recarga automática de módulos en dev. `lumen serve`.

### Fase 83: Playground Web ✅
Editor online con ejecución en navegador.

### Fases 86-87: AOT Compilation ✅
- 86: Transpilación a C + gcc/clang -O3
- 87: Backend Cranelift (base)

---

## Comandos CLI

| Comando | Descripción |
|---------|-------------|
| `lumen run <file>` | Ejecuta fuente .nv o bytecode .nvc |
| `lumen build <file>` | Compila a .nvc |
| `lumen check <file>` | Verifica sintaxis + semántica |
| `lumen disasm <file>` | Desensambla .nvc |
| `lumen fmt <file>` | Formatea código |
| `lumen repl` | Modo interactivo |
| `lumen new <name>` | Crea proyecto |
| `lumen test <file>` | Ejecuta tests |
| `lumen lint <file>` | Análisis estático |
| `lumen doc <file>` | Genera documentación HTML |
| `lumen debug <file>` | Inicia depurador |
| `lumen serve` | Hot reload + playground |
| `lumen lsp` | Servidor LSP |
| `lumen run -L <dir> <file>` | Ejecuta con ruta de librerías |

---

## Bytecode (.nvc)

- **Version**: 6
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-46
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays
  - 33-34: Closures
  - 35-37: Structs
  - 38-40: Result
  - 41-42: Option
  - 43: Enum
  - 44-45: Tuples
  - 46: Mod (módulo %)

---

## Estructura del proyecto

```
crates/
  lumen-lexer/    → token.rs, lexer.rs, error.rs
  lumen-parser/   → ast.rs, parser.rs, error.rs
  lumen-sema/     → sema.rs, loader.rs, error.rs
  lumen-ir/       → ir.rs, builder.rs
  lumen-codegen/  → bytecode.rs, codegen.rs, disasm.rs
  lumen-vm/       → vm.rs, value.rs
  lumen-cli/      → main.rs
  lumen-fmt/      → lib.rs
  lumen-repl/     → lib.rs
  lumen-project/  → lib.rs
  lumen-lsp/      → main.rs
  lumen-doc/      → main.rs
  lumen-aot/      → lib.rs
  lumen-pkg/      → lib.rs
docs/spec/        → grammar.ebnf, bytecode-format.md, error-codes.md, vm-spec.md
examples/         → *.nv (44 ejemplos funcionales)
stdlib/           → *.nv (texto, matematicas, coleccion, fecha, archivos, matrices)
scripts/          → PowerShell CI/CD (pre-commit, pre-vuelo)
```
""")

# ==============================================================
# CHANGELOG.md — Historial de versiones actualizado
# ==============================================================
write("CHANGELOG.md", """\
# Changelog

Todos los cambios importantes del proyecto LÚMEN se documentan aquí.

---

## v1.6.0 — Julio 2026

### Agregado
- **Fase 65: Guard Let** — `sea patron = expr sino { romper/retornar/continuar }`.
  Desugaring en IR builder a JmpIf/Jmp. `Stmt::GuardLet` en AST + parser + sema.
- **Fase 66: Operator Overloading** — `impl Suma for Punto` con método `fn sumar(self, otro)`.
  - `Expr::Binary` ahora tiene `resolved_method: Option<String>`.
  - Sema: `resolve_operator_overloads()` post-analysis con `HashMap<String, TypeInfo>`.
  - IR builder: emite `Call` en lugar de `Binary` cuando hay overload resuelto.
  - Traits `Suma`, `Resta`, `Multiplica`, `Divide` implementables en structs.
- **Fase 67: Extension Methods** — `impl Rasgo para TipoPrimitivo`.
  - `type_to_impl_name()` soporta resolución de tipos nativos: `entero`, `texto`, `decimal`, etc.
- **Fase 68: Tipos Asociados en Traits** — `tipo Item;` en rasgos y `tipo Item = T;` en impl.
  - AST: `AssociatedType` e `ImplAssociatedType`. Sema e IR completos.
- **Fase 70: Impl Trait return** — `funcion impl Rasgo foo() { retornar expr }`.
  - `Type::ImplTrait(String)` en AST. Parseo en `parse_type()`.
- **Fases 71-74: LSP Server (`lumen-lsp`)** — Diagnósticos en vivo, Autocompletado,
  Go-to-definition y Hover. Protocolo JSON-RPC sobre stdin/stdout.
- **Fase 75: lumen doc (`lumen-doc`)** — Generador de documentación HTML desde `///`.
- **Fase 76: Debugger** — Depurador interactivo con breakpoints, step, continue e inspect.
- **Fase 77: lumen fmt avanzado** — Soporte para `.lumen-fmt.toml` (`indent_spaces`, etc.).
- **Fase 78: lumen lint** — Análisis estático: código muerto y complejidad ciclomática.
- **Fase 79: REPL Pro** — Historial persistente, multilínea, resaltado, autocompletado.
- **Fase 80: Package Manager (`lumen-pkg`)** — `lumen install`, registry, lock file.
- **Fase 81: Build Incremental** — Caché de módulos para builds más rápidos.
- **Fase 82: Hot Reload** — `lumen serve` con recarga automática en dev.
- **Fase 83: Playground Web** — Editor online con ejecución en navegador.
- **Fases 86-87: AOT Compilation (`lumen-aot`)** — Transpilador C + backend Cranelift (base).
- **Roadmap fusionado** — Combinación del historial v1.0-v1.2 con el roadmap extendido v3.0.

### Cambiado
- Version bump a 1.6.0 en workspace.
- Trait impl functions reciben `"self"` como primer parámetro automáticamente.
- `docs/roadmap.md` expandido de 180 a 237 líneas con tablas detalladas de fases 0-60.
- `AGENTS.md` actualizado con fases 59-60, 68, 71-87 completadas.

### Corregido
- Encoding UTF-8 en scripts de CI/CD (pre-commit PowerShell).
- Pipeline de docs actualizado con referencias a crates nuevos.

---

## v1.2.0 — Julio 2026

### Agregado
- **Stdlib** — Módulos nativos: `matematicas`, `texto`, `coleccion`, `fecha`.
- **E/S de Archivos** — Builtins `leer_archivo`, `escribir_archivo`, `existe_archivo`.
- **Stack Traces** — Pila de llamadas completa en errores de runtime.
- **Mensajes de Error Mejorados** — Subrayado exacto con caret (`^^^^`) y colores ANSI.
- **Fases 42-57** — Inferencia de tipos, métodos en structs, diccionarios, string interpolation,
  rangos, constantes, string indexing, conversiones, operador ternario, loop labels,
  pattern matching exhaustivo + guardas, genéricos con bounds, matrices 2D.
- **Fase 58** — Enums avanzados con datos asociados.

### Corregido
- Advertencias de Clippy (CI verde en todos los targets).

---

## v1.1.0 — Julio 2026

### Agregado
- **Fase 21: For-Each Loop** — `para x en expr` / `for x in expr`. 31 tests.
- **Fase 22: Opcion<T>** — `opcion<T>` con `algun(valor)` y `ninguno`. 15 tests.
- **Fase 23: Enums/Tipos Suma** — `enum Nombre { Variante, Variante(tipo) }`. 20 tests.
- **Fase 24: Tuplas** — `(tipo, tipo)` y acceso `.0`, `.1`. 4 tests.
- **Fase 25: Destructuring** — `entero x, texto y = expr`, wildcard `_`. 14 tests.
- **Fase 26: Genéricos Básicos** — `<T, U>` en funciones y structs. 17 tests.

### Cambiado
- Workspace version a 1.1.0.
- CI corre en branches `master` y `main`.
- MSRV actualizado a 1.82.

---

## v1.0.0 — Julio 2026

Release inicial de LÚMEN. Lenguaje de programación educativo en español con pipeline completo
Lexer → Parser → Sema → IR → Codegen → VM. 21 fases completadas.
""")

# ==============================================================
# MARKETING.md — Posicionamiento actualizado a v1.6.0
# ==============================================================
write("MARKETING.md", """\
# LÚMEN — El Lenguaje que Ilumina

**v1.6.0 — Documento de Posicionamiento y Visión**

> *"Programar no debería ser un lujo en inglés. Debería ser un derecho en tu idioma."*

---

## 🎯 Misión

Democratizar la educación en programación para los 500 millones de hispanohablantes del mundo,
eliminando la barrera del inglés como requisito previo para aprender a programar.

LÚMEN existe porque creemos que **el idioma no debería determinar quién puede ser programador**.

---

## 💡 Lo Bueno

### 1. Español Nativo, Inglés Opcional

LÚMEN es el único lenguaje de programación moderno donde **el español es ciudadano de primera clase**.

```nv
// Español — natural, sin fricción
si edad >= 18 {
    imprimir("Mayor de edad");
} sino {
    imprimir("Menor de edad");
}
```

```nv
importar ingles;   // Ahora puedes usar if/else/while/for también
```

**No existe otro lenguaje que haga esto.** Python, JavaScript, Rust — todos asumen inglés.

### 2. Compilador de Calidad Industrial

LÚMEN no es un intérprete de juguete. Tiene un pipeline de compilación completo:

```
Código Fuente (.nv)
  → Lexer (tokenización)
  → Parser (AST)
  → Análisis Semántico (verificación de tipos)
  → IR Intermedio (optimización)
  → Codegen (bytecode)
  → VM Stack-Based (ejecución)
```

- **307 tests** automatizados (0 fallos)
- **Prop-testing** con miles de casos aleatorios
- **Constant folding**, **dead code elimination**, **shared pools**
- **Fuzzing** con 3 targets cargo-fuzz

### 3. Mensajes de Error que Enseñan

```
  E031 No puedes asignar un valor de tipo 'Texto' a una variable de tipo 'Entero'
  --> programa.nv:3:12
   |
 2 | entero x = 0;
 3 | entero y = "hola";
   |            ^^^^^^
   |
   Ayuda: Usa un valor de tipo 'Entero' en lugar de 'Texto'
```

### 4. Tipado Estático sin Dolor

```nv
x = 42;       // El compilador deduce: entero
nombre = "Ana"; // El compilador deduce: texto
```

- Type safety en tiempo de compilación
- Genéricos con bounds (`<T: Numerico>`)
- `Resultado<T, E>` para manejo de errores sin excepciones
- `Opcion<T>` para nulabilidad segura
- Sum types con `enum`

### 5. Baterías Incluidas (v1.6.0)

| Herramienta | Comando |
|-------------|---------|
| Formateador | `lumen fmt archivo.nv` |
| REPL Pro | `lumen repl` |
| Test runner | `lumen test archivo.nv` |
| Lint estático | `lumen lint archivo.nv` |
| Docs HTML | `lumen doc archivo.nv` |
| Depurador | `lumen debug archivo.nv` |
| Hot Reload | `lumen serve` |
| LSP (VS Code) | `lumen lsp` |
| Scaffolding | `lumen new mi_proyecto` |
| Package Mgr | `lumen install <pkg>` |
| Stdlib | `texto`, `matematicas`, `coleccion`, `fecha`, `archivos`, `matrices` |

### 6. Lenguaje Avanzado

Las siguientes características de lenguaje avanzado están completadas al 100%:

- **OR Patterns**: `caso Rojo | Verde:`
- **Guard Let**: `sea x = expr sino { romper }`
- **Operator Overloading**: `impl Suma for MiTipo`
- **Extension Methods**: `impl MiRasgo for entero`
- **Tipos Asociados**: `tipo Item;` en traits
- **Impl Trait return**: `-> impl Mostrable`
- **Pattern Matching Pro**: exhaustivo con guardas, rangos y strings

### 7. LSP — Developer Experience Premium

El servidor LSP (`lumen lsp`) provee en VS Code:
- ✅ Diagnósticos en tiempo real
- ✅ Autocompletado de símbolos
- ✅ Go-to-definition
- ✅ Hover con información de tipos

### 8. Stack Traces en Runtime

```
Error: Índice 5 fuera de rango (largo: 3)
Pila de llamadas:
  · procesar_lista en programa.nv:12
  · main en programa.nv:28
```

### 9. Interpolación de Strings

```nv
imprimir("Hola {nombre}, tienes {edad} años y mides {altura}m");
```

### 10. AOT Compilation (Base)

- Transpilador a C con gcc/clang -O3
- Backend Cranelift (base, en progreso)

---

## ⚠️ Lo Malo (Honestidad Total)

### 1. Todavía Joven (v1.6.0)

LÚMEN tiene menos de 2 años. No está en producción en ningún lado. Úsalo para aprender,
enseñar, prototipar — no para tu sistema bancario. Todavía.

### 2. Ecosistema Emergente

El package manager existe pero el registry de terceros es nuevo. La stdlib cubre lo básico
pero no hay frameworks web, ORMs ni SDKs de cloud todavía.

### 3. WASM y Binarios Nativos Completos — En Progreso

AOT (Cranelift/WASM) está en las fases 87-89, actualmente en desarrollo.

### 4. Sin Tutorial Interactivo

LENGUAJE.md y HERRAMIENTAS.md son completos, pero faltan:
- Tutorial interactivo web
- Libro digital completo
- Videos oficiales

---

## 🔭 Visión a Largo Plazo

### v2.0 — Distribución y Madurez
- WASM completo (ejecución en navegador sin servidor)
- Compilación nativa cross-platform (Linux/macOS/Windows)
- Single binary installer
- Benchmarks automatizados
- Plugins API

### v2.5 — Stdlib y Concurrencia
- Colecciones avanzadas, Red, Serde/JSON, Base de datos
- Threads, Async runtime, Paralelismo, Actores

### v3.0 — Ecosistema Completo
- GUI nativa + TUI
- AI/ML + DataFrames
- Cloud SDKs (AWS, GCP, Azure)
- Docker & K8s
- Comunidad autosustentable

---

## 🎓 Posicionamiento

### ¿Para quién es LÚMEN?

| Perfil | ¿LÚMEN es para ti? |
|--------|---------------------|
| **Estudiante hispanohablante** aprendiendo a programar | ✅ Perfecto. Sin barrera de idioma. |
| **Profesor de programación** en secundaria/universidad | ✅ Ideal. Diseñado para enseñar conceptos. |
| **Autodidacta** que quiere crear scripts y herramientas | ✅ Bueno. Sintaxis clara, errores útiles, REPL. |
| **Desarrollador profesional** buscando producción | ⚠️ Espera. El ecosistema está madurando. |
| **Startup** que necesita backend en producción | ❌ Todavía no. Usa Rust, Go o Python por ahora. |

### ¿Contra quién compite LÚMEN?

| Lenguaje | Fortaleza | Debilidad vs LÚMEN |
|----------|-----------|---------------------|
| **Python** | Ecosistema masivo | Sintaxis inglesa obligatoria, sin tipos estáticos |
| **JavaScript** | Web nativo, ubicuo | Sintaxis inglesa, comportamiento impredecible |
| **Rust** | Performance, seguridad | Curva de aprendizaje brutal, sintaxis inglesa |
| **Go** | Simplicidad, concurrencia | Sintaxis inglesa, sin genéricos expresivos |

**LÚMEN ocupa un nicho único**: primer lenguaje moderno para hispanohablantes con tipos
estáticos, genéricos, traits, pattern matching y mensajes de error pedagógicos.

---

## 📣 Pitch de 30 Segundos

> "LÚMEN es un lenguaje de programación diseñado para que cualquier persona de habla hispana
> pueda aprender a programar sin saber inglés. Escribes `si`, `mientras`, `funcion` — como
> piensas. Tiene tipos estáticos como Rust, inferencia como TypeScript, mensajes de error que
> te enseñan, LSP completo para VS Code, depurador integrado, y todo viene incluido:
> formateador, REPL Pro, tests, docs y hot reload."

---

## 🏷️ Taglines

- *"Programar en tu idioma, compilar con esteroides."*
- *"El lenguaje que no te obliga a aprender inglés."*
- *"Rust para principiantes, en español."*
- *"Porque programar ya es difícil. El idioma no debería serlo."*
- *"LÚMEN: donde el código se lee como se piensa."*
- *"TypeScript safety, Python simplicity, Spanish first."*

---

## 🔗 Links

- **Repositorio**: [github.com/omarPVP123131/Lumen](https://github.com/omarPVP123131/Lumen)
- **Manual**: `LENGUAJE.md`
- **Herramientas**: `HERRAMIENTAS.md`
- **Roadmap**: `docs/roadmap.md`
- **Reportar bugs**: GitHub Issues
- **Contribuir**: `docs/contributing.md`

---

## 🧠 Filosofía de Diseño

1. **El idioma no es la barrera.** Si un concepto se puede explicar en español, el código también.
2. **Los errores enseñan.** Cada mensaje de error debe ser una mini-lección.
3. **Baterías incluidas.** fmt, test, repl, lint, doc, debug, lsp — todo en la caja.
4. **Tipado sin ceremonia.** Seguridad de tipos sin 50 anotaciones.
5. **Transparencia.** El compilador explica qué hace y por qué.
6. **Honestidad.** Si algo no funciona, lo decimos. Sin marketing falso.

---

*LÚMEN v1.6.0 — Julio 2026 · Hecho con convicción, no con hype.*
""")

# ==============================================================
# docs/cli.md — Referencia CLI completa y actualizada
# ==============================================================
write("docs/cli.md", """\
# Referencia CLI de LÚMEN — v1.6.0

## Uso General

```bash
lumen <comando> [opciones] <archivo>
```

---

## Comandos

### `run` — Ejecutar programa

```bash
lumen run programa.nv       # Ejecuta fuente .nv
lumen run programa.nvc      # Ejecuta bytecode compilado
lumen run -L ./libs prog.nv # Con directorio de librerías
```

Compila y ejecuta en un solo paso. El pipeline completo se ejecuta en memoria.

---

### `build` — Compilar a bytecode

```bash
lumen build programa.nv     # Genera programa.nvc
```

---

### `check` — Verificar sintaxis y semántica

```bash
lumen check programa.nv     # Análisis completo sin ejecutar
```

---

### `disasm` — Desensamblar bytecode

```bash
lumen disasm programa.nvc   # Muestra instrucciones en texto legible
```

Útil para aprendizaje: cada instrucción de la VM con sus operandos.

---

### `fmt` — Formatear código

```bash
lumen fmt archivo.nv        # Formatea en su lugar
lumen fmt --check archivo.nv # Solo verifica sin modificar
```

Soporta configuración vía `.lumen-fmt.toml`:

```toml
indent_spaces = 4
max_line_length = 100
trailing_newline = true
```

---

### `repl` — REPL interactivo

```bash
lumen repl
```

Características: historial persistente, edición multilínea, resaltado de sintaxis,
autocompletado de símbolos.

---

### `new` — Crear proyecto

```bash
lumen new mi_proyecto
```

Genera scaffolding con `lumen.toml`, `src/main.nv`, y estructura de proyecto estándar.

---

### `test` — Ejecutar tests

```bash
lumen test tests.nv
```

Ejecuta todas las funciones que empiecen con `test_`. Usa `afirmar(expr)` para assertions.

---

### `lint` — Análisis estático

```bash
lumen lint programa.nv
```

Detecta: código muerto, variables no usadas, complejidad ciclomática alta, imports no usados.

---

### `doc` — Generar documentación

```bash
lumen doc programa.nv       # Genera HTML en ./docs/
lumen doc programa.nv -o mi_docs/
```

Extrae comentarios `///` y genera documentación HTML estática.

---

### `debug` — Depurador interactivo

```bash
lumen debug programa.nv
```

Comandos en el depurador:
- `break <linea>` — Añadir breakpoint
- `step` / `s` — Ejecutar siguiente instrucción
- `continue` / `c` — Continuar hasta siguiente breakpoint
- `inspect <var>` — Ver valor de variable
- `quit` — Salir

---

### `serve` — Servidor de desarrollo

```bash
lumen serve programa.nv
```

Inicia servidor con hot reload. Recarga automáticamente al detectar cambios en `.nv`.

---

### `lsp` — Servidor LSP

```bash
lumen lsp
```

Servidor LSP para VS Code y editores compatibles. Provee:
- Diagnósticos en tiempo real (`publishDiagnostics`)
- Autocompletado (`textDocument/completion`)
- Ir a definición (`textDocument/definition`)
- Hover con tipos (`textDocument/hover`)

---

## Opciones Globales

| Opción | Descripción |
|--------|-------------|
| `-L <dir>` / `--lib-dir <dir>` | Directorio de búsqueda para importar módulos |
| `--version` | Versión del compilador |
| `--help` | Mensaje de ayuda |

---

## Códigos de Salida

| Código | Significado |
|--------|-------------|
| 0 | Éxito |
| 1 | Error del usuario (sintaxis, semántica, runtime) |
| 2 | Error interno (bug del compilador) |

---

## Ejemplos

```bash
# Ejecutar un programa
lumen run examples/hello.nv

# Compilar y luego ejecutar bytecode
lumen build examples/func.nv
lumen run examples/func.nvc

# Verificar sin ejecutar
lumen check examples/loop.nv

# Desensamblar
lumen disasm examples/func.nvc

# Formatear todos los .nv
lumen fmt src/*.nv

# Ejecutar tests
lumen test tests/unit.nv

# Programa con imports desde directorio
lumen run -L ./stdlib programa.nv

# Generar docs
lumen doc src/main.nv -o ./docs

# Depurar
lumen debug programa.nv
```
""")

# ==============================================================
# docs/language.md — Referencia de sintaxis actualizada
# ==============================================================
write("docs/language.md", """\
# Referencia del Lenguaje LÚMEN — v1.6.0

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
| Lógicos | `&&` / `y`, `\\|\\|` / `o`, `!` / `no` |
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
""")

print("Todos los archivos escritos correctamente.")
