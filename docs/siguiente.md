# Siguientes Pasos — Roadmap LÚMEN v2.3 → v3.0

**Estado actual:** Fases 0-185 completas + Self-hosting total (mapas O(1)) + **Self-hosting puro (Sprint 5: fixpoint confirmado, 31 Julio 2026)**. Lenguaje, herramientas, stdlib, concurrencia, GUI/TUI/GFX, portabilidad.

---

## 🟢 Prioridad Alta — Fácil + Alto Impacto

| Área | Complejidad | Impacto | Razón |
|------|-------------|---------|-------|
| **AI/ML (Fases 186-200)** | 🔴 Alta | 🔥 Alto | Tensores, redes neuronales, DataFrames. Feature diferenciadora. |
| **Docs & Comunidad** | 🟢 Baja | 🔥 Muy Alto | Documentación, tutoriales, web, ejemplos. Lo que más necesita un lenguaje nuevo. |
| **Playground Web completo** | 🟡 Media | 🔥 Alto | `wasm-pack` + playground funcional con ejemplos interactivos. |

## 🟡 Prioridad Media — Moderado + Alto Impacto

| Área | Complejidad | Impacto | Razón |
|------|-------------|---------|-------|
| **Optimización VM para self-hosting** | 🟡 Media | 🟡 Medio | El self-compile puro tarda ~200s (VM LÚMEN interpretada); optimizar haría el bootstrap práctico |
| **Fixes y pulido** | 🟢 Baja | 🔥 Alto | Estabilidad general, edge cases |

## 🔵 Prioridad Baja — Muy Complejo + Nicho

| Área | Complejidad | Impacto | Razón |
|------|-------------|---------|-------|
| **Cloud (Fases 201-220)** | 🔴 Muy Alta | 🟡 Medio | AWS, GCP, Azure, K8s. Depende de ecosistema maduro. |
| **Extensiones VS Code/JetBrains** | 🟡 Media | 🟡 Medio | Atraería usuarios pero requiere mantenimiento constante. |

---

## 🎯 Recomendación

### ✅ Completado: **Self-hosting Rápido (30 Julio 2026)**
- `__compile_nv` builtin ejecuta pipeline Rust nativo (lex→parse→sema→ir→codegen)
- `compiler_v2.nv` reescrito: usa `__compile_nv` en vez del pipeline LÚMEN-in-LÚMEN lento
- Builtins string eficientes: `__str_subcadena`, `__str_concat_list`, `__str_starts_with`, `__str_to_chars`
- ArrayGet optimizado: `chars().collect()` → `chars().nth()`
- **Autocompilación: 533ms** (de >5min a 533ms)

### ✅ Completado: **Self-hosting Total (30 Julio 2026)**
- `Value::Map(Vec<...>)` → `HashMap<Value, Value>` con `Hash`/`Eq` manual
- `__map_get`/`__map_set`/`__map_contains`: O(n) → O(1)
- Sets union/inter/diff: O(n²) → O(n)
- `codegen_to_nvc`: lookups directos O(1)
- El parser LÚMEN-in-LÚMEN ahora tiene mapas O(1)

### ✅ Completado: **Self-hosting Puro (31 Julio 2026)**
- `compiler_v4.nv` autocontenido (55,308 bytes): lexer+parser+codegen+main sin imports
- Pipeline puro: lexer.nv → parser.nv → codegen.nv → `__codegen_a_nvc` → .nvc
- **Fixpoint confirmado**: `compiler_v4_self.nvc` (54,712 bytes, 49 funciones) se recompila a sí mismo con resultado idéntico en 3 runs (193s/203s/197s)
- 11 bugs críticos arreglados (saltos, TryUnwrap, print multi-arg, break/continue, escapes, keywords, forward decls)
- LÚMEN ya no depende de `__compile_nv` para compilarse

### 🟢 En curso: **Sprint 6-8 — LÚMEN autosuficiente (dogfooding total)**

**Sprint 6 — Imports + Gramática completa (compiler_v5 modular)** — prueba de fuego: 115/115 compilan, 29/115 ejecutan correctamente
- **6.1 `importar` en el pipeline puro** (P0 — ~50 ejemplos fallan por esto): resolver módulos, fusionar ASTs, prefijo `modulo_` en funciones y calls internos; compiler_v5 deja de ser concatenación (lexer/parser/codegen como módulos reales)
- **6.2 Keywords core faltantes**: `const`, `para`/`para cada` (for+foreach), `estructura` (campos, `T { ... }`, acceso `.campo`), `enum`/`opcion`/`resultado` (+ variantes), `elegir`/`sea` (match + if-let/while-let), closures `|x|`, params default `b = 10`, genéricos `<T>` (sintaxis), destructuring `_`, tuplas `(...)`
- **6.3 Runtime**: arrays anidados `arr[i][j]`, ArraySet `arr[i] = x`, propagación de errores TryUnwrap en top-level (actualmente silenciosa)
- **Verificación**: `fuego.ps1` → 100% OK+CORRECTO en todos los ejemplos que Rust puede ejecutar

**Sprint 7 — VM en LÚMEN + optimización (~200s → <10s)**
- 7.1 `vm.nv` — ejecutador de .nvc en LÚMEN puro (stack, frames, dispatch de opcodes, builtins)
- 7.2 Bootstrapping doble: compiler_v5 compila vm.nv → vm.nvc corre en la VM LÚMEN → **0 dependencias de Rust**
- 7.3 Optimización VM Rust: tabla de salto, internamiento de strings, evitar clones (3.7M instrs del self-compile)
- 7.4 Fixpoint doble: compilador Y VM compilados y ejecutados por sí mismos

**Sprint 8 — Dogfooding completo (release v2.4.0)**
- 8.1 Compilar el stdlib COMPLETO con compiler_v5 (matematicas, texto, coleccion, fecha, json, csv, red, tui, graficos)
- 8.2 Ejecutar los 115 ejemplos con la cadena 100% LÚMEN
- 8.3 Benchmarks vs Rust (compilación y ejecución)
- 8.4 Docs + AGENTS v2.4.0 + release

### Después: **AI/ML (Fases 186-200) + Docs**
- AI/ML + Data Science — feature más solicitada para un lenguaje moderno
- Tensores, redes neuronales, DataFrames desde stdlib
- Binding a librerías C (llama.cpp, ONNX)
- Docs + Comunidad — sitio web, tutorial interactivo, playground web

### Después: **Producción & Cloud (Fases 201-220)**
- AWS Lambda runtime, WASM plugins, extensión VS Code, empaquetado nativo
