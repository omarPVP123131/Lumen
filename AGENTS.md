# AGENTS.md — Diario de construcción de LÚMEN

**v1.1.0 — Release: Julio 2026**

---

## Testing (Actual)

| Crate | Tests | Type |
|-------|-------|------|
| lumen-lexer | 24 | unit |
| lumen-parser | 42 | unit |
| lumen-sema | 43 | unit |
| lumen-ir | 20 | unit + folding |
| lumen-codegen | 13 | unit |
| lumen-codegen | 5 | proptest |
| lumen-vm | 45 | unit |
| lumen-vm | 102 | e2e |
| **Total** | **294** | |

**0 warnings, 294 tests passing.**

---

## Fases completadas

### Fase 0-15: Infraestructura base ✅
Lexer, parser, sema, IR, bytecode, VM, CLI, arrays, control de flujo avanzado.

### Fase 16: Funciones avanzadas ✅
- Parámetros default (`funcion foo(entero a, entero b = 10)`)
- Lambdas IIFE (`funcion(x) { retornar x; }(5)`)
- Lambdas asignables (`dup = funcion(x) { retornar x*2; }; dup(5)`)
- Closures con `Value::Func(String)`, `FuncRef`/`CallValue` opcodes
- `Type::Func { param_types, return_type }` en AST y TypeInfo

### Fase 17: Estructuras/Objetos ✅
- `estructura Nombre { campo: tipo, ... }` — declaración
- `Nombre { campo: expr, ... }` — inicialización
- `expr.campo` — acceso a campo
- `expr.campo = valor` — asignación de campo
- `Type::Struct(String)`, `TypeInfo::Struct { name, fields }`
- `Value::Struct { name, fields }` en VM
- Opcodes: `StructNew(35)`, `StructGet(36)`, `StructSet(37)`

### Fase 18: Módulos ✅
- `importar "ruta.nv"`, `importar modulo`, `importar ... como alias`
- ModuleLoader: resolución de rutas, detección circular (E063)
- Prefixado de nombres (scope-tracked, builtins exentos)
- CLI: `-L`/`--lib-dir` para rutas de búsqueda

### Fase 19: Optimizaciones ✅
- IR: constant folding (aritmética, booleanos, comparaciones, strings, mixto Int/Float)
- IR: dead code elimination (Nop removal)
- Bytecode: shared constant pools (string_cache, int_cache, num_cache, name_cache)
- VM: function index cache (`HashMap<String, usize>`)

### Fase 20: v1.0 — Release ✅
- README.md completo con features, ejemplos, CLI, instalación
- Versionado SemVer 1.0.0

### Fase 21: For-Each ✅
- Sintaxis: `para x en expr` / `for x in expr`
- Token: `En`/`In` en lexer
- AST: `Stmt::ForEach { var_name, expr, body }`
- Parser: `parse_foreach()` con flag `no_struct_init`
- Sema: verifica que `expr` sea `Lista`
- IR: desugaring a while-loop con `ArrayLen`/`ArrayGet`/`Store`
- Tests: 4 parser, 6 sema, 9 e2e

### Fase 22: Resultado<T, E> ✅
- Sintaxis: `resultado<entero, texto>` como tipo
- Variantes: `exito(valor)` y `error(mensaje)`
- `intentar`/`try` para propagación automática
- AST: `Expr::Exito`, `Expr::Error`, `Expr::Intentar`, `Type::Resultado`
- Sema: `TypeInfo::Resultado { ok, err }`, validación completa
- IR: `Instr::ResultOk`, `Instr::ResultErr`, `Instr::TryUnwrap`
- Bytecode: `ResultOk(38)`, `ResultErr(39)`, `TryUnwrap(40)`
- VM: `Value::Exito(Box<Value>)`, `Value::Error(Box<Value>)`, propagación

### Fase 23: Opcion/Optional Type ✅
- Sintaxis: `opcion<T>`, valores `algun(valor)` y `ninguno`
- Token: `Opcion`/`Option`, `Algun`/`Some`, `Ninguno`/`None`
- AST: `Type::Opcion(Box<Type>)`, `Expr::Algun { expr }`, `Expr::Ninguno`
- Sema: `TypeInfo::Opcion(Box<TypeInfo>)`, `ninguno` asignable a cualquier `Opcion<T>`
- IR: `Instr::OptionSome`, `Instr::OptionNone`
- Bytecode: `OptionSome(41)`, `OptionNone(42)`
- VM: `Value::Opcion(Option<Box<Value>>)`, comparación por igualdad
- Tests: 5 sema, 10 e2e

### Fase 24: Enums/Tipos Suma ✅
- Sintaxis: `enum Nombre { Variante, Variante(tipo, ...) }`
- Constructor: `Nombre::Variante` o `Nombre::Variante(valor)`
- `DoubleColon` (::) para acceso a variantes
- Sema: validación de tipos en construcción y matching
- VM: `Value::Enum { name, variant, fields }`, opcode `EnumCtor(43)`
- Tests: 5 sema, 15 e2e

### Fase 25: Tuplas ✅
- Sintaxis: `(tipo, tipo)` y `(expr, expr)`
- Acceso por índice: `tupla.0`, `tupla.1`
- VM: `Value::Tuple(Vec<Value>)`, opcodes `TupleNew(44)`, `TupleAccess(45)`
- Tests: 4 e2e

### Fase 26: Destructuring ✅
- Sintaxis: `entero x, texto y = expr` (declaración) y `x, y = expr` (asignación)
- Wildcard `_`: `entero x, _ = (1, 2)` ignora elementos
- AST: `Decl::Destructure`, `Stmt::Destructure`, `DestructureTarget`
- Parser: desugaring en `parse_destructure_decl()` y `parse_destructure_assign_stmt()`
- Sema: valida tupla, verifica tipos/aridad, registra variables, omite `_`
- IR: temp variable `__dt_N` + `Load`/`TupleAccess(i)`/`Store` por cada target
- Loader: prefixing de nombres en targets de destructuración
- Tests: 14 e2e

### Fase 27: Genéricos Básicos ✅
- Sintaxis: `<T, U>` en funciones y estructuras
- Soporte en llamadas: `identidad<entero>(42)`
- Soporte en structs: `Par<entero, texto> { ... }`
- Implementación: Type erasure (compile-time checking)
- Parser: `parse_type_params`, `parse_type_args`, tracking de contexto genérico
- Sema: sustitución de tipos en firmas y validación
- Tests: 6 parser, 5 sema, 6 e2e

---

## Fases en construcción

### Fase 28: Stdlib — matematicas, texto ✅
- Módulos `.nv` en `stdlib/`
- `matematicas`: `abs`, `max`, `min`, `potencia`, `raiz`, `seno`, `coseno`
- `texto`: `longitud`, `mayusculas`, `minusculas`, `recortar`, `dividir`, `contiene`

### Fase 29: Stdlib — coleccion, fecha ✅
- `coleccion`: `invertir`, `ordenar`, `primero`, `ultimo`, `contar`
- `fecha`: `ahora` (timestamp UNIX)

### Fase 30: E/S de Archivos ✅
- `leer_archivo`, `escribir_archivo`, `existe_archivo` como builtins de VM
- Manejo de errores integrado (retorna errores en runtime)

### Fase 31: Stack Traces ✅
- VM mantiene call stack con `CallFrame { func_name, return_ip }`
- `VmError::with_stack()` muestra pila de llamadas completa
- `call_stack.push()` en Call y CallValue; pop en Ret y TryUnwrap

### Fase 32: Mensajes de Error Mejorados ✅
- Subrayado de posición exacta con caret (`^^^^`)
- Colores ANSI en terminal (rojo error, azul ubicación, verde subrayado, amarillo ayuda)
- Errores de `si`/`mientras` sin paréntesis muestran sugerencia con sintaxis correcta
- `show_error()` en CLI: muestra línea fuente, columna y subrayado
- `--help`/`-h`: ayuda completa con ejemplos de sintaxis básica

### Fase 33: Fuzzing ⏳
- `cargo-fuzz` para lexer, parser, decoder

### Fase 34: Property-Based Testing ⏳
- Round-trip invariants con proptest

### Fase 35: lumen fmt ⏳
- Formateador automático de código

### Fase 36: lumen repl ⏳
- Bucle interactivo

### Fase 37: lumen test ⏳
- Framework de testing nativo con `test`/`afirmar`

### Fase 38: lumen.toml + lumen new ⏳
- Manifiesto de proyecto

### Fase 39: Benchmarks + GitHub Releases ⏳
- Suite Criterion + binarios precompilados

---

## Comandos CLI

| Comando | Descripción |
|---------|-------------|
| `lumen run <file>` | Ejecuta fuente .nv o bytecode .nvc |
| `lumen build <file>` | Compila a .nvc |
| `lumen check <file>` | Verifica sintaxis + semántica |
| `lumen disasm <file>` | Desensambla .nvc |
| `lumen run -L <dir> <file>` | Ejecuta con ruta de librerías |

---

## Bytecode (.nvc)

- **Version**: 6
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-45
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays (ArrayNew, ArrayGet, ArraySet, ArrayLen, ArrayPush)
  - 33-34: Closures (FuncRef, CallValue)
  - 35-37: Structs (StructNew, StructGet, StructSet)
  - 38-40: Result (ResultOk, ResultErr, TryUnwrap)
  - 41-42: Option (OptionSome, OptionNone)
  - 43: Enum (EnumCtor)
  - 44-45: Tuples (TupleNew, TupleAccess)

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
docs/spec/        → grammar.ebnf, bytecode-format.md, error-codes.md, vm-spec.md
examples/         → *.nv (21+ ejemplos funcionales)
stdlib/           → *.nv (librería estándar)
tests/            → integration_test.rs
```
