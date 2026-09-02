// Benchmark de referencia en C — cubre TODAS las áreas del lenguaje LÚMEN
// (recursión, bucles, primos, strings, arrays, structs, enums, mapas,
// closures y unicode). Mismos algoritmos que benchmarks/lumen/*.nv
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/* ── fib ── */
static long long fib(int n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

/* ── sum ── */
static long long sum(long long n) {
    long long acc = 0;
    for (long long i = 0; i < n; i++) acc += i;
    return acc;
}

/* ── primes ── */
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

/* ── strings ── */
static long long strings(long n) {
    long long total = 0;
    char buf[64];
    for (long i = 0; i < n; i++) {
        int len = snprintf(buf, sizeof buf, "item-%ld-fin", i);
        total += len;
    }
    return total;
}

/* ── arrays ── */
static long long arrays(long n) {
    long long* xs = (long long*)malloc(sizeof(long long) * n);
    for (long i = 0; i < n; i++) xs[i] = i;
    long long acc = 0;
    for (long i = 0; i < n; i++) acc += xs[i];
    free(xs);
    return acc;
}

/* ── structs: campo + método en bucle ── */
typedef struct { long long x, y; } Punto;
static long long punto_sumar(Punto p) { return p.x + p.y; }
static long long structs(long n) {
    Punto p = {0, 0};
    long long acc = 0;
    for (long long i = 0; i < n; i++) {
        p.x = i;
        p.y = i + 1;
        acc += punto_sumar(p);
    }
    return acc;
}

/* ── enums: construcción + dispatch en bucle ── */
enum { OP_SUMA, OP_NADA };
static long long enums(long n) {
    long long acc = 0;
    for (long long i = 0; i < n; i++) {
        int tag = OP_SUMA;
        long long a = i, b = i + 1;
        if (tag == OP_SUMA) acc += a + b;
        else acc += 1;
    }
    return acc;
}

/* ── maps (mapa plano clave/valor, como el runtime C de LÚMEN) ── */
typedef struct { long long k, v; } KV;
static long long maps(long n) {
    KV* d = NULL;
    long long len = 0, cap = 0;
    for (long long i = 0; i < n; i++) {
        if (len == cap) { cap = cap ? cap * 2 : 8; d = (KV*)realloc(d, sizeof(KV) * cap); }
        d[len].k = i; d[len].v = i * 2; len++;
    }
    long long total = 0;
    for (long long i = 0; i < n; i++)
        for (long long j = 0; j < len; j++)
            if (d[j].k == i) { total += d[j].v; break; }
    free(d);
    return total;
}

/* ── closures: llamada a función en bucle ── */
static long long doble(long long x) { return x * 2; }
static long long closures(long n) {
    long long acc = 0;
    for (long long i = 0; i < n; i++) acc += doble(i);
    return acc;
}

/* ── unicode: conteo de codepoints UTF-8 + acceso por índice ── */
static long long utf8_len(const char* s) {
    long long n = 0;
    for (const unsigned char* p = (const unsigned char*)s; *p; p++)
        if ((*p & 0xC0) != 0x80) n++;  /* byte no-continuación = inicio de char */
    return n;
}
static long long utf8_char_len(const char* s, long long idx) {
    long long i = 0;
    const unsigned char* p = (const unsigned char*)s;
    while (*p) {
        if (i == idx) {
            const unsigned char* q = p + 1;
            while ((*q & 0xC0) == 0x80) q++;
            return (long long)(q - p);
        }
        while ((*p & 0xC0) == 0x80) p++;
        p++; i++;
    }
    return 0;
}
static long long unicode(long n) {
    const char* s = "h\xC3\xA9llo w\xC3\xB6rld \xF0\x9F\x9A\x80 caf\xC3\xA9"; /* "héllo wörld 🚀 café" */
    long long total = 0;
    for (long i = 0; i < n; i++) total += utf8_len(s) + utf8_char_len(s, 2);
    return total;
}

int main(int argc, char** argv) {
    if (argc < 2) { fprintf(stderr, "uso: bench_all <tarea>\n"); return 2; }
    const char* t = argv[1];
    if (!strcmp(t, "fib")) printf("fib:%lld\n", fib(26));
    else if (!strcmp(t, "sum")) printf("sum:%lld\n", sum(10000000));
    else if (!strcmp(t, "primes")) printf("primes:%lld\n", primes(20000));
    else if (!strcmp(t, "strings")) printf("strings:%lld\n", strings(200000));
    else if (!strcmp(t, "arrays")) printf("arrays:%lld\n", arrays(200000));
    else if (!strcmp(t, "structs")) printf("structs:%lld\n", structs(200000));
    else if (!strcmp(t, "enums")) printf("enums:%lld\n", enums(100000));
    else if (!strcmp(t, "maps")) printf("maps:%lld\n", maps(1000));
    else if (!strcmp(t, "closures")) printf("closures:%lld\n", closures(200000));
    else if (!strcmp(t, "unicode")) printf("unicode:%lld\n", unicode(200000));
    else { fprintf(stderr, "tarea desconocida: %s\n", t); return 2; }
    return 0;
}
