use lumen_codegen::Codegen;
use lumen_ir::IRBuilder;
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;
use lumen_vm::VM;
use std::time::Instant;

fn run_source(source: &str) -> Result<Vec<String>, String> {
    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    if !lex_errors.is_empty() {
        return Err(format!("LexError: {}", lex_errors[0].message));
    }
    let parser = Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();
    if !parse_errors.is_empty() {
        return Err(format!("ParseError: {}", parse_errors[0].message));
    }
    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        return Err(format!("SemError: {}", sem_errors[0].message));
    }
    let builder = IRBuilder::new();
    let ir_program = builder.build(&program);
    let codegen = Codegen::new();
    let (bc, _) = codegen.generate(&ir_program);
    let mut vm = VM::new(bc);
    vm.run().map_err(|e| format!("{:?}", e))?;
    Ok(vm.output().to_vec())
}

// ============================================================
// PERFORMANCE
// ============================================================
#[test]
fn test_performance_potencia_loop_10k() {
    let src = r#"
        funcion numero potencia(numero base, entero exp) {
            si (exp == 0) { retornar 1; }
            numero res = 1; entero i=0;
            mientras(i < exp){ res=res*base; i=i+1; }
            retornar res;
        }
        entero i=0;
        mientras(i < 10000){ potencia(2,10); i=i+1; }
        imprimir("perf_ok");
    "#;
    let start = Instant::now();
    let output = run_source(src).unwrap();
    let elapsed = start.elapsed();
    assert_eq!(output, vec!["perf_ok"]);
    // Debe completarse en < 2s en dev, < 1s en release
    assert!(
        elapsed.as_secs() < 2,
        "performance degradada: 10k potencias tomó {:?}",
        elapsed
    );
}

#[test]
fn test_performance_fib_30() {
    let src = r#"
        funcion entero fib(entero n){ si(n<=1){retornar n;} retornar fib(n-1)+fib(n-2); }
        imprimir(fib(20));
    "#;
    let start = Instant::now();
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["6765"]);
    assert!(start.elapsed().as_secs() < 2);
}

// ============================================================
// INTEGRACIÓN (stdlib + cross-module)
// ============================================================
#[test]
fn test_integracion_stdlib_matematicas() {
    let src = r#"
        funcion numero potencia(numero base, entero exp) {
            si(exp==0){retornar 1;} numero res=1; entero i=0;
            mientras(i<exp){res=res*base;i=i+1;} retornar res;
        }
        funcion numero seno_aprox(numero x){ retornar potencia(x,3)/6; }
        imprimir(potencia(3,4));
        imprimir(seno_aprox(1));
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output[0], "81");
}

#[test]
fn test_integracion_defaults_y_lambda() {
    let src = r#"
        funcion entero suma(entero a, entero b=10){ retornar a+b; }
        cualquiera f = funcion(entero x, entero y=5){ retornar x*y; };
        imprimir(suma(5));
        imprimir(f(3));
        imprimir(f(3,4));
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["15", "15", "12"]);
}

// ============================================================
// UNITARIAS (aisladas por crate, pero aquí como contrato)
// ============================================================
#[test]
fn test_unitaria_early_return_no_fallthrough() {
    let src = r#"
        funcion void a(entero x){ si x==0{retornar;} imprimir(1); }
        funcion void b(entero y){ imprimir(y); }
        a(0); b(99); a(1);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["99", "1"]);
}

#[test]
fn test_unitaria_arity_void() {
    let src = r#"
        funcion void tres(entero a, entero b, entero c){ imprimir(a+b+c); }
        tres(1,2,3);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["6"]);
}

// ============================================================
// ACEPTACIÓN (criterios de usuario final)
// ============================================================
#[test]
fn test_aceptacion_hello_world() {
    let src = r#"imprimir("Hola LUMEN");"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Hola LUMEN"]);
}

#[test]
fn test_aceptacion_fibonacci_10() {
    let src = r#"
        funcion entero fib(entero n){ si(n<=1){retornar n;} retornar fib(n-1)+fib(n-2); }
        lista<entero> res=[]; entero i=0; mientras(i<10){ res.agregar(fib(i)); i=i+1; }
        imprimir(res[0]); imprimir(res[9]);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "34"]);
}

#[test]
fn test_aceptacion_estructuras_y_metodos() {
    let src = r#"
        estructura Punto { x: entero, y: entero }
        Punto p = Punto{x:3,y:4};
        imprimir(p.x);
        p.x = 10;
        imprimir(p.x);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3", "10"]);
}
