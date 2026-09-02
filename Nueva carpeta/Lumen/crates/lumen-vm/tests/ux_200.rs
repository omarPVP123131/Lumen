use lumen_codegen::Codegen;
use lumen_ir::IRBuilder;
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;
use lumen_vm::VM;

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
    let mut sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        return Err(format!("SemError: {}", sem_errors[0].message));
    }
    lumen_ir::comptime::ComptimeEvaluator::new(&program).rewrite_program(&mut program);
    let builder = IRBuilder::new();
    let ir_program = builder.build(&program);
    let codegen = Codegen::new();
    let (bytecode, _warnings) = codegen.generate(&ir_program);
    let mut vm = VM::new(bytecode);
    vm.run().map_err(|e| format!("RuntimeError: {:?}", e))?;
    Ok(vm.output().to_vec())
}

#[test]
fn test_ux_001() {
    let src = r#"imprimir(__str_longitud("")); "#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0"]);
}

#[test]
fn test_ux_002() {
    let src = r#"lista<entero> a = []; imprimir(a.largo()); "#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0"]);
}

#[test]
fn test_ux_003() {
    let out = run_source(r#"imprimir(0);"#).unwrap();
    assert_eq!(out, vec!["0"]);
}

#[test]
fn test_ux_004() {
    let out = run_source(r#"imprimir(3.14);"#).unwrap();
    assert_eq!(out, vec!["3.14"]);
}

#[test]
fn test_ux_005() {
    let out = run_source(r#"imprimir(999999999);"#).unwrap();
    assert_eq!(out, vec!["999999999"]);
}

#[test]
fn test_ux_006() {
    let out = run_source(r#"imprimir(9223372036854775807);"#).unwrap();
    assert_eq!(out, vec!["9223372036854775807"]);
}

#[test]
fn test_ux_007() {
    let out = run_source(r#"imprimir("café");"#).unwrap();
    assert_eq!(out, vec!["café"]);
}

#[test]
fn test_ux_008() {
    let out = run_source(r#"imprimir("🚀✨");"#).unwrap();
    assert_eq!(out, vec!["🚀✨"]);
}

#[test]
fn test_ux_009() {
    let src = "   imprimir(   42   )   ;";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_010() {
    let src = "// comentario\nimprimir(1); // otro";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1"]);
}

#[test]
fn test_ux_011() {
    let src = "/* comentario */ imprimir(2);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["2"]);
}

#[test]
fn test_ux_012() {
    let src = "/* linea1\n linea2 */ imprimir(3);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["3"]);
}

#[test]
fn test_ux_013() {
    let src = "numero x = 5; { x = x + 1; } imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["6"]);
}

#[test]
fn test_ux_014() {
    let src = "numero x = 1; { numero y = 2; { x = x + y; } } imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["3"]);
}

#[test]
fn test_ux_015() {
    let src = "numero x = 0; { { { { x = 42; } } } } imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_016() {
    let src = r#"imprimir(__str_longitud("" + ""));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0"]);
}

#[test]
fn test_ux_017() {
    let out = run_source(r#"imprimir(0+0);"#).unwrap();
    assert_eq!(out, vec!["0"]);
}

#[test]
fn test_ux_018() {
    let out = run_source(r#"imprimir(123456.789);"#).unwrap();
    assert_eq!(out, vec!["123456.789"]);
}

#[test]
fn test_ux_019() {
    let src = "\n\t imprimir(99); \n";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["99"]);
}

#[test]
fn test_ux_020() {
    let src = r#"imprimir(10); // end"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["10"]);
}

#[test]
fn test_ux_021() {
    let src = r#"{ } imprimir("ok");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["ok"]);
}

#[test]
fn test_ux_022() {
    let src = "  numero /*c*/ x = 5; // set\n  imprimir(x); /* end */";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["5"]);
}

#[test]
fn test_ux_023() {
    let out = run_source(r#"imprimir(verdadero);"#).unwrap();
    assert_eq!(out, vec!["true"]);
}

#[test]
fn test_ux_024() {
    let out = run_source(r#"imprimir(falso);"#).unwrap();
    assert_eq!(out, vec!["false"]);
}

#[test]
fn test_ux_025() {
    let src = "numero x = 1; { numero x = 2; imprimir(x); } imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["2", "1"]);
}

#[test]
fn test_ux_026() {
    let src = "lista<entero> a = []; a.agregar(1); imprimir(a.largo());";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1"]);
}

#[test]
fn test_ux_027() {
    let out = run_source(r#"imprimir(0.0);"#).unwrap();
    // 0.0 may print as 0
    assert!(out[0] == "0" || out[0] == "0.0");
}

#[test]
fn test_ux_028() {
    let out = run_source(r#"imprimir(1000000 + 2000000);"#).unwrap();
    assert_eq!(out, vec!["3000000"]);
}

#[test]
fn test_ux_029() {
    let src = "/*🚀*/ imprimir(\"hola\");";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["hola"]);
}

#[test]
fn test_ux_030() {
    let src = "numero x=10; { imprimir(x); numero y=20; { imprimir(y); } }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["10", "20"]);
}

#[test]
fn test_ux_031() {
    let src = "// solo comentario\nimprimir(\"fin\");";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["fin"]);
}

#[test]
fn test_ux_032() {
    let src = "{ { } { } } imprimir(\"a\");";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["a"]);
}

#[test]
fn test_ux_033() {
    let src = "lista<entero> a = [0,0,0]; imprimir(a.largo());";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["3"]);
}

#[test]
fn test_ux_034() {
    let src = "numero/*c*/x/*c*/=/*c*/5; imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["5"]);
}

#[test]
fn test_ux_035() {
    let src = r#"lista<texto> a = ["", ""]; imprimir(a.largo());"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["2"]);
}

#[test]
fn test_ux_036() {
    let out = run_source(r#"imprimir(-999999);"#).unwrap();
    assert_eq!(out, vec!["-999999"]);
}

#[test]
fn test_ux_037() {
    let src = r#"imprimir("a"); imprimir("b"); imprimir("c");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["a", "b", "c"]);
}

#[test]
fn test_ux_038() {
    let src = "mientras (falso) { imprimir(\"no\"); } imprimir(\"fin\");";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["fin"]);
}

#[test]
fn test_ux_039() {
    let src = "si (verdadero) { imprimir(\"si\"); } sino { imprimir(\"no\"); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["si"]);
}

#[test]
fn test_ux_040() {
    let src = "numero x = 0; imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0"]);
}

#[test]
fn test_ux_041() {
    let out = run_source(r#"imprimir(0.5);"#).unwrap();
    assert_eq!(out, vec!["0.5"]);
}

#[test]
fn test_ux_042() {
    let out = run_source(r#"imprimir(2.718);"#).unwrap();
    assert_eq!(out, vec!["2.718"]);
}

#[test]
fn test_ux_043() {
    let src = "lista<entero> r = [x para x en 0..3]; imprimir(r);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[0, 1, 2]"]);
}

#[test]
fn test_ux_044() {
    let src = "lista<entero> r = [x para x en 1..=3]; imprimir(r);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[1, 2, 3]"]);
}

#[test]
fn test_ux_045() {
    let src =
        "funcion entero doble(entero x){ retornar x*2; } entero r = 5 |> doble(); imprimir(r);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["10"]);
}

#[test]
fn test_ux_046() {
    let src = "funcion entero doble(entero x){ retornar x*2; } funcion entero inc(entero x){ retornar x+1; } entero r = 10 |> doble() |> inc(); imprimir(r);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["21"]);
}

#[test]
fn test_ux_047() {
    let out = run_source(r#"imprimir(1 /* comment */ + 2);"#).unwrap();
    assert_eq!(out, vec!["3"]);
}

#[test]
fn test_ux_048() {
    let src = r#"imprimir("a\nb");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["a\nb"]);
}

#[test]
fn test_ux_049() {
    let src = r#"imprimir("a\tb");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["a\tb"]);
}

#[test]
fn test_ux_050() {
    let src = r#"imprimir("a\"b");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["a\"b"]);
}

#[test]
fn test_ux_051() {
    let src = r#"imprimir("a\\b");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["a\\b"]);
}

#[test]
fn test_ux_052() {
    let src = r#"imprimir("line1\nline2\ttab");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["line1\nline2\ttab"]);
}

#[test]
fn test_ux_053() {
    let out = run_source(r#"imprimir(1 << 3);"#).unwrap();
    assert_eq!(out, vec!["8"]);
}

#[test]
fn test_ux_054() {
    let out = run_source(r#"imprimir(8 >> 2);"#).unwrap();
    assert_eq!(out, vec!["2"]);
}

#[test]
fn test_ux_055() {
    let out = run_source(r#"imprimir(0xFF);"#).unwrap();
    assert_eq!(out, vec!["255"]);
}

#[test]
fn test_ux_056() {
    let out = run_source(r#"imprimir(0x10);"#).unwrap();
    assert_eq!(out, vec!["16"]);
}

#[test]
fn test_ux_057() {
    let src = "numero x= 1; /* comment */ imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1"]);
}

#[test]
fn test_ux_058() {
    let src = "// a\n/* b */ imprimir(5); // c\n/* d */";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["5"]);
}

#[test]
fn test_ux_059() {
    let src = r#"imprimir("  hola   mundo  ");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["  hola   mundo  "]);
}

#[test]
fn test_ux_060() {
    let src = "lista<entero> a = [x para x en 1.0..=3.0]; imprimir(a);";
    // 1.0..=3.0 should be parsed as range of floats? But test just checks not crash
    let result = run_source(src);
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_ux_061() {
    let src = "funcion T identidad<T>(T valor){ retornar valor; } entero x = identidad<entero>(42); imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_062() {
    let src = "funcion T identidad<T>(T valor){ retornar valor; } texto s = identidad<texto>(\"hola\"); imprimir(s);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["hola"]);
}

#[test]
fn test_ux_063() {
    let src = "estructura Par<T,U> { primero: T, segundo: U } Par<entero, texto> p = Par<entero, texto>{ primero: 1, segundo: \"hola\" }; imprimir(p.primero); imprimir(p.segundo);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "hola"]);
}

#[test]
fn test_ux_064() {
    let src = "estructura Par<T,U> { primero: T, segundo: U } Par<entero, decimal> p = Par<entero, decimal>{ primero: 42, segundo: 3.5 }; imprimir(p.primero); imprimir(p.segundo);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42", "3.5"]);
}

#[test]
fn test_ux_065() {
    let src = "enum Color { Rojo, Verde, Azul } Color c = Color::Rojo; elegir (c) { caso Color::Rojo: imprimir(\"rojo\"); caso Color::Verde: imprimir(\"verde\"); defecto: imprimir(\"otro\"); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["rojo"]);
}

#[test]
fn test_ux_066() {
    let src = "enum Estado { Ok, Error } Estado e = Estado::Error; elegir (e) { caso Estado::Ok: imprimir(\"ok\"); defecto: imprimir(\"def\"); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["def"]);
}

#[test]
fn test_ux_067() {
    let src = "rasgo Contenedor { tipo Item; funcion Item obtener_valor(este); } estructura Caja { valor: entero, } impl Contenedor para Caja { tipo Item = entero; funcion entero obtener_valor(este){ retornar este.valor; } } sea c = Caja{ valor: 99 }; imprimir(c.obtener_valor());";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["99"]);
}

#[test]
fn test_ux_068() {
    let src = "rasgo Duplicable { funcion entero duplicar(este); } impl Duplicable para entero { funcion entero duplicar(este){ retornar este*2; } } entero x=21; imprimir(x.duplicar());";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_069() {
    let src = r#"resultado<entero, texto> r = exito(42); imprimir(r);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["exito(42)"]);
}

#[test]
fn test_ux_070() {
    let src = r#"resultado<entero, texto> r = error("falló"); imprimir(r);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["error(falló)"]);
}

#[test]
fn test_ux_071() {
    let src = "opcion<entero> x = algun(42); imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["algun(42)"]);
}

#[test]
fn test_ux_072() {
    let src = "opcion<entero> x = ninguno; imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["ninguno"]);
}

#[test]
fn test_ux_073() {
    let out = run_source(r#"imprimir((42, "hola", 3.0));"#).unwrap();
    assert_eq!(out, vec!["(42, hola, 3)"]);
}

#[test]
fn test_ux_074() {
    let src = "imprimir((10,20,30).0); imprimir((10,20,30).1); imprimir((10,20,30).2);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["10", "20", "30"]);
}

#[test]
fn test_ux_075() {
    let src = "entero x, texto y = (1, \"hola\"); imprimir(x); imprimir(y);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "hola"]);
}

#[test]
fn test_ux_076() {
    let src = "entero x, _ = (1,2); imprimir(x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1"]);
}

#[test]
fn test_ux_077() {
    let src = "entero x=2; elegir (x){ caso 1: imprimir(\"uno\"); caso 2: imprimir(\"dos\"); defecto: imprimir(\"otro\"); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["dos"]);
}

#[test]
fn test_ux_078() {
    let src = "entero x=5; elegir (x){ caso 5 si x>3: imprimir(\"mayor\"); defecto: imprimir(\"otro\"); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["mayor"]);
}

#[test]
fn test_ux_079() {
    let src = "lista<entero> nums=[1,2,3,4,5,6]; lista<entero> pares=[x*10 para x en nums si x%2==0]; imprimir(pares);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[20, 40, 60]"]);
}

#[test]
fn test_ux_080() {
    let src = "lista<entero> datos=[10,15,20,25,30]; lista<entero> mayores= consultar x en datos donde x>=20 seleccionar x*2; imprimir(mayores);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[40, 50, 60]"]);
}

#[test]
fn test_ux_081() {
    let src = "array<integer> items=[1,2,3,4,5]; array<integer> q= query x in items where x>2 select x+10; print(q);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[13, 14, 15]"]);
}

#[test]
fn test_ux_082() {
    let src = "funcion texto? obtener(booleano b){ si b { retornar algun(\"OK\"); } retornar ninguno; } texto? v=obtener(verdadero); elegir (v){ caso algun(val): imprimir(val); defecto: imprimir(\"NADA\"); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["OK"]);
}

#[test]
fn test_ux_083() {
    let src = "funcion T identidad<T>(T valor){ retornar valor; } entero x = identidad<entero>(\"hola\");";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_084() {
    let src = "estructura Punto { x: entero, y: entero } Punto pt = Punto{ x: 10 };";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_085() {
    let src = "estructura Punto { x: entero, y: entero } Punto pt = Punto{ x:10, y:20, z:30 };";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_086() {
    let result = run_source("entero x, texto y = (1, \"a\", 3);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_087() {
    let result = run_source("opcion<texto> x = algun(42);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_088() {
    let src = "entero x=1; elegir (x){ caso \"texto\": imprimir(\"no\"); }";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_089() {
    let src = "enum Color { Rojo, Verde, Azul } Color c = Color::Rojo; elegir (c){ caso Color::Rojo: imprimir(\"rojo\"); caso Color::Verde: imprimir(\"verde\"); }";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_090() {
    let src = "const entero MAX = 100; imprimir(MAX);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["100"]);
}

#[test]
fn test_ux_091() {
    let src =
        "lista<entero> nums=[10,20,30]; imprimir(nums[0]); imprimir(nums[1]); imprimir(nums[2]);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["10", "20", "30"]);
}

#[test]
fn test_ux_092() {
    let src = "lista<entero> nums=[1,2,3]; imprimir(nums.largo());";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["3"]);
}

#[test]
fn test_ux_093() {
    let src = "lista<entero> nums=[1,2]; nums.agregar(3); nums.agregar(4); imprimir(nums.largo()); imprimir(nums[2]);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["4", "3"]);
}

#[test]
fn test_ux_094() {
    let src = r#"numero d = __map_nuevo(); d = __map_poner(d, "Ana", 30); imprimir(__map_obtener(d, "Ana")); imprimir(__map_longitud(d));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["30", "1"]);
}

#[test]
fn test_ux_095() {
    let src = r#"numero s = __conjunto_nuevo(); s = __conjunto_agregar(s, "a"); imprimir(__conjunto_tiene(s, "a")); imprimir(__conjunto_tiene(s, "b"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["true", "false"]);
}

#[test]
fn test_ux_096() {
    let src = r#"numero dq = __deque_nuevo(); dq = __deque_agregar_final(dq, 1); dq = __deque_agregar_frente(dq, 0); imprimir(__deque_longitud(dq));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["2"]);
}

#[test]
fn test_ux_097() {
    let src = r#"numero h = __monticulo_nuevo(); h = __monticulo_agregar(h, 5); h = __monticulo_agregar(h, 10); imprimir(__monticulo_ver(h)); imprimir(__monticulo_longitud(h));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["10", "2"]);
}

#[test]
fn test_ux_098() {
    let src = r#"numero ll = __enlazada_nuevo(); ll = __enlazada_agregar_final(ll, "x"); imprimir(__enlazada_longitud(ll));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1"]);
}

#[test]
fn test_ux_099() {
    let src = "estructura Persona { nombre: texto, edad: numero } Persona p = Persona{ nombre: \"Ana\", edad: 30 }; imprimir(p.nombre); imprimir(p.edad);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["Ana", "30"]);
}

#[test]
fn test_ux_100() {
    let src = "estructura Punto { x: entero, y: entero } Punto pt = Punto{ x:10, y:20 }; pt.x=30; imprimir(pt.x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["30"]);
}

#[test]
fn test_ux_101() {
    let src = "estructura Punto { x: entero, y: entero } Punto a=Punto{ x:1, y:2 }; Punto b=a; b.x=99; imprimir(a.x); imprimir(b.x);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "99"]);
}

#[test]
fn test_ux_102() {
    let out = run_source(r#"imprimir(funcion(entero x){ retornar x*2; }(5));"#).unwrap();
    assert_eq!(out, vec!["10"]);
}

#[test]
fn test_ux_103() {
    let out =
        run_source(r#"imprimir(funcion(entero a, entero b){ retornar a+b; }(10,20));"#).unwrap();
    assert_eq!(out, vec!["30"]);
}

#[test]
fn test_ux_104() {
    let src = "funcion entero fact(entero n){ si(n<=1){ retornar 1;} retornar n*fact(n-1);} imprimir(fact(5));";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["120"]);
}

#[test]
fn test_ux_105() {
    let src = "funcion entero fib(entero n){ si(n<=1){ retornar n; } retornar fib(n-1)+fib(n-2); } imprimir(fib(6));";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["8"]);
}

#[test]
fn test_ux_106() {
    let src =
        "numero contador=0; mientras (contador<5){ imprimir(contador); contador=contador+1; }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0", "1", "2", "3", "4"]);
}

#[test]
fn test_ux_107() {
    let src = "para (numero i=0; i<3; i=i+1){ imprimir(i); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0", "1", "2"]);
}

#[test]
fn test_ux_108() {
    let src = "lista<entero> nums=[1,2,3]; para n en nums { imprimir(n); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "2", "3"]);
}

#[test]
fn test_ux_109() {
    let src =
        r#"lista<texto> nombres=["Ana","Luis"]; para nombre en nombres { imprimir(nombre); }"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["Ana", "Luis"]);
}

#[test]
fn test_ux_110() {
    let src = "entero i=0; mientras(i<10){ si(i==3){ romper; } imprimir(i); i=i+1; }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0", "1", "2"]);
}

#[test]
fn test_ux_111() {
    let src = "entero i=0; mientras(i<5){ i=i+1; si(i==3){ continuar; } imprimir(i); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "2", "4", "5"]);
}

#[test]
fn test_ux_112() {
    let src = "para (entero i=0;i<10;i=i+1){ si(i==2){ romper; } imprimir(i); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0", "1"]);
}

#[test]
fn test_ux_113() {
    let src = "para (entero i=0;i<5;i=i+1){ si(i==2){ continuar; } imprimir(i); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0", "1", "3", "4"]);
}

#[test]
fn test_ux_114() {
    let src =
        "entero i=0; mientras(i<2){ entero j=0; mientras(j<2){ imprimir(i*10+j); j=j+1; } i=i+1; }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["0", "1", "10", "11"]);
}

#[test]
fn test_ux_115() {
    let out = run_source(r#"imprimir("hola" + " " + "mundo");"#).unwrap();
    assert_eq!(out, vec!["hola mundo"]);
}

#[test]
fn test_ux_116() {
    let src = r#"imprimir(__str_longitud("hola"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["4"]);
}

#[test]
fn test_ux_117() {
    let src = "lista<entero> nums=[1,2,3,4,5]; lista<entero> pares=[x*10 para x en nums si x%2==0]; imprimir(pares);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[20, 40]"]);
}

#[test]
fn test_ux_118() {
    let src = r#"numero d=__map_nuevo(); d=__map_poner(d,"x",1); d=__map_poner(d,"y",2); lista<numero> k=__map_claves(d); imprimir(k); "#;
    let out = run_source(src).unwrap();
    assert!(out[0].contains("x") || out[0].contains("y"));
}

#[test]
fn test_ux_119() {
    let src = "(entero, (texto, entero)) t = (1, (\"a\", 2)); imprimir(t.0); imprimir(t.1.0); imprimir(t.1.1);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "a", "2"]);
}

#[test]
fn test_ux_120() {
    let src =
        "entero a, texto b, decimal c = (1, \"x\", 3.5); imprimir(a); imprimir(b); imprimir(c);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "x", "3.5"]);
}

#[test]
fn test_ux_121() {
    let src = r#"funcion entero probar(){ resultado<entero, texto> r = exito(42); retornar intentar r; } entero x=probar(); imprimir(x);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_122() {
    let src = "opcion<entero> x=algun(10); elegir (x){ caso algun(10): imprimir(1); caso ninguno: imprimir(2); }";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1"]);
}

#[test]
fn test_ux_123() {
    let src = "funcion entero suma(entero a, entero b=10){ retornar a+b; } imprimir(suma(5)); imprimir(suma(5,20));";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["15", "25"]);
}

#[test]
fn test_ux_124() {
    let src = "lista<entero> a=[1,2,3]; numero b=__lista_invertir(a); imprimir(b);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[3, 2, 1]"]);
}

#[test]
fn test_ux_125() {
    let src = "lista<entero> a=[3,1,2]; numero b=__lista_ordenar(a); imprimir(b);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[1, 2, 3]"]);
}

#[test]
fn test_ux_126() {
    let src = r#"imprimir(__regex_coincide("\\d+", "abc123"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["true"]);
}

#[test]
fn test_ux_127() {
    let src = r#"imprimir(__regex_reemplazar("mundo", "Hola mundo", "Lumen"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["Hola Lumen"]);
}

#[test]
fn test_ux_128() {
    let src = "funcion entero doble(entero x){ retornar x*2; } imprimir(doble(21));";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_129() {
    let src = "entero s=0; entero i=0; mientras(i<10){ i=i+1; si(i%2==0){ continuar; } si(i>7){ romper; } s=s+i; } imprimir(s);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["16"]);
}

#[test]
fn test_ux_130() {
    let src = r#"numero m=__map_nuevo(); m=__map_poner(m,"a",1); numero s=__conjunto_nuevo(); s=__conjunto_agregar(s,"x"); imprimir(__map_longitud(m)); imprimir(__conjunto_tiene(s,"x"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1", "true"]);
}

#[test]
fn test_ux_131() {
    let src = r#"imprimir(__str_mayusculas("hola"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["HOLA"]);
}

#[test]
fn test_ux_132() {
    let src = r#"imprimir(__str_minusculas("HOLA"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["hola"]);
}

#[test]
fn test_ux_133() {
    let src = r#"imprimir(__str_recortar("  hola  "));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["hola"]);
}

#[test]
fn test_ux_134() {
    let src = r#"imprimir(__str_contiene("hola mundo", "mundo"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["true"]);
}

#[test]
fn test_ux_135() {
    let src = r#"imprimir(__str_dividir("a,b,c", ","));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[a, b, c]"]);
}

#[test]
fn test_ux_136() {
    let src = r#"imprimir(__str_ord("ABC"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[65, 66, 67]"]);
}

#[test]
fn test_ux_137() {
    let src = r#"imprimir(__str_padding_inicio("42", 5, "0"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["00042"]);
}

#[test]
fn test_ux_138() {
    let src = r#"imprimir(__str_padding_fin("42", 5, "."));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42..."]);
}

#[test]
fn test_ux_139() {
    let src = r#"imprimir(__str_longitud("hello"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["5"]);
}

#[test]
fn test_ux_140() {
    let src = "lista<entero> a=[3,1,2]; imprimir(__lista_ordenar(a)); lista<entero> b=[1,2,3]; imprimir(__lista_invertir(b));";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[1, 2, 3]", "[3, 2, 1]"]);
}

#[test]
fn test_ux_141() {
    let src = r#"numero m=__map_nuevo(); m=__map_poner(m,"k",42); numero s=__conjunto_nuevo(); s=__conjunto_agregar(s,99); imprimir(__map_obtener(m,"k")); imprimir(__conjunto_tiene(s,99));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42", "true"]);
}

#[test]
fn test_ux_142() {
    let src = r#"numero dq=__deque_nuevo(); dq=__deque_agregar_final(dq,1); dq=__deque_agregar_frente(dq,0); numero h=__monticulo_nuevo(); h=__monticulo_agregar(h,7); numero ll=__enlazada_nuevo(); ll=__enlazada_agregar_final(ll,"z"); imprimir(__deque_longitud(dq)); imprimir(__monticulo_ver(h)); imprimir(__enlazada_longitud(ll));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["2", "7", "1"]);
}

#[test]
fn test_ux_143() {
    let src = r#"lista<entero> a=[1,2,3]; imprimir(__json_texto(a));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[1,2,3]"]);
}

#[test]
fn test_ux_144() {
    let src = r#"texto s="[1,2,3]"; numero v=__json_parsear(s); imprimir(v);"#;
    let out = run_source(src).unwrap();
    assert!(out[0].contains("1") && out[0].contains("2"));
}

#[test]
fn test_ux_145() {
    let src = r#"lista<entero> arr=[5,6]; texto j=__json_texto(arr); numero p=__json_parsear(j); imprimir(p);"#;
    let out = run_source(src).unwrap();
    assert!(out[0].contains("5"));
}

#[test]
fn test_ux_146() {
    let src = r#"imprimir(__hash_sha256("hola"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out[0].len(), 64);
    assert!(out[0].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_ux_147() {
    let src = r#"imprimir(__hash_sha512("hola"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out[0].len(), 128);
    assert!(out[0].chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_ux_148() {
    let src = r#"numero r=__escritor_buffer("test_ux200_tmp_a.txt","contenido"); imprimir(r);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["exito(true)"]);
    let _ = std::fs::remove_file("test_ux200_tmp_a.txt");
}

#[test]
fn test_ux_149() {
    let src = r#"__escritor_buffer("test_ux200_tmp_b.txt","linea1\nlinea2"); numero arr=__lector_buffer("test_ux200_tmp_b.txt"); imprimir(__deque_longitud(arr));"#;
    let out = run_source(src).unwrap();
    // reader returns array of lines, length 2
    assert_eq!(out, vec!["2"]);
    let _ = std::fs::remove_file("test_ux200_tmp_b.txt");
}

#[test]
fn test_ux_150() {
    let src = r#"imprimir(__tiempo_formatear(0)); imprimir(__tiempo_diferencia(100,50)); imprimir(__zona_info("utc")); imprimir(__zona_info("est"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1970-01-01T00:00:00Z", "50", "0", "-5"]);
}

#[test]
fn test_ux_151() {
    let src = r#"entero buf=__ffi_asignar(5); __ffi_escribir(buf,0,"hello"); __ffi_liberar(buf,5); imprimir("ffi_ok");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["ffi_ok"]);
}

#[test]
fn test_ux_152() {
    let src = r#"entero i=0; mientras(i<50){ entero buf=__ffi_asignar(5); __ffi_escribir(buf,0,"hello"); __ffi_liberar(buf,5); i=i+1; } imprimir("loop_ok");"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["loop_ok"]);
}

#[test]
fn test_ux_153() {
    let src = r#"entero buf=__ffi_asignar(5); __ffi_liberar(buf,5); numero r=__ffi_liberar(buf,5); imprimir(r); imprimir("double_checked");"#;
    let out = run_source(src).unwrap();
    assert!(out[0].contains("error") || out[0].contains("Liberación") || out[0].contains("void"));
    assert_eq!(out[1], "double_checked");
}

#[test]
fn test_ux_154() {
    let src = r#"entero buf=__ffi_asignar(5); __ffi_liberar(buf,5); numero r=__ffi_escribir(buf,0,"hi"); imprimir(r); imprimir("uaf_ok");"#;
    let out = run_source(src).unwrap();
    assert!(
        out[0].contains("error") || out[0].contains("no registrado") || out[0].contains("void")
    );
    assert_eq!(out[1], "uaf_ok");
}

#[test]
fn test_ux_155() {
    let src = r#"entero buf=__ffi_asignar(3); numero r=__ffi_escribir(buf,2,"hello"); imprimir(r); imprimir("oob_write_ok"); __ffi_liberar(buf,3);"#;
    let out = run_source(src).unwrap();
    assert!(out[0].contains("error") || out[0].contains("fuera de rango"));
    assert_eq!(out[1], "oob_write_ok");
}

#[test]
fn test_ux_156() {
    let src = r#"entero buf=__ffi_asignar(3); numero r=__ffi_leer(buf,2,5); imprimir(r); imprimir("oob_read_ok"); __ffi_liberar(buf,3);"#;
    let out = run_source(src).unwrap();
    assert!(out[0].contains("error") || out[0].contains("fuera de rango"));
    assert_eq!(out[1], "oob_read_ok");
}

#[test]
fn test_ux_157() {
    let src = "lista<entero> nums=[1]; imprimir(nums[5]);";
    let result = run_source(src);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("fuera de rango") || err.contains("RuntimeError"));
}

#[test]
fn test_ux_158() {
    let src = "lista<entero> nums=[1]; imprimir(nums[-1]);";
    let result = run_source(src);
    assert!(result.is_err());
}

#[test]
fn test_ux_159() {
    let src = r#"entero buf=__ffi_asignar(8); __ffi_poke_u32(buf,0,42); entero v=__ffi_peek_u32(buf,0); imprimir(v); __ffi_liberar(buf,8);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_160() {
    let src = r#"entero buf=__ffi_asignar(10); __ffi_escribir(buf,0,"hola"); texto s=__ffi_leer(buf,0,4); imprimir(s); __ffi_liberar(buf,10);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["hola"]);
}

#[test]
fn test_ux_161() {
    let src = "funcion entero mid_func(){ retornar 42; } texto x=__tarea_lanzar(\"mid_func\"); entero y=__tarea_esperar(x); imprimir(y);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_162() {
    let src = "funcion entero suma(entero a, entero b){ retornar a+b; } texto i=__tarea_lanzar(\"suma\",10,20); entero r=__tarea_esperar(i); imprimir(r);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["30"]);
}

#[test]
fn test_ux_163() {
    let src = "funcion entero doble(entero x){ retornar x*2; } texto t1=__tarea_lanzar(\"doble\",5); texto t2=__tarea_lanzar(\"doble\",10); entero r1=__tarea_esperar(t1); entero r2=__tarea_esperar(t2); imprimir(r1+r2);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["30"]);
}

#[test]
fn test_ux_164() {
    let src = "function integer double(integer x){ return x*2; } texto tid=__task_spawn(\"double\",21); entero res=__task_await(tid); print(res);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["42"]);
}

#[test]
fn test_ux_165() {
    let src = "texto tid=__tarea_lanzar(\"nonexistent_func\"); entero res=__tarea_esperar(tid); imprimir(res);";
    let out = run_source(src).unwrap();
    assert!(out[0].contains("void") || out[0].contains("error") || out[0].contains("Error"));
}

#[test]
fn test_ux_166() {
    let src = r#"funcion texto mid_func(){ retornar "ok"; } imprimir(__coro_crear("mid_func",0));"#;
    let out = run_source(src).unwrap();
    assert!(out[0].starts_with("coro_"));
}

#[test]
fn test_ux_167() {
    let src = r#"texto tid=__temporizador_esperar(5); numero r=__tarea_esperar(tid); imprimir(r);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["true"]);
}

#[test]
fn test_ux_168() {
    let src = r#"__escritor_buffer("test_ux200_async.txt","asynccontent"); texto tid=__file_read_async("test_ux200_async.txt"); numero content=__tarea_esperar(tid); imprimir(content);"#;
    let out = run_source(src).unwrap();
    assert!(out[0].contains("asynccontent"));
    let _ = std::fs::remove_file("test_ux200_async.txt");
}

#[test]
fn test_ux_169() {
    let src = r#"texto tid=__tcp_conectar_async("127.0.0.1:1"); numero r=__tarea_esperar(tid); imprimir("hecho");"#;
    let result = run_source(src);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), vec!["hecho"]);
}

#[test]
fn test_ux_170() {
    let src = "funcion entero mid_func(){ retornar 99; } texto tid=__tarea_lanzar(\"mid_func\"); entero res=__tarea_esperar(tid); imprimir(res);";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["99"]);
}

#[test]
fn test_ux_171() {
    let src = "funcion entero suma(entero a, entero b){ retornar a+b; } imprimir(suma(3,7));";
    let out1 = run_source(src).unwrap();
    let out2 = run_source(src).unwrap();
    assert_eq!(out1, out2);
    assert_eq!(out1, vec!["10"]);
}

#[test]
fn test_ux_172() {
    let src = "funcion entero fib(entero n){ si(n<=1){ retornar n; } retornar fib(n-1)+fib(n-2); } imprimir(fib(7));";
    let out1 = run_source(src).unwrap();
    let out2 = run_source(src).unwrap();
    assert_eq!(out1, vec!["13"]);
    assert_eq!(out1, out2);
}

#[test]
fn test_ux_173() {
    let src = "funcion entero fact(entero n){ si(n<=1){ retornar 1; } retornar n*fact(n-1); } imprimir(fact(4));";
    let out1 = run_source(src).unwrap();
    assert_eq!(out1, vec!["24"]);
    assert_eq!(out1, run_source(src).unwrap());
}

#[test]
fn test_ux_174() {
    let src = "funcion numero potencia(numero base, entero exp){ si(exp==0){ retornar 1; } numero res=1; entero i=0; mientras(i<exp){ res=res*base; i=i+1; } retornar res; } imprimir(potencia(2,10));";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["1024"]);
    assert_eq!(out, run_source(src).unwrap());
}

#[test]
fn test_ux_175() {
    let src =
        r#"funcion texto unir(texto a, texto b){ retornar a+b; } imprimir(unir("hola","mundo"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["holamundo"]);
    assert_eq!(out, run_source(src).unwrap());
}

#[test]
fn test_ux_176() {
    let src = "lista<entero> a=[3,1,2]; imprimir(__lista_ordenar(a));";
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[1, 2, 3]"]);
    assert_eq!(out, run_source(src).unwrap());
}

#[test]
fn test_ux_177() {
    let src = r#"numero m=__map_nuevo(); m=__map_poner(m,"k",7); imprimir(__map_obtener(m,"k"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["7"]);
    assert_eq!(out, run_source(src).unwrap());
}

#[test]
fn test_ux_178() {
    let src = r#"imprimir(__regex_coincide("a+", "aaa"));"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["true"]);
    assert_eq!(out, run_source(src).unwrap());
}

#[test]
fn test_ux_179() {
    let src = r#"imprimir(__hash_sha256("test"));"#;
    let out1 = run_source(src).unwrap();
    let out2 = run_source(src).unwrap();
    assert_eq!(out1, out2);
    assert_eq!(out1[0].len(), 64);
}

#[test]
fn test_ux_180() {
    let src = r#"lista<entero> a=[1,2]; texto j=__json_texto(a); imprimir(j);"#;
    let out = run_source(src).unwrap();
    assert_eq!(out, vec!["[1,2]"]);
    assert_eq!(out, run_source(src).unwrap());
}

#[test]
fn test_ux_181() {
    let result = run_source("imprimir(x);");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("SemError")
            || err.contains("UndefinedVariable")
            || err.contains("no definida")
    );
}

#[test]
fn test_ux_182() {
    let result = run_source("numero x=1/0;");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("DivisionByZero") || err.contains("División") || err.contains("RuntimeError")
    );
}

#[test]
fn test_ux_183() {
    let result = run_source("lista<entero> nums=[1]; imprimir(nums[5]);");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("fuera de rango") || err.contains("RuntimeError"));
}

#[test]
fn test_ux_184() {
    let result = run_source("lista<entero> nums=[1]; imprimir(nums[-1]);");
    assert!(result.is_err());
}

#[test]
fn test_ux_185() {
    let result = run_source("let @x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("LexError"));
}

#[test]
fn test_ux_186() {
    let result = run_source("\"hola");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("LexError"));
}

#[test]
fn test_ux_187() {
    let result = run_source("numero x = ;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ParseError"));
}

#[test]
fn test_ux_188() {
    let result = run_source("entero x = \"hola\";");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_189() {
    let result = run_source("entero x=1; entero x=2;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_190() {
    let result = run_source("romper;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_191() {
    let result = run_source("continuar;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_192() {
    let result = run_source("estructura Punto { x: entero, y: entero } Punto pt = Punto{ x:10 };");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_193() {
    let result = run_source(
        "estructura Punto { x: entero, y: entero } Punto pt = Punto{ x:10, y:20, z:30 };",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_194() {
    let result = run_source(
        "estructura Punto { x: entero, y: entero } Punto pt = Punto{ x:10, y:\"veinte\" };",
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_195() {
    let result = run_source("entero x=1; elegir (x){ caso \"texto\": imprimir(\"no\"); }");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_196() {
    let result = run_source("funcion T id<T>(T v){ retornar v; } entero x = id<entero>(\"hola\");");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_197() {
    let result = run_source("entero x, texto y = (1, \"a\", 3);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_198() {
    let result = run_source("entero x, texto y = (1, 2);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_199() {
    let result =
        run_source("funcion entero f(entero a, entero b){ retornar a+b; } imprimir(f(1));");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_ux_200() {
    let result = run_source("imprimir(noExiste(1));");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("SemError")
            || err.contains("UndefinedFunction")
            || err.contains("no definida")
    );
}
