# Plan v3.1 — Mejoras Propuestas (21 Ago 2026) — Post-Producción v3.1.4

**Estado base:** v3.1.4 Producción Real publicado (21 Ago 2026) — **917 tests** (616 e2e + 9 production, 673 vm tests), **bench 8** (`cargo bench -p lumen-bench`), **headless `es_headless()`** centralizado (`stdlib/graficos.nv`, `LUMEN_HEADLESS`/`CI`), **CHUNK_VERSION 7** con `FuncMeta.defaults` persistidos, **fixes escalables** `last_significant()` + `label_counter` global, CI `headless-check` (`LUMEN_HEADLESS=1 CI=1`). Ver [docs/produccion.md](produccion.md). Antes: 167 bugs v3.1.4 (720/393/372), aarch64 fix.

Este plan prioriza **evidencia sobre promesas**. Cada ítem tiene criterio de aceptación medible. Parte del checklist `docs/produccion.md`.

---

## P0 — Producción Real v3.1.4 ✅ COMPLETADO (21 Ago 2026)

**Entregado:**
- `builder last_significant()` + `label_counter` global (fallthrough `Variable 'a'/'n'` — `matematicas.nv` `Variable 'n'`)
- `vm FuncMeta.defaults` persistidos `CHUNK_VERSION 7` (`Int/Float/Str/Bool`) + `bind_args` unificado (`Call`/`CallValue`/`run_function` con defaults reales)
- `stdlib/graficos.nv:es_headless()` centralizado (`getenv CI/LUMEN_HEADLESS` vía `__ffi`, `peek!=0`)
- Bench 8 (`cargo bench -p lumen-bench` — 4 prod: fallthrough, defaults, matematicas, headless) + reporte `target/criterion/report/index.html`
- Tests 616 e2e (4 regresión) + 9 production = 673 vm tests, 917 workspace (`cargo test --workspace`)
- CI `headless-check` job Linux `env: LUMEN_HEADLESS=1 CI=1` con `cargo test`, `lumen check examples`, `cargo test --test production`, `cargo bench -- --quick`
- Docs `docs/produccion.md` con checklist único; `VERSION` 3.1.4, decode v6+7

**Comandos verificación:**
```bash
cargo test --workspace
cargo bench -p lumen-bench
LUMEN_HEADLESS=1 CI=1 cargo test --workspace
LUMEN_HEADLESS=1 CI=1 cargo run --bin lumen -- check examples
```

---

## P1 — Estabilidad de Demos (hecho v3.1.4, pendiente de fix profundo)

**Hecho 21 Ago:**
- `charts_demo`, `graficos_avanzado_demo`, `tui_temas_demo` ahora detectan `CI=1` / `LUMEN_HEADLESS=1` y salen con `Headless/CI detectado — demo omitida` (0) en lugar de `Variable 'a/radius' no definida` o `AV 0xC0000005`.
- `lumen check examples` sigue **389/389**; `lumen run` con `CI=1` ahora **no falla** en esas 3 demos.

**Pendiente fix profundo (VM loader):**
- **Síntoma:** `Variable 'a' / 'alpha' / 'radius' no definida` al llamar funciones de `graficos_canvas.nv` con param `a` (alpha) desde otro módulo (`graficos_charts`). Repro: `repro_b.nv` (init SDL + grafico_barras) falla incluso tras `cargo build --release`.
- **Hipótesis:** `loader.rs` prefija incorrectamente params/locals single-letter o confunde `Call @84 Nop @4` (arity 4 vs 3). Disasm muestra `Call @84 Nop @4` para call de 3 args — desalineación de aridad/store.
- **Próximo paso:** Añadir test `test_cross_module_alpha_param` en `crates/lumen-vm/tests/e2e.rs` que importe `graficos_canvas` y llame `limpiar_pantalla` vía otro módulo; instrumentar `vm.rs:5003` para logear `locals` keys y `store` de params. Fix: corregir `collect_module_declarations` para no prefijar params/locals; añadir test `infratests` de loader.

---

## P2 — CI/CD y Benchmarks (completado v3.1.4 + próximo)

**Hecho v3.1.4:**
- `reports/BENCHMARK.md` creado (VM 856ms / C 22ms / Cranelift 5.6ms; aot 38/38; self-hosting fixpoint 5s).
- `gui_ffi.rs` fix verificado en `cargo build --release` (dev 7s, release 62s) y `cargo clippy -D warnings` 0.
- **Nuevo:** `crates/lumen-bench` 8 benches (4 prod nuevos) — `cargo bench -p lumen-bench` + CI `headless-check` con `LUMEN_HEADLESS=1 CI=1` + `--quick` (ver `docs/produccion.md` §2.2 y §3).

**Propuesto v3.1 (1 día):**
- [ ] CI: job `fuzz-smoke` nightly (5min, `cargo fuzz run` con 4 corpora) `continue-on-error: true`, artefacto corpus. No bloquea release.
- [ ] CI: publicar `BENCHMARK.md` + `target/criterion/report/index.html` como artefacto en cada tag (upload-artifact) y validar regresión >10%.
- [x] CI: `lumen check examples` + `CI=1 lumen run` de demos como gate — **HECHO** en `headless-check` con `LUMEN_HEADLESS=1`.

---

## P3 — DX y Roadmap (próximo sprint, 1–2 semanas)

### DX de alto impacto / bajo coste

1. **Version honesty:** `Cargo.toml` ya es 3.1.4, pero `lumen --version` se lee de `CARGO_PKG_VERSION`. Añadir `VERSION` como source of truth en `build.rs` (include_str! + `env::set_var`) para que `lumen --version` y `VERSION` nunca diverjan.
2. **Sugerencias `did you mean`:** ya existe `suggest_examples` para `lumen run typo` — extender a `lumen check typo` y a `importar "typo.nv"` (levenshtein en loader).
3. **Repro unificado:** `scripts/repro.ps1 <file.nv>` que haga `check + run + run CI=1` y volcado de `target/pre-vuelo-report.json` para triage rápido.

### Playground L2/L3 (según `plan-playground.md`)

- F3.3 Compartir (gist-like, URL con código base64)
- F5.1 Time-travel debugger en web (replay de `vm.output()` steps)
- F5.2 Tests en playground (ejecutar `testing.nv` asserts y mostrar badge)
- F8.1 Exportar `.nvc` descargable (ya hecho parcialmente)

Priorizar **F5.2** (tests) — convierte playground en herramienta de aprendizaje, no solo demo.

### AI/ML Fases 186-200 (según `roadmap.md`)

No empezar por tensores. Secuencia recomendada:

1. **Fase 186–188 (stdlib base):** `tensor.nv` con `Tensor{data, shape}` + ops `add/mul/matmul` sin autograd — validar con 5 tests e2e y bench vs `matriz_simd.nv`.
2. **Fase 189–191 (autograd):** `autograd.nv` con `Variable{data, grad, backward}` y `AdamW` — portar `demo_autograd` existente y fuzz.
3. **Fase 192–194 (GGUF):** `gguf.nv` loader Q4_K_M para `phi-3-mini` — integrar con `scheduler.nv` M:N ya existente.

Cada fase **requiere** 2 ejemplos + `cargo test -p lumen-vm` + `lumen check` + `BENCHMARK.md` antes de avanzar. No aceptar "feature completa" sin `cargo test` verde.

---

## Cómo medir v3.1.4 DONE ✅

- `cargo test --workspace` **917** (616 e2e + 9 production) 0 FAILED — **HECHO** (era 414→917)
- `cargo bench -p lumen-bench` **8 benches** (4 prod) — **HECHO**
- `cargo test --test production` **9 production** — **HECHO**
- `CHUNK_VERSION` **7** (decode v6+7) con `FuncMeta.defaults` — **HECHO**
- `stdlib/graficos.nv:es_headless()` + `LUMEN_HEADLESS=1` — **HECHO**

## Cómo medir v3.1 DONE (siguiente)

- `cargo test --workspace` 917 → **≥925** (nuevos tests de loader alpha + tui headless profundos)
- `lumen check examples` 389 → **389** (sin regresión)
- `LUMEN_HEADLESS=1 CI=1 lumen run examples/charts_demo.nv` → **0** con mensaje headless (ya hecho, ahora centralizado)
- `cargo bench -p lumen-bench` 8 → **8** con regresión <10% gate en CI
- `cargo build --release -p lumen-cli` **<65s** (sin regresión)
- `reports/BENCHMARK.md` actualizado en cada tag con `target/criterion` artifact

---

*LÚMEN v3.1 — foco en estabilidad (VM loader) y DX medible, antes de escalar a AI.*

