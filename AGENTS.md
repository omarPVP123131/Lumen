# AGENTS.md — Diario de construcción de LÚMEN

**v1.1.0 — Release: Julio 2026**

---

## Testing (Final)

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

### Fase 0-15: Infraestructura base
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
- Version bytecode: 5

### Fase 18: Módulos ✅
- `importar "ruta.nv"`, `importar modulo`, `importar ... como alias`
- ModuleLoader nuevo: resolución de rutas, detección circular (E063)
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
- 213 tests, 0 warnings
- Pendiente: crates.io, GitHub Release con binarios

### Fase 21: For-Each ✅
- Sintaxis: `para x en expr` / `for x in expr`
- Token: `En`/`In` en lexer
- AST: `Stmt::ForEach { var_name, expr, body }`
- Parser: `parse_foreach()` con flag `no_struct_init` para evitar ambigüedad con struct init
- Sema: verifica que `expr` sea `Lista`, define variable del ciclo en nuevo scope
- IR: desugaring a while-loop con `ArrayLen`/`ArrayGet`/`Store`
- Sin cambios en bytecode o VM (reutiliza opcodes existentes)
- Tests: 4 parser, 6 sema, 9 e2e
- Ejemplo: `examples/foreach.nv`

### Fase 22: Opcion/Optional Type ✅
- Sintaxis: `opcion<T>`, valores `algun(valor)` y `ninguno`
- Token: `Opcion`/`Option`, `Algun`/`Some`, `Ninguno`/`None`
- AST: `Type::Opcion(Box<Type>)`, `Expr::Algun { expr }`, `Expr::Ninguno`
- Parser: parseo de tipo `opcion<T>`, expresión `algun(expr)`, keyword `ninguno`
- Sema: `TypeInfo::Opcion(Box<TypeInfo>)`, `ninguno` asignable a cualquier `Opcion<T>`
- IR: `Instr::OptionSome`, `Instr::OptionNone`
- Bytecode: `OptionSome(41)`, `OptionNone(42)`
- VM: `Value::Opcion(Option<Box<Value>>)`, comparación por igualdad
- Tests: 5 sema, 10 e2e

### Fase 23: Enums/Tipos Suma ✅
- Sintaxis: `enum Nombre { Variante, Variante(tipo, ...) }`
- Constructor: `Nombre::Variante` o `Nombre::Variante(valor)`
- `DoubleColon` (::) para acceso a variantes
- Sema: validación de tipos en construcción y matching
- VM: `Value::Enum { name, variant, fields }`, opcode `EnumCtor(43)`
- Tests: 5 sema, 15 e2e

### Fase 24: Tuplas ✅
- Sintaxis: `(tipo, tipo)` y `(expr, expr)`
- Acceso por índice: `tupla.0`, `tupla.1`
- VM: `Value::Tuple(Vec<Value>)`, opcodes `TupleNew(44)`, `TupleAccess(45)`
- Tests: 4 e2e

### Fase 25: Destructuring ✅
- Sintaxis: `entero x, texto y = expr` (declaración) y `x, y = expr` (asignación)
- Wildcard `_`: `entero x, _ = (1, 2)` ignora elementos
- AST: `Decl::Destructure`, `Stmt::Destructure`, `DestructureTarget`
- Parser: desugaring en `parse_destructure_decl()` y `parse_destructure_assign_stmt()`
- Sema: valida tupla, verifica tipos/aridad, registra variables, omite `_`
- IR: temp variable `__dt_N` + `Load`/`TupleAccess(i)`/`Store` por cada target
- Sin cambios en bytecode o VM (reutiliza `TupleAccess` existente)
- Loader: prefixing de nombres en targets de destructuración
- Tests: 14 e2e (declaración, asignación, wildcard, errores de tipo/aridad, 3 elementos, expresiones, inglés)

### Fase 26: Genéricos Básicos ✅
- Sintaxis: `<T, U>` en funciones y estructuras
- Soporte en llamadas: `identidad<entero>(42)`
- Soporte en structs: `Par<entero, texto> { ... }`
- Implementación: Type erasure (compile-time checking)
- Parser: `parse_type_params`, `parse_type_args`, tracking de contexto genérico
- Sema: sustitución de tipos en firmas y validación
- Tests: 6 parser, 5 sema, 6 e2e

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

- **Version**: 5
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-37
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays (ArrayNew, ArrayGet, ArraySet, ArrayLen, ArrayPush)
  - 33-34: Closures (FuncRef, CallValue)
  - 35-37: Structs (StructNew, StructGet, StructSet)

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
tests/            → integration_test.rs
```
