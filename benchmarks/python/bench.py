# Benchmark de referencia en Python — mismos algoritmos que benchmarks/lumen/*.nv
import sys


def fib(n):
    if n < 2:
        return n
    return fib(n - 1) + fib(n - 2)


def sum_loop(n):
    acc = 0
    i = 0
    while i < n:
        acc += i
        i += 1
    return acc


def es_primo(n):
    if n < 2:
        return False
    if n == 2:
        return True
    if n % 2 == 0:
        return False
    i = 3
    while i * i <= n:
        if n % i == 0:
            return False
        i += 2
    return True


def primes(lim):
    c = 0
    k = 2
    while k < lim:
        if es_primo(k):
            c += 1
        k += 1
    return c


def strings(n):
    total = 0
    i = 0
    while i < n:
        s = "item-" + str(i) + "-fin"
        total += len(s)
        i += 1
    return total


def arrays(n):
    xs = []
    i = 0
    while i < n:
        xs.append(i)
        i += 1
    acc = 0
    j = 0
    while j < n:
        acc += xs[j]
        j += 1
    return acc


if __name__ == "__main__":
    t = sys.argv[1] if len(sys.argv) > 1 else ""
    if t == "fib":
        print("fib:%d" % fib(26))
    elif t == "sum":
        print("sum:%d" % sum_loop(10_000_000))
    elif t == "primes":
        print("primes:%d" % primes(20_000))
    elif t == "strings":
        print("strings:%d" % strings(200_000))
    elif t == "arrays":
        print("arrays:%d" % arrays(200_000))
    else:
        print("uso: bench.py <fib|sum|primes|strings|arrays>", file=sys.stderr)
        sys.exit(2)
