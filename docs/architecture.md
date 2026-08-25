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
│             │  46 opcodes
│             │  Result/Option/Enum/Tuple/Map
└─────────────┘
    │
    ▼
┌─────────────┐
│   Tools     │  CLI (single binary)
│             │  fmt, repl, lsp, doc, pkg, aot
│             │  debug, test, lint, install
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
  lumen-cli/       main.rs (binario único)
  lumen-fmt/       lib.rs
  lumen-repl/      lib.rs
  lumen-lsp/       lib.rs
  lumen-doc/       lib.rs
  lumen-aot/       lib.rs
  lumen-pkg/       lib.rs
  lumen-project/   lib.rs
  lumen-api/       lib.rs
  lumen-plugin/    lib.rs
  lumen-bench/     benches/benchmarks.rs
docs/
  spec/            grammar.ebnf, bytecode-format.md,
                   error-codes.md, vm-spec.md
  language.md      Referencia del lenguaje
  cli.md           Referencia CLI
  architecture.md  Este documento
  roadmap.md       Roadmap de desarrollo
  contributing.md  Guía de contribución
examples/          *.nv (programas de ejemplo)
stdlib/            *.nv (biblioteca estándar)
scripts/           *.ps1 (CI/CD, hooks, installers)
fuzz/              fuzz targets
```

## Bytecode (.nvc) — v3.3.0 Producción

- **Versión**: 7 (`CHUNK_VERSION 7`, decode acepta 6 y 7 para compat con `.nvc` antiguos)
- **Novedad v3.1.4:** `FuncMeta.defaults: Vec<Option<DefaultValue>>` (`Int/Float/Str/Bool`) persistidos en el chunk para `bind_args` unificado (`Call`/`CallValue`/`run_function`/hilos usan defaults reales cuando `i>=args.len()` en vez de `Void`/`pop()` corrupto). Ver `docs/produccion.md` §1.3.
- **Magic**: `LUMN` (4 bytes)
- **Opcodes**: 0-46 + 52-53 (MatchType/MatchPayload) y rangos
  - 0-27: Core (Push, Pop, Add, Sub, Jmp, Call, Ret, Print, etc.)
  - 28-32: Arrays (ArrayNew, ArrayGet, ArraySet, ArrayLen, ArrayPush)
  - 33-34: Closures (FuncRef, CallValue)
  - 35-37: Structs (StructNew, StructGet, StructSet)
  - 38-40: Result (ResultOk, ResultErr, ResultUnwrap)
  - 41-42: Option (OptionSome, OptionNone)
  - 43: Enum (EnumCtor)
  - 44-45: Tuples (TupleNew, TupleAccess)
  - 46: Mod
  - 52-53: MatchType/MatchPayload (if-let / elegir con payloads)

## Modo Headless y Bench (v3.3.0)

- **Headless centralizado:** `stdlib/graficos.nv:es_headless()` usa `getenv("CI"/"LUMEN_HEADLESS")` vía `__ffi` (`msvcrt`/`libc`/`libSystem`) → `iniciar()`/`ventana()` retornan `false/0` sin `SDL_Init`. Demos con `si !iniciar() { retornar; }` salen con `init_fail_ok`. CI `headless-check` con `LUMEN_HEADLESS=1 CI=1`. Ver `docs/produccion.md` §1.4 y §3.
- **Bench formal 8 benches** (`crates/lumen-bench/benches/benchmarks.rs`): `lexer_tokenize`, `parser_parse`, `pipeline_full`, `vm_fib_20` + 4 prod `prod_fallthrough_early_return`, `prod_defaults_callvalue`, `prod_matematicas_potencia`, `prod_graficos_headless`. `cargo bench -p lumen-bench` (reporte `target/criterion/report/index.html`).
- **Builder escalable:** `last_significant()` + `label_counter` global (ver `docs/produccion.md` §1.1) garantiza terminador correcto y evita colisión de labels.

## Producción Real — Checklist

Ver `docs/produccion.md` checklist final: `cargo test --workspace` (917), `cargo bench`, `lumen check examples` 389/389, `LUMEN_HEADLESS=1` headless, `CHUNK_VERSION 7` con fallback v6.

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

