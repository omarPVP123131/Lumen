# Siguientes Pasos — Roadmap LÚMEN v2.4 → v3.0

**Estado actual:** Fases 0-185 completas + Self-hosting puro (Sprint 5-6: fixpoint confirmado) + **Sprint 7: VM en LÚMEN (`vm.nv`) COMPLETADO** (fixpoint 861s → 20.1s, 43x COW Arc) + **Sprint 8: dogfooding — fuego 116/116 compilan · 108 CORRECTOS** (6 Ago 2026). Pendiente: bootstrapping doble + release v2.4.0.

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
| **Bootstrapping doble + release v2.4.0** | 🟡 Media | 🟡 Alto | vm.nv compilada por el compilador LÚMEN (hito final del self-hosting) + tag/release |
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

### 🟢 Completado: **Sprint 6-8 — LÚMEN autosuficiente (dogfooding total, 31 Jul - 6 Ago 2026)**

- **6.1 `importar` en el pipeline puro** ✅ — resolver `_imp_*` + `parser_parsear_con_base`, self-import detectado (canonicalize en loader.rs)
- **6.2-6.4 Gramática real** ✅ — `sea`/`const` (VarDecl), StructInit `T {}` → mapas, `.campo` → Index+Texto, `elegir`/`defecto:`/`caso`, enum `Nombre::Miembro(args)`, `algun`/`ninguno`/`exito`/`error`, cortocircuito `&&`/`||`, closures IIFE, params default, traits `rasgo`/`impl`/`este`, cast `como`
- **6.3 Runtime** ✅ — arrays anidados, ArraySet `arr[i] = x`, TryUnwrap top-level con `__tipo_de(fin)`, floats con `.`, lexer CRLF/UTF-8 seguro (`__str_subcadena_chars` nativo)
- **FIXPOINT v4 CONFIRMADO** ✅ — self/self2 byte-IDENTICAL (SHA-256 90048DC9…), 5s
- **Sprint 7 — VM en LÚMEN** ✅ — `vm.nv` (dispatch 0-46, corutinas reales, handlers JSON/tarea/coro/crypto/fs/env/tiempo/hilo/mutex/calendario), demo 120s → 0.9s, batería 39/40
- **Sprint 8 — Dogfooding** ✅ — stdlib completo compila; **fuego.ps1: 116/116 compilan · 108 CORRECTOS · 4 INCOMPATIBLES · 4 TIMEOUT · 0 fallos**; benchmarks: compile x5.4, run x231 (mediana x2-6)
- **Pendiente:** bootstrapping doble (compiler_v4 compila vm.nv → vm.nvc corre en VM LÚMEN → 0 dependencias de Rust) → release v2.4.0

### Después: **AI/ML (Fases 186-200) + Docs**
- AI/ML + Data Science — feature más solicitada para un lenguaje moderno
- Tensores, redes neuronales, DataFrames desde stdlib
- Binding a librerías C (llama.cpp, ONNX)
- Docs + Comunidad — sitio web, tutorial interactivo, playground web

### Después: **Producción & Cloud (Fases 201-220)**
- AWS Lambda runtime, WASM plugins, extensión VS Code, empaquetado nativo
