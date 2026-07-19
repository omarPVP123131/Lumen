# AGENTS.md — Diario de construcción de LÚMEN

## Fecha de inicio: Julio 2026

Este documento registra el proceso de construcción del lenguaje LÚMEN desde cero, fase por fase.

---

## Fase 0: Cimientos del proyecto

### Decisión inicial
- **Lenguaje**: Rust (edition 2021)
- **Estructura**: Monorepo con Cargo workspace
- **Crates**: `lumen-lexer`, `lumen-parser`, `lumen-sema`, `lumen-ir`, `lumen-codegen`, `lumen-vm`, `lumen-cli`
- **MSRV**: Rust 1.70+
- **Testing**: `cargo test` con unit tests e integration tests

### Convenciones
- **Commits**: Conventional Commits
- **Versionado**: SemVer estricto (0.1.0 actual)
- **Bytecode**: Formato `.nvc` versionado con magic number `LUMN`

### Fixes aplicados (Julio 2026)
- **Parser**: Se corrigieron `parse_function`, `parse_if`, `parse_while`, `parse_for`, `parse_return` para consumir el keyword con `self.peek().span; self.advance();` antes de parsear (antes usaban `self.previous().span` que era incorrecto porque el keyword era confirmado por `check()` pero nunca consumido)
- **Parser**: Se corrigió `parse_assignment` para usar `self.peek()` en lugar de `self.previous()` — ahora lee el identificador del token actual y lo consume
- **Parser**: Se agregó soporte para keywords callables (`imprimir`, `print`, `leer`, `read`) como expresiones de llamada a función en `parse_primary`
- **Parser**: Se extrajo lógica común a `parse_call_or_ident()` para evitar duplicación
- **IR Builder**: Se agregó creación automática de función `__main__` para código de nivel superior (top-level code). Se corrigió `finalize_func` para retornar a `__main__` después de procesar cada función declarada.
- **IR**: Cambio de `HashMap` a `BTreeMap` en `Program.funcs` para orden determinístico de funciones.
- **Codegen**: Se reemplazó `instr_size` con índices de instrucción directos en resolución de etiquetas. Se agregó `FuncMeta` al bytecode con nombre, parámetros y start.
- **Bytecode**: Se agregó `funcs: Vec<FuncMeta>` encode/decode al formato `.nvc`.
- **VM**: Se implementó `Ret` handler en `execute_simple` (antes era no-op, el handler real estaba en `execute_with_idx` donde nunca se invocaba). Se implementó `Call` handler para funciones de usuario con call stack, scoped locals y binding de parámetros. La VM arranca en `__main__`.
- **CLI**: Se corrigió flujo `run_source` para eliminar `Print` posterior a `Call` (elimina output "void").
- **Warnings**: Se eliminaron ~25 warnings de compilación (dead_code, unused_vars, unused_mut, unused_imports). Todos los crates compilan sin warnings.
- **Tests**: 76 tests unitarios pasan en todos los crates. flujo `cargo run -- run examples/*.nv` 100% funcional.

---

## Fase 1: Especificación formal del lenguaje

### Gramática EBNF
Definida en `docs/spec/grammar.ebnf` — cubre expresiones, sentencias, declaraciones.

### Sistema de tipos
- `numero` → f64 (unificado en v1.0, se separará en `entero`/`decimal` en Fase 9)
- `texto` → String
- `booleano` → bool
- Sin coerciones implícitas

### Códigos de error
- `E001`–`E010`: Errores léxicos
- `E011`–`E030`: Errores sintácticos
- `E031`–`E050`: Errores semánticos

---

## Fase 2: Lexer

### Implementación
- Lexer manual (transparencia pedagógica)
- Cada token lleva línea/columna exacta
- Errores recuperables (no aborta en primer error)
- Soporte: `//` comentarios, `/* */` bloque (no anidado)
- Escapes en strings: `\n`, `\t`, `\"`, `\\`

### Tests: +150 casos unitarios

---

## Fase 3: Parser y AST

### Implementación
- Recursive descent + Pratt parsing para expresiones
- Error recovery (sincronización en `;` y `}`)
- AST serializable a JSON para debugging

### Tests: Snapshot testing, round-trip

---

## Fase 4: Análisis semántico

### Implementación
- Symbol table con scopes anidados
- Type checking estático
- Mensajes de error pedagógicos con snippet y sugerencia

---

## Fase 5: IR (Representación Intermedia)

### Implementación
- Three-address code
- CFG por función
- Constant folding como optimización básica

---

## Fase 6: Generador de bytecode

### Implementación
- Stack-based instruction set
- Formato `.nvc` con header versionado
- Encoder/decoder simétricos
- Disassembler

---

## Fase 7: Máquina Virtual

### Implementación
- Stack-based VM (fetch-decode-execute)
- Call stack con frames
- Arena heap para strings
- NaN boxing para valores (postergado)

---

## Fase 8: CLI completa

### Comandos
- `lumen run <file>` — ejecuta fuente o bytecode
- `lumen build <file>` — compila a `.nvc`
- `lumen check <file>` — verifica sintaxis+semántica
- `lumen disasm <file>` — desensambla bytecode

---

## Testing (Actual)

| Crate | Tests | Type |
|-------|-------|------|
| lumen-lexer | 24 | unit |
| lumen-parser | 28 | unit |
| lumen-sema | 27 | unit |
| lumen-ir | 5 | unit |
| lumen-codegen | 13 | unit |
| lumen-codegen | 5 | proptest |
| lumen-vm | 45 | unit |
| lumen-vm | 41 | e2e |
| **Total** | **188** | |

---

### Fase 14: Arrays/Listas ✅

**Objetivo**: Tipos de datos compuestos con `lista<Type>` dinámico.

- [x] Lexer: tokens `[`, `]`, `.`, `lista`/`array`
- [x] AST: `Type::Lista`, `Expr::List`/`Index`/`MethodCall`
- [x] Parser: tipos genéricos, literales `[...]`, indexación `expr[expr]`, métodos `expr.nombre(args)`
- [x] Sema: `TypeInfo::Lista`, type checking de métodos (`agregar`/`push`, `largo`/`len`/`length`)
- [x] IR: `ArrayNew`, `ArrayGet`, `ArraySet`, `ArrayLen`, `ArrayPush`
- [x] Bytecode: 5 nuevas opcodes (28-32), versión 3
- [x] Codegen: emitir opcodes de array
- [x] VM: `Value::Array(Vec<Value>)`, handlers para todas las opcodes
- [x] Ejemplo: `examples/arrays.nv` funcional
- [x] 8 tests e2e de arrays
- [x] 173 tests pasan, 0 warnings

**Esfuerzo**: ~8-12 hours

---

### Fase 15: Control de flujo avanzado ✅

**Objetivo**: `romper` (break), `continuar` (continue), `elegir` (switch/match).

- [x] Lexer: tokens `Romper`/`Break`, `Continuar`/`Continue`, `Elegir`/`Match`, `Caso`/`Case`, `Defecto`/`Default`, `Colon`
- [x] AST: `Stmt::Break`, `Stmt::Continue`, `Stmt::Match`, `MatchArm`
- [x] Parser: parseo completo con casos, defecto, síncrono en recovery
- [x] Sema: `loop_depth` tracking, error E054/E055 fuera de ciclo, tipo de armas E056
- [x] IR: `LoopLabels` stack, `Break`/`Continue` → `Jmp`, `Match` → cadena de comparaciones
- [x] Ejemplos: `examples/break.nv`, `examples/continue.nv`, `examples/match.nv`
- [x] 10 tests e2e de control de flujo
- [x] 183 tests pasan, 0 warnings

---

### Fase 15: Control de flujo avanzado

**Objetivo**: `romper` (break), `continuar` (continue), `elegir` (switch/match).

- **`romper`/`continuar`**: Lexer tokens, parser stmts, IR labels + jumps, VM ya lo soporta (solo falta cableado)
- **`elegir`/`match`**: Pattern matching básico con tipos, exhaustiveness check en sema
- **AST**: `Stmt::Break`, `Stmt::Continue`, `Stmt::Match { expr, arms }`
- **IR**: `Instr::Break(target_label)`, `Instr::Continue(target_label)`
- **Esfuerzo**: ~4-6 hours · **Dependencias**: Fase 9

---

### Fase 16: Funciones avanzadas

**Objetivo**: Closures, funciones anónimas, parámetros default.

- **Closures**: Captura de entorno (by value), `Fn` type en IR y VM
- **Funciones anónimas**: `funcion(x, y) { retornar x + y; }` como expresión
- **Parámetros default**: `funcion foo(numero x = 42) { ... }`
- **AST**: `Expr::Lambda { params, body }`, `Type::Function`
- **VM**: Closure representation (code pointer + captured env)
- **Esfuerzo**: ~10-16 hours · **Dependencias**: Fase 14

---

### Fase 17: Estructuras/Objetos

**Objetivo**: Tipos compuestos (`estructura`/`struct`, `objeto`/`object`).

- **Sintaxis**: `estructura Persona { nombre: texto, edad: numero }`
- **Instanciación**: `Persona { nombre: "Ana", edad: 30 }`
- **Acceso**: `persona.nombre`
- **AST**: `Decl::Struct`, `Expr::StructInit`, `Expr::FieldAccess`
- **VM**: `Value::Struct(HashMap<String, Value>)`
- **Esfuerzo**: ~6-10 hours · **Dependencias**: Fase 13, Fase 14

---

### Fase 18: Módulos y archivos múltiples

**Objetivo**: `importar`/`import`, separación en archivos, namespaces.

- **Sintaxis**: `importar "math.nv";` o `importar math;`
- **Loader**: resolver rutas relativas, evitar circular imports
- **IR**: merge de programas, namespacing de funciones
- **CLI**: `lumen run main.nv --lib-dir ./lib`
- **Esfuerzo**: ~6-8 hours · **Dependencias**: Fase 17

---

### Fase 19: Optimizaciones

**Objetivo**: Mejorar performance de bytecode y VM.

- **IR**: Constant folding (ya existe básico), dead code elimination, inlining de funciones pequeñas
- **Codegen**: Pool de strings/nums compartido entre funciones, eliminación de Nops redundantes
- **VM**: NaN boxing para Values (postergado de Fase 7), arena allocator para strings, threaded code dispatch
- **Esfuerzo**: ~8-12 hours · **Dependencias**: Fase 13-18

---

### Fase 20: v1.0 — Release

**Objetivo**: Publicación estable del lenguaje.

- [ ] Documentación completa (tutorial, referencia, examples)
- [ ] README.md con badges, ejemplos, instrucciones de instalación
- [ ] Publicar en crates.io
- [ ] Release en GitHub con binarios precompilados
- [ ] Changelog v1.0

**Esfuerzo**: ~4-6 hours · **Dependencias**: Fases 13-19

---

## Leyenda de esfuerzo

| Esfuerzo | Horas estimadas |
|----------|-----------------|
| Pequeño | 2-4h |
| Mediano | 4-8h |
| Grande | 8-16h |
| Masivo | 16h+ |

**Prioridad recomendada**: Fase 9 → 10 → 11 → 12 (infraestructura), luego Fase 13 → 14 → 15 (features core), luego Fase 16 → 17 → 18 (features avanzadas), luego Fase 19 → 20 (polish).

---

## Ejemplos

### hello.nv
```
imprimir("¡Hola, LÚMEN!");
```

### loop.nv
```
numero contador = 0;
mientras (contador < 5) {
    imprimir(contador);
    contador = contador + 1;
}
```

### func.nv
```
funcion numero suma(numero a, numero b) {
    retornar a + b;
}
numero resultado = suma(3, 7);
imprimir(resultado);
```
