#!/usr/bin/env python3
"""
LÚMEN — Suite de regresión y pruebas cruzadas (QA bugs A–P).

Fija con pruebas el comportamiento de los bugs corregidos en la auditoría
v3.94.22 para evitar regresiones:

  A  `dueno T` auto-desreferencia campos e indexación
  B  hilos/mutex: typo en nombre de función es error (check + runtime)
  C  `en_tiempo_compilacion` puro pliega; impuro es error E090
  D  binding de `si sea` es por valor (comportamiento documentado)
  E  overflow wraparound documentado + aritmética verificada
  F  stdlib oficial: los 69 módulos compilan
  G  mensaje E031 explica la normalización de `cualquiera` a `decimal`
  H  casts reales `X como T` (entero/decimal/booleano/texto) — VM y AOT
  I  `continuar`/`romper` dentro de `para ... en` (antes E055/E070)
  J  `~` bitnot plegado no duplica el operando en llamadas multi-arg
  K  slice de rango `a[x..y]`/`s[x..y]` con paridad VM/AOT
  L  lambdas `funcion (...) {...}` y `|x| ...` con paridad VM/AOT

Uso:
  python scripts/regresion_qa.py --lumen ./target/debug/lumen --stdlib stdlib
"""
import argparse
import pathlib
import subprocess
import sys
import tempfile
import os

# ── Casos: (nombre, fuente, modo, espera_exit_0, subcadena_esperada) ─────────
CASES = [
    # ── A: dueno auto-desreferencia ──────────────────────────────────────────
    {
        "name": "A_dueno_campos_e_index",
        "mode": "run",
        "expect_exit0": True,
        "expect": ["simple:7", "generico:3", "lista:9", "metodo:2", "enum:99"],
        "source": """
estructura Recurso { id: entero }
estructura Caja<T> { valor: T }
estructura Item { valor: entero }
enum Estado { Listo(entero), Vacio }
funcion vacio consumir_simple(dueno Recurso r) { imprimir("simple:", a_texto(r.id)); }
funcion vacio consumir_generico(dueno Caja<entero> c) { imprimir("generico:", a_texto(c.valor)); }
funcion vacio consumir_lista(dueno lista<Item> l) { imprimir("lista:", a_texto(l[0].valor)); }
funcion vacio consumir_metodo(dueno lista<Item> l) { imprimir("metodo:", a_texto(l.largo())); }
funcion vacio consumir_enum(dueno Estado e) {
    si sea Estado::Listo(x) = e {
        imprimir("enum:", a_texto(x));
    } sino {
        imprimir("enum:vacio");
    }
}
funcion entero principal() {
    consumir_simple(Recurso { id: 7 });
    consumir_generico(Caja { valor: 3 });
    consumir_lista([Item { valor: 9 }]);
    consumir_metodo([Item { valor: 1 }, Item { valor: 2 }]);
    consumir_enum(Estado::Listo(99));
    retornar 0;
}
""",
    },
    # ── B: typo en literal → error en check ──────────────────────────────────
    {
        "name": "B_typo_check_E042",
        "mode": "check",
        "expect_exit0": False,
        "expect": ["E042", "tarea_pessada"],
        "source": """
importar "concurrencia.nv";
funcion entero principal() {
    cualquiera handle = hilo_lanzar1("tarea_pessada", 10);
    cualquiera salida = hilo_esperar(handle);
    imprimir(a_texto(salida));
    retornar 0;
}
""",
    },
    # ── B: typo vía variable → error explícito en runtime (no void) ─────────
    {
        "name": "B_typo_runtime_error",
        "mode": "run",
        "expect_exit0": True,
        "expect": ["no definida", "tarea_pessada"],
        "forbidden": ["void"],
        "source": """
importar "concurrencia.nv";
funcion entero principal() {
    texto nombre = "tarea_pessada";
    cualquiera handle = hilo_lanzar1(nombre, 10);
    cualquiera salida = hilo_esperar(handle);
    imprimir(a_texto(salida));
    retornar 0;
}
""",
    },
    # ── B: nombre correcto sigue funcionando ─────────────────────────────────
    {
        "name": "B_ok_resultado",
        "mode": "run",
        "expect_exit0": True,
        "expect": ["20"],
        "source": """
importar "concurrencia.nv";
funcion cualquiera tarea(entero n) { retornar n * 2; }
funcion entero principal() {
    cualquiera handle = hilo_lanzar1("tarea", 10);
    cualquiera salida = hilo_esperar(handle);
    imprimir(a_texto(salida));
    retornar 0;
}
""",
    },
    # ── C: comptime impuro → error E090 ──────────────────────────────────────
    {
        "name": "C_comptime_impuro_E090",
        "mode": "run",
        "expect_exit0": False,
        "expect": ["E090"],
        "source": """
funcion entero con_efecto() { imprimir("EFECTO!"); retornar 42; }
funcion entero principal() {
    entero x = en_tiempo_compilacion { con_efecto() };
    imprimir(a_texto(x));
    retornar 0;
}
""",
    },
    # ── C: comptime puro pliega y ejecuta ────────────────────────────────────
    {
        "name": "C_comptime_puro_pliega",
        "mode": "run",
        "expect_exit0": True,
        "expect": ["65578"],
        "source": """
funcion entero principal() {
    entero x = en_tiempo_compilacion { (1024 * 1024) / 16 + 42 };
    imprimir(a_texto(x));
    retornar 0;
}
""",
    },
    # ── D: binding por valor de `si sea` (documentado) ───────────────────────
    {
        "name": "D_binding_por_valor",
        "mode": "run",
        "expect_exit0": True,
        "expect": ["nivel:1"],
        "source": """
estructura Config { nivel: entero }
funcion entero principal() {
    opcion<Config> cfg = algun(Config { nivel: 1 });
    si sea algun(c) = cfg {
        c.nivel = 5;
    }
    si sea algun(c) = cfg {
        imprimir("nivel:", a_texto(c.nivel));
    }
    retornar 0;
}
""",
    },
    # ── E: wraparound documentado ────────────────────────────────────────────
    {
        "name": "E_wraparound_documentado",
        "mode": "run",
        "expect_exit0": True,
        "expect": ["-9223372036854775808"],
        "source": """
funcion entero principal() {
    entero max_val = 9223372036854775807;
    entero salida = max_val + 1;
    imprimir(a_texto(salida));
    retornar 0;
}
""",
    },
    # ── E: aritmética verificada ─────────────────────────────────────────────
    {
        "name": "E_aritmetica_verificada",
        "mode": "run",
        "expect_exit0": True,
        "expect": ["exito(30)", "Desbordamiento", "División por cero"],
        "source": """
importar "matematicas.nv";
funcion entero principal() {
    imprimir(suma_verificada(10, 20));
    imprimir(suma_verificada(9223372036854775807, 1));
    imprimir(division_verificada(1, 0));
    retornar 0;
}
""",
    },
    # ── G: mensaje E031 explica cualquiera→decimal ───────────────────────────
    {
        "name": "G_mensaje_normalizacion",
        "mode": "check",
        "expect_exit0": False,
        "expect": ["E031", "normaliza"],
        "source": """
funcion cualquiera identidad(cualquiera x) { retornar x; }
funcion entero principal() {
    entero c = identidad(42) + 1;
    imprimir(a_texto(c));
    retornar 0;
}
""",
    },
    # ── H: casts reales `X como T` (VM y AOT) ────────────────────────────────
    {
        "name": "H_cast_real",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["i:3", "d:5", "b:true", "t:hola", "z:false"],
        "source": """
funcion entero principal() {
    imprimir("i:", a_texto(3.9 como entero));
    imprimir("d:", a_texto(5 como decimal));
    imprimir("b:", a_texto(5 como booleano));
    imprimir("t:", "hola" como texto);
    imprimir("z:", a_texto(0 como booleano));
    retornar 0;
}
""",
    },
    # ── I: continuar/romper dentro de `para ... en` ──────────────────────────
    {
        "name": "I_foreach_continuar_romper",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["suma:8"],
        "source": """
funcion entero principal() {
    entero suma = 0;
    para n en [1, 2, 3, 4, 5, 6] {
        si n == 2 { continuar; }
        si n == 5 { romper; }
        suma = suma + n;
    }
    imprimir("suma:", a_texto(suma));
    retornar 0;
}
""",
    },
    # ── J: bitnot `~` no duplica el operando en llamadas multi-arg ───────────
    {
        "name": "J_bitnot_multiarg",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["A:1-9", "solo:-9", "B:-92"],
        "source": """
funcion entero principal() {
    imprimir("A:", 1, ~8);
    imprimir("solo:", ~8);
    imprimir("B:", ~8, 2);
    retornar 0;
}
""",
    },
    # ── K: slice de rango `a[x..y]` / `s[x..y]` con paridad VM/AOT ───────────
    {
        "name": "K_rodaja_rango",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["e:bcd", "i:bcde", "len:2", "v:2030"],
        "source": """
funcion entero principal() {
    texto s = "abcdef";
    imprimir("e:", s[1..4], " i:", s[1..=4]);
    lista<entero> a = [10, 20, 30, 40, 50];
    imprimir("len:", a_texto(a[1..3].largo()), " v:", a_texto(a[1..3][0]), a_texto(a[1..3][1]));
    retornar 0;
}
""",
    },
    # ── L: lambdas `funcion (...) {...}` y `|x| ...` (paridad VM/AOT) ────────
    {
        "name": "L_lambdas",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["triple:21", "inc:42"],
        "source": """
funcion entero principal() {
    sea triple = |x| x * 3;
    sea inc = funcion (entero n) { retornar n + 1; };
    imprimir("triple:", a_texto(triple(7)), " inc:", a_texto(inc(41)));
    retornar 0;
}
""",
    },
    # ── M: mapa (paridad VM/AOT; tamaño que cabe en el backend C de valor) ───
    {
        "name": "M_mapa_basico",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["maps:999000", "len:1000"],
        "source": """
funcion entero principal() {
    sea d = __map_nuevo();
    entero i = 0;
    mientras (i < 1000) {
        d = __map_poner(d, i, i * 2);
        i = i + 1;
    }
    numero total = 0;
    i = 0;
    mientras (i < 1000) {
        total = total + __map_obtener(d, i);
        i = i + 1;
    }
    imprimir("maps:", a_texto(total), " len:", a_texto(__map_longitud(d)));
    retornar 0;
}
""",
    },
    # ── N: rango guardado en variable como índice (paridad VM/AOT) ───────────
    {
        "name": "N_rango_variable_indice",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["vm_arr:2", "vm_str:bc", "k:30"],
        "source": """
funcion entero principal() {
    lista<entero> a = [10, 20, 30, 40];
    sea r = 1..3;
    imprimir("vm_arr:", a_texto(a[r].largo()));
    texto s = "abcde";
    imprimir("vm_str:", s[r]);
    entero k = 2;
    imprimir("k:", a_texto(a[k]));
    retornar 0;
}
""",
    },
    # ── O: ternarios inline dentro de llamadas, en bucle (paridad VM/AOT) ────
    {
        "name": "O_ternarios_bucle",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["x:104060", "x:203060", "x:204050"],
        "source": """
funcion entero principal() {
    entero i = 0;
    mientras (i < 3) {
        imprimir("x:", i == 0 ? 10 : 20, i == 1 ? 30 : 40, i == 2 ? 50 : 60);
        i = i + 1;
    }
    retornar 0;
}
""",
    },
    # ── P: closure lambda captura por valor/snapshot (paridad VM/AOT) ────────
    {
        "name": "P_closure_captura",
        "mode": "parity",
        "expect_exit0": True,
        "expect": ["cont1:0", "a:5", "b:12", "f1:11", "f2:3", "x:1", "total:60"],
        "source": """
funcion entero principal() {
    entero cont1 = 0;
    sea inc = |n: entero| { cont1 = cont1 + n; retornar cont1; };
    entero a = inc(5);
    entero b = inc(7);
    imprimir("cont1:", a_texto(cont1), " a:", a_texto(a), " b:", a_texto(b));
    entero x = 1;
    sea f1 = |n: entero| { x = x + n; retornar x; };
    sea f2 = |n: entero| { x = x * n; retornar x; };
    imprimir("f1:", a_texto(f1(10)), " f2:", a_texto(f2(3)), " x:", a_texto(x));
    entero total = 0;
    entero i = 0;
    mientras (i < 5) {
        sea g = |n: entero| n + i;
        total = total + g(10);
        i = i + 1;
    }
    imprimir("total:", a_texto(total));
    retornar 0;
}
""",
    },
]


def run_one(lumen_bin, stdlib_dirs, mode, source):
    with tempfile.NamedTemporaryFile("w", suffix=".nv", delete=False, encoding="utf-8") as f:
        f.write(source)
        path = f.name
    try:
        if mode == "parity":
            # VM vs AOT-C: misma salida y mismo código de salida.
            vm = subprocess.run(
                [str(lumen_bin), "run"] + stdlib_dirs + [path],
                capture_output=True, timeout=30,
            )
            vm_out = (vm.stdout or b"").decode(errors="ignore")
            b = subprocess.run(
                [str(lumen_bin), "build", "--native"] + stdlib_dirs + [path],
                capture_output=True, timeout=60,
            )
            exe = pathlib.Path(path).with_suffix("")
            if b.returncode != 0 or not exe.exists():
                os.unlink(path)
                return 1, f"(build native falló: {(b.stderr or b'').decode(errors='ignore')[:300]})"
            aot = subprocess.run([str(exe)], capture_output=True, timeout=30)
            aot_out = (aot.stdout or b"").decode(errors="ignore")
            exe.unlink(missing_ok=True)
            if vm.returncode != aot.returncode or vm_out != aot_out:
                return 1, f"PARIDAD ROTA\nVM({vm.returncode}): {vm_out}\nAOT({aot.returncode}): {aot_out}"
            return 0, vm_out
        cmd = [str(lumen_bin), mode] + stdlib_dirs + [path]
        r = subprocess.run(cmd, capture_output=True, timeout=30)
        out = (r.stdout or b"").decode(errors="ignore") + (r.stderr or b"").decode(errors="ignore")
        return r.returncode, out
    finally:
        try:
            os.unlink(path)
        except OSError:
            pass


def stdlib_gate(lumen_bin, stdlib_dir):
    """Bug F: todos los módulos de la stdlib compilan limpio."""
    fails = []
    mods = sorted(pathlib.Path(stdlib_dir).glob("*.nv"))
    for m in mods:
        r = subprocess.run(
            [str(lumen_bin), "check", "-L", str(stdlib_dir), "-L", str(stdlib_dir / "compiler"), str(m)],
            capture_output=True, timeout=30,
        )
        if r.returncode != 0:
            fails.append(m.name)
    return mods, fails


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--lumen", required=True)
    ap.add_argument("--stdlib", required=True)
    args = ap.parse_args()

    lumen = pathlib.Path(args.lumen)
    stdlib = pathlib.Path(args.stdlib)
    stdlib_dirs = ["-L", str(stdlib), "-L", str(stdlib / "compiler")]

    total = 0
    failed = 0

    print("═══ LÚMEN · Regresión QA (A–P) ═══")
    for case in CASES:
        total += 1
        rc, out = run_one(lumen, stdlib_dirs, case["mode"], case["source"])
        ok_exit = (rc == 0) == case["expect_exit0"]
        ok_subs = all(s in out for s in case["expect"])
        ok_forbidden = all(s not in out for s in case.get("forbidden", []))
        if ok_exit and ok_subs and ok_forbidden:
            print(f"  ✓ {case['name']}")
        else:
            failed += 1
            print(f"  ✗ {case['name']}  (exit={rc}, esperaba exit0={case['expect_exit0']})")
            for s in case["expect"]:
                if s not in out:
                    print(f"      falta subcadena: {s!r}")
            for s in case.get("forbidden", []):
                if s in out:
                    print(f"      apareció subcadena prohibida: {s!r}")

    # ── F: gate de stdlib ────────────────────────────────────────────────────
    total += 1
    mods, fails = stdlib_gate(lumen, stdlib)
    if fails:
        failed += 1
        print(f"  ✗ F_stdlib_compila  ({len(fails)} módulos fallan: {', '.join(fails)})")
    else:
        print(f"  ✓ F_stdlib_compila  ({len(mods)} módulos compilan limpio)")

    print(f"═══ Resultado: {total - failed}/{total} OK ═══")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
