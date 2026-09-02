#!/usr/bin/env python3
"""v3.94.24 — Benchmark de TODAS las áreas del lenguaje LÚMEN.

Áreas cubiertas: recursión (fib), bucles (sum), primos (primes), strings
(strings), arrays (arrays), structs+métodos (structs), enums+elegir (enums),
mapas (maps), closures/lambdas (closures) y strings unicode (unicode).

Compara 3 implementaciones por tarea:
  lumen-vm      : intérprete VM (target/debug/lumen run)
  lumen-aotc    : compilación nativa C -O3 (lumen build --native)
  c             : referencia C -O3 (bench_all.c)

Uso:
  python3 benchmarks/run_bench_all.py [--lumen target/debug/lumen]
Salida: benchmarks/results/benchmark_all.csv + informe_all.md
"""
import argparse
import os
import pathlib
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parent.parent
RES = ROOT / "benchmarks" / "results"
LUMEN_DIR = ROOT / "benchmarks" / "lumen"

TASKS = ["fib", "sum", "primes", "strings", "arrays", "structs", "enums", "maps", "closures", "unicode"]
EXPECTED = {
    "fib": "fib:121393",
    "sum": "sum:49999995000000",
    "primes": "primes:2262",
    "strings": "strings:2888890",
    "arrays": "arrays:19999900000",
    "structs": "structs:40000000000",
    "enums": "enums:10000000000",
    "maps": "maps:999000",
    "closures": "closures:39999800000",
    "unicode": "unicode:3800000",
}


def measure(cmd, cwd=ROOT, runs=3):
    """Devuelve (mejor_segundos, salida). Repite `runs` veces y toma el mínimo."""
    best = float("inf")
    out = ""
    for _ in range(runs):
        t0 = time.perf_counter()
        p = subprocess.run(cmd, cwd=str(cwd), stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        dt = time.perf_counter() - t0
        out = (p.stdout or b"").decode(errors="replace").strip()
        if p.returncode != 0:
            return None, f"(exit {p.returncode})"
        best = min(best, dt)
    return best, out


def build_aotc(lumen, task):
    src = LUMEN_DIR / f"{task}.nv"
    r = subprocess.run([str(lumen), "build", "--native", str(src)],
                       capture_output=True, text=True)
    exe = LUMEN_DIR / task
    if r.returncode == 0 and exe.exists():
        return exe
    return None


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lumen", default=str(ROOT / "target" / "debug" / "lumen"))
    ap.add_argument("--tasks", default=None, help="coma-separadas; por defecto todas")
    args = ap.parse_args()

    lumen = pathlib.Path(args.lumen)
    if not lumen.exists():
        sys.exit(f"Falta {lumen} — compila con cargo build -p lumen-cli")

    tasks = TASKS if not args.tasks else args.tasks.split(",")

    # C de referencia
    bench_c = RES / "bench_all_c"
    cc = subprocess.run(["gcc", "-O3", str(ROOT / "benchmarks" / "c" / "bench_all.c"),
                         "-o", str(bench_c), "-lm"], capture_output=True, text=True)
    have_c = cc.returncode == 0 and bench_c.exists()

    RES.mkdir(parents=True, exist_ok=True)
    rows = []

    def run_task(task, impl, cmd):
        dt, out = measure(cmd)
        ok = dt is not None and EXPECTED.get(task, "") in (out or "")
        state = "ok" if ok else ("fail" if dt is None else "mismatch")
        rows.append((task, impl, dt, state, out if not ok else ""))
        dt_s = f"{dt:.3f}s" if dt is not None else "FAIL"
        print(f"  {impl:12s} {task:9s} {dt_s:>8s}  {state.upper()}")

    print(f"Benchmark de TODAS las áreas (VM vs AOT-C vs C) — {len(tasks)} tareas\n")
    for t in tasks:
        print(f"— {t}")
        run_task(t, "lumen-vm", [str(lumen), "run", str(LUMEN_DIR / f"{t}.nv")])
        exe = build_aotc(lumen, t)
        if exe:
            run_task(t, "lumen-aotc", [str(exe)])
        else:
            rows.append((t, "lumen-aotc", None, "buildfail", ""))
            print(f"  {'lumen-aotc':12s} {t:9s} {'FAIL':>8s}  BUILDFAIL")
        if have_c:
            run_task(t, "c", [str(bench_c), t])

    # CSV
    with open(RES / "benchmark_all.csv", "w") as f:
        f.write("tarea,implementacion,segundos,estado\n")
        for task, impl, dt, state, _ in rows:
            f.write(f"{task},{impl},{dt if dt is not None else ''},{state}\n")

    # Markdown
    impl_order = ["lumen-vm", "lumen-aotc", "c"]
    md = ["# Benchmark LÚMEN — todas las áreas del lenguaje (v3.94.24)\n",
          "Mejor de 3 ejecuciones (segundos de pared). Mismo algoritmo en cada lenguaje.\n",
          "| Tarea | Área | lumen-vm | lumen-aotc | C (-O3) |",
          "|---|:---|---:|---:|---:|"]
    by = {(t, i): (dt, st) for t, i, dt, st, _ in rows}
    areas = {
        "fib": "recursión", "sum": "bucles", "primes": "primos",
        "strings": "strings", "arrays": "arrays", "structs": "structs",
        "enums": "enums", "maps": "mapas", "closures": "closures",
        "unicode": "unicode",
    }
    for t in tasks:
        line = [f"**{t}**", areas.get(t, "")]
        for impl in impl_order:
            if (t, impl) in by:
                dt, st = by[(t, impl)]
                if dt is None:
                    line.append("FAIL")
                elif st != "ok":
                    line.append(f"⚠ {dt:.3f}s")
                else:
                    line.append(f"{dt:.3f}s")
            else:
                line.append("—")
        md.append("| " + " | ".join(line) + " |")
    md.append("")
    # Notas
    md.append("## Notas")
    md.append("- `maps` usa 1 000 claves: el backend C de mapas es de valor (O(n) por inserción, "
              "sin recolección), por lo que tamaños grandes solo son viables en la VM "
              "(mapa persistente ImMap). Ver AUDITORIA_V39424.md, ítem 8.")
    md.append("- `lumen-vm` corre con el binario de depuración; `lumen-aotc` compila a C -O3.")
    md.append("")
    with open(RES / "informe_all.md", "w") as f:
        f.write("\n".join(md))
    print("\nListo: benchmarks/results/benchmark_all.csv + informe_all.md")


if __name__ == "__main__":
    main()
