#!/usr/bin/env python3
"""
LÚMEN CI Gate — 0 crashes, 0 fallos no permitidos, lista explícita de omitidos.

Uso:
  python scripts/ci_gate.py --lumen ./target/debug/lumen --stdlib stdlib --examples examples
  LUMEN_HEADLESS=1 CI=1 python scripts/ci_gate.py --lumen ./target/release/lumen ...

Criterios de producción (evaluación v3.1.4):
- 389 ejemplos deben ser 389 válidos en `lumen check`
- Ejecución masiva con LUMEN_HEADLESS=1 CI=1 timeout 8s
  - PASS: exit 0
  - FAIL: exit 1 con error funcional
  - TIMEOUT: >8s
  - CRASH: exit 134 (abort), 139 (segfault), 132, etc. o señal
- Gate: 0 CRASH, 0 FAIL no permitido, TIMEOUT solo si el archivo tiene // @interactive
- FAIL solo permitido si el archivo tiene // @expected_failure
- Lista explícita de omitidos se genera en el reporte.

Solo usa el binario distribuido (lumen) y stdlib/examples del paquete, no el árbol de fuentes del compilador.
"""
import argparse
import pathlib
import subprocess
import os
import sys
import concurrent.futures
import time

EXPECTED_FAILURE_TAG = "@expected_failure"
INTERACTIVE_TAG = "@interactive"

# Los 3 P0 crashes que deben estar corregidos — si alguno crashea, es bloqueante
P0_CRASH_FILES = {"tui_puro.nv", "tui_temas_demo.nv", "tui_jr.nv"}

def check_file_tags(path):
    text = pathlib.Path(path).read_text(encoding="utf-8", errors="ignore")
    has_expected = EXPECTED_FAILURE_TAG in text
    has_interactive = INTERACTIVE_TAG in text
    return has_expected, has_interactive

def run_one(lumen_bin, stdlib_dir, example_path, timeout=8):
    env = os.environ.copy()
    env["LUMEN_HEADLESS"] = "1"
    env["CI"] = "1"
    # Usar -L stdlib -L stdlib/compiler como en la evaluación
    cmd = [str(lumen_bin), "run", "-L", str(stdlib_dir), "-L", str(stdlib_dir / "compiler"), str(example_path)]
    start = time.time()
    try:
        result = subprocess.run(
            cmd,
            capture_output=True,
            text=False,
            timeout=timeout,
            env=env,
        )
        elapsed = time.time() - start
        out = (result.stdout or b"").decode(errors="ignore") + (result.stderr or b"").decode(errors="ignore")
        # Detectar crash por código de salida
        if result.returncode in (134, 139, 132, -6, -11):  # abort, segfault, SIGABRT, SIGSEGV
            return "CRASH", result.returncode, out, elapsed
        if result.returncode == 0:
            return "PASS", 0, out, elapsed
        else:
            # FAIL funcional
            return "FAIL", result.returncode, out, elapsed
    except subprocess.TimeoutExpired as e:
        elapsed = time.time() - start
        out = ""
        if e.stdout:
            out += e.stdout.decode(errors="ignore") if isinstance(e.stdout, bytes) else str(e.stdout)
        if e.stderr:
            out += e.stderr.decode(errors="ignore") if isinstance(e.stderr, bytes) else str(e.stderr)
        return "TIMEOUT", 124, out, elapsed

def main():
    # Fix Windows console encoding for emoji/ANSI
    try:
        sys.stdout.reconfigure(encoding="utf-8", errors="ignore")
        sys.stderr.reconfigure(encoding="utf-8", errors="ignore")
    except Exception:
        pass
    parser = argparse.ArgumentParser(description="LÚMEN CI Gate")
    parser.add_argument("--lumen", required=True, help="Path to lumen binary (packaged artifact)")
    parser.add_argument("--stdlib", required=True, help="Path to stdlib dir (packaged)")
    parser.add_argument("--examples", required=True, help="Path to examples dir (packaged)")
    parser.add_argument("--timeout", type=int, default=8, help="Timeout per example in seconds")
    parser.add_argument("--workers", type=int, default=8, help="Parallel workers")
    args = parser.parse_args()

    lumen_bin = pathlib.Path(args.lumen)
    stdlib_dir = pathlib.Path(args.stdlib)
    examples_dir = pathlib.Path(args.examples)

    if not lumen_bin.exists():
        print(f"ERROR: lumen binary not found: {lumen_bin}", file=sys.stderr)
        sys.exit(2)
    if not stdlib_dir.exists():
        print(f"ERROR: stdlib not found: {stdlib_dir}", file=sys.stderr)
        sys.exit(2)
    if not examples_dir.exists():
        print(f"ERROR: examples not found: {examples_dir}", file=sys.stderr)
        sys.exit(2)

    # 1. Check 389 válidos
    print("=== LUMEN CI GATE ===")
    print(f"lumen: {lumen_bin}")
    print(f"stdlib: {stdlib_dir}")
    print(f"examples: {examples_dir}")
    print(f"timeout: {args.timeout}s, workers: {args.workers}")
    print()

    # Check
    print("--- lumen check ---")
    check_cmd = [str(lumen_bin), "check", str(examples_dir)]
    result = subprocess.run(check_cmd, capture_output=True, text=False)
    out_check = (result.stdout or b"").decode(errors="ignore") + (result.stderr or b"").decode(errors="ignore")
    print(out_check)
    if result.returncode != 0:
        print("FAIL: lumen check failed", file=sys.stderr)
        sys.exit(1)
    # Parse "389 válidos" from output
    if "389" not in out_check:
        print("WARN: check output doesn't mention 389", file=sys.stderr)

    # 2. Run 389 examples
    print("\n--- lumen run 389 examples (LUMEN_HEADLESS=1 CI=1) ---")
    examples = sorted(examples_dir.rglob("*.nv"))
    print(f"Found {len(examples)} examples")
    if len(examples) != 389:
        print(f"WARN: expected 389, found {len(examples)}", file=sys.stderr)

    results = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=args.workers) as executor:
        future_to_path = {
            executor.submit(run_one, lumen_bin, stdlib_dir, p, args.timeout): p
            for p in examples
        }
        for future in concurrent.futures.as_completed(future_to_path):
            path = future_to_path[future]
            try:
                status, code, out, elapsed = future.result()
            except Exception as e:
                status, code, out, elapsed = "CRASH", -1, str(e), 0
            results[path] = (status, code, out, elapsed)
            rel = path.relative_to(examples_dir)
            print(f"{status:7} {rel} ({elapsed:.2f}s)")

    # 3. Clasificar
    counts = {"PASS": 0, "FAIL": 0, "TIMEOUT": 0, "CRASH": 0}
    allowed_fail = []
    allowed_timeout = []
    not_allowed_fail = []
    not_allowed_timeout = []
    crashes = []

    for path, (status, code, out, elapsed) in results.items():
        counts[status] += 1
        has_expected, has_interactive = check_file_tags(path)
        rel = str(path.relative_to(examples_dir))
        if status == "CRASH":
            crashes.append((rel, code, out))
        elif status == "FAIL":
            if has_expected:
                allowed_fail.append(rel)
            else:
                not_allowed_fail.append((rel, out))
        elif status == "TIMEOUT":
            if has_interactive:
                allowed_timeout.append(rel)
            else:
                not_allowed_timeout.append(rel)
        # P0 check: if any of the 3 known crash files still crashes, it's block
        if path.name in P0_CRASH_FILES and status == "CRASH":
            print(f"P0 BLOCKER: {rel} still crashes (exit {code})", file=sys.stderr)

    print("\n=== RESULTADO EJECUTIVO ===")
    print(f"PASS: {counts['PASS']}, FAIL: {counts['FAIL']}, TIMEOUT: {counts['TIMEOUT']}, CRASH: {counts['CRASH']}")
    print(f"Total: {sum(counts.values())} (expected 389)")
    print()
    print(f"Allowed FAIL (expected_failure): {len(allowed_fail)}")
    for f in sorted(allowed_fail):
        print(f"  - {f}")
    print()
    print(f"Allowed TIMEOUT (interactive): {len(allowed_timeout)}")
    for f in sorted(allowed_timeout):
        print(f"  - {f}")
    print()

    if crashes:
        print("=== BLOQUEADORES P0 — CRASHES ===", file=sys.stderr)
        for rel, code, out in crashes:
            print(f"  {rel} exit {code}", file=sys.stderr)
            print(out[:500], file=sys.stderr)
        print("Gate: 0 crashes — FAILED", file=sys.stderr)
        sys.exit(1)

    if not_allowed_fail:
        print("=== FALLOS NO PERMITIDOS (sin @expected_failure) ===", file=sys.stderr)
        for rel, out in not_allowed_fail:
            print(f"  {rel}", file=sys.stderr)
            print(out[:300], file=sys.stderr)
        print(f"Gate: 0 fallos no permitidos — FAILED ({len(not_allowed_fail)} encontrados)", file=sys.stderr)
        sys.exit(1)

    if not_allowed_timeout:
        print("=== TIMEOUTS NO PERMITIDOS (sin @interactive) ===", file=sys.stderr)
        for rel in not_allowed_timeout:
            print(f"  {rel}", file=sys.stderr)
        print(f"Gate: TIMEOUT solo permitido con @interactive — FAILED ({len(not_allowed_timeout)} encontrados)", file=sys.stderr)
        sys.exit(1)

    # Lista explícita de omitidos
    print("=== LISTA EXPLÍCITA DE OMITIDOS (permitidos) ===")
    print(f"Expected failures ({len(allowed_fail)}): {', '.join(sorted(allowed_fail))}")
    print(f"Interactive timeouts ({len(allowed_timeout)}): {', '.join(sorted(allowed_timeout))}")
    print()
    print("Gate: 0 crashes, 0 fallos no permitidos — PASSED")
    print(f"PASS {counts['PASS']}/389 ({counts['PASS']/389*100:.2f}%) — {'APTO' if counts['PASS'] >= 389 - len(allowed_fail) - len(allowed_timeout) else 'NO APTO'} para producción si gate pasó")

if __name__ == "__main__":
    main()
