# Roadmap de LÚMEN — v1.1.0

**Versión actual: 1.1.0 — 294 tests, 0 warnings, fases 0-32 completadas**

---

## Fases completadas ✅ (0-27)

| Fase | Descripción | Estado |
|------|-------------|--------|
| 0 | Cimientos (workspace, CI, tooling) | ✅ |
| 1 | Especificación formal (gramática EBNF, códigos de error) | ✅ |
| 2 | Lexer (57 tokens, 33 keywords bilingües) | ✅ |
| 3 | Parser + AST (19 Expr, 14 Stmt, 12 Type variants) | ✅ |
| 4 | Análisis semántico (28 códigos de error, 12 TypeInfo variants) | ✅ |
| 5 | IR (27 instrucciones, constant folding, DCE) | ✅ |
| 6 | Bytecode v6 (45 opcodes, LUMN magic, 4 constant pools) | ✅ |
| 7 | VM stack-based (12 Value variants, 46 opcodes ejecutados) | ✅ |
| 8 | CLI (`run`/`build`/`check`/`disasm`) | ✅ |
| 9 | Split numérico (`entero` i64, `decimal` f64, `numero` alias) | ✅ |
| 10 | Arrays (`lista<T>`, index, `agregar`, `largo`) | ✅ |
| 11 | Strings (concatenación, comparación, escapes) | ✅ |
| 12 | Booleanos (`verdadero`/`falso`, `&&`/`||`, `!`) | ✅ |
| 13 | Control avanzado (`romper`, `continuar`, `elegir`/`match`) | ✅ |
| 14 | Parámetros default (`funcion foo(a, b = 10)`) | ✅ |
| 15 | Lambdas IIFE (`funcion(x){...}(5)`) | ✅ |
| 16 | Closures (lambdas asignables, `FuncRef`/`CallValue`) | ✅ |
| 17 | Estructuras (`estructura`, init, field access) | ✅ |
| 18 | Módulos (`importar`, ModuleLoader, `-L`) | ✅ |
| 19 | Optimizaciones (constant folding, DCE, shared pools) | ✅ |
| 20 | Release v1.0 (README, SemVer, docs separados) | ✅ |
| 21 | For-Each (`para x en expr`, desugaring a while-loop) | ✅ |
| 22 | Resultado\<T, E\> (`exito`, `error`, `intentar`, opcodes 38-40) | ✅ |
| 23 | Opcion\<T\> (`algun`, `ninguno`, opcodes 41-42) | ✅ |
| 24 | Enums/Tipos Suma (`enum`, `::`, opcode 43) | ✅ |
| 25 | Tuplas (`(a, b)`, opcodes 44-45) | ✅ |
| 26 | Destructuring (`x, y = expr`, wildcard `_`) | ✅ |
| 27 | Genéricos Básicos (`<T, U>`, type erasure) | ✅ |

---

## Fases completadas (continuación) ✅

| Fase | Descripción | Estado |
|------|-------------|--------|
| 28 | Stdlib: `matematicas`, `texto` | ✅ |
| 29 | Stdlib: `coleccion`, `fecha` | ✅ |
| 30 | E/S de Archivos (`leer_archivo`, `escribir_archivo`, `existe_archivo`) | ✅ |
| 31 | Stack Traces (`CallFrame`, `VmError::with_stack()`) | ✅ |
| 32 | Mensajes de Error Mejorados (subrayado, colores, `show_error()`) | ✅ |

## Fases en construcción 🔨

---

### Fase 33 — Fuzzing
**Objetivo**: Robustez ante entrada arbitraria.

- `cargo-fuzz` para lexer, parser, decoder de bytecode
- Corpus de entrada inicial

**Criterio de aceptación**:
- [ ] 100k+ iteraciones de fuzzing sin panics
- [ ] CI ejecuta fuzzing como paso separado
- [ ] Todo panic detectado se convierte en bug y test

---

### Fase 34 — Property-Based Testing
**Objetivo**: Invariantes verificadas con proptest.

- Round-trip: AST → JSON → AST
- Round-trip: IR → bytecode → decode
- Propiedades: "parsear(formatear(AST)) == AST"

**Criterio de aceptación**:
- [ ] Suite de proptest con 1000+ casos por propiedad
- [ ] 0 fallos conocidos en las propiedades
- [ ] CI ejecuta proptest

---

### Fase 35 — `lumen fmt`
**Objetivo**: Formateador automático de código.

```bash
lumen fmt programa.nv
```

- Formateo consistente (indentación, espacios, saltos de línea)
- AST round-trip como base
- Modo `--check` para CI

**Criterio de aceptación**:
- [ ] Formatea todos los ejemplos de `examples/` sin errores
- [ ] `--check` retorna código de salida 1 si hay diferencias
- [ ] Round-trip: formatear dos veces produce el mismo resultado
- [ ] Tests: 10+ casos de formateo

---

### Fase 36 — `lumen repl`
**Objetivo**: Bucle interactivo para experimentación.

```bash
$ lumen repl
LÚMEN v1.1.0 — Escribe 'salir' para terminar
> entero x = 5
> imprimir(x * 2)
10
>
```

- Historial de comandos
- Evaluación línea por línea
- Persistencia de variables entre líneas

**Criterio de aceptación**:
- [ ] REPL funcional con todas las características del lenguaje
- [ ] Manejo de errores sin salir del REPL
- [ ] Comando `salir`/`exit` para terminar
- [ ] Tests: simulación de sesión REPL

---

### Fase 37 — `lumen test`
**Objetivo**: Framework de testing nativo.

```lumen
test "suma básica" {
    afirmar(suma(2, 3) == 5);
}
```

```bash
lumen test tests/
```

**Criterio de aceptación**:
- [ ] Parseo de bloques `test`
- [ ] `lumen test` descubre y ejecuta tests
- [ ] Reporte con color: ✓ y ✗
- [ ] Código de salida 0 si todos pasan, 1 si alguno falla
- [ ] Tests del propio framework

---

### Fase 38 — `lumen.toml` (Manifiesto de Proyecto)
**Objetivo**: Configuración de proyectos LÚMEN.

```toml
[proyecto]
nombre = "mi-app"
version = "0.1.0"
autor = "Tu Nombre"

[dependencias]
matematicas = "1.0"
```

- `lumen new <nombre>` genera un proyecto con `lumen.toml`
- Resolución de dependencias desde rutas locales

**Criterio de aceptación**:
- [ ] Parseo de `lumen.toml`
- [ ] `lumen new` crea estructura de proyecto
- [ ] Dependencias locales se resuelven
- [ ] Tests: crear, configurar, compilar proyecto

---

### Fase 39 — Benchmarks Públicos
**Objetivo**: Medir y documentar el rendimiento.

- Suite con Criterion para parse, codegen, VM
- Comparación contra v1.0.x para detectar regresiones

**Criterio de aceptación**:
- [ ] Benchmarks para: parse, codegen, VM (fibonacci, loop, call)
- [ ] Resultados documentados en `docs/performance.md`
- [ ] CI ejecuta benchmarks y falla si hay regresión > 10%

---

### Fase 40 — Binarios Precompilados + GitHub Release
**Objetivo**: Distribución sin necesidad de compilar.

- GitHub Actions compila para Windows, Linux, macOS
- Artefactos: `.tar.gz` (Linux/macOS), `.zip` (Windows)
- Release notes automáticas

**Criterio de aceptación**:
- [ ] Workflow de release en CI
- [ ] Binarios para las 3 plataformas
- [ ] Release en GitHub con notas

---

## Más allá de v2.0

Una vez alcanzada la fase 40:

### Herramientas
- LSP server con autocompletado y diagnósticos en tiempo real
- Extensión VS Code con syntax highlighting
- Linter (`lumen lint`) con reglas de estilo configurables
- Debugger: breakpoints, step-over/step-into, watch variables

### Educación y Ecosistema
- Playground web compilado a WASM
- Visualización del stack en tiempo real
- Modo "clase": exportar ejecución paso a paso
- Tutorial interactivo embebido en el playground

### Madurez del compilador
- NaN-boxing para representación compacta de valores
- FFI para llamar C desde LÚMEN
- Tail call optimization
- Concurrencia: hilos ligeros / async-await
- JIT compilation para hot paths
- Self-hosting: compilador de LÚMEN escrito en LÚMEN
- Target WASM nativo
- Macros / metaprogramación
