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
└──────┬──────┘
       │
       ▼
┌─────────────┐
│    VM       │  crates/lumen-vm
│             │  Stack-based
│             │  Call frames
│             │  37 opcodes
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

- **Versión**: 5
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-37
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays (ArrayNew, ArrayGet, ArraySet, ArrayLen, ArrayPush)
  - 33-34: Closures (FuncRef, CallValue)
  - 35-37: Structs (StructNew, StructGet, StructSet)

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
