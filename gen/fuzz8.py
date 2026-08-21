#!/usr/bin/env python3
"""Fuzzer de RECHAZO: programas malformados que el compilador debe rechazar.

Los fuzzers anteriores (`fuzz6.py`, `fuzz7.py`) comprueban que un programa
*válido* dé el resultado correcto. Este comprueba lo contrario, que es donde
vivía BUG-151: que un programa *inválido* no se ejecute como si nada.

El fallo original: al faltar el `{` de un bloque, el parser descartaba la
sentencia entera sin emitir ningún error y el cuerpo se reejecutaba después
como bloque suelto — sin su condición. `si (1 == 2) basura { imprimir("x"); }`
imprimía `x`, con código de salida 0 y `lumen check` dando el visto bueno.

El oráculo aquí no es un valor, sino un veredicto: **rc != 0**, y además el
programa NO debe imprimir ninguna de sus marcas centinela. Un programa que
falla pero ejecuta parte del cuerpo antes es igual de peligroso.

Uso:  python3 fuzz8.py [n_casos]
"""
import random
import os
import sys

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fz12")
os.makedirs(OUT, exist_ok=True)

# Marca centinela: si aparece en la salida, el cuerpo se ejecutó pese al error.
S = "CENTINELA_NO_DEBE_SALIR"


def w(cuerpo):
    return "funcion vacio main() {\n" + cuerpo + "\n}\n"


def c_si_sin_llave(r):
    return w('    si (1 == %d) basura { imprimir("%s"); }' % (r.randint(2, 9), S))


def c_mientras_sin_llave(r):
    return w('    mientras (1 == %d) basura { imprimir("%s"); }' % (r.randint(2, 9), S))


def c_sino_sin_llave(r):
    return w(
        '    si (1 == 1) { imprimir("ok"); } sino basura { imprimir("%s"); }' % S
    )


def c_funcion_sin_llave(r):
    return 'funcion vacio f() basura { imprimir("%s"); }\n' % S + w("    f();")


def c_si_sin_bloque(r):
    """Sin llaves en absoluto: `si cond sentencia;` no es sintaxis de LÚMEN."""
    return w('    si (1 == %d) imprimir("%s");' % (r.randint(2, 9), S))


def c_para_sin_llave(r):
    return w('    para i en 1..=%d basura { imprimir("%s"); }' % (r.randint(2, 5), S))


def c_llave_sin_cerrar(r):
    return 'funcion vacio main() {\n    si (1 == 1) { imprimir("%s");\n' % S


def c_condicion_vacia(r):
    return w('    si () { imprimir("%s"); }' % S)


def c_variable_no_declarada(r):
    return w('    si (no_existe_%d == 1) { imprimir("%s"); }' % (r.randint(1, 99), S))


def c_llamada_inexistente(r):
    return w('    funcion_que_no_existe_%d();\n    imprimir("%s");' % (r.randint(1, 99), S))


def c_tipo_incompatible(r):
    return w('    entero x = "texto";\n    imprimir("%s");' % S)


def c_parentesis_desbalanceado(r):
    return w('    si ((1 == 1) { imprimir("%s"); }' % S)


GEN = [
    c_si_sin_llave,
    c_mientras_sin_llave,
    c_sino_sin_llave,
    c_funcion_sin_llave,
    c_si_sin_bloque,
    c_para_sin_llave,
    c_llave_sin_cerrar,
    c_condicion_vacia,
    c_variable_no_declarada,
    c_llamada_inexistente,
    c_tipo_incompatible,
    c_parentesis_desbalanceado,
]


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 120
    r = random.Random(151)  # semilla = bug de origen
    for k in range(n):
        f = GEN[k % len(GEN)]
        base = os.path.join(OUT, "r%04d" % k)
        with open(base + ".nv", "w") as fh:
            fh.write(f(r))
        with open(base + ".tag", "w") as fh:
            fh.write(f.__name__ + "\n")
    print("generados %d casos de rechazo en %s" % (n, OUT))


main()
