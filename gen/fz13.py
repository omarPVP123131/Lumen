#!/usr/bin/env python3
"""fz13: fuzzer diferencial de regex VM <-> binario nativo.

BUG-080 y BUG-166 fueron el mismo fallo: el regex nativo no coincidia con el de
la VM. Se arreglo dos veces porque nadie comparaba las dos implementaciones de
forma sistematica. Este fuzzer genera patrones y sujetos al azar y exige que
`lumen run` y el binario de `lumen build --native` den exactamente lo mismo.
"""
import random, subprocess, sys, os, tempfile

RAIZ = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def _busca_binario():
    """El script puede vivir en `<repo>/gen/` o en `<algo>/gen/` con el repo en
    `<algo>/lumen-src/`. Se admiten ambas disposiciones, y `LUMEN_BIN` manda."""
    env = os.environ.get("LUMEN_BIN")
    if env:
        return env
    exe = "lumen.exe" if os.name == "nt" else "lumen"
    cands = [
        os.path.join(RAIZ, "target", "release", exe),
        os.path.join(RAIZ, "lumen-src", "target", "release", exe),
        os.path.join(os.environ.get("LUMEN_RAIZ", RAIZ), "target", "release", exe),
    ]
    for c in cands:
        if os.path.isfile(c) and os.access(c, os.X_OK):
            return c
    sys.stderr.write("no encuentro el binario de lumen; probado:\n  " +
                     "\n  ".join(cands) + "\nfija LUMEN_BIN si esta en otro sitio\n")
    sys.exit(2)

LUMEN = _busca_binario()

ATOMOS = [r"\d", r"\w", r"\s", r"\D", r"\W", r"\S", ".", "a", "b", "c", "1", "x",
          "[a-z]", "[0-9]", "[^0-9]", "[abc]", "[^ab]", r"\.", "@", "_"]
CUANT = ["", "", "", "*", "+", "?", "{2}", "{1,2}", "{2,}", "{0,3}"]
SUJETOS = ["", "a", "ab", "abc", "123", "a1b2", "hola mundo", "  x", "mail@web",
           "aaa", "AB12", "a.b", "x_y", "42x", "sin numeros", "ababc", "foo"]

def patron(rng):
    n = rng.randint(1, 4)
    p = ""
    for _ in range(n):
        a = rng.choice(ATOMOS)
        if rng.random() < 0.15:
            a = "(" + rng.choice(ATOMOS) + rng.choice(["", "|" + rng.choice(ATOMOS)]) + ")"
        p += a + rng.choice(CUANT)
    if rng.random() < 0.2:
        p = "^" + p
    if rng.random() < 0.2:
        p = p + "$"
    if rng.random() < 0.15:
        p = p + "|" + rng.choice(ATOMOS)
    return p

def esc(s):
    return s.replace("\\", "\\\\").replace('"', '\\"')

def main():
    total = int(sys.argv[1]) if len(sys.argv) > 1 else 200
    rng = random.Random(int(sys.argv[2]) if len(sys.argv) > 2 else 13)
    casos = []
    for _ in range(total):
        casos.append((patron(rng), rng.choice(SUJETOS)))
    lineas = []
    for p, s in casos:
        lineas.append('imprimir(__regex_coincide("%s", "%s"));' % (esc(p), esc(s)))
        lineas.append('imprimir(__regex_reemplazar("%s", "%s", "#"));' % (esc(p), esc(s)))
    tmp = tempfile.mkdtemp(prefix="fz13_")
    src = os.path.join(tmp, "fz13.nv")
    binp = os.path.join(tmp, "fz13.bin")
    open(src, "w").write("\n".join(lineas) + "\n")

    vm = subprocess.run([LUMEN, "run", src], capture_output=True, text=True, timeout=300)
    b = subprocess.run([LUMEN, "build", "--native", src, "-o", binp],
                       capture_output=True, text=True, timeout=600)
    if b.returncode != 0:
        print("no compila el binario nativo:", b.stderr[-400:]); return 2
    nat = subprocess.run([binp], capture_output=True, text=True, timeout=300)
    if nat.returncode != 0:
        print("el binario nativo salio con rc=%d  stderr=%r" % (nat.returncode, nat.stderr[-300:]))
        print("ultima linea emitida: %r" % (nat.stdout.strip().split("\n")[-1:],))

    a = vm.stdout.replace("\r\n", "\n").strip().split("\n")
    c = nat.stdout.replace("\r\n", "\n").strip().split("\n")
    diffs = 0
    if len(a) != len(c):
        print("distinto numero de lineas: vm=%d nativo=%d" % (len(a), len(c)))
        diffs += 1
    for i in range(min(len(a), len(c))):
        if a[i] != c[i]:
            k = i // 2
            p, s = casos[k]
            print("DIFF /%s/ vs %r => vm=%r nativo=%r" % (p, s, a[i], c[i]))
            diffs += 1
            if diffs > 20: break
    print("== fz13 regex: total=%d comparaciones=%d diffs=%d ==" % (total, len(a), diffs))
    return 1 if diffs else 0

sys.exit(main())
