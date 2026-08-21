#!/usr/bin/env python3
"""Fuzzer diferencial de CLOSURES y CAPTURA DE VARIABLES (zona BUG-148/149).

Igual que `fuzz6.py`, la salida esperada se calcula **en Python** en lugar de
comparar LÚMEN consigo mismo: los fallos de esta zona (BUG-148, BUG-149) no
eran divergencias entre backends, sino que ambos coincidían en perder la
captura.

El modelo de la semántica, deducido sondeando el intérprete:

  * la captura es **por referencia**: la closure ve los cambios que la
    envolvente haga después de crearla, y sus mutaciones se ven fuera;
  * cada evaluación de la expresión-lambda produce una **instancia
    independiente**, así que dos closures de la misma factoría no comparten
    estado (`crear()` dos veces ⇒ dos contadores separados);
  * pasar por valor no interviene aquí: lo capturado es la variable, no su
    valor en el momento.

Uso:  python3 fuzz7.py [n_casos]
"""
import random
import os
import sys

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "fz11")
os.makedirs(OUT, exist_ok=True)


def caso_captura_simple(r):
    """Captura de una variable del entorno, sin mutación."""
    x = r.randint(1, 50)
    a = r.randint(1, 50)
    src = [
        "entero x = %d;" % x,
        "sea f = funcion(entero a) { retornar a + x; };",
        "imprimir(f(%d));" % a,
    ]
    return "\n".join(src) + "\n", [str(a + x)], "captura_simple"


def caso_captura_ve_cambio_posterior(r):
    """La captura es por referencia: ve lo que pase después de crearla."""
    x0 = r.randint(1, 20)
    x1 = r.randint(50, 99)
    a = r.randint(1, 9)
    src = [
        "entero x = %d;" % x0,
        "sea f = funcion(entero a) { retornar a + x; };",
        "x = %d;" % x1,
        "imprimir(f(%d));" % a,
    ]
    return "\n".join(src) + "\n", [str(a + x1)], "captura_ve_cambio_posterior"


def caso_contador_devuelto(r):
    """BUG-148: una closure devuelta debe sobrevivir a su factoría."""
    n = r.randint(2, 4)
    src = [
        "funcion cualquiera crear() {",
        "    entero n = 0;",
        "    sea f = funcion() { n = n + 1; retornar n; };",
        "    retornar f;",
        "}",
        "",
        "sea c = crear();",
    ]
    esp = []
    for k in range(1, n + 1):
        src.append("imprimir(c());")
        esp.append(str(k))
    return "\n".join(src) + "\n", esp, "contador_devuelto"


def caso_instancias_aisladas(r):
    """Dos closures de la misma factoría no comparten estado."""
    b1 = r.randint(1, 50)
    b2 = r.randint(51, 99)
    a = r.randint(1, 9)
    src = [
        "funcion cualquiera crear(entero base) {",
        "    sea f = funcion(entero a) { retornar a + base; };",
        "    retornar f;",
        "}",
        "",
        "sea g1 = crear(%d);" % b1,
        "sea g2 = crear(%d);" % b2,
        "imprimir(g1(%d));" % a,
        "imprimir(g2(%d));" % a,
        "imprimir(g1(%d));" % a,
    ]
    return (
        "\n".join(src) + "\n",
        [str(a + b1), str(a + b2), str(a + b1)],
        "instancias_aisladas",
    )


def caso_mutacion_visible_fuera(r):
    """BUG-149: la mutación de una captura se ve en la variable original."""
    d = r.randint(1, 9)
    veces = r.randint(2, 4)
    src = [
        "funcion vacio p() {",
        "    entero x = 0;",
        "    sea inc = funcion(entero n) { x = x + n; retornar x; };",
    ]
    acc = 0
    esp = []
    for _ in range(veces):
        acc += d
        src.append("    imprimir(inc(%d));" % d)
        esp.append(str(acc))
    src.append("    imprimir(x);")
    esp.append(str(acc))
    src += ["}", "p();"]
    return "\n".join(src) + "\n", esp, "mutacion_visible_fuera"


def caso_captura_parametro(r):
    """Capturar el parámetro de la función envolvente."""
    base = r.randint(10, 99)
    a = r.randint(1, 9)
    src = [
        "funcion cualquiera hacer(entero base) {",
        "    sea f = funcion(entero a) { retornar a * base; };",
        "    retornar f;",
        "}",
        "",
        "sea g = hacer(%d);" % base,
        "imprimir(g(%d));" % a,
    ]
    return "\n".join(src) + "\n", [str(a * base)], "captura_parametro"


def caso_captura_lista(r):
    """Capturar una lista y mutarla desde la closure."""
    n = r.randint(2, 3)
    l = [r.randint(0, 9) for _ in range(n)]
    v = r.randint(10, 99)
    src = [
        "funcion vacio p() {",
        "    lista<entero> l = [%s];" % ", ".join(str(x) for x in l),
        "    sea f = funcion() { agregar(l, %d); retornar largo(l); };" % v,
        "    imprimir(f());",
        "    imprimir(l);",
        "}",
        "p();",
    ]
    l2 = l + [v]
    return (
        "\n".join(src) + "\n",
        [str(len(l2)), "[" + ", ".join(str(x) for x in l2) + "]"],
        "captura_lista",
    )


def caso_dos_capturas(r):
    """Una closure que captura dos variables distintas."""
    x = r.randint(1, 30)
    y = r.randint(1, 30)
    src = [
        "funcion cualquiera hacer() {",
        "    entero x = %d;" % x,
        "    entero y = %d;" % y,
        "    sea f = funcion() { retornar x * 100 + y; };",
        "    retornar f;",
        "}",
        "",
        "sea g = hacer();",
        "imprimir(g());",
    ]
    return "\n".join(src) + "\n", [str(x * 100 + y)], "dos_capturas"


def caso_closure_en_bucle(r):
    """Crear closures dentro de un bucle: cada vuelta, su propia instancia."""
    n = r.randint(2, 3)
    src = [
        "funcion cualquiera hacer(entero k) {",
        "    sea f = funcion() { retornar k * 10; };",
        "    retornar f;",
        "}",
        "",
    ]
    esp = []
    for i in range(n):
        src.append("sea f%d = hacer(%d);" % (i, i + 1))
    for i in range(n):
        src.append("imprimir(f%d());" % i)
        esp.append(str((i + 1) * 10))
    return "\n".join(src) + "\n", esp, "closure_en_bucle"


def caso_closure_anidada(r):
    """Una closure dentro de otra closure."""
    a = r.randint(1, 20)
    b = r.randint(1, 20)
    src = [
        "funcion cualquiera externa(entero a) {",
        "    sea media = funcion(entero b) {",
        "        sea interna = funcion() { retornar a + b; };",
        "        retornar interna();",
        "    };",
        "    retornar media;",
        "}",
        "",
        "sea m = externa(%d);" % a,
        "imprimir(m(%d));" % b,
    ]
    return "\n".join(src) + "\n", [str(a + b)], "closure_anidada"


def caso_closure_recursiva(r):
    """Lambda que se llama a sí misma (BUG-060)."""
    n = r.randint(3, 6)
    src = [
        "sea fact = funcion(entero n) {",
        "    si n <= 1 { retornar 1; }",
        "    retornar n * fact(n - 1);",
        "};",
        "imprimir(fact(%d));" % n,
    ]
    f = 1
    for k in range(2, n + 1):
        f *= k
    return "\n".join(src) + "\n", [str(f)], "closure_recursiva"


def caso_closure_como_argumento(r):
    """Pasar una closure a otra función y llamarla allí."""
    x = r.randint(1, 30)
    a = r.randint(1, 9)
    src = [
        "funcion entero aplicar(cualquiera f, entero v) { retornar f(v); }",
        "",
        "entero x = %d;" % x,
        "sea doble = funcion(entero a) { retornar a * 2 + x; };",
        "imprimir(aplicar(doble, %d));" % a,
    ]
    return "\n".join(src) + "\n", [str(a * 2 + x)], "closure_como_argumento"


GEN = [
    caso_captura_simple,
    caso_captura_ve_cambio_posterior,
    caso_contador_devuelto,
    caso_instancias_aisladas,
    caso_mutacion_visible_fuera,
    caso_captura_parametro,
    caso_captura_lista,
    caso_dos_capturas,
    caso_closure_en_bucle,
    caso_closure_anidada,
    caso_closure_recursiva,
    caso_closure_como_argumento,
]


def main():
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 240
    r = random.Random(148)  # semilla = número del bug de origen
    for k in range(n):
        src, esp, tag = GEN[k % len(GEN)](r)
        base = os.path.join(OUT, "c%04d" % k)
        with open(base + ".nv", "w") as fh:
            fh.write(src)
        with open(base + ".exp", "w") as fh:
            fh.write("\n".join(esp) + "\n")
        with open(base + ".tag", "w") as fh:
            fh.write(tag + "\n")
    print("generados %d casos en %s" % (n, OUT))


main()
