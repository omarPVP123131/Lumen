// Suite variada — C++ de referencia (g++ -O2).
// Mismos algoritmos que la versión C: quicksort 100k con LCG, matmul 48x48,
// sieve 1M, dict 30k (std::unordered_map). Salida: "nombre:valor".
#include <cstdio>
#include <cstdint>
#include <vector>
#include <unordered_map>

static uint64_t lcg_next(uint64_t &x) {
    x = (x * 1103515245u + 12345u) % 2147483648u;
    return x;
}

static void qs_swap(std::vector<int> &a, int i, int j) {
    int t = a[i]; a[i] = a[j]; a[j] = t;
}

static void qsort_lumen(std::vector<int> &a, int izq, int der) {
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
    std::vector<int> a(n);
    for (int i = 0; i < n; i++) a[i] = i;
    uint64_t x = 42;
    for (int i = 0; i < n; i++) {
        int j = (int)(lcg_next(x) % (uint64_t)n);
        qs_swap(a, i, j);
    }
    qsort_lumen(a, 0, n - 1);
    int64_t total = 0;
    for (int i = 0; i < n; i++) total += (int64_t)a[i] * (i + 1);
    return total;
}

static int64_t bench_matmul(int n) {
    std::vector<std::vector<int>> A(n, std::vector<int>(n)), B(n, std::vector<int>(n));
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
    return total;
}

static int bench_sieve(int lim) {
    std::vector<char> criba(lim + 1, 1);
    for (int p = 2; p * p <= lim; p++) {
        if (criba[p]) {
            for (int m = p * p; m <= lim; m += p) criba[m] = 0;
        }
    }
    int cuenta = 0;
    for (int q = 2; q <= lim; q++) cuenta += criba[q];
    return cuenta;
}

static int64_t bench_dict(int n) {
    std::unordered_map<int, int64_t> d;
    d.reserve((size_t)n * 2);
    for (int i = 0; i < n; i++) d[i] = i;
    for (int i = 0; i < n; i += 2) d[i] = d[i]; /* re-escritura sin cambio */
    int64_t total = 0;
    for (int i = 0; i < n; i++) total += d[i];
    return total;
}

int main() {
    std::printf("sort:%lld\n", (long long)bench_sort(100000));
    std::printf("matmul:%lld\n", (long long)bench_matmul(48));
    std::printf("sieve:%d\n", bench_sieve(1000000));
    std::printf("dict:%lld\n", (long long)bench_dict(30000));
    return 0;
}
