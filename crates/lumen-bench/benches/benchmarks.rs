use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn lexer_bench(c: &mut Criterion) {
    let source = r#"
funcion entero fib(entero n) {
    si n <= 1 { retornar n; }
    retornar fib(n-1) + fib(n-2);
}
imprimir(fib(20));
"#;

    c.bench_function("lexer_tokenize", |b| {
        b.iter(|| {
            let (tokens, errs) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            assert!(errs.is_empty());
            black_box(tokens);
        })
    });
}

fn parser_bench(c: &mut Criterion) {
    let source = r#"
funcion entero fib(entero n) {
    si n <= 1 { retornar n; }
    retornar fib(n-1) + fib(n-2);
}
imprimir(fib(20));
"#;
    let (tokens, _) = lumen_lexer::Lexer::new(source).tokenize();

    c.bench_function("parser_parse", |b| {
        b.iter(|| {
            let (ast, errs) = lumen_parser::Parser::new(black_box(tokens.clone())).parse();
            assert!(errs.is_empty());
            black_box(ast);
        })
    });
}

fn pipeline_bench(c: &mut Criterion) {
    let source = r#"
funcion entero fib(entero n) {
    si n <= 1 { retornar n; }
    retornar fib(n-1) + fib(n-2);
}
imprimir(fib(20));
"#;

    c.bench_function("pipeline_full", |b| {
        b.iter(|| {
            let (tokens, e) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            assert!(e.is_empty());
            let (mut ast, e2) = lumen_parser::Parser::new(tokens).parse();
            assert!(e2.is_empty());
            let errs = lumen_sema::SemanticAnalyzer::new().analyze(&mut ast);
            assert!(errs.is_empty());
            let ir = lumen_ir::IRBuilder::new().build(&ast);
            let (bc, _) = lumen_codegen::Codegen::new().generate(&ir);
            black_box(bc);
        })
    });
}

fn vm_exec_bench(c: &mut Criterion) {
    let source = r#"
funcion entero fib(entero n) {
    si n <= 1 { retornar n; }
    retornar fib(n-1) + fib(n-2);
}
imprimir(fib(20));
"#;

    c.bench_function("vm_fib_20", |b| {
        b.iter(|| {
            let (tokens, e) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            assert!(e.is_empty());
            let (mut ast, e2) = lumen_parser::Parser::new(tokens).parse();
            assert!(e2.is_empty());
            let errs = lumen_sema::SemanticAnalyzer::new().analyze(&mut ast);
            assert!(errs.is_empty());
            let ir = lumen_ir::IRBuilder::new().build(&ast);
            let (bc, _) = lumen_codegen::Codegen::new().generate(&ir);
            let mut vm = lumen_vm::VM::new(bc);
            vm.run().unwrap();
            black_box(());
        })
    });
}

criterion_group!(
    benches,
    lexer_bench,
    parser_bench,
    pipeline_bench,
    vm_exec_bench
);
criterion_main!(benches);
