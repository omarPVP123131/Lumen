// Benchmark de referencia en C++ — mismos algoritmos que benchmarks/lumen/*.nv
#include <cstdio>
#include <cstdint>
#include <cstring>
#include <string>
#include <vector>

static long long fib(int n) {
    if (n < 2) return n;
    return fib(n - 1) + fib(n - 2);
}

static long long sum(long long n) {
    long long acc = 0;
    for (long long i = 0; i < n; i++) acc += i;
    return acc;
}

static bool es_primo(long long n) {
    if (n < 2) return false;
    if (n == 2) return true;
    if (n % 2 == 0) return false;
    for (long long i = 3; i * i <= n; i += 2)
        if (n % i == 0) return false;
    return true;
}

static long long primes(long long lim) {
    long long c = 0;
    for (long long k = 2; k < lim; k++)
        if (es_primo(k)) c++;
    return c;
}

static long long strings(long n) {
    long long total = 0;
    for (long i = 0; i < n; i++) {
        std::string s = "item-" + std::to_string(i) + "-fin";
        total += (long long)s.size();
    }
    return total;
}

static long long arrays(long n) {
    std::vector<long long> xs;
    xs.reserve(n);
    for (long i = 0; i < n; i++) xs.push_back(i);
    long long acc = 0;
    for (long long v : xs) acc += v;
    return acc;
}

int main(int argc, char** argv) {
    if (argc < 2) { fprintf(stderr, "uso: bench <fib|sum|primes|strings|arrays>\n"); return 2; }
    const char* t = argv[1];
    if (!strcmp(t, "fib")) printf("fib:%lld\n", fib(26));
    else if (!strcmp(t, "sum")) printf("sum:%lld\n", sum(10000000LL));
    else if (!strcmp(t, "primes")) printf("primes:%lld\n", primes(20000));
    else if (!strcmp(t, "strings")) printf("strings:%lld\n", strings(200000));
    else if (!strcmp(t, "arrays")) printf("arrays:%lld\n", arrays(200000));
    else { fprintf(stderr, "tarea desconocida: %s\n", t); return 2; }
    return 0;
}
