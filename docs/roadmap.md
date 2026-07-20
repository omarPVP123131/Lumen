# Roadmap de LÚMEN

Camino hacia **v1.0 estable**, dividido en 40 fases. Las fases 0-20 están completadas; las fases 21-40 son requisitos de aceptación para declarar la versión 1.0 final.

---

## Fases completadas ✅ (0-20)

| Fase | Descripción | Criterio de Aceptación |
|------|-------------|------------------------|
| 0 | Cimientos (workspace, CI, tooling) | `cargo build --all` funciona, CI en verde |
| 1 | Especificación formal | Gramática EBNF en `docs/spec/`, códigos de error definidos |
| 2 | Lexer | Tokeniza cualquier fuente sin panics, errores recuperables |
| 3 | Parser + AST | AST correcto para todos los ejemplos, error recovery |
| 4 | Análisis semántico | Type checking, scopes, detección de errores semánticos |
| 5 | IR | Three-address code correcto, constant folding básico |
| 6 | Bytecode | `.nvc` encode/decode round-trip, disassembler legible |
| 7 | VM | Ejecuta bytecode correctamente, stack traces en errores |
| 8 | CLI | `run`/`build`/`check`/`disasm` funcionales |
| 9 | Split numérico | `entero` i64, `decimal` f64, `numero` alias |
| 10 | Arrays | `lista<T>`, index, `agregar`, `largo` |
| 11 | Strings | Concatenación, comparación, escapes |
| 12 | Booleanos | `verdadero`/`falso`, `&&` cortocircuito, `!` |
| 13 | Control avanzado | `romper`, `continuar`, `elegir`/`match` |
| 14 | Parámetros default | `funcion foo(a, b = 10)` |
| 15 | Lambdas IIFE | `funcion(x){...}(5)` |
| 16 | Closures | Lambdas asignables, `Type::Func`, `CallValue` |
| 17 | Estructuras | `estructura`, init, field access, `StructNew/Get/Set` |
| 18 | Módulos | `importar`, ModuleLoader, detección circular, flag `-L` |
| 19 | Optimizaciones | Constant folding, DCE, shared pools, func cache |
| 20 | Release interno | README landing, docs/ separados, SemVer 1.0.0 |
| 21 | `resultado<T, E>` | Manejo de errores seguro con `exito`, `error` e `intentar` |

---

## Fases pendientes → v1.0 final (22-40)

### Fase 21 — `resultado<T, E>` ✅
**Objetivo**: Manejo de errores sin panics mediante tipo `resultado`.

- Sintaxis: `resultado<entero, texto>` como tipo
- Variantes: `exito(valor)` y `error(mensaje)`
- Coincidencia de patrones básica para desempaquetar
- Función `?`/`intentar` para propagación automática

**Criterio de aceptación**:
- [x] Parseo de sintaxis `resultado<T, E>`
- [x] Type checking: el tipo debe coincidir en asignaciones y retornos
- [x] Ejecución en VM: `exito(5)` produce `Value::Result(Ok(5))`
- [x] Propagación: `intentar` extrae el valor o retorna el error
- [x] Tests: caso de éxito, caso de error, propagación anidada

---

### Fase 22 — `opcion<T>` (Optional Type) ✅
**Objetivo**: Tipo opcional/nullable seguro sin null pointers.

- Sintaxis: `opcion<entero>`, valores `algun(valor)` y `ninguno`
- Coerción explícita requerida (no hay null implícito)
- Desempaquetado seguro con `elegir`

**Criterio de aceptación**:
- [x] Parseo de `opcion<T>`, `algun`, `ninguno`
- [x] Sema: `ninguno` puede asignarse a cualquier `opcion<T>`
- [x] VM: `algun(5)` → `Value::Opcion(Some(5))`
- [x] Acceso seguro: comparación y match con `elegir`
- [x] Tests: 5 sema + 10 e2e cubriendo variantes y errores

---

### Fase 23 — Enums / Tipos Suma ✅
**Objetivo**: Tipos suma con variantes nombradas.

```lumen
enum Color {
    Rojo,
    Verde(entero),
    Azul(texto, entero)
}
```

- Sintaxis: `enum Nombre { Variante, Variante(tipo, ...) }`
- Coincidencia con `elegir` para desempaquetar variantes
- Type checking exhaustivo (opcional en v1.0)

**Criterio de aceptación**:
- [x] Parseo de declaración `enum`
- [x] Constructores de variantes como expresiones
- [x] `elegir` con variantes de enum (pattern matching)
- [x] VM: `Value::Enum { name, variant, fields }`
- [x] Tests: enum simple, con datos, match exhaustivo

---

### Fase 24 — Tuplas ✅
**Objetivo**: Tuplas como tipo nativo para agrupar valores heterogéneos.

```lumen
entero, texto par = (1, "hola");
imprimir(par.0); // 1
```

- Sintaxis de tipo: `(tipo, tipo, ...)`
- Indexación por posición: `tupla.0`, `tupla.1`
- Soporte en IR y VM como valor compuesto

**Criterio de aceptación**:
- [x] Parseo de tipo tupla y expresión tupla
- [x] Type checking: número de elementos y tipos deben coincidir
- [x] VM: `Value::Tuple(Vec<Value>)`
- [x] Acceso por índice con validación de rango
- [x] Tests: 5+ casos

---

### Fase 25 — Destructuring ✅
**Objetivo**: Desempaquetar tuplas, structs y enums en asignaciones y parámetros.

```lumen
entero x, texto y = (1, "hola");      // tupla
entero edad, texto nom = persona;      // struct (por orden/nombre)
funcion(entero a, texto b) { ... }     // parámetros con destructuring
```

- Asignación con destructuring
- Parámetros de función con destructuring
- Anidación permitida

**Criterio de aceptación**:
- [x] Parseo de asignación con múltiples targets
- [x] Parsing de parámetros con destructuring
- [x] Type checking: número de targets vs valor
- [x] VM: lógica de desempaquetado
- [x] Tests: tupla, struct, anidado, error en mismatch

---

### Fase 26 — Genéricos Básicos ✅
**Objetivo**: Funciones y tipos genéricos.

```lumen
funcion T identidad<T>(T valor) { retornar valor; }
estructura Par<T, U> { primero: T, segundo: U }
```

- Parámetros de tipo en funciones y structs
- Inferencia de tipos en llamadas
- Monomorfización o type erasure (decidir)

**Criterio de aceptación**:
- [x] Parseo de parámetros de tipo `<T, U>`
- [x] Type checking: sustitución de tipos
- [x] IR/codegen/VM: funcionamiento correcto
- [x] Tests: identidad, par, struct genérico

---

### Fase 27 — Librería Estándar: `matematicas`, `texto`
**Objetivo**: Módulos de librería estándar para operaciones comunes.

- `matematicas`: `abs`, `max`, `min`, `potencia`, `raiz`, `seno`, `coseno`
- `texto`: `longitud`, `mayusculas`, `minusculas`, `recortar`, `dividir`, `contiene`

**Criterio de aceptación**:
- [ ] Módulos `.nv` en `stdlib/` que se importan con `importar matematicas`
- [ ] Cada función documentada y testeada
- [ ] Coerciones numéricas manejadas correctamente
- [ ] Tests: 3+ casos por función

---

### Fase 28 — Librería Estándar: `coleccion`, `fecha`
**Objetivo**: Más módulos de librería estándar.

- `coleccion`: `map`, `filtrar`, `reducir`, `ordenar`, `invertir`
- `fecha`: `ahora`, `formatear`, `diferencia`, `sumar_dias`

**Criterio de aceptación**:
- [ ] Módulos funcionales con tests
- [ ] `coleccion` itera sobre `lista<T>` correctamente
- [ ] `fecha` maneja timezones UTC
- [ ] Tests: 3+ casos por función

---

### Fase 29 — E/S de Archivos
**Objetivo**: Leer y escribir archivos desde LÚMEN.

```lumen
texto contenido = leer_archivo("datos.txt");
escribir_archivo("salida.txt", "contenido");
```

- Funciones `leer_archivo`, `escribir_archivo`, `existe_archivo`
- Manejo de errores con `resultado`
- Rutas relativas y absolutas

**Criterio de aceptación**:
- [ ] Funciones implementadas como builtins de VM
- [ ] `leer_archivo` retorna `resultado<texto, texto>`
- [ ] `escribir_archivo` retorna `resultado<void, texto>`
- [ ] Path traversal detection
- [ ] Tests: leer, escribir, error en archivo inexistente

---

### Fase 30 — Stack Traces en Runtime
**Objetivo**: Mensajes de error con pila de llamadas completa.

- Cuando ocurre un error en runtime, mostrar:
  ```
  Error: División por cero
    en factorial (línea 5)
    en main (línea 12)
  ```
- Integrar con el sistema de errores existente

**Criterio de aceptación**:
- [ ] VM mantiene call stack con información de línea
- [ ] Errores muestran 3+ niveles de pila
- [ ] Coincide línea correcta del código fuente
- [ ] Tests: error en función anidada

---

### Fase 31 — Mensajes de Error Mejorados
**Objetivo**: Errores con subrayado, colores y sugerencias.

```
Error E042: Tipo incompatible en línea 7, columna 12
  esperado: entero
  recibido: texto
  
  >   entero x = "hola"
                  ^^^^^^
  Sugerencia: Usa un valor numérico como 42
```

- Subrayado de la posición exacta del error
- Colores en terminal (con flag `--no-color`)
- Sugerencias específicas por código de error

**Criterio de aceptación**:
- [ ] Cada error E0xx tiene subrayado y sugerencia
- [ ] Colores funcionales en Windows/Linux/macOS
- [ ] `--no-color` desactiva colores
- [ ] Tests: verificar formato de salida de errores

---

### Fase 32 — Fuzzing
**Objetivo**: Robustez ante entrada arbitraria.

- `cargo-fuzz` para lexer: generate tokens sin panics
- `cargo-fuzz` para parser: genera AST sin panics
- `cargo-fuzz` para decoder de bytecode: decode sin panics
- Corpus de entrada inicial

**Criterio de aceptación**:
- [ ] 100k+ iteraciones de fuzzing sin panics
- [ ] CI ejecuta fuzzing como paso separado
- [ ] Todo panic detectado se convierte en bug y test

---

### Fase 33 — Property-Based Testing
**Objetivo**: Invariantes verificadas con proptest.

- Round-trip: AST → JSON → AST
- Round-trip: IR → bytecode → decode
- Propiedades: "parsear(formatear(AST)) == AST"
- Propiedades: "ejecutar(compilar(src)).salida == esperado"

**Criterio de aceptación**:
- [ ] Suite de proptest con 1000+ casos por propiedad
- [ ] 0 fallos conocidos en las propiedades
- [ ] CI ejecuta proptest

---

### Fase 34 — `lumen fmt`
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

### Fase 35 — `lumen repl`
**Objetivo**: Bucle interactivo para experimentación.

```bash
$ lumen repl
LÚMEN v1.0 — Escribe 'salir' para terminar
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

### Fase 36 — `lumen test`
**Objetivo**: Framework de testing nativo.

```lumen
// archivo_test.nv
test "suma básica" {
    afirmar(suma(2, 3) == 5);
}

test "resta negativa" {
    afirmar(resta(2, 5) == -3);
}
```

```bash
lumen test tests/
```

- Keyword `test` para definir casos
- `afirmar`/`assert` para verificaciones
- Reporte de resultados: pass/fail con contadores

**Criterio de aceptación**:
- [ ] Parseo de bloques `test`
- [ ] `lumen test` descubre y ejecuta tests
- [ ] Reporte con color: ✓ y ✗
- [ ] Código de salida 0 si todos pasan, 1 si alguno falla
- [ ] Tests del propio framework

---

### Fase 37 — `lumen.toml` (Manifiesto de Proyecto)
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
- Soporte para `--manifest-path`

**Criterio de aceptación**:
- [ ] Parseo de `lumen.toml`
- [ ] `lumen new` crea estructura de proyecto
- [ ] Dependencias locales se resuelven
- [ ] Tests: crear, configurar, compilar proyecto

---

### Fase 38 — Benchmarks Públicos
**Objetivo**: Medir y documentar el rendimiento.

- Suite de benchmarks reproducible con Criterion
- Comparación contra v0.x para detectar regresiones
- Publicación en `docs/performance.md`

**Criterio de aceptación**:
- [ ] Benchmarks para: parse, codegen, VM (fibonacci, loop, call)
- [ ] Resultados documentados en `docs/performance.md`
- [ ] CI ejecuta benchmarks y falla si hay regresión > 10%
- [ ] Gráfica de rendimiento por release

---

### Fase 39 — Binarios Precompilados + GitHub Release
**Objetivo**: Distribución sin necesidad de compilar.

- GitHub Actions compila para Windows, Linux, macOS
- Artefactos: `.tar.gz` (Linux/macOS), `.zip` (Windows)
- Release notes automáticas

**Criterio de aceptación**:
- [ ] Workflow de release en CI
- [ ] Binarios para las 3 plataformas
- [ ] Release en GitHub con notas
- [ ] Instalación: descargar, extraer, `./lumen run hello.nv`

---

### Fase 40 — Release Oficial v1.0
**Objetivo**: LÚMEN listo para distribución pública desde GitHub.

- Tag `v1.0.0` en GitHub (✅ completado)
- Release notes completas
- GitHub Release con assets
- README actualizado con badges

**Criterio de aceptación**:
- [ ] Release en GitHub con notas y tag v1.0.0
- [ ] Binarios para las 3 plataformas (CI)
- [ ] README actualizado
- [ ] Anuncio público

---

## Más allá de v1.0

Una vez alcanzada la fase 40, estas funcionalidades se explorarán en versiones posteriores sin fecha definida:

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
- Compilación separada de módulos
- Sistema de traits (interfaces)
- NaN-boxing para representación compacta de valores
- FFI para llamar C desde LÚMEN
- Tail call optimization
- Concurrencia: hilos ligeros / async-await
- JIT compilation para hot paths
- Self-hosting: compilador de LÚMEN escrito en LÚMEN
- Target WASM nativo
- Macros / metaprogramación
