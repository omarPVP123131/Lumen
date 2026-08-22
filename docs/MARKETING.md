# LÚMEN — El Lenguaje que Ilumina

**v3.0.0 — Documento de Posicionamiento y Visión**

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

### 2. Compilador de Calidad Industrial — v3.0.0 Producción Real

LÚMEN no es un intérprete de juguete. Tiene un pipeline de compilación completo:

```
Código Fuente (.nv)
  → Lexer (tokenización)
  → Parser (AST)
  → Análisis Semántico (verificación de tipos)
  → IR Intermedio (builder last_significant() + label_counter global)
  → Codegen (bytecode CHUNK_VERSION 7 + FuncMeta.defaults)
  → VM Stack-Based (bind_args unificado, 917 tests, bench 8)
```

- **917 tests** automatizados (616 e2e + 9 production + resto workspace, 0 fallos) — ver `docs/produccion.md`
- **8 benches** criterion (`cargo bench -p lumen-bench`: lexer, parser, pipeline, vm_fib_20 + 4 prod) — bench formal producción
- **Modo headless centralizado** (`stdlib/graficos.nv:es_headless()` con `LUMEN_HEADLESS`/`CI`, CI `headless-check`)
- **Prop-testing** con miles de casos aleatorios
- **Constant folding**, **dead code elimination**, **shared pools**
- **Fuzzing** con 4 targets cargo-fuzz (structs/listas, closures, rechazo y regex) sin divergencias
- **CHUNK_VERSION 7** con defaults persistidos y compat v6, `VERSION` 3.0.0

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

### 5. Baterías Incluidas (v3.0.0 Producción Real)

| Herramienta | Comando | Novedad v3.0.0 |
|-------------|---------|----------------|
| Formateador | `lumen fmt archivo.nv` |  |
| REPL Pro | `lumen repl` |  |
| Test runner | `lumen test archivo.nv` | 917 tests (616 e2e + 9 production) |
| Lint estático | `lumen lint archivo.nv` |  |
| Docs HTML | `lumen doc archivo.nv` |  |
| Depurador | `lumen debug archivo.nv` |  |
| Hot Reload | `lumen serve` |  |
| LSP (VS Code) | `lumen lsp` |  |
| Scaffolding | `lumen new mi_proyecto` |  |
| Package Mgr | `lumen install <pkg>` |  |
| Bench | `cargo bench -p lumen-bench` | 8 benches (4 prod nuevos) |
| Headless | `LUMEN_HEADLESS=1 lumen run` | `stdlib/graficos.nv:es_headless()` centralizado |
| Stdlib | `texto`, `matematicas`, `coleccion`, `fecha`, `archivos`, `matrices` | `graficos.nv es_headless()` + `matematicas.nv` fix `Variable 'n'` |
| Producción | `docs/produccion.md` | Checklist `CHUNK_VERSION 7` + CI `headless-check` |

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

## ✅ Producción Real v3.0.0 (21 Ago 2026) — NUEVO

**LÚMEN v3.0.0 ya es deployable** con checklist de producción en [docs/produccion.md](produccion.md):

- Fixes escalables: `last_significant()` + `label_counter` global, `CHUNK_VERSION 7` con `FuncMeta.defaults` + `bind_args` unificado, `es_headless()` centralizado
- 917 tests (616 e2e + 9 production), 8 benches (`cargo bench -p lumen-bench`), 389/389 `lumen check`, CI `headless-check` con `LUMEN_HEADLESS=1 CI=1`
- `cargo build --release --target <target>` genera binarios Windows/Linux/macOS/Android/WASM listos para producción
- Usa `LUMEN_HEADLESS=1` para CI/headless; `stdlib/graficos.nv:es_headless()` evita `SDL_Init` sin display

## ⚠️ Lo Malo (Honestidad Total — actualizado v3.0.0)

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
| **Desarrollador profesional** buscando producción | ✅ Ya — v3.0.0 Producción Real (917 tests, bench 8, `LUMEN_HEADLESS`, `CHUNK_VERSION 7`, ver `docs/produccion.md`) |
| **Startup** que necesita backend en producción | ✅ Ya — `lumen bundle`/`lumen build --release` + `cargo test --workspace` + headless CI, ver `docs/produccion.md` |

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

*LÚMEN v3.0.0 — Agosto 2026 · Hecho con convicción, no con hype.*
