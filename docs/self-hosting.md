# PLAN: LÚMEN — Independencia Total (Self-Hosting como C/Rust)

**Objetivo:** LÚMEN se autocompila sin depender de Rust. El compilador, la VM y el runtime están escritos en LÚMEN. Bootstrap ocurre UNA sola vez con Rust. A partir de ahí, LÚMEN vive por sí mismo.

---

## Arquitectura Final

```
┌─────────────────────────────────────────────────────────────┐
│                   CÓDIGO FUENTE (.nv)                       │
└──────────────────────────┬──────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              COMPILADOR LÚMEN (escrito en LÚMEN)            │
│                                                             │
│  lexer.nv ──→ parser.nv ──→ sema.nv ──→ codegen.nv ──→ .nvc│
│                                                             │
│  Compilado a binario nativo vía Cranelift AOT               │
│  Bootstrapped UNA SOLA VEZ desde Rust                       │
└──────────────────────────┬──────────────────────────────────┘
                           ↓
┌─────────────────────────────────────────────────────────────┐
│              VM LÚMEN (escrita en LÚMEN)                    │
│                                                             │
│  vm_core.nv ──→ Cranelift AOT ──→ lumen_vm.exe             │
│                                                             │
│  Ejecuta bytecode .nvc sin Rust                             │
│  Bootstrapped UNA SOLA VEZ desde Rust                       │
└─────────────────────────────────────────────────────────────┘

Resultado: 0 líneas de Rust en producción.
Rust solo existió como "padre" en el bootstrap inicial.
```

---

## Fases del Plan

### 🟢 Fase 1: Lexer LÚMEN (Sprint 1)

**Duración:** 1-2 días | **Dificultad:** Fácil

| Tarea | Descripción | Validación |
|-------|-------------|------------|
| 1.1 | `stdlib/compiler/lexer.nv` — tokeniza fuente `.nv` | Output idéntico al lexer Rust |
| 1.2 | Soporte para todos los tokens: keywords ES/EN, números, strings, operadores | `cargo test` compara ambos outputs |
| 1.3 | Reporte de errores léxicos con línea/columna | Mismos errores que lexer Rust |
| 1.4 | Ejemplo: `lumen run stdlib/compiler/test_lexer.nv` | Tokeniza `demo_completo.nv` sin errores |

**API del lexer:**
```nv
funcion lista<Token> lexer_tokenizar(texto source)
funcion list<Token> lexer_tokenize(texto source)
```

**Estructura Token:**
```nv
estructura Token {
    tipo: texto,      // "Ident", "Numero", "String", "Si", "Funcion", ...
    valor: texto,     // el texto del token
    linea: entero,    // línea en el fuente
    columna: entero,  // columna en el fuente
}
```

---

### 🟡 Fase 2: Parser LÚMEN (Sprint 2)

**Duración:** 2-3 días | **Dificultad:** Media

| Tarea | Descripción | Validación |
|-------|-------------|------------|
| 2.1 | `stdlib/compiler/parser.nv` — parseo recursivo por descenso | AST idéntico al parser Rust |
| 2.2 | Expresiones, declaraciones, funciones, structs, enums | Parsea `demo_completo.nv` |
| 2.3 | Manejo de errores con sincronización | Errores coinciden con parser Rust |
| 2.4 | AST representado como `__map_*` + `__list_*` | Walkeable e inspeccionable |

**API del parser:**
```nv
funcion AST parser_parsear(lista<Token> tokens)
funcion AST parser_parse(list<Token> tokens)
```

**Estructura AST (representación con builtins):**
```nv
// Cada nodo AST es un numero (mapa) con tipo y campos
numero nodo = __map_nuevo();
nodo = __map_poner(nodo, "tipo", "DeclaracionVariable");
nodo = __map_poner(nodo, "nombre", "x");
nodo = __map_poner(nodo, "tipo_variable", "entero");
nodo = __map_poner(nodo, "valor", expresion_inicial);
```

---

### 🟡 Fase 3: Codegen LÚMEN (Sprint 3)

**Duración:** 2-3 días | **Dificultad:** Media

| Tarea | Descripción | Validación |
|-------|-------------|------------|
| 3.1 | `stdlib/compiler/codegen.nv` — AST → bytecode `.nvc` | Bytecode válido, ejecuta en VM |
| 3.2 | Genera opcodes: Push, Pop, Add, Sub, Call, Ret, etc. | `lumen run output.nvc` funciona |
| 3.3 | Soporte para funciones, structs, enums, control flow | `demo_completo.nv` compila |
| 3.4 | Formato `.nvc` v7: magic `LUMN`, version, opcodes | `lumen disasm output.nvc` legible |

**API del codegen:**
```nv
funcion Bytecode codegen_generar(AST arbol)
funcion Bytecode codegen_generate(AST tree)
```

---

### 🟢 Fase 4: Bootstrap del Compilador (Sprint 4)

**Duración:** 1 día | **Dificultad:** Fácil

| Paso | Acción |
|------|--------|
| 4.1 | Compilar `lexer.nv + parser.nv + codegen.nv` con el compilador Rust |
| 4.2 | Output: `lumen_compiler.nvc` |
| 4.3 | Verificar: `lumen_compiler.nvc` compila `hello.nv` → output correcto |
| 4.4 | **Hito:** El compilador de LÚMEN ahora existe como bytecode ✅ |

---

### 🔴 Fase 5: Self-Verificación del Compilador (Sprint 5)

**Duración:** 1-2 días | **Dificultad:** Crítica

| Paso | Acción |
|------|--------|
| 5.1 | Ejecutar `lumen_compiler.nvc` sobre `lexer.nv + parser.nv + codegen.nv` |
| 5.2 | Output: `lumen_compiler_v2.nvc` |
| 5.3 | **Comparar:** `lumen_compiler.nvc == lumen_compiler_v2.nvc` byte a byte |
| 5.4 | Si son idénticos → **SELF-HOSTING DEL COMPILADOR VERIFICADO** ✅ |
| 5.5 | Si no, depurar el compilador LÚMEN hasta que coincidan |

---

### 🔴 Fase 6: VM Core en LÚMEN (Sprint 6)

**Duración:** 3-5 días | **Dificultad:** Alta

| Tarea | Descripción |
|-------|-------------|
| 6.1 | `stdlib/compiler/vm_core.nv` — VM minimalista en LÚMEN |
| 6.2 | Stack basado en listas, IP, locals como `__map_*` |
| 6.3 | Opcodes core: Push, Pop, Add, Sub, Mul, Div, Jmp, Call, Ret, Print, Load, Store |
| 6.4 | File I/O vía builtins existentes (`__leer_archivo`, `__escribir_archivo`) |
| 6.5 | **NO incluye:** FFI, crypto, threads, GUI (se cargan como plugins `.dll`) |
| 6.6 | VM compila a binario nativo vía Cranelift AOT → `lumen_vm.exe` |

**API de la VM:**
```nv
funcion void vm_ejecutar(Bytecode bc)
funcion void vm_execute(Bytecode bc)

funcion texto vm_salida()        // output del programa
funcion texto vm_output()        // program output
```

---

### 🟢 Fase 7: Bootstrap de la VM (Sprint 7)

**Duración:** 1 día | **Dificultad:** Fácil

| Paso | Acción |
|------|--------|
| 7.1 | Compilar `vm_core.nv` con Rust → `lumen_vm.nvc` |
| 7.2 | Cranelift AOT → `lumen_vm.exe` (binario nativo) |
| 7.3 | Verificar: `lumen_vm.exe hello.nvc` → "Hola LÚMEN" |
| 7.4 | **Hito:** Runtime nativo sin Rust ✅ |

---

### 🔴 Fase 8: Self-Verificación de la VM (Sprint 8)

**Duración:** 1-2 días | **Dificultad:** Crítica

| Paso | Acción |
|------|--------|
| 8.1 | Ejecutar `lumen_vm.exe` con `vm_core.nv` → nuevo `lumen_vm.nvc` |
| 8.2 | Cranelift AOT → `lumen_vm_v2.exe` |
| 8.3 | Comparar binarios (`lumen_vm.exe` vs `lumen_vm_v2.exe`) |
| 8.4 | Si idénticos → **INDEPENDENCIA TOTAL VERIFICADA** ✅ |

---

## Resumen de Sprints

| Sprint | Fase | Días | Hito |
|--------|------|------|------|
| 1 | Lexer LÚMEN | 1-2 | Tokeniza sin Rust |
| 2 | Parser LÚMEN | 2-3 | AST sin Rust |
| 3 | Codegen LÚMEN | 2-3 | Genera `.nvc` sin Rust |
| 4 | Bootstrap compilador | 1 | `lumen_compiler.nvc` existe |
| 5 | Self-verificación | 1-2 | Compilador se autocompila |
| 6 | VM Core LÚMEN | 3-5 | Runtime en LÚMEN |
| 7 | Bootstrap VM | 1 | `lumen_vm.exe` nativo |
| 8 | Self-verificación VM | 1-2 | VM se autocompila |
| **Total** | | **13-19 días** | Independencia total |

---

## Lo que NO se implementa en LÚMEN (plugins externos)

| Componente | Motivo | Alternativa |
|-----------|--------|-------------|
| **FFI** (`libloading`) | Necesita OS-level linking | Plugin `.dll` externo |
| **Crypto** (BCrypt) | API Windows específica | Plugin `.dll` |
| **SDL2 / GUI** | Depende de DLLs externas | Plugin `.dll` |
| **Threads** | Necesita `std::thread` | Plugin `.dll` |
| **AOT (Cranelift)** | Demasiado complejo para LÚMEN | Se mantiene en Rust como backend externo |

La VM core solo ejecuta bytecode. Las extensiones se cargan como `.dll` desde LÚMEN vía `__ffi_cargar`. Esto es igual que Lua, Python, Node.js — el core es mínimo, las extensiones son nativas.

---

## Comparativa: Actual vs Self-Hosting

| | Actual (v2.0) | Fase 5 | Fase 8 (Final) |
|---|---|---|---|
| **Compilador** | Rust | LÚMEN | LÚMEN |
| **VM** | Rust | Rust | LÚMEN → nativo |
| **Compila LÚMEN** | ✅ | ✅ | ✅ |
| **Compila la VM** | ❌ | ❌ | ✅ |
| **Depende de Rust para compilar** | ✅ | **NO** | **NO** |
| **Depende de Rust para ejecutar** | ✅ | ✅ | **NO** |
| **Self-hosting** | ❌ | ✅ Parcial | ✅ **Completo** |

---

## Principio de Bootstrap

El bootstrap solo ocurre UNA vez:

```
┌─────────────────────────────────────────────────────────────────┐
│  ETAPA 1: Bootstrap inicial (una sola vez, luego se descarta)    │
│                                                                  │
│  lexer.nv + parser.nv + codegen.nv                               │
│         ↓                                                        │
│  Compilador Rust (EXISTENTE) → lumen_compiler.nvc                │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│  ETAPA 2: Self-compilación                                       │
│                                                                  │
│  lumen_compiler.nvc → VM Rust → compila lexer+parser+codegen     │
│                              → lumen_compiler_v2.nvc             │
│                                                                  │
│  ¿lumen_compiler.nvc == lumen_compiler_v2.nvc? → ✅              │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│  ETAPA 3: Independencia                                          │
│                                                                  │
│  vm_core.nv → compila con Rust (última vez) → lumen_vm.exe       │
│  lumen_vm.exe ejecuta lumen_compiler.nvc → compila vm_core.nv    │
│                                          → lumen_vm_v2.exe       │
│                                                                  │
│  ¿lumen_vm.exe == lumen_vm_v2.exe? → ✅                          │
│                                                                  │
│  A PARTIR DE AQUÍ: 0 dependencias de Rust                        │
└─────────────────────────────────────────────────────────────────┘
```

---

> **"Si quieres construir un barco, no le digas a la gente que recoja madera. Haz que añoren el mar."**
> — Antoine de Saint-Exupéry
