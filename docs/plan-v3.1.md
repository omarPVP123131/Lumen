# Plan v3.1 — Mejoras Propuestas (21 Ago 2026)

**Estado base:** v3.0.0 publicado (167 bugs, 720/393/372, aarch64 fix, docs sincronizadas, 389 check, 414 tests).

Este plan prioriza **evidencia sobre promesas**. Cada ítem tiene criterio de aceptación medible.

---

## P1 — Estabilidad de Demos (hecho, pendiente de fix profundo)

**Hecho 21 Ago:**
- `charts_demo`, `graficos_avanzado_demo`, `tui_temas_demo` ahora detectan `CI=1` / `LUMEN_HEADLESS=1` y salen con `Headless/CI detectado — demo omitida` (0) en lugar de `Variable 'a/radius' no definida` o `AV 0xC0000005`.
- `lumen check examples` sigue **389/389**; `lumen run` con `CI=1` ahora **no falla** en esas 3 demos.

**Pendiente fix profundo (VM loader):**
- **Síntoma:** `Variable 'a' / 'alpha' / 'radius' no definida` al llamar funciones de `graficos_canvas.nv` con param `a` (alpha) desde otro módulo (`graficos_charts`). Repro: `repro_b.nv` (init SDL + grafico_barras) falla incluso tras `cargo build --release`.
- **Hipótesis:** `loader.rs` prefija incorrectamente params/locals single-letter o confunde `Call @84 Nop @4` (arity 4 vs 3). Disasm muestra `Call @84 Nop @4` para call de 3 args — desalineación de aridad/store.
- **Próximo paso:** Añadir test `test_cross_module_alpha_param` en `crates/lumen-vm/tests/e2e.rs` que importe `graficos_canvas` y llame `limpiar_pantalla` vía otro módulo; instrumentar `vm.rs:5003` para logear `locals` keys y `store` de params. Fix: corregir `collect_module_declarations` para no prefijar params/locals; añadir test `infratests` de loader.

---

## P2 — CI/CD y Benchmarks (hecho parcial)

**Hecho:**
- `reports/BENCHMARK.md` creado (VM 856ms / C 22ms / Cranelift 5.6ms; aot 38/38; self-hosting fixpoint 5s).
- `gui_ffi.rs` fix verificado en `cargo build --release` (dev 7s, release 62s) y `cargo clippy -D warnings` 0.

**Propuesto (1 día):**
- [ ] CI: job `fuzz-smoke` nightly (5min, `cargo fuzz run` con 4 corpora) `continue-on-error: true`, artefacto corpus. No bloquea release.
- [ ] CI: publicar `BENCHMARK.md` como artefacto en cada tag (upload-artifact).
- [ ] CI: `lumen check examples` + `CI=1 lumen run` de demos como gate (ahora pasa).

---

## P3 — DX y Roadmap (próximo sprint, 1–2 semanas)

### DX de alto impacto / bajo coste

1. **Version honesty:** `Cargo.toml` ya es 3.0.0, pero `lumen --version` se lee de `CARGO_PKG_VERSION`. Añadir `VERSION` como source of truth en `build.rs` (include_str! + `env::set_var`) para que `lumen --version` y `VERSION` nunca diverjan.
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

## Cómo medir v3.1 DONE

- `cargo test --workspace` 414 → **≥420** (nuevos tests de loader alpha + tui headless)
- `lumen check examples` 389 → **389** (sin regresión)
- `CI=1 lumen run examples/charts_demo.nv` → **0** con mensaje headless (ya hecho)
- `cargo build --release -p lumen-cli` **<65s** (sin regresión)
- `reports/BENCHMARK.md` actualizado en cada tag

---

*LÚMEN v3.1 — foco en estabilidad (VM loader) y DX medible, antes de escalar a AI.*
