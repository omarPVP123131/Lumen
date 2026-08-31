#!/usr/bin/env python3
"""ci_bench_suite — Suite variada de benchmarks (v3.5.40+).

Corre las 4 tareas aditivas (sort/matmul/sieve/dict) en los 5 lenguajes:
C, C++, Rust, Python y Lúmen (VM con JIT), y VERIFICA cada salida contra
el checksum exacto calculado de antemano con implementaciones de
referencia. Es la suite que usa la CI multiplataforma (Windows/macOS/
Linux) — los binarios se compilan en un directorio temporal, nunca en
benchmarks/ (no contamina el árbol).

Uso:  python3 scripts/ci_bench_suite.py [--quick]

Salida: tabla "tarea | lenguaje | tiempo | checksum" y resumen.
Exit 0 si TODOS los checksums coinciden en TODOS los lenguajes presentes;
si un toolchain falta (p. ej. gcc en Windows), la tarea se omite con aviso
y NO falla la suite.
"""
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
QUICK = "--quick" in sys.argv

GOLDEN = {
    "sort": "sort:333333333300000",
    "matmul": "matmul:520167",
    "sieve": "sieve:78498",
    "dict": "dict:449985000",
}
TASKS = ["sort", "matmul", "sieve", "dict"]


def _lumen_bin():
    for cand in (
        os.path.join(ROOT, "target", "release", "lumen"),
        os.path.join(ROOT, "target", "release", "lumen.exe"),
        os.path.join(ROOT, "target", "debug", "lumen"),
        os.path.join(ROOT, "target", "debug", "lumen.exe"),
    ):
        if os.path.exists(cand):
            return cand
    return None


def _run(cmd, timeout=180):
    t0 = time.perf_counter()
    r = subprocess.run(cmd, capture_output=True, text=True, timeout=timeout)
    return r, (time.perf_counter() - t0) * 1000.0


def _build_c(tmp, src, cc, extra=()):
    out = os.path.join(tmp, "suite_c" + (".exe" if os.name == "nt" else ""))
    r = subprocess.run([cc, "-O2", *extra, src, "-o", out],
                       capture_output=True, text=True)
    return out if r.returncode == 0 else None


def _build_rust(tmp, src):
    out = os.path.join(tmp, "suite_rs" + (".exe" if os.name == "nt" else ""))
    r = subprocess.run(["rustc", "-O", src, "-o", out],
                       capture_output=True, text=True)
    return out if r.returncode == 0 else None


def _find_cc(candidates):
    for cc in candidates:
        if shutil.which(cc):
            return cc
    return None


def main():
    lumen = _lumen_bin()
    results = []  # (tarea, lenguaje, ms, checksum_ok, salida)
    tmp = tempfile.mkdtemp(prefix="lumen_suite_")

    # ── Lúmen (VM, JIT ON) ──
    lumen_ok = lumen is not None
    if not lumen_ok:
        print("[aviso] no se encontró target/release/lumen — compila el release primero")

    # ── C ──
    c_bin = None
    cc = _find_cc(["gcc", "cc", "clang"])
    if cc:
        c_bin = _build_c(tmp, os.path.join(ROOT, "benchmarks", "c", "bench_suite.c"), cc)

    # ── C++ ──
    cpp_bin = None
    cxx = _find_cc(["g++", "c++", "clang++"])
    if cxx:
        cpp_bin = _build_c(tmp, os.path.join(ROOT, "benchmarks", "cpp", "bench_suite.cpp"), cxx)

    # ── Rust ──
    rs_bin = None
    if shutil.which("rustc"):
        rs_bin = _build_rust(tmp, os.path.join(ROOT, "benchmarks", "rust", "bench_suite.rs"))

    py = os.path.join(ROOT, "benchmarks", "python", "bench_suite.py")

    n_ok = n_bad = 0
    # Los binarios C/C++/Rust/Python imprimen las 4 líneas de una vez;
    # Lúmen corre UN archivo .nv por tarea.
    monolitos = []
    if c_bin:
        monolitos.append(("c", c_bin))
    if cpp_bin:
        monolitos.append(("cpp", cpp_bin))
    if rs_bin:
        monolitos.append(("rust", rs_bin))
    if shutil.which("python3") or shutil.which("python"):
        pyexe = "python3" if shutil.which("python3") else "python"
        monolitos.append(("python", [pyexe, py]))
    for lang, argv in monolitos:
        try:
            r, ms = _run(argv)
        except subprocess.TimeoutExpired:
            for tarea in TASKS:
                results.append((tarea, lang, -1.0, False, "(timeout)"))
            n_bad += 4
            continue
        lineas = [l.strip() for l in r.stdout.splitlines() if l.strip()]
        for idx, tarea in enumerate(TASKS):
            linea = lineas[idx] if idx < len(lineas) else ""
            ok = linea == GOLDEN[tarea]
            results.append((tarea, lang, ms / 4.0, ok, linea))
            n_ok += ok
            n_bad += (not ok)
    if lumen_ok:
        for tarea in TASKS:
            try:
                r, ms = _run([lumen, "run",
                              os.path.join(ROOT, "benchmarks", "lumen", f"{tarea}.nv")])
            except subprocess.TimeoutExpired:
                results.append((tarea, "lumen", -1.0, False, "(timeout)"))
                n_bad += 1
                continue
            lineas = [l.strip() for l in r.stdout.splitlines() if l.strip()]
            linea = lineas[0] if lineas else ""
            ok = linea == GOLDEN[tarea]
            results.append((tarea, "lumen", ms, ok, linea))
            n_ok += ok
            n_bad += (not ok)

    # ── Tabla ──
    print(f"\n{'tarea':8s} {'lenguaje':8s} {'ms':>10s}  checksum")
    print("-" * 52)
    for tarea, lang, ms, ok, linea in results:
        marca = "✓" if ok else "✗ FALLA"
        msv = f"{ms:8.1f}" if ms >= 0 else "timeout"
        print(f"{tarea:8s} {lang:8s} {msv:>10s}  {marca}  {linea}")
    print("-" * 52)
    print(f"Checksums: {n_ok} OK | {n_bad} FALLAS")
    shutil.rmtree(tmp, ignore_errors=True)
    return 0 if n_bad == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
