#!/usr/bin/env python3
"""Suite variada — Python de referencia (CPython).
Mismos algoritmos que la versión C: quicksort 100k con LCG, matmul 48x48,
sieve 1M, dict 30k (dict nativo). Salida: "nombre:valor"."""


def lcg_next(x):
    return (x * 1103515245 + 12345) % 2147483648


def qsort_lumen(a, izq, der):
    if izq >= der:
        return
    pivote = a[izq]
    i, j = izq, der
    while i <= j:
        while a[i] < pivote:
            i += 1
        while a[j] > pivote:
            j -= 1
        if i <= j:
            a[i], a[j] = a[j], a[i]
            i += 1
            j -= 1
    qsort_lumen(a, izq, j)
    qsort_lumen(a, i, der)


def bench_sort(n):
    a = list(range(n))
    x = 42
    for i in range(n):
        x = lcg_next(x)
        j = x % n
        a[i], a[j] = a[j], a[i]
    qsort_lumen(a, 0, n - 1)
    return sum(a[i] * (i + 1) for i in range(n))


def bench_matmul(n):
    A = [[(i * j) % 5 for j in range(n)] for i in range(n)]
    B = [[(i + j) % 7 for j in range(n)] for i in range(n)]
    total = 0
    for i in range(n):
        for j in range(n):
            s = 0
            for k in range(n):
                s += A[i][k] * B[k][j]
            total += s
    return total


def bench_sieve(lim):
    criba = [1] * (lim + 1)
    p = 2
    while p * p <= lim:
        if criba[p]:
            for m in range(p * p, lim + 1, p):
                criba[m] = 0
        p += 1
    return sum(criba[2:])


def bench_dict(n):
    d = {}
    for i in range(n):
        d[i] = i
    for i in range(0, n, 2):
        d[i] = d[i]  # re-escritura sin cambio de suma
    return sum(d[i] for i in range(n))


if __name__ == "__main__":
    print(f"sort:{bench_sort(100000)}")
    print(f"matmul:{bench_matmul(48)}")
    print(f"sieve:{bench_sieve(1000000)}")
    print(f"dict:{bench_dict(30000)}")
