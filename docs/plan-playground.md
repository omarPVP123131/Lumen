# Plan del Playground Web — LÚMEN v2.4.2+

> Estado: PLAN v2 (Alt C: niveles de madurez) · 14 Ago 2026
> Estructura: **9 features × 3 niveles (L1 funcional → L2 pulido → L3 avanzado) = 27 fases**, cada una con criterios de aceptación verificables.
> ✅ **Ronda L1 COMPLETADA (14 Ago):** F1.1, F2.1, F3.1, F3.2, F4.1, F9.1 — ver "Estado actual".

---

## 1. Estado actual (lo que ya tenemos)

| Componente | Detalle |
|---|---|
| `lumen serve [--port N]` | Servidor HTTP estático en Rust puro (`crates/lumen-cli/src/main.rs`), sin Python. MIME types, headers COOP/COEP, 404, anti path-traversal, redirección `/` → `/web/index.html`. Endpoints: `GET /api/health`, `GET /api/examples`, `GET /api/examples/{file}`, `POST /api/run` (VM Rust nativa → `{ok,output}`/`{ok,error}`). Verificado (200/404/JSON). |
| Runtime WASM | `crates/lumen-wasm/src/lib.rs`: `LumenRuntime` wasm-bindgen con `run`, `run_with_files`, `check`, `tokenize`, `compile_to_bytes`, `version`, `register_js_function`. Pipeline completo (lexer→parser→sema→IR→codegen→VM) en el browser. |
| Stdlib embebida | `crates/lumen-wasm/build.rs` genera `embedded_stdlib.rs` (31 archivos incl) + `ModuleLoader::with_memory_files` resuelve imports desde memoria (F3.1). |
| Editor CodeMirror 6 | `web/vendor/cm/` (11 módulos ESM planos, vendor local sin CDN) + modo LUMEN generado desde `token.rs` (74 keywords, `StreamLanguage` + Catppuccin). **Autosave localStorage, error-line marking, `Ctrl+Enter`, gutter, autocompletado (`Ctrl+Space` + keywords/snippets), minimapa** (F2.1 + F2.3). |
| UI | `crates/lumen-wasm/web/index.html`: toggle **WASM ↔ Servidor** (persistente vía `localStorage`), historial de ejecuciones (hasta 10 runs), **selector con categorías, búsqueda, favoritos, marcador "importar"**, 128 ejemplos (API `/api/examples` + fallback `embedded_examples.js`), 3 pestañas (Salida/Consola/JS Interop), statusbar con tiempo, toast, 17 bridges JS. Versión v2.4.2. |
| Descargar .nvc | `compile_to_bytes(source)` → `Uint8Array` → Blob descargable (F9.1). |
| Build WASM | `wasm-pack build crates/lumen-wasm --target web` + `pkg/` en .gitignore (regenerable). |
| Batería F4.1 | 128 ejemplos embebidos en `embedded_examples.js` (autogenerado por `gen-embedded-examples.ps1`). |
| Tests | cargo test 0 fallos (lexer 27, parser 45, sema 56, ir 20, vm 45, e2e 166, aot 4, api 5, etc. ~380). |

## 2. Gaps que el plan resuelve

1. **Sin resaltado de sintaxis** — editor `<textarea>` plano.
2. **Imports/stdlib NO funcionan en el navegador** — `ModuleLoader` lee del filesystem; en WASM no hay filesystem → todo `importar "*.nv"` falla. **Gap más grande.**
3. Sin persistencia (localStorage), sin compartir por URL, sin descargar/subir `.nv`.
4. Selector con 18 ejemplos hardcodeados, no los 117 reales del repo.
5. Sin AST/Disasm/Debugger visual.
6. Sin hot reload.
7. Sin programación en bloques.
8. Sin compilación nativa desde el playground (depende de Etapa 3 AOT→Rust).
9. Sin CI wasm ni despliegue online (hoy solo local).

---

## 3. Plan por features y niveles (27 fases)

Leyenda: **L1** = funcional (fase base) · **L2** = pulido · **L3** = avanzado.
Orden de ejecución global sugerido: **todas las L1 → todas las L2 → todas las L3** (cada feature queda usable desde su L1).

---

### F1 — Servidor web (`lumen serve`)

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F1.1 | Endpoints `/api/health` y `/api/examples` (índice: nombre, archivo, descripción de los 117 ejemplos) + `GET /api/examples/{file}` con contenido | `curl /api/health` → 200 JSON `{status,version}`; índice lista 117 entradas y cada `{file}` devuelve el contenido exacto del repo; 404 correcto para archivo inexistente; `/api/*` con headers CORS y sin cache agresiva |
| L2 | F1.2 | Cache del índice (mtime + ETag) + `--port` por env var `LUMEN_PORT` | ✅ Segunda petición con `If-None-Match` correcto → 304; cambiar un ejemplo en disco → el índice refleja el cambio sin reiniciar; `LUMEN_PORT=9000 lumen serve` escucha en 9000 |
| L3 | F1.3 | `GET /api/compile-native` (POST del `.nv` → responde `.nvc` descargable hoy; exe tras Etapa 3 AOT) + `GET /api/meta` (versión del compilador, stdlib embebida, features) | POST válido devuelve `.nvc` ejecutable con `lumen run`; POST inválido devuelve 422 con el error del compilador; `/api/meta` informa `aot:false` hasta Etapa 3 |

### F2 — Editor (CodeMirror)

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F2.1 | CodeMirror 6 vendor local (`web/vendor/codemirror/`) + gramática LÚMEN v1: keywords ES/EN, strings, números, comentarios, operadores. Tema Catppuccin igual al CSS actual | La lista de keywords se genera del lexer real (script `gen-lumen-mode.js`), no a mano; los 18 ejemplos se ven coloreados correctamente; `Ctrl+Enter` sigue ejecutando; 0 errores en consola del browser |
| L2 | F2.2 | Autosave localStorage (2s), compartir por URL (hash base64 `#code=...` + botón copiar enlace), descargar `.nv`, subir `.nv` (file picker) | ✅ Recargar página restaura el código; abrir enlace en otra pestaña restaura el código; roundtrip descargar→subir byte-idéntico; persistencia separada por ejemplo |
| L3 | F2.3 | Línea de error resaltada en gutter (mapa span→línea), autocompletado (`Ctrl+Space`: keywords + snippets), minimapa | ✅ Un E042/E020 muestra la línea exacta subrayada y el panel de errores navega a ella; `Ctrl+Space` inserta keywords/snippets sin errores de parse; minimapa sincronizado con scroll |

### F3 — Runtime-lenguaje completo en browser (loader virtual)

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F3.1 | `crates/lumen-wasm/build.rs` genera `embedded_stdlib.rs` (`include_str!` por archivo, sin deps nuevas) + `ModuleLoader::with_memory_files(HashMap)` en lumen-sema que resuelve imports desde memoria antes de disco | `build.rs` genera el módulo con TODOS los `.nv` de `stdlib/`; nuevo unit test en lumen-sema: import desde memoria sin tocar disco; los 166 e2e y 378 tests existentes siguen verdes (compatibilidad total) |
| L2 | F3.2 | `run_lumen` (wasm) usa el loader virtual; builtins de filesystem/red/env (`__fs_*`, `__env_*`, `__ffi_*`, `__red_*`) retornan error controlado (no panic) en browser | `importar "texto.nv"` compila y ejecuta en browser; smoke test wasm: `test_stdlib_mini`, `test_texto_std`, `test_json_avanzado` con imports == output CLI (salvo timestamps); un `__fs_*` devuelve mensaje claro tipo `no disponible en navegador`, sin panic |
| L3 | F3.3 | Batería completa de paridad: `demo_completo`, `test_migracion`, `jr_fecha`, `tui_jr` (secciones sin GUI) contra output CLI; panel "soporte" por builtin en UI | ≥20 ejemplos con imports byte-idénticos a CLI en un harness de test automatizado del wasm (wasm-pack test); la UI lista qué builtins están disponibles/ausentes en browser |

### F4 — Ejemplos

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F4.1 | Selector poblado con los 117 ejemplos reales: fetch a `/api/examples` cuando hay servidor; fallback a JSON embebido (funciona en `file://` y GitHub Pages) | Con servidor: selector lista 117 nombres con categoría; sin servidor: al menos 40 ejemplos clave disponibles; cambio de ejemplo carga el código y actualiza línea de números |
| L2 | F4.2 | Categorías (basics/functions/data/pro/stdlib), búsqueda, favoritos en localStorage, marcador "usa stdlib/importar" | ✅ Filtrar por categoría y búsqueda textual funciona; favoritos persisten entre sesiones; el marcador detecta `importar` en el código del ejemplo |
| L3 | F4.3 | Ejemplos interactivos: los que usan bridges JS (canvas, DOM, alert) se ejecutan con su bridge activo; botón "probar" que corre un ejemplo con asserts y muestra ✓/✗ | ≥5 ejemplos interactivos funcionan (p.ej. canvas, título de página); el botón "probar" reporta pass/fail por assert con salida visible |

### F5 — Insights (AST / Disasm)

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F5.1 | Pestañas nuevas: **Tokens** (mejorada) y **AST** — runtime wasm expone `ast_dump(source)` (JSON con estructura del AST) | El AST mostrado para `demo_completo` refleja su estructura real (funciones, llamadas, literales); tokens ya funcional; navegación por pestañas sin romper el estado del editor |
| L2 | F5.2 | Pestaña **Disasm** — runtime wasm expone `disasm_bytecode(source)` reusando `lumen_codegen::disassemble` | El texto mostrado es idéntico (byte a byte) a `lumen disasm` del .nvc del mismo source (verificado con 3 ejemplos) |
| L3 | F5.3 | Stats del programa (funciones, instrucciones, opcodes más usados), diff entre disasm de dos fuentes, exportar disasm/AST a archivo | Stats numéricamente correctos para `demo_completo` (verificados contra el VM); diff resalta diferencias; exportación descarga `.txt`/`.json` |

### F6 — Debugger visual

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F6.1 | Runtime wasm expone `debug_init(source)`/`debug_step()`/`debug_state()` (ip, pila, output); UI con step/run/reset sobre la vista Disasm | `step` avanza ip de una instrucción a la vez; pila mostrada coincide con `vm.stack_top()`; reset vuelve al inicio; run termina igual que el botón Ejecutar |
| L2 | F6.2 | Breakpoints clickeando el gutter del Disasm + panel de call stack + pausa en error | Break en la ip marcada (verificado con loop); call stack listado al romper en una llamada anidada; un runtime error pausa mostrando pila y mensaje |
| L3 | F6.3 | Watch de variables (nombres + valores del scope actual), hover de valores, mapa aproximado ip→línea fuente (v2 documentado como aproximado) | Watch lista variables del frame actual actualizadas tras cada step; hover sobre una ip muestra instrucción y valores; indicador visual "aproximado" en el mapeo línea |

### F7 — Programación en bloques (Blockly)

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F7.1 | Blockly vendor local (core + `es`) + toggle **Bloques ↔ Código** | El toggle cambia de vista sin perder el código; los bloques base (imprimir, variable, número, si/sino, mientras) generan LÚMEN textual válido |
| L2 | F7.2 | Generador completo bloques→LÚMEN: categorías variables, funciones simples, operadores, listas; **5 ejemplos base** disponibles en ambas vistas | Los 5 ejemplos generados desde bloques compilan y producen el mismo output que su versión de código; editar código manualmente desactiva la vista bloques con aviso visual (v1: sync solo bloques→código, documentado) |
| L3 | F7.3 | Categorías avanzadas (enums, resultado/opcion, structs, llamadas a bridges JS), bloques anidados con validación de tipos, "Bloques" como panel secundario (no reemplaza al editor) | Un programa con `elegir`/`resultado` se construye desde bloques y ejecuta; la validación previene tipos incompatibles (entero donde va texto); ambos paneles visibles simultáneamente |

### F8 — Hot reload / UX

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F8.1 | Botón **auto-ejecutar**: al guardar (Ctrl+S) o tras 1s de inactividad en el editor → run automático | Auto-run dispara con el código actual; el botón on/off persiste; no interfiere con Ctrl+Enter |
| L2 | F8.2 | `lumen serve --watch` (polling mtime 2s de `examples/` + `stdlib/`, sin dep de `notify`) + ETag en `/api/examples` | Tocar un ejemplo en disco → aparece/actualiza en el selector sin recargar la página (≤3s); ETag correcto en respuestas del índice |
| L3 | F8.3 | Push SSE (EventSource) de cambios + indicador visual "ejemplos actualizados" + re-ejecución automática del ejemplo activo si cambió en disco | Cambiar un ejemplo en disco → banner + re-run del ejemplo activo sin interacción; SSE se reconecta solo si se cae; fallback a polling 2s si SSE no disponible |

### F9 — Nativo + CI/CD

| Nivel | Fase | Descripción | Criterios de aceptación |
|---|---|---|---|
| L1 | F9.1 | **Descargar `.nvc`** desde el browser: runtime expone `compile_to_bytes(source) -> Uint8Array` → Blob descargable + **CI wasm** en GitHub Actions (build wasm-pack + smoke test) | El `.nvc` descargado ejecuta con `lumen run` y produce el mismo output; job `wasm` verde en push (build + test pkg); CI reporta tamaño del wasm |
| L2 | F9.2 | **GitHub Pages**: deploy automático de `web/` + `pkg/` → playground online | El playground es accesible en la URL Pages; el selector usa el fallback JSON embebido (sin `/api`); build del pkg y deploy en el mismo workflow |
| L3 | F9.3 | `/api/compile-native` con **AOT→Rust (Etapa 3)**: el servidor genera `.rs` autocontenido y compila con rustc → devuelve el exe; botón "⚡ Nativo" con estado del servidor | Tras Etapa 3: POST devuelve exe funcional (fib nativo corre); sin rustc en el servidor → 503 con mensaje claro; el botón muestra disponibilidad real de `/api` |

---

## 4. Detalles técnicos clave

### Loader virtual (F3)
```rust
// lumen-sema/loader.rs
pub struct ModuleLoader {
    files: Vec<PathBuf>,                    // search paths (existente)
    memory_files: HashMap<String, String>,  // NUEVO: stdlib embebida
}
impl ModuleLoader {
    pub fn with_memory_files(mem: HashMap<String, String>) -> Self { ... }
    // resolve_path: si existe en memoria (ruta normalizada) → memoria;
    // si no → disco (comportamiento actual, intacto).
}
```
- `build.rs` de lumen-wasm genera `embedded_stdlib.rs` con `pub const STDLIB_FILES: &[(&str, &str)]` (nombre + contenido). Regenerar el wasm incluye la stdlib vigente automáticamente.
- El prefijado de imports existente (`texto___str_*`) y `is_builtin` no cambian; solo cambia la fuente de lectura.

### Gramática CodeMirror (F2)
- Keywords extraídas del lexer real (`crates/lumen-lexer/src/lexer.rs`, mapa `kw`) vía script `gen-lumen-mode.js` → `web/vendor/lumen-mode.js`. No duplicar la lista a mano.

### Debugger (F6)
- El VM ya soporta `vm.debug`, `vm.step()`, `vm.set_breakpoint()`, `vm.stack_top()`. v1: breakpoints sobre la vista **Disasm** (exactos por ip). v2: mapa aproximado ip→línea fuente (documentado como aproximado).

### Blockly (F7)
- Vendor local sin CDN: `blockly_compressed.js`, `blocks_compressed.js`, `es.js`. Generador = pipeline propio bloque→LÚMEN textual, independiente del parser del lenguaje.

### Hot reload (F8)
- v1: polling mtime (sin dep `notify`). v2: SSE con fallback polling.

### Nativo (F9)
- `/api/compile-native` depende de la **Etapa 3 AOT→Rust**. Hasta entonces: descarga `.nvc` (funcional) + `aot:false` en `/api/meta`.

## 5. Riesgos y mitigaciones

| Riesgo | Mitigación |
|---|---|
| Tamaño del wasm (~1.3MB + stdlib ~200KB) | gzip del `.wasm` en build-wasm.ps1; reportar tamaño en CI (F9.1) |
| COOP/COEP puede afectar `__js_eval` en algunos browsers | Verificar Chrome/Firefox; `__js_call` por bridge no usa eval cuando el bridge existe |
| Loader virtual vs prefijado de imports | Reusar el mismo `resolve_imports` + `is_builtin`; solo cambia la fuente de lectura (memoria vs disco) |
| Blockly aumenta el estado de la UI | Vista toggle separada; el estado de bloques solo vive en esa vista |
| `/api/*` solo existe con servidor | Fallback JSON embebido (F4.1) — el playground funciona incluso en `file://` |
| Hot reload con polling vs watcher real | v1: polling mtime (sin dep). v2: SSE con fallback polling |
| Mapa ip→línea fuente impreciso (F6.3) | Documentado como "aproximado" en la UI; breakpoints exactos sobre Disasm |

## 6. Orden de ejecución

```
Ronda L1: F1.1 → F2.1 → F3.1 → F3.2 → F4.1 → F9.1  (playground usable: servidor, editor, stdlib, ejemplos, CI)
Ronda L2: F1.2 → F2.2 → F3.3 → F4.2 → F5.1 → F5.2 → F6.1 → F6.2 → F8.1 → F8.2 → F9.2
Ronda L3: F1.3 → F2.3 → F4.3 → F5.3 → F6.3 → F7.1 → F7.2 → F7.3 → F8.3 → F9.3
```
(Las fases dentro de cada ronda son independientes y pueden ejecutarse en el orden que convenga a cada sesión.)

## 7. Qué NO entra en v1

- Edición colaborativa en tiempo real (multi-usuario).
- Backend con cuentas/guardado en nube (Alt 3 — evaluable después de F9.3).
- Compilación WASI `cargo` en el browser (el VM ya es la opción real).
- Sync bidireccional código↔bloques (v1: solo bloques→código, documentado).
