// Benchmark de referencia en Rust — mismos algoritmos que benchmarks/lumen/*.nv
use std::env;

fn fib(n: i64) -> i64 {
    if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
}

fn sum(n: i64) -> i64 {
    let mut acc: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        acc += i;
        i += 1;
    }
    acc
}

fn es_primo(n: i64) -> bool {
    if n < 2 { return false; }
    if n == 2 { return true; }
    if n % 2 == 0 { return false; }
    let mut i: i64 = 3;
    while i * i <= n {
        if n % i == 0 { return false; }
        i += 2;
    }
    true
}

fn primes(lim: i64) -> i64 {
    let mut c: i64 = 0;
    let mut k: i64 = 2;
    while k < lim {
        if es_primo(k) { c += 1; }
        k += 1;
    }
    c
}

fn strings(n: i64) -> i64 {
    let mut total: i64 = 0;
    let mut i: i64 = 0;
    while i < n {
        let s = format!("item-{}-fin", i);
        total += s.len() as i64;
        i += 1;
    }
    total
}

fn arrays(n: i64) -> i64 {
    let mut xs: Vec<i64> = Vec::with_capacity(n as usize);
    let mut i: i64 = 0;
    while i < n {
        xs.push(i);
        i += 1;
    }
    let mut acc: i64 = 0;
    let mut j: usize = 0;
    while j < xs.len() {
        acc += xs[j];
        j += 1;
    }
    acc
}

fn main() {
    let t = env::args().nth(1).expect("uso: bench <fib|sum|primes|strings|arrays>");
    match t.as_str() {
        "fib" => println!("fib:{}", fib(26)),
        "sum" => println!("sum:{}", sum(10_000_000)),
        "primes" => println!("primes:{}", primes(20_000)),
        "strings" => println!("strings:{}", strings(200_000)),
        "arrays" => println!("arrays:{}", arrays(200_000)),
        _ => panic!("tarea desconocida: {}", t),
    }
}
