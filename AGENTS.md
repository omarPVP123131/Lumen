# AGENTS.md — Diario de construcción de LÚMEN

**v1.6.0 — Release: Julio 2026**

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
| lumen-vm | 109 | e2e |
| lumen-fmt | 2 | unit |
| lumen-repl | 2 | unit |
| lumen-project | 1 | unit |
| **Total** | **~306** | |

**~9 warnings, ~306 tests passing. 43/43 ejemplos funcionando.**

---

## Fases completadas

### Fase 0-15: Infraestructura base ✅
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

### Fase 21-27: Features del lenguaje ✅
For-Each, Resultado<T,E>, Opcion, Enums, Tuplas, Destructuring, Genéricos.

### Fase 28-30: Stdlib + Archivos ✅
matematicas, texto, coleccion, fecha, archivos.

### Fase 31: Stack Traces ✅

### Fase 32: Mensajes de Error Mejorados ✅
Caret, ANSI, preview multi-línea, conteo de errores.

### Fase 33: Fuzzing ✅
3 targets cargo-fuzz (lexer, parser, decoder).

### Fase 34: Property-Based Testing ✅
Proptest en codegen.

### Fase 35: lumen fmt ✅
`lumen fmt` formatea código .nv. Crate `lumen-fmt`.

### Fase 36: lumen repl ✅
REPL interactivo. Crate `lumen-repl`.

### Fase 37: lumen test ✅
`lumen test` ejecuta funciones `test_*`.

### Fase 38: lumen.toml + lumen new ✅
`lumen new` scaffolding. Crate `lumen-project`.

### Fase 39: CI/CD + Releases ✅
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

### Fase 59: Closures Pro 🔄
Captura por valor/referencia. En desarrollo.

### Fase 60: Async/Await 📋
Planificado para v2.0.

### Fase 65: Guard Let ✅
`sea patron = expr sino { romper/retornar/continuar }`. Desugaring en IR builder a JmpIf/Jmp.
`Stmt::GuardLet` en AST, `parse_guard_let()` en parser, sema + loader, IR builder.

### Fase 66: Operator Overloading ✅
Vía trait method convention (`impl Suma for Punto`). `Expr::Binary.resolved_method`.
Sema: `resolve_operator_overloads()` post‑analysis walk con HashMap de inferencia de tipos.
IR builder: `resolved_method` → `Call` o `Binary` nativo.

### Fase 69: Where Clauses ⏭️
Saltado — `<T: Rasgo>` syntax ya soporta bounds.

### Fase 70: Impl Trait return ✅
`funcion impl Rasgo foo() { retornar expr; }`. `Type::ImplTrait(String)` en AST.
Parseo en `parse_type()`, mapea a `TypeInfo::TypeVar` en sema (tipo opaco resuelto en llamada).

### Fase 71: Tipos Asociados en Traits ✅
`tipo Item;` en traits y `tipo Item = T;` en impl. `AssociatedType` e `ImplAssociatedType` en AST, sema e IR.

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
docs/spec/        → grammar.ebnf, bytecode-format.md, error-codes.md, vm-spec.md
examples/         → *.nv (42 ejemplos funcionales + desafiantes)
stdlib/           → *.nv (librería estándar: texto, matematicas, coleccion, fecha, archivos, matrices)
scripts/          → PowerShell CI/CD (pre-commit, pre-vuelo)
```
