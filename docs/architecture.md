# Arquitectura del Compilador LÚMEN

## Pipeline de Compilación

```
Fuente .nv
    │
    ▼
┌─────────────┐
│   Lexer     │  crates/lumen-lexer
│             │  Texto → Tokens
│             │  Recuperación de errores
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Parser    │  crates/lumen-parser
│             │  Recursive descent + Pratt
│             │  Tokens → AST
│             │  Sincronización en errores
└──────┬──────┘
       │
       ▼
┌─────────────┐
│ ModuleLoader│  crates/lumen-sema (loader.rs)
│             │  Resuelve importar/import
│             │  Flatten + prefix de nombres
│             │  Detección circular (E063)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Sema       │  crates/lumen-sema
│             │  Type checking
│             │  Scope management
│             │  Type inference
│             │  Operator overload resolution
│             │  Impl trait verification
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   IR        │  crates/lumen-ir
│             │  Three-address code
│             │  Constant folding
│             │  Dead code elimination
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Codegen    │  crates/lumen-codegen
│             │  IR → Bytecode
│             │  Shared constant pools
│             │  Formato .nvc
│             │  Property-based testing (proptest)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    VM       │  crates/lumen-vm
│             │  Stack-based
│             │  Call frames
│             │  37 opcodes base
│             │  + Result/Option/Enum/Tuple (8)
│             │  + Mod (1)
└─────────────┘
```

## Estructura del Proyecto

```
crates/
  lumen-lexer/     token.rs, lexer.rs, error.rs
  lumen-parser/    ast.rs, parser.rs, error.rs
  lumen-sema/      sema.rs, loader.rs, error.rs
  lumen-ir/        ir.rs, builder.rs
  lumen-codegen/   bytecode.rs, codegen.rs, disasm.rs
  lumen-vm/        vm.rs, value.rs
  lumen-cli/       main.rs
  lumen-fmt/      lib.rs
  lumen-repl/     lib.rs
  lumen-project/  lib.rs
docs/
  spec/            grammar.ebnf, bytecode-format.md,
                   error-codes.md, vm-spec.md
  language.md      Referencia del lenguaje
  cli.md           Referencia CLI
  architecture.md  Este documento
  roadmap.md       Roadmap de desarrollo
  contributing.md  Guía de contribución
examples/          *.nv (programas de ejemplo)
tests/             integration_test.rs
```

## Bytecode (.nvc)

- **Versión**: 6
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-46
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays (ArrayNew, ArrayGet, ArraySet, ArrayLen, ArrayPush)
  - 33-34: Closures (FuncRef, CallValue)
  - 35-37: Structs (StructNew, StructGet, StructSet)
  - 38-40: Result (ResultOk, ResultErr, ResultUnwrap)
  - 41-42: Option (OptionSome, OptionNone)
  - 43: Enum (EnumCtor)
  - 44-45: Tuples (TupleNew, TupleAccess)
  - 46: Mod

## Value System (VM)

La VM maneja valores a través del enum `Value`:

- `Value::Int(i64)` — Entero
- `Value::Float(f64)` — Decimal
- `Value::Str(String)` — Texto
- `Value::Bool(bool)` — Booleano
- `Value::Array(Vec<Value>)` — Lista
- `Value::Func(String)` — Referencia a función
- `Value::Struct { name, fields }` — Estructura
- `Value::Void` — Vacío
