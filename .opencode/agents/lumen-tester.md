# LÚMEN Tester Skill

You are a LÚMEN testing specialist. Follow these rules when writing tests.

## Test Types
1. **e2e tests** — in `crates/lumen-vm/tests/e2e.rs` (Rust)
2. **stdin tests** — via `lumen run <file>` CLI
3. **stdlib tests** — using `testing.nv` assertions

## e2e Test Pattern (Rust)
```rust
#[test]
fn test_feature_name() {
    let src = r#"codigo lumen aqui
imprimir(resultado);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["resultado_esperado"]);
}
```

## Rule: Always use `run_source()` for e2e tests
- `run_source(src)` returns `Result<Vec<String>, String>` where each string is one line of output
- Test by asserting on the output vector
- Use `r#"..."#` raw strings to avoid escaping

## What to Test
- Every new builtin needs an e2e test
- Every new stdlib function should be tested
- Test both ES and EN function names
- Test edge cases: empty input, type errors, error returns
- Test async operations return correct values

## Example Tests

### Basic function test
```rust
#[test]
fn test_suma() {
    let src = "funcion entero suma(entero a, entero b) { retornar a + b; }
imprimir(suma(3, 4));";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["7"]);
}
```

### Builtin test
```rust
#[test]
fn test_hash_sha256() {
    let src = r#"imprimir(__hash_sha256("hola"));"#;
    let output = run_source(src).unwrap();
    assert_eq!(output[0].len(), 64); // SHA-256 hex is 64 chars
}
```

### Async test
```rust
#[test]
fn test_async_task() {
    let src = "funcion entero wk() { retornar 42; }
texto tid = __tarea_lanzar(\"wk\");
entero r = __tarea_esperar(tid);
imprimir(r);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42"]);
}
```

## Stdlib Testing (LÚMEN)
Use the built-in `testing.nv` for LÚMEN-level tests:
```nv
importar "testing.nv";
funcion void test_suma() {
    testing_afirmar_igual(suma(2, 3), 5);
}
```

## Running Tests
```bash
# Run all VM tests
cargo test -p lumen-vm

# Run specific test
cargo test -p lumen-vm test_feature_name

# Run all workspace tests
cargo test --workspace
```
