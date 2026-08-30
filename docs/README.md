# 📚 Documentación de LÚMEN v3.5.7

> Índice central de la documentación. **Estado real del proyecto** (29/30 ago 2026):
> 956 tests en verde (JIT ON y OFF), pre-commit completo en verde
> (fmt + clippy `-D warnings` + tests + `lumen check examples` 396/396),
> ci_gate 392/389 PASS ×2, fixpoint self-hosting byte-idéntico, y las
> rondas de rendimiento v3.5.31 → v3.5.37 (TOTAL de benchmarks 590 ms →
> **267 ms** con JIT, fib 4.4 ms ≈ 2× el C).

## Estructura

```
docs/
├── README.md              ← este índice
├── MARKETING.md           — material promocional del proyecto
├── guias/                 — guías de usuario y formación
│   ├── GUIA_RAPIDA_UX.md          — guía rápida de experiencia de uso
│   ├── CURRICULUM_7_DIAS.md       — plan de estudio de 7 días
│   └── LIBRO_OFICIAL_LUMEN.md     — el libro oficial del lenguaje
├── referencia/            — referencias técnicas del lenguaje y herramientas
│   ├── LENGUAJE.md                — referencia del lenguaje (ES)
│   ├── language.md                — referencia del lenguaje (EN)
│   ├── ESPECIFICACION_FORMAL_LUMEN.md — especificación formal completa
│   ├── HERRAMIENTAS.md            — catálogo de herramientas (REPL, LSP, debugger)
│   └── cli.md                     — referencia completa de comandos CLI
├── arquitectura/          — cómo está construido por dentro
│   ├── architecture.md            — pipeline del compilador (lexer → … → VM)
│   └── jit.md                     — arquitectura real del JIT (Tier-1/Tier-2/Tier-R)
├── desarrollo/            — planes, procesos y estado de desarrollo
│   ├── AGENTS.md                  — convenciones para agentes de IA
│   ├── contributing.md            — guía de contribución
│   ├── roadmap.md                 — roadmap oficial de fases
│   ├── siguiente.md               — siguientes pasos
│   ├── produccion.md              — checklist único de producción
│   ├── self-hosting.md            — historia del bootstrap (compiler_v4 en LÚMEN)
│   ├── plan-playground.md / plan-v3.1.md — planes históricos
│   └── progress-2026-08-01.md     — bitácora de progreso
├── spec/                  — especificaciones de bajo nivel
│   ├── bytecode-format.md         — formato .nvc byte a byte (incl. opcodes fusionados)
│   ├── error-codes.md             — catálogo de códigos de error
│   ├── vm-spec.md                 — especificación del VM
│   └── grammar.ebnf                — gramática EBNF
└── informes/              — informes y reportes históricos
    ├── AUDIT_REPORT.md            — auditoría del repo
    ├── BENCHMARK.md               — reporte de benchmarks (VM/JIT vs C/Rust/Python)
    ├── EXECUTIVE_SUMMARY.md       — resumen ejecutivo
    ├── LUMEN_REPORT.md            — reporte integral del proyecto
    ├── TEST_REPORT.md             — reporte de tests y verificación
    ├── fixpoint_status.md         — estado del fixpoint self-hosting (autogenerado)
    └── perf_v3530_cranelift.md    — análisis de rendimiento AOT Cranelift
```

Otros documentos en la raíz:

- [`README.md`](../README.md) — cara pública del proyecto (resumen, inicio rápido, benchmarks).
- [`CHANGELOG.md`](../CHANGELOG.md) — historial completo de versiones (arriba: rondas de rendimiento v3.5.31 → v3.5.37).
- [`info.md`](../info.md) — compendio técnico integral (enciclopedia maestra).

## Estado real — un vistazo

| Área | Estado |
|---|---|
| Compilación | `cargo build --release` 0 errores, 0 warnings |
| Linter | `cargo clippy --all -- -D warnings` — 0 warnings |
| Formato | `cargo fmt -- --check` — limpio |
| Tests | 956/956 con JIT ON y 956/956 con `LUMEN_JIT=0` |
| Ejemplos | `lumen check examples` — 396 archivos, 0 errores |
| Gate de CI | `ci_gate.py` 392 PASS / 0 crashes ×2 (JIT ON/OFF) |
| Self-hosting | fixpoint byte-idéntico (170985 B, sha256 `02b0460d…`) |
| Rendimiento JIT | TOTAL 267 ms vs 1542 ms intérprete (5.8×); fib 4.4 ms (23×) |
