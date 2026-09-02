// Benchmark de referencia en C — mismos algoritmos que benchmarks/lumen/*.nv
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

static long long fib(int n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

static long long sum(long long n) {
    long long acc = 0;
    for (long long i = 0; i < n; i++) acc += i;
    return acc;
}

static int es_primo(long long n) {
    if (n < 2) return 0;
    if (n == 2) return 1;
    if (n % 2 == 0) return 0;
    for (long long i = 3; i * i <= n; i += 2)
        if (n % i == 0) return 0;
    return 1;
}

static long long primes(long long lim) {
    long long c = 0;
    for (long long k = 2; k < lim; k++)
        if (es_primo(k)) c++;
    return c;
}

static long long strings(long n) {
    long long total = 0;
    char buf[64];
    for (long i = 0; i < n; i++) {
        int len = snprintf(buf, sizeof buf, "item-%ld-fin", i);
        total += len;
    }
    return total;
}

static long long arrays(long n) {
    long long* xs = (long long*)malloc(sizeof(long long) * n);
    for (long i = 0; i < n; i++) xs[i] = i;
    long long acc = 0;
    for (long i = 0; i < n; i++) acc += xs[i];
    free(xs);
    return acc;
}

int main(int argc, char** argv) {
    if (argc < 2) { fprintf(stderr, "uso: bench <fib|sum|primes|strings|arrays>\n"); return 2; }
    const char* t = argv[1];
    if (!strcmp(t, "fib")) printf("fib:%lld\n", fib(26));
    else if (!strcmp(t, "sum")) printf("sum:%lld\n", sum(10000000));
    else if (!strcmp(t, "primes")) printf("primes:%lld\n", primes(20000));
    else if (!strcmp(t, "strings")) printf("strings:%lld\n", strings(200000));
    else if (!strcmp(t, "arrays")) printf("arrays:%lld\n", arrays(200000));
    else { fprintf(stderr, "tarea desconocida: %s\n", t); return 2; }
    return 0;
}
