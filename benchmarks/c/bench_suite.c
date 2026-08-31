/* Suite variada — C de referencia (gcc -O2).
 * 4 tareas con checksum exacto: sort (quicksort 100k, LCG x=42),
 * matmul 48x48 (A=(i*j)%5, B=(i+j)%7), sieve 1M (78498),
 * dict 30k (hash table por sondeo lineal). Salida: "nombre:valor". */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>

/* ── sort: quicksort por índice con shuffle LCG determinista ── */
static uint64_t lcg_next(uint64_t *x) {
    *x = (*x * 1103515245u + 12345u) % 2147483648u;
    return *x;
}

static void qs_swap(int *a, int i, int j) {
    int t = a[i]; a[i] = a[j]; a[j] = t;
}

static void qsort_lumen(int *a, int izq, int der) {
    if (izq >= der) return;
    int pivote = a[izq];
    int i = izq, j = der;
    while (i <= j) {
        while (a[i] < pivote) i++;
        while (a[j] > pivote) j--;
        if (i <= j) { qs_swap(a, i, j); i++; j--; }
    }
    qsort_lumen(a, izq, j);
    qsort_lumen(a, i, der);
}

static int64_t bench_sort(int n) {
    int *a = malloc((size_t)n * sizeof(int));
    for (int i = 0; i < n; i++) a[i] = i;
    uint64_t x = 42;
    for (int i = 0; i < n; i++) {
        int j = (int)(lcg_next(&x) % (uint64_t)n);
        qs_swap(a, i, j);
    }
    qsort_lumen(a, 0, n - 1);
    int64_t total = 0;
    for (int i = 0; i < n; i++) total += (int64_t)a[i] * (i + 1);
    free(a);
    return total;
}

/* ── matmul: C = A·B con A=(i*j)%5, B=(i+j)%7 ── */
static int64_t bench_matmul(int n) {
    int (*A)[48] = malloc((size_t)n * sizeof(*A));
    int (*B)[48] = malloc((size_t)n * sizeof(*B));
    for (int i = 0; i < n; i++)
        for (int j = 0; j < n; j++) {
            A[i][j] = (i * j) % 5;
            B[i][j] = (i + j) % 7;
        }
    int64_t total = 0;
    for (int i = 0; i < n; i++)
        for (int j = 0; j < n; j++) {
            int64_t s = 0;
            for (int k = 0; k < n; k++) s += (int64_t)A[i][k] * B[k][j];
            total += s;
        }
    free(A); free(B);
    return total;
}

/* ── sieve: criba de Eratóstenes hasta 1M, cuenta de primos ── */
static int bench_sieve(int lim) {
    char *criba = calloc((size_t)lim + 1, 1);
    for (int i = 0; i <= lim; i++) criba[i] = 1;
    for (int p = 2; p * p <= lim; p++) {
        if (criba[p]) {
            for (int m = p * p; m <= lim; m += p) criba[m] = 0;
        }
    }
    int cuenta = 0;
    for (int q = 2; q <= lim; q++) cuenta += criba[q];
    free(criba);
    return cuenta;
}

/* ── dict: tabla hash de sondeo lineal, 30k entradas ── */
#define DICT_CAP 65536
#define DICT_EMPTY (-1)

static int64_t bench_dict(int n) {
    int64_t *keys = malloc(DICT_CAP * sizeof(int64_t));
    int64_t *vals = malloc(DICT_CAP * sizeof(int64_t));
    for (int i = 0; i < DICT_CAP; i++) keys[i] = DICT_EMPTY;
    for (int i = 0; i < n; i++) {
        unsigned h = (unsigned)i & (DICT_CAP - 1);
        while (keys[h] != DICT_EMPTY) h = (h + 1) & (DICT_CAP - 1);
        keys[h] = i; vals[h] = i;
    }
    for (int i = 0; i < n; i += 2) {
        unsigned h = (unsigned)i & (DICT_CAP - 1);
        while (keys[h] != i) h = (h + 1) & (DICT_CAP - 1);
        vals[h] = i; /* re-escritura sin cambio de suma */
    }
    int64_t total = 0;
    for (int i = 0; i < n; i++) {
        unsigned h = (unsigned)i & (DICT_CAP - 1);
        while (keys[h] != i) h = (h + 1) & (DICT_CAP - 1);
        total += vals[h];
    }
    free(keys); free(vals);
    return total;
}

int main(void) {
    printf("sort:%lld\n", (long long)bench_sort(100000));
    printf("matmul:%lld\n", (long long)bench_matmul(48));
    printf("sieve:%d\n", bench_sieve(1000000));
    printf("dict:%lld\n", (long long)bench_dict(30000));
    return 0;
}
