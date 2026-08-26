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

// ============================================================
// PRODUCCIÓN: benchmarks de regresión escalable (fallthrough, defaults, matematicas, headless)
// ============================================================

fn prod_fallthrough_bench(c: &mut Criterion) {
    let source = r#"
        funcion void foo(entero r, entero g, entero b){ si r==0{retornar;} imprimir(r); }
        funcion void bar(entero r, entero g, entero b, entero a){ imprimir(a); }
        foo(0,0,0); foo(1,2,3); bar(1,2,3,99);
    "#;
    c.bench_function("prod_fallthrough_early_return", |b| {
        b.iter(|| {
            let (tokens, _) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            let (mut ast, _) = lumen_parser::Parser::new(tokens).parse();
            lumen_sema::SemanticAnalyzer::new().analyze(&mut ast);
            let ir = lumen_ir::IRBuilder::new().build(&ast);
            let (bc, _) = lumen_codegen::Codegen::new().generate(&ir);
            let mut vm = lumen_vm::VM::new(bc);
            vm.run().unwrap();
            black_box(());
        })
    });
}

fn prod_defaults_bench(c: &mut Criterion) {
    let source =
        r#"cualquiera f = funcion(entero a, entero b=10){ retornar a+b; }; f(5); f(5,20);"#;
    c.bench_function("prod_defaults_callvalue", |b| {
        b.iter(|| {
            let (tokens, _) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            let (mut ast, _) = lumen_parser::Parser::new(tokens).parse();
            lumen_sema::SemanticAnalyzer::new().analyze(&mut ast);
            let ir = lumen_ir::IRBuilder::new().build(&ast);
            let (bc, _) = lumen_codegen::Codegen::new().generate(&ir);
            let mut vm = lumen_vm::VM::new(bc);
            vm.run().unwrap();
            black_box(());
        })
    });
}

fn prod_matematicas_bench(c: &mut Criterion) {
    let source = r#"
        funcion numero potencia(numero base, entero exp){
            si(exp==0){retornar 1;} numero res=1; entero i=0;
            mientras(i<exp){res=res*base;i=i+1;} retornar res;
        }
        potencia(2,10); potencia(3,7); potencia(5,5);
    "#;
    c.bench_function("prod_matematicas_potencia", |b| {
        b.iter(|| {
            let (tokens, _) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            let (mut ast, _) = lumen_parser::Parser::new(tokens).parse();
            lumen_sema::SemanticAnalyzer::new().analyze(&mut ast);
            let ir = lumen_ir::IRBuilder::new().build(&ast);
            let (bc, _) = lumen_codegen::Codegen::new().generate(&ir);
            let mut vm = lumen_vm::VM::new(bc);
            vm.run().unwrap();
            black_box(());
        })
    });
}

fn prod_headless_bench(c: &mut Criterion) {
    // Valida el path headless centralizado es_headless() sin tocar SDL (solo pipeline)
    let source = r#"
        funcion booleano es_headless(){ si 1==1 { retornar falso; } retornar verdadero; }
        es_headless();
    "#;
    c.bench_function("prod_graficos_headless", |b| {
        b.iter(|| {
            let (tokens, _) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            let (mut ast, _) = lumen_parser::Parser::new(tokens).parse();
            lumen_sema::SemanticAnalyzer::new().analyze(&mut ast);
            let ir = lumen_ir::IRBuilder::new().build(&ast);
            let (bc, _) = lumen_codegen::Codegen::new().generate(&ir);
            black_box(bc);
        })
    });
}


fn prod_ref_mut_bench(c: &mut Criterion) {
    // v3.3+: MakeRef/write-back en bucle cerrado
    let source = r#"
        funcion vacio sumar(prestado mut entero total, entero v) { total = total + v; }
        estructura Cont { n: entero }
        impl Cont {
            funcion vacio inc(prestado mut este) { este.n = este.n + 1; }
        }
        funcion vacio main() {
            entero t = 0;
            sea c = Cont { n: 0 };
            para i en [1, 2, 3, 4, 5] {
                sumar(t, i);
                c.inc();
            }
            imprimir(t);
            imprimir(c.n);
        }
    "#;
    c.bench_function("prod_ref_mut_writeback", |b| {
        b.iter(|| {
            let (tokens, _) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            let (mut ast, _) = lumen_parser::Parser::new(tokens).parse();
            lumen_sema::SemanticAnalyzer::new().analyze(&mut ast);
            let ir = lumen_ir::IRBuilder::new().build(&ast);
            let (bc, _) = lumen_codegen::Codegen::new().generate(&ir);
            let mut vm = lumen_vm::VM::new(bc);
            let _ = vm.run();
            black_box(vm.output().to_vec());
        })
    });
}

fn prod_comptime_bench(c: &mut Criterion) {
    // v3.3+: plegado comptime con llamadas a funciones puras
    let source = r#"
        funcion entero fib(n: entero) {
            si (n < 2) { retornar n; }
            retornar fib(n - 1) + fib(n - 2);
        }
        funcion vacio main() {
            sea f = comptime { fib(15) };
            imprimir(f);
        }
    "#;
    c.bench_function("prod_comptime_fold", |b| {
        b.iter(|| {
            let (tokens, _) = lumen_lexer::Lexer::new(black_box(source)).tokenize();
            let (mut ast, _) = lumen_parser::Parser::new(tokens).parse();
            lumen_ir::comptime::ComptimeEvaluator::new(&ast).rewrite_program(&mut ast);
            black_box(ast);
        })
    });
}
criterion_group!(
    benches,
    lexer_bench,
    parser_bench,
    pipeline_bench,
    vm_exec_bench,
    prod_fallthrough_bench,
    prod_defaults_bench,
    prod_matematicas_bench,
    prod_headless_bench,
    prod_ref_mut_bench,
    prod_comptime_bench
);
criterion_main!(benches);
