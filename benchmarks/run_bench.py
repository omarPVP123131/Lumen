#!/usr/bin/env python3
"""v3.5.18 — Harness de benchmark: Lúmen (VM / AOT-C / Cranelift / LLVM) vs
C, C++, Rust y Python. Mide tiempo de pared (s) y RSS pico (MB) por tarea.

Uso:  python3 benchmarks/run_bench.py
Salida: benchmarks/results/benchmark.csv + benchmarks/results/informe.md
"""
import os
import subprocess
import sys
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
LUMEN = os.path.join(ROOT, "target", "release", "lumen")
RES = os.path.join(ROOT, "benchmarks", "results")
TASKS = ["fib", "sum", "primes", "strings", "arrays"]
EXPECTED = {
    "fib": "fib:121393",
    "sum": "sum:49999995000000",
    "primes": "primes:2262",
    "strings": "strings:2888890",
    "arrays": "arrays:19999900000",
}


def run_measured(cmd, cwd=ROOT):
    """Ejecuta cmd; devuelve (segundos, rss_mb, stdout). RSS exacto vía wait4."""
    t0 = time.perf_counter()
    p = subprocess.Popen(
        cmd, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL
    )
    out = p.stdout.read()
    p.stdout.close()
    try:
        _, _, rusage = os.wait4(p.pid, 0)
        rss_mb = rusage.ru_maxrss / 1024.0  # KB → MB (Linux)
    except ChildProcessError:
        p.wait()
        rss_mb = float("nan")
    dt = time.perf_counter() - t0
    return dt, rss_mb, out.decode(errors="replace").strip()


def build_lumen_variants():
    """Compila cada tarea Lúmen con AOT-C y Cranelift; devuelve rutas."""
    variants = {}
    for t in TASKS:
        src = os.path.join(ROOT, "benchmarks", "lumen", f"{t}.nv")
        # AOT-C
        r = subprocess.run(
            [LUMEN, "build", "--c", src], capture_output=True, text=True
        )
        exe_c = os.path.join(ROOT, "benchmarks", "lumen", t)
        if r.returncode == 0 and os.path.exists(exe_c):
            dest = os.path.join(RES, f"exe_{t}_aotc")
            os.replace(exe_c, dest)
            variants.setdefault(t, {})["aotc"] = [dest]
        else:
            print(f"[aviso] AOT-C fallo en {t}: {r.stderr[:200]}")
        # Cranelift
        r = subprocess.run(
            [LUMEN, "build", "--rust", src], capture_output=True, text=True
        )
        if r.returncode == 0 and os.path.exists(exe_c):
            dest = os.path.join(RES, f"exe_{t}_cranelift")
            os.replace(exe_c, dest)
            variants.setdefault(t, {})["cranelift"] = [dest]
        else:
            print(f"[aviso] Cranelift fallo en {t}: {r.stderr[:200]}")
        # LLVM (solo si hay clang)
        r = subprocess.run(
            [LUMEN, "build", "--llvm", src], capture_output=True, text=True
        )
        ll = os.path.join(ROOT, "benchmarks", "lumen", f"{t}.ll")
        exe_ll = os.path.join(ROOT, "benchmarks", "lumen", t)
        if r.returncode == 0 and os.path.exists(exe_ll):
            dest = os.path.join(RES, f"exe_{t}_llvm")
            os.replace(exe_ll, dest)
            variants.setdefault(t, {})["llvm"] = [dest]
        elif os.path.exists(ll):
            print(f"[info] LLVM genero IR de {t} (sin clang no se enlaza aqui)")
    return variants


def build_refs():
    """Compila las referencias C/C++/Rust si faltan o no son ejecutables."""
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    jobs = [
        (os.path.join(RES, "bench_c"), ["gcc", "-O3", os.path.join(root, "benchmarks", "c", "bench.c"), "-o", os.path.join(RES, "bench_c"), "-lm"]),
        (os.path.join(RES, "bench_cpp"), ["g++", "-O3", "-std=c++17", os.path.join(root, "benchmarks", "cpp", "bench.cpp"), "-o", os.path.join(RES, "bench_cpp")]),
        (os.path.join(RES, "bench_rs"), None),  # rustc se resuelve abajo
    ]
    rustc = os.path.expanduser("~/.cargo/bin/rustc")
    if not os.path.exists(rustc):
        rustc = "rustc"
    jobs[2] = (os.path.join(RES, "bench_rs"), [rustc, "-O", os.path.join(root, "benchmarks", "rust", "bench.rs"), "-o", os.path.join(RES, "bench_rs")])
    for out, cmd in jobs:
        if os.path.exists(out) and os.access(out, os.X_OK):
            continue
        try:
            subprocess.run(cmd, check=True, capture_output=True)
            os.chmod(out, 0o755)
        except Exception as e:
            print(f"[aviso] no pude compilar {out}: {e}")


def main():
    os.makedirs(RES, exist_ok=True)
    build_refs()
    if not os.path.exists(LUMEN):
        print("Falta target/release/lumen — ejecuta cargo build --release primero")
        sys.exit(1)
    print("Compilando variantes Lúmen (AOT-C / Cranelift / LLVM)...")
    variants = build_lumen_variants()

    rows = []  # (task, impl, secs, rss_mb, salida_ok)
    impls = []

    def add(task, impl, cmd):
        dt, rss, out = run_measured(cmd)
        ok = EXPECTED.get(task, "") in out
        state = "ok" if ok else ("oom" if out == "" else "mismatch")
        if not ok:
            print(f"[ALERTA] {impl}/{task}: estado={state} salida={out[:120]!r}")
        rows.append((task, impl, dt, rss, state))
        print(f"  {impl:12s} {task:8s} {dt:9.3f}s {rss:9.1f} MB {state.upper()}")

    print("Corriendo benchmarks...")
    for t in TASKS:
        print(f"— tarea: {t}")
        add(t, "lumen-vm", [LUMEN, "run", os.path.join(ROOT, "benchmarks", "lumen", f"{t}.nv")])
        for var in ("aotc", "cranelift", "llvm"):
            if t in variants and var in variants[t]:
                add(t, f"lumen-{var}", variants[t][var])
        add(t, "c", [os.path.join(RES, "bench_c"), t])
        add(t, "cpp", [os.path.join(RES, "bench_cpp"), t])
        add(t, "rust", [os.path.join(RES, "bench_rs"), t])
        add(t, "python", [sys.executable, os.path.join(ROOT, "benchmarks", "python", "bench.py"), t])

    # CSV
    with open(os.path.join(RES, "benchmark.csv"), "w") as f:
        f.write("tarea,implementacion,segundos,rss_mb,estado\n")
        for task, impl, dt, rss, state in rows:
            f.write(f"{task},{impl},{dt:.4f},{rss:.1f},{state}\n")

    # Markdown
    impl_order = ["lumen-vm", "lumen-aotc", "lumen-cranelift", "lumen-llvm", "c", "cpp", "rust", "python"]
    seen = []
    for _, impl, _, _, _ in rows:
        if impl not in seen:
            seen.append(impl)
    impl_order = [i for i in impl_order if i in seen] + [i for i in seen if i not in impl_order]

    md = []
    md.append("# Benchmark LÚMEN vs C / C++ / Rust / Python (v3.5.29)\n")
    md.append("Mismo algoritmo en cada lenguaje. `Tiempo` = segundos de pared (mejor medición única);")
    md.append("`RSS` = memoria pico del proceso en MB.\n")
    md.append("| Tarea | " + " | ".join(impl_order) + " |")
    md.append("|---|" + "---|" * len(impl_order))
    by = {}
    for task, impl, dt, rss, state in rows:
        by[(task, impl)] = (dt, rss, state)
    for t in TASKS:
        line = [f"**{t}**"]
        for impl in impl_order:
            if (t, impl) in by:
                dt, rss, state = by[(t, impl)]
                if state == "ok":
                    line.append(f"{dt:.3f}s / {rss:.0f}MB")
                elif state == "oom":
                    line.append(f"OOM ({dt:.1f}s)")
                else:
                    line.append(f"⚠ {dt:.3f}s")
            else:
                line.append("—")
        md.append("| " + " | ".join(line) + " |")
    md.append("")
    with open(os.path.join(RES, "informe.md"), "w") as f:
        f.write("\n".join(md))
    print("\nListo: benchmarks/results/benchmark.csv + informe.md")


if __name__ == "__main__":
    main()
