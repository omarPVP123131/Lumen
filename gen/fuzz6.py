#!/usr/bin/env python3
"""Fuzzer diferencial de STRUCTS, LISTAS y `prestado mut` (zona del BUG-008).

El BUG-008 era que un struct se pasaba por referencia mientras que un
`lista<T>` se pasaba por valor: dos reglas distintas para la misma sintaxis.
Tras el fix la regla es única —por valor se copia, `prestado mut` aliasea—, y
este fuzzer la somete a presión combinando ambos mundos: structs que contienen
listas, listas de structs, cadenas de llamadas y alias por asignación.

Lo importante es que la salida esperada se calcula **en Python**, con un modelo
propio de la semántica, y no comparando la VM contra sí misma. Así se detectan
tres cosas distintas:

  1. una divergencia VM ↔ binario nativo,
  2. que ambos backends coincidan en un resultado incorrecto —el modo de fallo
     del BUG-008, que no habría saltado con una comparación VM↔nativo—,
  3. un aliasing accidental: que una mutación se escape a través de una copia.

Uso:  python3 fuzz6.py [n_casos]
"""
import random
import os
import sys
import copy

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fz10")
os.makedirs(OUT, exist_ok=True)


def fmt_lista(xs):
    return "[" + ", ".join(str(x) for x in xs) + "]"


# ─────────────────────── generadores de casos ───────────────────────


def caso_struct_prestado_vs_valor(r):
    """El corazón del BUG-008: la misma función con y sin `prestado mut`."""
    x0 = r.randint(0, 9)
    v1 = r.randint(10, 99)
    v2 = r.randint(10, 99)
    src = [
        "estructura P { x: entero, }",
        "",
        "funcion vacio por_valor(P p) { p.x = %d; }" % v1,
        "funcion vacio por_ref(prestado mut P p) { p.x = %d; }" % v2,
        "",
        "sea a = P{x: %d};" % x0,
        "por_valor(a);",
        "imprimir(a.x);",
        "por_ref(a);",
        "imprimir(a.x);",
    ]
    # Modelo: por valor no se ve; por referencia sí.
    return "\n".join(src) + "\n", [str(x0), str(v2)], "struct_prestado_vs_valor"


def caso_lista_prestada_vs_valor(r):
    n = r.randint(2, 4)
    l = [r.randint(0, 9) for _ in range(n)]
    i = r.randrange(n)
    v1 = r.randint(10, 99)
    v2 = r.randint(10, 99)
    src = [
        "funcion vacio por_valor(lista<entero> l) { l[%d] = %d; }" % (i, v1),
        "funcion vacio por_ref(prestado mut lista<entero> l) { l[%d] = %d; }" % (i, v2),
        "",
        "lista<entero> a = %s;" % fmt_lista(l),
        "por_valor(a);",
        "imprimir(a);",
        "por_ref(a);",
        "imprimir(a);",
    ]
    esp = [fmt_lista(l)]
    l2 = list(l)
    l2[i] = v2
    esp.append(fmt_lista(l2))
    return "\n".join(src) + "\n", esp, "lista_prestada_vs_valor"


def caso_struct_con_lista(r):
    """Struct que contiene una lista: la copia debe ser profunda."""
    n = r.randint(2, 3)
    l = [r.randint(0, 9) for _ in range(n)]
    i = r.randrange(n)
    v1 = r.randint(10, 99)
    v2 = r.randint(10, 99)
    src = [
        "estructura S { l: lista<entero>, }",
        "",
        "funcion vacio por_valor(S s) { s.l[%d] = %d; }" % (i, v1),
        "funcion vacio por_ref(prestado mut S s) { s.l[%d] = %d; }" % (i, v2),
        "",
        "sea a = S{l: %s};" % fmt_lista(l),
        "por_valor(a);",
        "imprimir(a.l);",
        "por_ref(a);",
        "imprimir(a.l);",
    ]
    esp = [fmt_lista(l)]
    l2 = list(l)
    l2[i] = v2
    esp.append(fmt_lista(l2))
    return "\n".join(src) + "\n", esp, "struct_con_lista"


def caso_lista_de_structs(r):
    n = r.randint(2, 3)
    xs = [r.randint(0, 9) for _ in range(n)]
    i = r.randrange(n)
    v = r.randint(10, 99)
    src = [
        "estructura P { x: entero, }",
        "",
        "funcion vacio por_ref(prestado mut lista<P> l) { l[%d].x = %d; }" % (i, v),
        "",
        "lista<P> a = [%s];" % ", ".join("P{x: %d}" % x for x in xs),
        "por_ref(a);",
    ]
    xs2 = list(xs)
    xs2[i] = v
    for k in range(n):
        src.append("imprimir(a[%d].x);" % k)
    return "\n".join(src) + "\n", [str(x) for x in xs2], "lista_de_structs"


def caso_agregar_prestado(r):
    """`agregar` a través de un préstamo debe verse fuera."""
    n = r.randint(1, 3)
    l = [r.randint(0, 9) for _ in range(n)]
    nuevos = [r.randint(10, 99) for _ in range(r.randint(1, 3))]
    cuerpo = " ".join("agregar(l, %d);" % v for v in nuevos)
    src = [
        "funcion vacio ap(prestado mut lista<entero> l) { %s }" % cuerpo,
        "funcion vacio ap_valor(lista<entero> l) { agregar(l, 777); }",
        "",
        "lista<entero> a = %s;" % fmt_lista(l),
        "ap_valor(a);",
        "imprimir(largo(a));",
        "ap(a);",
        "imprimir(a);",
        "imprimir(largo(a));",
    ]
    esp = [str(n)]
    l2 = l + nuevos
    esp.append(fmt_lista(l2))
    esp.append(str(len(l2)))
    return "\n".join(src) + "\n", esp, "agregar_prestado"


def caso_cadena_de_llamadas(r):
    """Un préstamo que se propaga a través de varias funciones."""
    x0 = r.randint(0, 9)
    d1 = r.randint(1, 9)
    d2 = r.randint(1, 9)
    src = [
        "estructura C { n: entero, }",
        "",
        "funcion vacio interna(prestado mut C c) { c.n = c.n + %d; }" % d2,
        "funcion vacio externa(prestado mut C c) {",
        "    c.n = c.n + %d;" % d1,
        "    interna(c);",
        "}",
        "",
        "sea c = C{n: %d};" % x0,
        "externa(c);",
        "imprimir(c.n);",
    ]
    return "\n".join(src) + "\n", [str(x0 + d1 + d2)], "cadena_de_llamadas"


def caso_alias_por_asignacion(r):
    """`sea b = a;` copia: mutar b no debe tocar a."""
    x0 = r.randint(0, 9)
    v = r.randint(10, 99)
    n = r.randint(2, 3)
    l = [r.randint(0, 9) for _ in range(n)]
    i = r.randrange(n)
    w = r.randint(10, 99)
    src = [
        "estructura P { x: entero, }",
        "",
        "sea a = P{x: %d};" % x0,
        "sea b = a;",
        "b.x = %d;" % v,
        "imprimir(a.x);",
        "imprimir(b.x);",
        "",
        "lista<entero> l1 = %s;" % fmt_lista(l),
        "lista<entero> l2 = l1;",
        "l2[%d] = %d;" % (i, w),
        "imprimir(l1);",
        "imprimir(l2);",
    ]
    l2 = list(l)
    l2[i] = w
    esp = [str(x0), str(v), fmt_lista(l), fmt_lista(l2)]
    return "\n".join(src) + "\n", esp, "alias_por_asignacion"


def caso_bucle_muta_prestado(r):
    """Mutación repetida dentro de un bucle a través del préstamo."""
    n = r.randint(2, 4)
    l = [r.randint(0, 9) for _ in range(n)]
    d = r.randint(1, 5)
    src = [
        "funcion vacio escala(prestado mut lista<entero> l, entero k) {",
        "    para i en 0..%d {" % n,
        "        l[i] = l[i] + k;",
        "    }",
        "}",
        "",
        "lista<entero> a = %s;" % fmt_lista(l),
        "escala(a, %d);" % d,
        "imprimir(a);",
    ]
    return (
        "\n".join(src) + "\n",
        [fmt_lista([x + d for x in l])],
        "bucle_muta_prestado",
    )


def caso_struct_anidado_prestado(r):
    """Préstamo de un struct con otro struct dentro."""
    x0 = r.randint(0, 9)
    v = r.randint(10, 99)
    src = [
        "estructura Interno { v: entero, }",
        "estructura Externo { i: Interno, }",
        "",
        "funcion vacio toca(prestado mut Externo e) { e.i.v = %d; }" % v,
        "funcion vacio no_toca(Externo e) { e.i.v = 555; }",
        "",
        "sea e = Externo{i: Interno{v: %d}};" % x0,
        "no_toca(e);",
        "imprimir(e.i.v);",
        "toca(e);",
        "imprimir(e.i.v);",
    ]
    return "\n".join(src) + "\n", [str(x0), str(v)], "struct_anidado_prestado"


def caso_devolver_struct(r):
    """Un struct devuelto por una función no debe compartir estado."""
    x0 = r.randint(0, 9)
    v = r.randint(10, 99)
    src = [
        "estructura P { x: entero, }",
        "",
        "funcion P construir(entero n) { retornar P{x: n}; }",
        "",
        "sea a = construir(%d);" % x0,
        "sea b = construir(%d);" % x0,
        "b.x = %d;" % v,
        "imprimir(a.x);",
        "imprimir(b.x);",
    ]
    return "\n".join(src) + "\n", [str(x0), str(v)], "devolver_struct"


def caso_dos_prestamos_secuenciales(r):
    """Dos préstamos consecutivos sobre el mismo dato."""
    n = r.randint(2, 3)
    l = [r.randint(0, 9) for _ in range(n)]
    i, j = r.randrange(n), r.randrange(n)
    v1, v2 = r.randint(10, 99), r.randint(10, 99)
    src = [
        "funcion vacio uno(prestado mut lista<entero> l) { l[%d] = %d; }" % (i, v1),
        "funcion vacio dos(prestado mut lista<entero> l) { l[%d] = %d; }" % (j, v2),
        "",
        "lista<entero> a = %s;" % fmt_lista(l),
        "uno(a);",
        "dos(a);",
        "imprimir(a);",
    ]
    l2 = list(l)
    l2[i] = v1
    l2[j] = v2
    return "\n".join(src) + "\n", [fmt_lista(l2)], "dos_prestamos_secuenciales"


def caso_matriz_prestada(r):
    """Lista de listas a través de un préstamo."""
    f = r.randint(2, 3)
    c = r.randint(2, 3)
    g = [[r.randint(0, 9) for _ in range(c)] for _ in range(f)]
    i, j = r.randrange(f), r.randrange(c)
    v = r.randint(10, 99)
    src = [
        "funcion vacio toca(prestado mut lista<lista<entero>> g) { g[%d][%d] = %d; }"
        % (i, j, v),
        "",
        "lista<lista<entero>> m = [%s];" % ", ".join(fmt_lista(fila) for fila in g),
        "toca(m);",
    ]
    g2 = copy.deepcopy(g)
    g2[i][j] = v
    for k in range(f):
        src.append("imprimir(m[%d]);" % k)
    return "\n".join(src) + "\n", [fmt_lista(fila) for fila in g2], "matriz_prestada"



def caso_prestar_campo(r):
    """BUG-147: pasar `s.l` a un `prestado mut` descartaba la mutación."""
    n = r.randint(2, 3)
    l = [r.randint(0, 9) for _ in range(n)]
    i = r.randrange(n)
    v = r.randint(10, 99)
    src = [
        "estructura S { l: lista<entero>, }",
        "",
        "funcion vacio toca(prestado mut lista<entero> l) { l[%d] = %d; }" % (i, v),
        "",
        "sea s = S{l: %s};" % fmt_lista(l),
        "toca(s.l);",
        "imprimir(s.l);",
    ]
    l2 = list(l)
    l2[i] = v
    return "\n".join(src) + "\n", [fmt_lista(l2)], "prestar_campo"


def caso_prestar_elemento(r):
    """BUG-147: pasar `l[i]` a un `prestado mut`."""
    n = r.randint(2, 3)
    xs = [r.randint(0, 9) for _ in range(n)]
    i = r.randrange(n)
    v = r.randint(10, 99)
    src = [
        "estructura P { x: entero, }",
        "",
        "funcion vacio toca(prestado mut P p) { p.x = %d; }" % v,
        "",
        "lista<P> l = [%s];" % ", ".join("P{x: %d}" % x for x in xs),
        "toca(l[%d]);" % i,
    ]
    xs2 = list(xs)
    xs2[i] = v
    for k in range(n):
        src.append("imprimir(l[%d].x);" % k)
    return "\n".join(src) + "\n", [str(x) for x in xs2], "prestar_elemento"


def caso_prestar_lista_anidada(r):
    """BUG-147: `agregar` a través de un préstamo de `m[i]`."""
    f = r.randint(2, 3)
    g = [[r.randint(0, 9) for _ in range(r.randint(1, 2))] for _ in range(f)]
    i = r.randrange(f)
    v = r.randint(10, 99)
    src = [
        "funcion vacio ap(prestado mut lista<entero> l) { agregar(l, %d); }" % v,
        "",
        "lista<lista<entero>> m = [%s];" % ", ".join(fmt_lista(x) for x in g),
        "ap(m[%d]);" % i,
    ]
    g2 = copy.deepcopy(g)
    g2[i].append(v)
    for k in range(f):
        src.append("imprimir(m[%d]);" % k)
    return "\n".join(src) + "\n", [fmt_lista(x) for x in g2], "prestar_lista_anidada"


def caso_prestar_campo_anidado(r):
    """BUG-147 sobre una cadena `e.i.v` de dos niveles."""
    x0 = r.randint(0, 9)
    v = r.randint(10, 99)
    src = [
        "estructura Interno { v: entero, }",
        "estructura Externo { i: Interno, }",
        "",
        "funcion vacio toca(prestado mut Interno x) { x.v = %d; }" % v,
        "",
        "sea e = Externo{i: Interno{v: %d}};" % x0,
        "toca(e.i);",
        "imprimir(e.i.v);",
    ]
    return "\n".join(src) + "\n", [str(v)], "prestar_campo_anidado"


GEN = [
    caso_struct_prestado_vs_valor,
    caso_lista_prestada_vs_valor,
    caso_struct_con_lista,
    caso_lista_de_structs,
    caso_agregar_prestado,
    caso_cadena_de_llamadas,
    caso_alias_por_asignacion,
    caso_bucle_muta_prestado,
    caso_struct_anidado_prestado,
    caso_devolver_struct,
    caso_dos_prestamos_secuenciales,
    caso_matriz_prestada,
    caso_prestar_campo,
    caso_prestar_elemento,
    caso_prestar_lista_anidada,
    caso_prestar_campo_anidado,
]


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 120
    r = random.Random(8)  # semilla = número del bug de origen
    for k in range(n):
        src, esp, tag = GEN[k % len(GEN)](r)
        base = os.path.join(OUT, "p%04d" % k)
        with open(base + ".nv", "w") as fh:
            fh.write(src)
        with open(base + ".exp", "w") as fh:
            fh.write("\n".join(esp) + "\n")
        with open(base + ".tag", "w") as fh:
            fh.write(tag + "\n")
    print("generados %d casos en %s" % (n, OUT))


main()
