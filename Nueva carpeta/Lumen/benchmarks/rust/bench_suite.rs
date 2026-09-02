// Suite variada — Rust de referencia (rustc -O).
// Mismos algoritmos que la versión C: quicksort 100k con LCG, matmul 48x48,
// sieve 1M, dict 30k (HashMap). Salida: "nombre:valor".
use std::collections::HashMap;

fn lcg_next(x: &mut u64) -> u64 {
    *x = (*x * 1103515245u64 + 12345u64) % 2147483648u64;
    *x
}

fn qs_swap(a: &mut [i32], i: usize, j: usize) {
    a.swap(i, j);
}

fn qsort_lumen(a: &mut [i32], izq: usize, der: usize) {
    if izq >= der {
        return;
    }
    let pivote = a[izq];
    let mut i = izq;
    let mut j = der;
    while i <= j {
        while a[i] < pivote {
            i += 1;
        }
        while a[j] > pivote {
            j -= 1;
        }
        if i <= j {
            qs_swap(a, i, j);
            i += 1;
            if j == 0 {
                break;
            }
            j -= 1;
        }
    }
    if j > izq {
        qsort_lumen(a, izq, j);
    }
    qsort_lumen(a, i, der);
}

fn bench_sort(n: usize) -> i64 {
    let mut a: Vec<i32> = (0..n as i32).collect();
    let mut x: u64 = 42;
    for i in 0..n {
        let j = (lcg_next(&mut x) % n as u64) as usize;
        a.swap(i, j);
    }
    qsort_lumen(a.as_mut_slice(), 0, n - 1);
    let mut total: i64 = 0;
    for i in 0..n {
        total += a[i] as i64 * (i as i64 + 1);
    }
    total
}

fn bench_matmul(n: usize) -> i64 {
    let mut a = vec![vec![0i64; n]; n];
    let mut b = vec![vec![0i64; n]; n];
    for i in 0..n {
        for j in 0..n {
            a[i][j] = ((i * j) % 5) as i64;
            b[i][j] = ((i + j) % 7) as i64;
        }
    }
    let mut total: i64 = 0;
    for i in 0..n {
        for j in 0..n {
            let mut s: i64 = 0;
            for k in 0..n {
                s += a[i][k] * b[k][j];
            }
            total += s;
        }
    }
    total
}

fn bench_sieve(lim: usize) -> i32 {
    let mut criba = vec![1u8; lim + 1];
    let mut p = 2usize;
    while p * p <= lim {
        if criba[p] == 1 {
            let mut m = p * p;
            while m <= lim {
                criba[m] = 0;
                m += p;
            }
        }
        p += 1;
    }
    let mut cuenta: i32 = 0;
    for q in 2..=lim {
        cuenta += criba[q] as i32;
    }
    cuenta
}

fn bench_dict(n: usize) -> i64 {
    let mut d: HashMap<i32, i64> = HashMap::with_capacity(n * 2);
    for i in 0..n {
        d.insert(i as i32, i as i64);
    }
    // Fase 2: re-escrituras (mismo valor) sobre claves pares.
    for i in (0..n).step_by(2) {
        let v = d[&(i as i32)];
        d.insert(i as i32, v);
    }
    let mut total: i64 = 0;
    for i in 0..n {
        total += d[&(i as i32)];
    }
    total
}

fn main() {
    println!("sort:{}", bench_sort(100000));
    println!("matmul:{}", bench_matmul(48));
    println!("sieve:{}", bench_sieve(1000000));
    println!("dict:{}", bench_dict(30000));
}
