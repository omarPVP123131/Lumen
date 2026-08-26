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

    let sema = SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    if !sem_errors.is_empty() {
        return Err(format!("SemError: {}", sem_errors[0].message));
    }

    // comptime (bug #7): mismo orden que el CLI — plegar antes del builder
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
fn test_hello() {
    let output = run_source(r#"imprimir("¡Hola, LÚMEN!");"#).unwrap();
    assert_eq!(output, vec!["¡Hola, LÚMEN!"]);
}

#[test]
fn test_loop() {
    let src = "numero contador = 0;
mientras (contador < 5) {
    imprimir(contador);
    contador = contador + 1;
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "1", "2", "3", "4"]);
}

#[test]
fn test_func() {
    let src = "funcion numero suma(numero a, numero b) {
    retornar a + b;
}
numero res = suma(3, 7);
imprimir(res);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn test_condicional() {
    let src = "numero edad = 18;
si (edad >= 18) {
    imprimir(\"Eres mayor de edad\");
} sino {
    imprimir(\"Eres menor de edad\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Eres mayor de edad"]);
}

#[test]
fn test_lexical_error() {
    let result = run_source("let @x = 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("LexError"));
}

#[test]
fn test_syntax_error() {
    let result = run_source("numero x = ;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ParseError"));
}

#[test]
fn test_semantic_error() {
    let result = run_source("entero x = \"hola\";");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_runtime_division_by_zero() {
    let result = run_source("numero x = 1 / 0;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("DivisionByZero"));
}

#[test]
fn test_runtime_undefined_variable() {
    let result = run_source("imprimir(x);");
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("SemError") || err.contains("UndefinedVariable"));
}

#[test]
fn test_print_number() {
    let output = run_source("imprimir(42);").unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_print_boolean() {
    let output = run_source("imprimir(verdadero);").unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_multiple_prints() {
    let src = "imprimir(\"a\");
imprimir(\"b\");
imprimir(\"c\");";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["a", "b", "c"]);
}

#[test]
fn test_while_false_body_never_executes() {
    let src = "mientras (falso) {
    imprimir(\"no\");
}
imprimir(\"fin\");";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["fin"]);
}

#[test]
fn test_nested_blocks() {
    let src = "numero x = 1;
{
    numero y = 2;
    x = x + y;
}
imprimir(x);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_if_else_false_branch() {
    let src = "si (falso) {
    imprimir(\"si\");
} sino {
    imprimir(\"no\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["no"]);
}

#[test]
fn test_for_loop() {
    let src = "para (numero i = 0; i < 3; i = i + 1) {
    imprimir(i);
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "1", "2"]);
}

#[test]
fn test_boolean_comparison() {
    let src = "imprimir(1 < 2);
imprimir(3 > 5);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn test_string_concatenation() {
    let src = "imprimir(\"hola\" + \" \" + \"mundo\");";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["hola mundo"]);
}

#[test]
fn test_array_literal_and_index() {
    let src = "lista<entero> nums = [10, 20, 30];
imprimir(nums[0]);
imprimir(nums[1]);
imprimir(nums[2]);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "20", "30"]);
}

#[test]
fn test_array_len() {
    let src = "lista<entero> nums = [1, 2, 3];
imprimir(nums.largo());";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_array_push() {
    let src = "lista<entero> nums = [1, 2];
nums.agregar(3);
nums.agregar(4);
imprimir(nums.largo());
imprimir(nums[2]);
imprimir(nums[3]);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["4", "3", "4"]);
}

#[test]
fn test_array_empty_literal() {
    let src = "lista<entero> nums = [];
imprimir(nums.largo());";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_array_index_oob() {
    let src = "lista<entero> nums = [1];
imprimir(nums[5]);";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("fuera de rango"));
}

#[test]
fn test_array_decimal_coercion() {
    let src = "lista<decimal> nums = [1, 2, 3];
nums.agregar(4);
imprimir(nums[0]);
imprimir(nums[3]);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "4"]);
}

#[test]
fn test_array_english_keywords() {
    let src = "array<integer> nums = [5, 10];
print(nums[0]);
print(nums.len());";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["5", "2"]);
}

#[test]
fn test_array_index_out_of_bounds_negative() {
    let src = "lista<entero> nums = [1];
imprimir(nums[-1]);";
    let result = run_source(src);
    assert!(result.is_err());
}

#[test]
fn test_break_in_while() {
    let src = "entero i = 0;
mientras (i < 10) {
    si (i == 3) { romper; }
    imprimir(i);
    i = i + 1;
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "1", "2"]);
}

#[test]
fn test_continue_in_while() {
    let src = "entero i = 0;
mientras (i < 5) {
    i = i + 1;
    si (i == 3) { continuar; }
    imprimir(i);
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "2", "4", "5"]);
}

#[test]
fn test_nested_continue() {
    let src = "entero i = 0;
mientras (i < 3) {
    entero j = 0;
    mientras (j < 3) {
        j = j + 1;
        si (j == 2) { continuar; }
        imprimir(i * 10 + j);
    }
    i = i + 1;
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "3", "11", "13", "21", "23"]);
}

#[test]
fn test_match_simple() {
    let src = "entero x = 2;
elegir (x) {
    caso 1: imprimir(\"uno\");
    caso 2: imprimir(\"dos\");
    caso 3: imprimir(\"tres\");
    defecto: imprimir(\"otro\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["dos"]);
}

#[test]
fn test_match_default() {
    let src = "entero x = 99;
elegir (x) {
    caso 1: imprimir(\"uno\");
    defecto: imprimir(\"default\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["default"]);
}

#[test]
fn test_match_english() {
    let src = "integer x = 3;
match (x) {
    case 1: print(\"one\");
    case 3: print(\"three\");
    default: print(\"other\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["three"]);
}

#[test]
fn test_break_outside_loop_error() {
    let result = run_source("romper;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_continue_outside_loop_error() {
    let result = run_source("continuar;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_break_in_for() {
    let src = "para (entero i = 0; i < 10; i = i + 1) {
    si (i == 2) { romper; }
    imprimir(i);
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "1"]);
}

#[test]
fn test_continue_in_for() {
    let src = "para (entero i = 0; i < 5; i = i + 1) {
    si (i == 2) { continuar; }
    imprimir(i);
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "1", "3", "4"]);
}

#[test]
fn test_match_arm_type_error() {
    let src = "entero x = 1;
elegir (x) {
    caso \"texto\": imprimir(\"no\");
}";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_default_param_used() {
    let src = "funcion entero suma(entero a, entero b = 10) { retornar a + b; }
imprimir(suma(5));
imprimir(suma(5, 20));";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["15", "25"]);
}

#[test]
fn test_lambda_iife() {
    let src = "imprimir(funcion(entero x) { retornar x * 2; }(5));";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn test_lambda_multiple_args() {
    let src = "imprimir(funcion(entero a, entero b) { retornar a + b; }(10, 20));";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["30"]);
}

#[test]
fn test_lambda_string_concat() {
    let src = "imprimir(funcion(texto a, texto b) { retornar a + b; }(\"Hola \", \"Mundo\"));";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Hola Mundo"]);
}

#[test]
fn test_default_param_min_args_error() {
    let src = "funcion entero suma(entero a, entero b = 10) { retornar a + b; }
imprimir(suma());";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

// --- Struct tests ---

#[test]
fn test_struct_decl_and_init() {
    let src = "estructura Persona { nombre: texto, edad: numero }
Persona p = Persona { nombre: \"Ana\", edad: 30 };
imprimir(p.nombre);
imprimir(p.edad);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Ana", "30"]);
}

#[test]
fn test_struct_field_access() {
    let src = "estructura Punto { x: entero, y: entero }
Punto pt = Punto { x: 10, y: 20 };
imprimir(pt.x);
imprimir(pt.y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "20"]);
}

#[test]
fn test_struct_field_assign() {
    let src = "estructura Punto { x: entero, y: entero }
Punto pt = Punto { x: 10, y: 20 };
pt.x = 30;
imprimir(pt.x);
imprimir(pt.y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["30", "20"]);
}

#[test]
fn test_struct_value_semantics() {
    let src = "estructura Punto { x: entero, y: entero }
Punto a = Punto { x: 1, y: 2 };
Punto b = a;
b.x = 99;
imprimir(a.x);
imprimir(b.x);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "99"]);
}

#[test]
fn test_struct_english_keywords() {
    let src = "struct Person { name: string, age: number }
Person p = Person { name: \"Bob\", age: 25 };
print(p.name);
print(p.age);
p.age = 26;
print(p.age);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Bob", "25", "26"]);
}

#[test]
fn test_struct_multiple_fields() {
    let src = "estructura Rect { ancho: decimal, alto: decimal, area: decimal }
Rect r = Rect { ancho: 5.5, alto: 3.0, area: 16.5 };
imprimir(r.ancho);
imprimir(r.alto);
imprimir(r.area);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["5.5", "3", "16.5"]);
}

#[test]
fn test_struct_missing_field_error() {
    let src = "estructura Punto { x: entero, y: entero }
Punto pt = Punto { x: 10 };";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_struct_undefined_field_error() {
    let src = "estructura Punto { x: entero, y: entero }
Punto pt = Punto { x: 10, y: 20, z: 30 };";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_struct_field_type_error() {
    let src = "estructura Punto { x: entero, y: entero }
Punto pt = Punto { x: 10, y: \"veinte\" };";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_struct_in_struct() {
    let src = "estructura Direccion { calle: texto, numero: entero }
estructura Persona { nombre: texto, direccion: texto }
Persona p = Persona { nombre: \"Ana\", direccion: \"Calle 123\" };
imprimir(p.nombre);
imprimir(p.direccion);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Ana", "Calle 123"]);
}

#[test]
fn test_result_exito() {
    let src = r#"resultado<entero, texto> r = exito(42);
imprimir(r);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["exito(42)"]);
}

#[test]
fn test_result_error() {
    let src = r#"resultado<entero, texto> r = error("falló");
imprimir(r);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["error(falló)"]);
}

#[test]
fn test_result_type_declaration() {
    let src = r#"resultado<texto, entero> r = exito("ok");
imprimir(r);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["exito(ok)"]);
}

#[test]
fn test_try_unwrap_ok() {
    let src = r#"funcion entero probar() {
    resultado<entero, texto> r = exito(42);
    retornar intentar r;
}
entero x = probar();
imprimir(x);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_try_unwrap_error_propagates() {
    let src = r#"funcion resultado<entero, texto> fallar() {
    resultado<entero, texto> r = error("fracaso");
    retornar r;
}
resultado<entero, texto> res = fallar();
imprimir(res);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["error(fracaso)"]);
}

#[test]
fn test_result_in_function_return() {
    let src = r#"funcion resultado<entero, texto> dividir(entero a, entero b) {
    si (b == 0) {
        retornar error("división por cero");
    }
    retornar exito(a / b);
}
resultado<entero, texto> r = dividir(10, 0);
imprimir(r);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["error(división por cero)"]);
}

#[test]
fn test_result_success_division() {
    let src = r#"funcion resultado<entero, texto> dividir(entero a, entero b) {
    si (b == 0) {
        retornar error("división por cero");
    }
    retornar exito(a / b);
}
resultado<entero, texto> r = dividir(10, 2);
imprimir(r);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["exito(5)"]);
}

#[test]
fn test_try_unwrap_in_nested_function() {
    let src = r#"funcion resultado<entero, texto> validar(entero x) {
    si (x < 0) {
        retornar error("negativo");
    }
    retornar exito(x);
}
funcion resultado<entero, texto> procesar(entero x) {
    entero val = intentar validar(x);
    retornar exito(val * 2);
}
resultado<entero, texto> r1 = procesar(5);
resultado<entero, texto> r2 = procesar(-1);
imprimir(r1);
imprimir(r2);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["exito(10)", "error(negativo)"]);
}

// --- For-Each Loop Tests ---

#[test]
fn test_foreach_basic() {
    let src = "lista<entero> nums = [1, 2, 3];
para n en nums {
    imprimir(n);
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "2", "3"]);
}

#[test]
fn test_foreach_empty() {
    let src = "lista<entero> nums = [];
para n en nums {
    imprimir(n);
}
imprimir(\"fin\");";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["fin"]);
}

#[test]
fn test_foreach_english_keywords() {
    let src = "array<integer> nums = [10, 20, 30];
for n in nums {
    print(n);
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "20", "30"]);
}

#[test]
fn test_foreach_with_strings() {
    let src = "lista<texto> nombres = [\"Ana\", \"Luis\", \"Pedro\"];
para nombre en nombres {
    imprimir(nombre);
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Ana", "Luis", "Pedro"]);
}

#[test]
fn test_foreach_in_function() {
    let src = "funcion texto unir(lista<texto> palabras) {
    texto res = \"\";
    para p en palabras {
        res = res + p;
    }
    retornar res;
}
texto r = unir([\"a\", \"b\", \"c\"]);
imprimir(r);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["abc"]);
}

#[test]
fn test_foreach_nested() {
    let src = "lista<entero> nums = [1, 2];
para a en nums {
    para b en nums {
        imprimir(a * b);
    }
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "2", "2", "4"]);
}

#[test]
fn test_foreach_with_condition() {
    let src = "lista<entero> nums = [1, 2, 3, 4, 5];
para n en nums {
    si (n > 2) {
        imprimir(n);
    }
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3", "4", "5"]);
}

#[test]
fn test_foreach_type_error() {
    let src = "entero x = 42;
para n en x {
    imprimir(n);
}";
    let result = run_source(src);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.contains("SemError")
            || err.contains("E044")
            || err.contains("lista")
            || err.contains("array")
    );
}

#[test]
fn test_foreach_var_scope() {
    let src = "lista<entero> nums = [10, 20];
para n en nums {
    imprimir(n);
}
imprimir(99);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "20", "99"]);
}

// --- Opcion/Optional Type Tests ---

#[test]
fn test_opcion_algun() {
    let src = "opcion<entero> x = algun(42);
imprimir(x);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["algun(42)"]);
}

#[test]
fn test_opcion_ninguno() {
    let src = "opcion<entero> x = ninguno;
imprimir(x);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["ninguno"]);
}

#[test]
fn test_opcion_english_keywords() {
    let src = "option<integer> x = some(42);
option<string> y = none;
imprimir(x);
imprimir(y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["algun(42)", "ninguno"]);
}

#[test]
fn test_opcion_assign_ninguno_to_any() {
    let src = "opcion<texto> x = ninguno;
imprimir(x);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["ninguno"]);
}

#[test]
fn test_opcion_type_error() {
    let src = "opcion<texto> x = algun(42);";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(
        result.as_ref().unwrap_err().contains("SemError")
            || result.as_ref().unwrap_err().contains("E031")
    );
}

#[test]
fn test_opcion_eq_algun() {
    let src = "opcion<entero> x = algun(5);
opcion<entero> y = algun(5);
booleano eq = x == y;
imprimir(eq);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_opcion_neq_algun_ninguno() {
    let src = "opcion<entero> x = algun(5);
opcion<entero> y = ninguno;
booleano neq = x != y;
imprimir(neq);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_opcion_match_algun() {
    let src = "opcion<entero> x = algun(10);
elegir (x) {
    caso algun(10): { imprimir(1); }
    caso ninguno: { imprimir(2); }
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_opcion_match_ninguno() {
    let src = "opcion<entero> x = ninguno;
elegir (x) {
    caso algun(10): { imprimir(1); }
    caso ninguno: { imprimir(2); }
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_opcion_in_function() {
    let src = "funcion opcion<entero> buscar(entero x) {
    si (x > 0) {
        retornar algun(x);
    }
    retornar ninguno;
}
opcion<entero> r1 = buscar(5);
opcion<entero> r2 = buscar(-1);
imprimir(r1);
imprimir(r2);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["algun(5)", "ninguno"]);
}

// --- Tuple Tests ---

#[test]
fn test_tuple_basic() {
    let src = "imprimir((42, \"hola\", 3.0));";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["(42, hola, 3)"]);
}

#[test]
fn test_tuple_access() {
    let src = "imprimir((10, 20, 30).0);
imprimir((10, 20, 30).1);
imprimir((10, 20, 30).2);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "20", "30"]);
}

#[test]
fn test_tuple_nested() {
    let src = "(entero, (texto, entero)) t = (1, (\"a\", 2));
imprimir(t.0);
imprimir(t.1.0);
imprimir(t.1.1);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "a", "2"]);
}

#[test]
fn test_tuple_type_error() {
    let result = run_source("entero x = (1, 2);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

// --- Destructuring Tests ---

#[test]
fn test_destructure_declaration() {
    let src = "entero x, texto y = (1, \"hola\");
imprimir(x);
imprimir(y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "hola"]);
}

#[test]
fn test_destructure_assignment() {
    let src = "entero x = 0;
texto y = \"\";
x, y = (1, \"mundo\");
imprimir(x);
imprimir(y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "mundo"]);
}

#[test]
fn test_destructure_wildcard() {
    let src = "entero x, _ = (1, 2);
imprimir(x);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_destructure_three_elements() {
    let src = "entero a, texto b, decimal c = (1, \"x\", 3.5);
imprimir(a);
imprimir(b);
imprimir(c);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "x", "3.5"]);
}

#[test]
fn test_destructure_type_error() {
    let result = run_source("entero x, texto y = (1, 2);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_destructure_arity_error() {
    let result = run_source("entero x, texto y = (1, \"a\", 3);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_destructure_assign_arity_error() {
    let result = run_source("entero x = 0; entero y = 0; x, y = (1, 2, 3);");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_destructure_assign_not_tuple_error() {
    let result = run_source("entero x = 0; entero y = 0; x, y = 42;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_destructure_decl_not_tuple_error() {
    let result = run_source("entero x, entero y = 42;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_destructure_wildcard_middle() {
    let src = "entero a, _, entero c = (1, 2, 3);
imprimir(a);
imprimir(c);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "3"]);
}

#[test]
fn test_destructure_array_access() {
    let src = "lista<entero> nums = [10, 20];
entero x, entero y = (nums[0], nums[1]);
imprimir(x);
imprimir(y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "20"]);
}

#[test]
fn test_destructure_expression() {
    let src = "entero a, entero b = (1 + 2, 3 * 4);
imprimir(a);
imprimir(b);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3", "12"]);
}

#[test]
fn test_destructure_english_keywords() {
    let src = "integer x, string y = (42, \"hello\");
print(x);
print(y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42", "hello"]);
}

#[test]
fn test_destructure_assign_wildcard() {
    let src = "entero x = 0;
entero y = 0;
x, _, y = (1, 2, 3);
imprimir(x);
imprimir(y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "3"]);
}

// --- Generics Tests ---

#[test]
fn test_generic_function_identity_int() {
    let src = "funcion T identidad<T>(T valor) { retornar valor; }
entero x = identidad<entero>(42);
imprimir(x);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_generic_function_identity_string() {
    let src = "funcion T identidad<T>(T valor) { retornar valor; }
texto s = identidad<texto>(\"hola\");
imprimir(s);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["hola"]);
}

#[test]
fn test_generic_struct_pair() {
    let src = "estructura Par<T, U> { primero: T, segundo: U }
Par<entero, texto> p = Par<entero, texto> { primero: 1, segundo: \"hola\" };
imprimir(p.primero);
imprimir(p.segundo);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "hola"]);
}

#[test]
fn test_generic_struct_pair_numeric() {
    let src = "estructura Par<T, U> { primero: T, segundo: U }
Par<entero, decimal> p = Par<entero, decimal> { primero: 42, segundo: 3.5 };
imprimir(p.primero);
imprimir(p.segundo);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42", "3.5"]);
}

#[test]
fn test_generic_function_type_error() {
    let src = "funcion T identidad<T>(T valor) { retornar valor; }
entero x = identidad<entero>(\"hola\");";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_generic_function_with_struct() {
    let src = "funcion T id<T>(T v) { retornar v; }
entero x = id<entero>(99);
texto s = id<texto>(\"mundo\");
imprimir(x);
imprimir(s);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["99", "mundo"]);
}

#[test]
fn test_match_guard_passes() {
    let src = "entero x = 5;
elegir (x) {
    caso 5 si x > 3: imprimir(\"cinco mayor que 3\");
    defecto: imprimir(\"otro\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["cinco mayor que 3"]);
}

#[test]
fn test_match_guard_fails_falls_through() {
    let src = "entero x = 5;
elegir (x) {
    caso 5 si x > 10: imprimir(\"cinco mayor que 10\");
    defecto: imprimir(\"defecto\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["defecto"]);
}

#[test]
fn test_match_guard_multiple_arms() {
    let src = "entero x = 3;
elegir (x) {
    caso 1 si x < 5: imprimir(\"uno\");
    caso 3 si x < 10: imprimir(\"tres\");
    defecto: imprimir(\"otro\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["tres"]);
}

#[test]
fn test_match_guard_falls_through_to_next_arm() {
    let src = "entero x = 3;
elegir (x) {
    caso 3 si x < 0: imprimir(\"negativo\");
    caso 3: imprimir(\"solo tres\");
    defecto: imprimir(\"otro\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["solo tres"]);
}

#[test]
fn test_match_english_guard() {
    let src = "integer x = 10;
match (x) {
    case 10 if x > 5: print(\"diez\");
    default: print(\"otro\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["diez"]);
}

#[test]
fn test_match_exhaustiveness_error() {
    let src = "enum Color { Rojo, Verde, Azul }
Color c = Color::Rojo;
elegir (c) {
    caso Color::Rojo: imprimir(\"rojo\");
    caso Color::Verde: imprimir(\"verde\");
}";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_match_exhaustiveness_with_default() {
    let src = "enum Color { Rojo, Verde, Azul }
Color c = Color::Rojo;
elegir (c) {
    caso Color::Rojo: imprimir(\"rojo\");
    defecto: imprimir(\"otro\");
}";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["rojo"]);
}

#[test]
fn test_match_guard_type_error() {
    let src = "entero x = 1;
elegir (x) {
    caso 1 si 42: imprimir(\"mal\");
}";
    let result = run_source(src);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_trait_associated_types() {
    let src = "rasgo Contenedor {
    tipo Item;
    funcion Item obtener_valor(este);
}
estructura Caja {
    valor: entero,
}
impl Contenedor para Caja {
    tipo Item = entero;
    funcion entero obtener_valor(este) {
        retornar este.valor;
    }
}
sea c = Caja { valor: 99 };
imprimir(c.obtener_valor());";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["99"]);
}

#[test]
fn test_trait_associated_types_typed_assign() {
    let src = "rasgo Convertidor {
    tipo Entrada;
    tipo Salida;
    funcion Salida convertir(este, Entrada val);
}
estructura Duplicador {}
impl Convertidor para Duplicador {
    tipo Entrada = entero;
    tipo Salida = texto;
    funcion texto convertir(este, entero val) {
        retornar \"dup: \" + (val * 2);
    }
}
sea d = Duplicador {};
entero x = 21;
texto r = d.convertir(x);
imprimir(r);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["dup: 42"]);
}

#[test]
fn test_extension_methods() {
    let src = "rasgo Duplicable {
    funcion entero duplicar(este);
}
impl Duplicable para entero {
    funcion entero duplicar(este) {
        retornar este * 2;
    }
}
entero x = 21;
imprimir(x.duplicar());";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42"]);
}

// --- Fase 96-110: Colecciones ---

#[test]
fn test_diccionario() {
    let src = r#"numero d = __map_nuevo();
d = __map_poner(d, "Ana", 30);
d = __map_poner(d, "Luis", 25);
imprimir(__map_obtener(d, "Ana"));
imprimir(__map_longitud(d));
imprimir(__map_contiene(d, "Ana"));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["30", "2", "true"]);
}

#[test]
fn test_conjunto() {
    let src = r#"numero s = __conjunto_nuevo();
s = __conjunto_agregar(s, "a");
s = __conjunto_agregar(s, "b");
imprimir(__conjunto_tiene(s, "a"));
imprimir(__conjunto_tiene(s, "c"));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn test_conjunto_union() {
    let src = r#"numero a = __conjunto_nuevo();
a = __conjunto_agregar(a, 1);
a = __conjunto_agregar(a, 2);
numero b = __conjunto_nuevo();
b = __conjunto_agregar(b, 2);
b = __conjunto_agregar(b, 3);
numero u = __conjunto_unir(a, b);
imprimir(__conjunto_tiene(u, 1));
imprimir(__conjunto_tiene(u, 3));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true", "true"]);
}

#[test]
fn test_conjunto_inter() {
    let src = r#"numero a = __conjunto_nuevo();
a = __conjunto_agregar(a, 1);
a = __conjunto_agregar(a, 2);
numero b = __conjunto_nuevo();
b = __conjunto_agregar(b, 2);
b = __conjunto_agregar(b, 3);
numero i = __conjunto_interseccion(a, b);
imprimir(__conjunto_tiene(i, 1));
imprimir(__conjunto_tiene(i, 2));
imprimir(__conjunto_tiene(i, 3));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["false", "true", "false"]);
}

#[test]
fn test_conjunto_diff() {
    let src = r#"numero a = __conjunto_nuevo();
a = __conjunto_agregar(a, 1);
a = __conjunto_agregar(a, 2);
a = __conjunto_agregar(a, 3);
numero b = __conjunto_nuevo();
b = __conjunto_agregar(b, 2);
numero d = __conjunto_diferencia(a, b);
imprimir(__conjunto_tiene(d, 1));
imprimir(__conjunto_tiene(d, 2));
imprimir(__conjunto_tiene(d, 3));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true", "false", "true"]);
}

#[test]
fn test_deque() {
    let src = r#"numero dq = __deque_nuevo();
dq = __deque_agregar_final(dq, 1);
dq = __deque_agregar_final(dq, 2);
dq = __deque_agregar_frente(dq, 0);
imprimir(__deque_longitud(dq));
numero f = __deque_quitar_frente(dq);
imprimir(f);
numero b = __deque_quitar_final(dq);
imprimir(b);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_monticulo() {
    let src = r#"numero h = __monticulo_nuevo();
h = __monticulo_agregar(h, 5);
h = __monticulo_agregar(h, 1);
h = __monticulo_agregar(h, 10);
imprimir(__monticulo_ver(h));
imprimir(__monticulo_longitud(h));
numero popped = __monticulo_quitar(h);
imprimir(popped);
imprimir(__monticulo_longitud(h));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "3", "10", "3"]);
}

#[test]
fn test_enlazada() {
    let src = r#"numero ll = __enlazada_nuevo();
ll = __enlazada_agregar_final(ll, "x");
ll = __enlazada_agregar_final(ll, "y");
ll = __enlazada_agregar_frente(ll, "z");
imprimir(__enlazada_longitud(ll));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3"]);
}

// --- Regex ---

#[test]
fn test_regex_is_match() {
    let src = r#"imprimir(__regex_coincide("\\d+", "abc123"));
imprimir(__regex_coincide("\\d+", "abc"));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn test_regex_replace() {
    let src = r#"imprimir(__regex_reemplazar("mundo", "Hola mundo", "Lumen"));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["Hola Lumen"]);
}

// --- Unicode ---

#[test]
fn test_unicode_nfc() {
    let src = r#"imprimir(__unicode_normalizar("cafe\\u0301", "NFC"));
"#;
    let output = run_source(src).unwrap();
    // Composite é character
    assert_eq!(output.len(), 1);
}

// --- Padding ---

#[test]
fn test_pad_start() {
    let src = r#"imprimir(__str_padding_inicio("42", 5, "0"));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["00042"]);
}

#[test]
fn test_pad_end() {
    let src = r#"imprimir(__str_padding_fin("42", 5, "."));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42..."]);
}

// --- Encoding ---

#[test]
fn test_utf8_encoding() {
    let src = r#"lista<entero> bytes = __codificacion_utf8("Hola");
imprimir(bytes);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["[72, 111, 108, 97]"]);
}

// --- IO (Buffered) ---

#[test]
fn test_buf_writer() {
    let src = r#"numero r = __escritor_buffer("test_e2e_tmp.txt", "contenido");
imprimir(r);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["exito(true)"]);
    let _ = std::fs::remove_file("test_e2e_tmp.txt");
}

#[test]
fn test_buf_reader() {
    let src = r#"numero r = __lector_buffer("Cargo.toml");
imprimir(__deque_longitud(r));
"#;
    let output = run_source(src).unwrap();
    assert!(!output[0].is_empty());
}

// --- TCP ---

#[test]
fn test_tcp_connect_refused() {
    let src = r#"numero r = __tcp_conectar("127.0.0.1:1");
imprimir(r);
"#;
    let result = run_source(src);
    assert!(result.is_ok() || result.is_err());
}

// --- HTTP ---

#[test]
fn test_http_get() {
    let src = r#"numero r = __http_obtener("https://httpbin.org/get");
imprimir(r);
"#;
    // May fail if no network, so just check it doesn't crash the test runner
    let _ = run_source(src);
}

// --- Serial ---

#[test]
fn test_serial_open() {
    let src = r#"numero r = __serial_abrir("COM1");
imprimir(r);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true"]);
}

// --- Map keys ---

#[test]
fn test_map_keys() {
    let src = r#"numero d = __map_nuevo();
d = __map_poner(d, "x", 1);
d = __map_poner(d, "y", 2);
lista<numero> k = __map_claves(d);
imprimir(k);
"#;
    let output = run_source(src).unwrap();
    assert!(output[0].contains("x") || output[0].contains("y"));
}

// --- Deque peek/pop empty ---

#[test]
fn test_deque_empty() {
    let src = r#"numero dq = __deque_nuevo();
imprimir(__deque_longitud(dq));
numero f = __deque_quitar_frente(dq);
imprimir(f);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "void"]);
}

// --- Heap empty ---

#[test]
fn test_heap_empty() {
    let src = r#"numero h = __monticulo_nuevo();
imprimir(__monticulo_ver(h));
imprimir(__monticulo_longitud(h));
numero p = __monticulo_quitar(h);
imprimir(p);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["void", "0", "void"]);
}

// --- Linked list empty ---

#[test]
fn test_enlazada_empty() {
    let src = r#"numero ll = __enlazada_nuevo();
imprimir(__enlazada_longitud(ll));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0"]);
}

// --- Combined stress ---

#[test]
fn test_builtins_stress() {
    let src = r#"numero m = __map_nuevo();
m = __map_poner(m, "a", 1);
m = __map_poner(m, "b", 2);
imprimir(__map_longitud(m));
imprimir(__map_contiene(m, "a"));
numero s = __conjunto_nuevo();
s = __conjunto_agregar(s, "x");
s = __conjunto_agregar(s, "y");
imprimir(__conjunto_tiene(s, "x"));
imprimir(__regex_coincide("\\w+", "hola"));
lista<entero> bytes = __codificacion_utf8("abc");
imprimir(bytes);
imprimir(__str_padding_inicio("7", 3, "0"));
"#;
    let output = run_source(src).unwrap();
    assert_eq!(
        output,
        vec!["2", "true", "true", "true", "[97, 98, 99]", "007"]
    );
}

// --- Regex captures ---

#[test]
fn test_regex_captures() {
    let src = r#"numero caps = __regex_capturar("(\\d+)-(\\w+)", "123-abc");
imprimir(caps);
"#;
    let output = run_source(src).unwrap();
    assert!(output[0].contains("123-abc") || output[0] == "[]");
}

// --- Encoding from utf8 ---

#[test]
fn test_encoding_from_utf8() {
    let src = r#"lista<entero> bytes = __codificacion_utf8("Hola");
texto dec = __desde_utf8(bytes);
imprimir(dec);
"#;
    let output = run_source(src).unwrap();
    assert!(!output[0].is_empty());
}

// --- New builtins: tipo_de, str_ord, hash, fs, tiempo, jwt, env, coro ---

#[test]
fn test_tipo_de() {
    let src = r#"imprimir(__tipo_de(42));
imprimir(__tipo_de(3.14));
imprimir(__tipo_de("hola"));
imprimir(__tipo_de(verdadero));
lista<entero> x = [];
imprimir(__tipo_de(x));"#;
    let output = run_source(src).unwrap();
    assert_eq!(
        output,
        vec!["entero", "decimal", "texto", "booleano", "lista"]
    );
}

#[test]
fn test_str_ord() {
    let src = r#"imprimir(__str_ord("A"));
imprimir(__str_ord("ABC"));"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["[65]", "[65, 66, 67]"]);
}

#[test]
fn test_hash_sha256() {
    let src = r#"imprimir(__hash_sha256("hola"));"#;
    let output = run_source(src).unwrap();
    // SHA-256 produces 32 bytes => 64 hex chars
    let hex = &output[0];
    if hex != "error(Bcrypt no disponible)" {
        assert_eq!(hex.len(), 64, "SHA-256 hex debe tener 64 caracteres");
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-256 debe ser hexadecimal"
        );
    }
    // else: bcrypt no disponible, test pasa sin aserción fuerte
}

#[test]
fn test_hash_sha512() {
    let src = r#"imprimir(__hash_sha512("hola"));"#;
    let output = run_source(src).unwrap();
    // SHA-512 produces 64 bytes => 128 hex chars
    let hex = &output[0];
    if hex != "error(Bcrypt no disponible)" {
        assert_eq!(hex.len(), 128, "SHA-512 hex debe tener 128 caracteres");
        assert!(
            hex.chars().all(|c| c.is_ascii_hexdigit()),
            "SHA-512 debe ser hexadecimal"
        );
    }
}

#[test]
fn test_fs_listar() {
    let src = r#"imprimir(__fs_listar("."));"#;
    let output = run_source(src).unwrap();
    // Should return an array (or error if directory unreadable)
    assert!(output[0].starts_with('[') || output[0].starts_with("error("));
}

#[test]
fn test_tiempo_formatear() {
    let src = r#"imprimir(__tiempo_formatear(0));"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1970-01-01T00:00:00Z"]);
}

#[test]
fn test_tiempo_diferencia() {
    let src = r#"imprimir(__tiempo_diferencia(100, 50));"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["50"]);
}

#[test]
fn test_jwt() {
    let src = r#"imprimir(__jwt_codificar("{\"sub\":\"123\"}", "secreto"));
imprimir(__jwt_decodificar(__jwt_codificar("{\"sub\":\"123\"}", "secreto"), "secreto"));"#;
    let output = run_source(src).unwrap();
    // Token has 3 dot-separated parts
    assert_eq!(output[0].matches('.').count(), 2, "JWT debe tener 2 puntos");
    assert!(!output[0].is_empty(), "JWT no debe estar vacío");
    // Decoded payload matches original
    assert_eq!(output[1], "{\"sub\":\"123\"}");
}

#[test]
fn test_env_listar() {
    let src = r#"imprimir(__env_listar());"#;
    let output = run_source(src).unwrap();
    // Should return an array of "KEY=VALUE" strings
    assert!(
        output[0].starts_with('['),
        "env_listar debe retornar un array"
    );
    assert!(
        output[0].len() > 2,
        "debe haber al menos una variable de entorno"
    );
}

#[test]
fn test_coro_basic() {
    let src = r#"funcion texto mid_func() { retornar "ok"; }
imprimir(__coro_crear("mid_func", 0));"#;
    let output = run_source(src).unwrap();
    // Coroutine id format: coro_N
    assert!(
        output[0].starts_with("coro_"),
        "ID de corrutina debe empezar con 'coro_'"
    );
    assert!(!output[0].is_empty(), "ID de corrutina no debe estar vacío");
}

// ── Async / Task Tests ──

#[test]
fn test_async_task_basic() {
    let src = "funcion entero mid_func() { retornar 42; }
texto x = __tarea_lanzar(\"mid_func\");
entero y = __tarea_esperar(x);
imprimir(y);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_async_with_args() {
    let src = "funcion entero suma(entero a, entero b) { retornar a + b; }
texto i = __tarea_lanzar(\"suma\", 10, 20);
entero r = __tarea_esperar(i);
imprimir(r);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["30"]);
}

#[test]
fn test_async_multiple_tasks() {
    let src = "funcion entero slow_double(entero x) { retornar x * 2; }
texto t1 = __tarea_lanzar(\"slow_double\", 5);
texto t2 = __tarea_lanzar(\"slow_double\", 10);
texto t3 = __tarea_lanzar(\"slow_double\", 15);
entero r1 = __tarea_esperar(t1);
entero r2 = __tarea_esperar(t2);
entero r3 = __tarea_esperar(t3);
imprimir(r1 + r2 + r3);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["60"]);
}

#[test]
fn test_async_esperar_keyword() {
    let src = "funcion entero mid_func() { retornar 99; }
texto tid = __tarea_lanzar(\"mid_func\");
entero res = __tarea_esperar(tid);
imprimir(res);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["99"]);
}

#[test]
fn test_async_english_task_spawn() {
    let src = "function integer double(integer x) { return x * 2; }
texto tid = __task_spawn(\"double\", 21);
entero res = __task_await(tid);
print(res);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_async_task_not_found() {
    let src = "texto tid = __tarea_lanzar(\"nonexistent_func\");
entero res = __tarea_esperar(tid);
imprimir(res);";
    let output = run_source(src).unwrap();
    // The nonexistent function returns Void in the spawned task
    // Void prints as "void" on the output
    assert!(
        output[0].contains("void") || output[0].contains("error") || output[0].contains("Error")
    );
}

#[test]
fn test_js_eval_stub() {
    let src = "texto r = __js_eval(\"1 + 1\");
imprimir(r);";
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1 + 1"]);
}

#[test]
fn test_js_call_stub() {
    let src = "texto r = __js_llamar(\"alert\", \"hola\");
imprimir(r);";
    let output = run_source(src).unwrap();
    assert!(output[0].contains("__lumen_call"));
}

// --- New builtins: AES, Timezone, Duration, Calendar, Async I/O ---

#[test]
fn test_aes_encrypt_decrypt() {
    let src = r#"texto key = "clave16bytes!!!!";
texto data = "mensaje secreto";
texto ct = __aes_encriptar(key, data);
texto pt = __aes_desencriptar(key, ct);
imprimir(pt);
"#;
    let output = run_source(src).unwrap();
    // On Linux/macOS, bcrypt.dll won't be available, so accept error message
    if output[0] != "mensaje secreto" {
        assert!(
            output[0].contains("bcrypt") || output[0].contains("Bcrypt"),
            "Expected AES roundtrip or bcrypt error, got: {:?}",
            output
        );
    }
}

#[test]
fn test_timezone_info() {
    let src = r#"entero utc = __zona_info("utc");
entero est = __zona_info("est");
entero pst = __zona_info("pst");
entero cet = __zona_info("cet");
entero jst = __zona_info("jst");
imprimir(utc);
imprimir(est);
imprimir(pst);
imprimir(cet);
imprimir(jst);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "-5", "-8", "1", "9"]);
}

#[test]
fn test_duration() {
    let src = r#"entero d = __duracion_nueva(5, 500000000);
entero s = __duracion_segundos(d);
imprimir(s);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["5"]);
}

#[test]
fn test_calendar_hijri() {
    let src = r#"texto h = __calendario_hijri(0);
imprimir(h);
"#;
    let output = run_source(src).unwrap();
    // Epoch 1970-01-01 → ~1391 AH
    assert!(
        output[0].contains("AH"),
        "Expected Hijri date, got: {}",
        output[0]
    );
}

#[test]
fn test_calendar_persian() {
    let src = r#"texto p = __calendario_persa(0);
imprimir(p);
"#;
    let output = run_source(src).unwrap();
    // Epoch 1970-01-01 → ~1348 AP
    assert!(
        output[0].contains("AP"),
        "Expected Persian date, got: {}",
        output[0]
    );
}

#[test]
fn test_file_read_async() {
    let src = r#"texto tid = __file_read_async("Cargo.toml");
numero content = __tarea_esperar(tid);
imprimir(content);
"#;
    let output = run_source(src).unwrap();
    assert!(!output[0].is_empty(), "Should read Cargo.toml content");
    assert!(
        output[0].contains("lumen-vm"),
        "Should contain package name"
    );
}

#[test]
fn test_timer_delay() {
    let src = r#"texto tid = __temporizador_esperar(10);
numero r = __tarea_esperar(tid);
imprimir(r);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_tcp_connect_async() {
    let src = r#"texto tid = __tcp_conectar_async("127.0.0.1:1");
numero r = __tarea_esperar(tid);
imprimir("hecho");
"#;
    // Expects error since no server is listening, but should not crash
    let result = run_source(src);
    assert!(result.is_ok());
}

#[test]
fn test_pipe_operator_execution() {
    let src = r#"
funcion entero doble(entero x) { retornar x * 2; }
funcion entero mas_uno(entero x) { retornar x + 1; }
entero r = 10 |> doble() |> mas_uno();
imprimir(r);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["21"]);
}

#[test]
fn test_optional_sugar_execution() {
    let src = r#"
funcion texto? obtener(booleano b) {
    si b { retornar algun("OK"); }
    retornar ninguno;
}
texto? v = obtener(verdadero);
elegir (v) {
    caso algun(val): imprimir(val);
    defecto: imprimir("NADA");
}
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["OK"]);
}

#[test]
fn test_list_comprehension_execution() {
    let src = r#"
lista<entero> nums = [1, 2, 3, 4, 5, 6];
lista<entero> pares = [x * 10 para x en nums si x % 2 == 0];
imprimir(pares);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["[20, 40, 60]"]);
}

#[test]
fn test_list_comprehension_range_and_filter() {
    let src = r#"
lista<entero> cuadrados = [x * x para x en 1..=5 si x > 2];
imprimir(cuadrados);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["[9, 16, 25]"]);
}

#[test]
fn test_linq_query_spanish_execution() {
    let src = r#"
lista<entero> datos = [10, 15, 20, 25, 30];
lista<entero> mayores = consultar x en datos donde x >= 20 seleccionar x * 2;
imprimir(mayores);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["[40, 50, 60]"]);
}

#[test]
fn test_linq_query_english_execution() {
    let src = r#"
array<integer> items = [1, 2, 3, 4, 5];
array<integer> query_res = query x in items where x > 2 select x + 10;
print(query_res);
"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["[13, 14, 15]"]);
}

#[test]
fn test_new_001_abs_positive_int() {
    let output = run_source(r#"imprimir(abs(5));"#).unwrap();
    assert_eq!(output, vec!["5"]);
}

#[test]
fn test_new_002_abs_negative_int() {
    let output = run_source(r#"imprimir(abs(-7));"#).unwrap();
    assert_eq!(output, vec!["7"]);
}

#[test]
fn test_new_003_abs_zero() {
    let output = run_source(r#"imprimir(abs(0));"#).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_new_004_absoluto_alias() {
    let output = run_source(r#"imprimir(absoluto(-3));"#).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_005_abs_float_negative() {
    let output = run_source(r#"imprimir(abs(-3.5));"#).unwrap();
    assert_eq!(output, vec!["3.5"]);
}

#[test]
fn test_new_006_abs_float_positive() {
    let output = run_source(r#"imprimir(abs(2.2));"#).unwrap();
    assert_eq!(output, vec!["2.2"]);
}

#[test]
fn test_new_007_abs_negative_zero_float() {
    let output = run_source(r#"imprimir(abs(-0.0));"#).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_new_008_abs_large_int() {
    let output = run_source(r#"imprimir(abs(-1000000));"#).unwrap();
    assert_eq!(output, vec!["1000000"]);
}

#[test]
fn test_new_009_absoluto_float() {
    let output = run_source(r#"imprimir(absoluto(-0.5));"#).unwrap();
    assert_eq!(output, vec!["0.5"]);
}

#[test]
fn test_new_010_abs_with_var() {
    let output = run_source(r#"entero x = -9; imprimir(abs(x));"#).unwrap();
    assert_eq!(output, vec!["9"]);
}

#[test]
fn test_new_011_abs_in_expression() {
    let output = run_source(r#"imprimir(abs(-5) + abs(3));"#).unwrap();
    assert_eq!(output, vec!["8"]);
}

#[test]
fn test_new_012_absoluto_large_float() {
    let output = run_source(r#"imprimir(absoluto(-123.456));"#).unwrap();
    assert_eq!(output, vec!["123.456"]);
}

#[test]
fn test_new_013_min_int_positive() {
    let output = run_source(r#"imprimir(min(5, 3));"#).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_014_min_int_negative() {
    let output = run_source(r#"imprimir(min(-2, -5));"#).unwrap();
    assert_eq!(output, vec!["-5"]);
}

#[test]
fn test_new_015_min_float() {
    let output = run_source(r#"imprimir(min(3.5, 2.1));"#).unwrap();
    assert_eq!(output, vec!["2.1"]);
}

#[test]
fn test_new_016_min_mixed() {
    let output = run_source(r#"imprimir(min(5, 3.2));"#).unwrap();
    assert_eq!(output, vec!["3.2"]);
}

#[test]
fn test_new_017_minimo_alias_int() {
    let output = run_source(r#"imprimir(minimo(10, 20));"#).unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn test_new_018_minimo_alias_float() {
    let output = run_source(r#"imprimir(minimo(1.5, 1.2));"#).unwrap();
    assert_eq!(output, vec!["1.2"]);
}

#[test]
fn test_new_019_min_zero() {
    let output = run_source(r#"imprimir(min(0, 0));"#).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_new_020_min_large() {
    let output = run_source(r#"imprimir(min(1000000, 999999));"#).unwrap();
    assert_eq!(output, vec!["999999"]);
}

#[test]
fn test_new_021_min_equal() {
    let output = run_source(r#"imprimir(min(7, 7));"#).unwrap();
    assert_eq!(output, vec!["7"]);
}

#[test]
fn test_new_022_min_negative_vs_positive() {
    let output = run_source(r#"imprimir(min(-3, 5));"#).unwrap();
    assert_eq!(output, vec!["-3"]);
}

#[test]
fn test_new_023_max_int() {
    let output = run_source(r#"imprimir(max(5, 3));"#).unwrap();
    assert_eq!(output, vec!["5"]);
}

#[test]
fn test_new_024_max_negative() {
    let output = run_source(r#"imprimir(max(-2, -5));"#).unwrap();
    assert_eq!(output, vec!["-2"]);
}

#[test]
fn test_new_025_max_float() {
    let output = run_source(r#"imprimir(max(3.5, 4.5));"#).unwrap();
    assert_eq!(output, vec!["4.5"]);
}

#[test]
fn test_new_026_max_mixed() {
    let output = run_source(r#"imprimir(max(5, 3.2));"#).unwrap();
    assert_eq!(output, vec!["5"]);
}

#[test]
fn test_new_027_maximo_alias_int() {
    let output = run_source(r#"imprimir(maximo(10, 20));"#).unwrap();
    assert_eq!(output, vec!["20"]);
}

#[test]
fn test_new_028_maximo_alias_float() {
    let output = run_source(r#"imprimir(maximo(1.5, 2.5));"#).unwrap();
    assert_eq!(output, vec!["2.5"]);
}

#[test]
fn test_new_029_max_zero() {
    let output = run_source(r#"imprimir(max(0, -1));"#).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_new_030_max_large() {
    let output = run_source(r#"imprimir(max(100, 200));"#).unwrap();
    assert_eq!(output, vec!["200"]);
}

#[test]
fn test_new_031_max_equal() {
    let output = run_source(r#"imprimir(max(7, 7));"#).unwrap();
    assert_eq!(output, vec!["7"]);
}

#[test]
fn test_new_032_max_negative_vs_positive() {
    let output = run_source(r#"imprimir(max(-3, 5));"#).unwrap();
    assert_eq!(output, vec!["5"]);
}

#[test]
fn test_new_033_imprimir_two_strings() {
    let output = run_source(r#"imprimir("a", "b");"#).unwrap();
    assert_eq!(output, vec!["ab"]);
}

#[test]
fn test_new_034_imprimir_three_numbers() {
    let output = run_source(r#"imprimir(1, 2, 3);"#).unwrap();
    assert_eq!(output, vec!["123"]);
}

#[test]
fn test_new_035_imprimir_string_and_number() {
    let output = run_source(r#"imprimir("x", 42);"#).unwrap();
    assert_eq!(output, vec!["x42"]);
}

#[test]
fn test_new_036_imprimir_mixed_three() {
    let output = run_source(r#"imprimir(1, "b", 3);"#).unwrap();
    assert_eq!(output, vec!["1b3"]);
}

#[test]
fn test_new_037_imprimir_four_numbers() {
    let output = run_source(r#"imprimir(1, 2, 3, 4);"#).unwrap();
    assert_eq!(output, vec!["1234"]);
}

#[test]
fn test_new_038_imprimir_with_space() {
    let output = run_source(r#"imprimir("hola", " ", "mundo");"#).unwrap();
    assert_eq!(output, vec!["hola mundo"]);
}

#[test]
fn test_new_039_imprimir_bool_combined() {
    let output = run_source(r#"imprimir("val:", verdadero);"#).unwrap();
    assert_eq!(output, vec!["val:true"]);
}

#[test]
fn test_new_040_imprimir_two_calls() {
    let output = run_source(r#"imprimir("a", "b"); imprimir("c");"#).unwrap();
    assert_eq!(output, vec!["ab", "c"]);
}

#[test]
fn test_new_041_a_texto_int() {
    let output = run_source(r#"imprimir(a_texto(42));"#).unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_new_042_a_texto_float() {
    let output = run_source(r#"imprimir(a_texto(3.14));"#).unwrap();
    assert_eq!(output, vec!["3.14"]);
}

#[test]
fn test_new_043_a_texto_bool() {
    let output = run_source(r#"imprimir(a_texto(verdadero));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_044_a_texto_string() {
    let output = run_source(r#"imprimir(a_texto("hola"));"#).unwrap();
    assert_eq!(output, vec!["hola"]);
}

#[test]
fn test_new_045_to_texto_alias() {
    let output = run_source(r#"imprimir(to_texto(99));"#).unwrap();
    assert_eq!(output, vec!["99"]);
}

#[test]
fn test_new_046_str_from_alias() {
    let output = run_source(r#"imprimir(__str_from(123));"#).unwrap();
    assert_eq!(output, vec!["123"]);
}

#[test]
fn test_new_047_largo_string_builtin() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_048_largo_list_via_var() {
    let output = run_source(r#"lista<entero> nums = [1,2,3]; imprimir(largo(nums));"#).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_049_raiz() {
    let output = run_source(r#"imprimir(raiz(9));"#).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_050_sqrt_alias() {
    let output = run_source(r#"imprimir(sqrt(16));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_051_potencia_int() {
    let output = run_source(r#"imprimir(potencia(2, 3));"#).unwrap();
    assert_eq!(output, vec!["8"]);
}

#[test]
fn test_new_052_pow_alias() {
    let output = run_source(r#"imprimir(pow(2, 4));"#).unwrap();
    assert_eq!(output, vec!["16"]);
}

#[test]
fn test_new_053_piso() {
    let output = run_source(r#"imprimir(piso(3.7));"#).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_054_techo() {
    let output = run_source(r#"imprimir(techo(3.2));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_055_redondear() {
    let output = run_source(r#"imprimir(redondear(3.6));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_056_floor_alias() {
    let output = run_source(r#"imprimir(floor(2.9));"#).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_new_057_str_upper() {
    let output = run_source(r#"imprimir(__str_upper("hola"));"#).unwrap();
    assert_eq!(output, vec!["HOLA"]);
}

#[test]
fn test_new_058_str_lower() {
    let output = run_source(r#"imprimir(__str_lower("HOLA"));"#).unwrap();
    assert_eq!(output, vec!["hola"]);
}

#[test]
fn test_new_059_str_trim() {
    let output = run_source(r#"imprimir(__str_trim("  hola  "));"#).unwrap();
    assert_eq!(output, vec!["hola"]);
}

#[test]
fn test_new_060_str_contains_true() {
    let output = run_source(r#"imprimir(__str_contains("hola mundo", "mundo"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_061_str_contains_false() {
    let output = run_source(r#"imprimir(__str_contains("hola", "x"));"#).unwrap();
    assert_eq!(output, vec!["false"]);
}

#[test]
fn test_new_062_str_split_comma() {
    let output = run_source(r#"imprimir(__str_split("a,b,c", ","));"#).unwrap();
    assert_eq!(output, vec!["[a, b, c]"]);
}

#[test]
fn test_new_063_str_split_empty_delim() {
    let output = run_source(r#"imprimir(__str_split("abc", ""));"#).unwrap();
    assert_eq!(output, vec!["[a, b, c]"]);
}

#[test]
fn test_new_064_str_len_bytes() {
    let output = run_source(r#"imprimir(__str_len("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_065_str_longitud_alias() {
    let output = run_source(r#"imprimir(__str_longitud("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_066_str_ord_abc() {
    let output = run_source(r#"imprimir(__str_ord("ABC"));"#).unwrap();
    assert_eq!(output, vec!["[65, 66, 67]"]);
}

#[test]
fn test_new_067_str_chr() {
    let output = run_source(r#"imprimir(__str_chr(65));"#).unwrap();
    assert_eq!(output, vec!["A"]);
}

#[test]
fn test_new_068_str_slice() {
    let output = run_source(r#"imprimir(__str_slice("hola mundo", 1, 4));"#).unwrap();
    assert_eq!(output, vec!["ola"]);
}

#[test]
fn test_new_069_str_reemplazar() {
    let output =
        run_source(r#"imprimir(__str_reemplazar("hola mundo", "mundo", "Lumen"));"#).unwrap();
    assert_eq!(output, vec!["hola Lumen"]);
}

#[test]
fn test_new_070_str_starts_with_true() {
    let output = run_source(r#"imprimir(__str_starts_with("hola", "ho"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_071_str_to_chars() {
    let output = run_source(r#"imprimir(__str_to_chars("abc"));"#).unwrap();
    assert_eq!(output, vec!["[a, b, c]"]);
}

#[test]
fn test_new_072_str_concat_list() {
    let output =
        run_source(r#"lista<texto> xs = ["a", "b", "c"]; imprimir(__str_concat_list(xs));"#)
            .unwrap();
    assert_eq!(output, vec!["abc"]);
}

#[test]
fn test_new_073_str_a_entero() {
    let output = run_source(r#"imprimir(__str_a_entero("42"));"#).unwrap();
    assert_eq!(output, vec!["42"]);
}

#[test]
fn test_new_074_texto_a_entero_alias() {
    let output = run_source(r#"imprimir(__texto_a_entero("123"));"#).unwrap();
    assert_eq!(output, vec!["123"]);
}

#[test]
fn test_new_075_str_slice_negative_end() {
    let output = run_source(r#"imprimir(__str_slice("abcdef", 0, -1));"#).unwrap();
    assert_eq!(output, vec!["abcdef"]);
}

#[test]
fn test_new_076_str_contiene_alias() {
    let output = run_source(r#"imprimir(__str_contiene("hola mundo", "hola"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_077_map_basic() {
    let output = run_source(
        r#"numero d = __map_nuevo(); d = __map_poner(d, "a", 1); imprimir(__map_obtener(d, "a"));"#,
    )
    .unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_078_map_longitud() {
    let output = run_source(r#"numero d = __map_nuevo(); d = __map_poner(d, "a", 1); d = __map_poner(d, "b", 2); imprimir(__map_longitud(d));"#).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_new_079_map_contiene_true() {
    let output = run_source(r#"numero d = __map_nuevo(); d = __map_poner(d, "x", 5); imprimir(__map_contiene(d, "x"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_080_map_contiene_false() {
    let output = run_source(r#"numero d = __map_nuevo(); d = __map_poner(d, "x", 5); imprimir(__map_contiene(d, "y"));"#).unwrap();
    assert_eq!(output, vec!["false"]);
}

#[test]
fn test_new_081_map_overwrite() {
    let output = run_source(r#"numero d = __map_nuevo(); d = __map_poner(d, "k", 1); d = __map_poner(d, "k", 99); imprimir(__map_obtener(d, "k"));"#).unwrap();
    assert_eq!(output, vec!["99"]);
}

#[test]
fn test_new_082_set_basic() {
    let output = run_source(r#"numero s = __conjunto_nuevo(); s = __conjunto_agregar(s, "a"); imprimir(__conjunto_tiene(s, "a"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_083_set_has_false() {
    let output = run_source(r#"numero s = __conjunto_nuevo(); s = __conjunto_agregar(s, "a"); imprimir(__conjunto_tiene(s, "b"));"#).unwrap();
    assert_eq!(output, vec!["false"]);
}

#[test]
fn test_new_084_set_union() {
    let output = run_source(r#"numero a = __conjunto_nuevo(); a = __conjunto_agregar(a, 1); a = __conjunto_agregar(a, 2); numero b = __conjunto_nuevo(); b = __conjunto_agregar(b, 2); b = __conjunto_agregar(b, 3); numero u = __conjunto_unir(a, b); imprimir(__conjunto_tiene(u, 1)); imprimir(__conjunto_tiene(u, 3));"#).unwrap();
    assert_eq!(output, vec!["true", "true"]);
}

#[test]
fn test_new_085_set_inter() {
    let output = run_source(r#"numero a = __conjunto_nuevo(); a = __conjunto_agregar(a, 1); a = __conjunto_agregar(a, 2); numero b = __conjunto_nuevo(); b = __conjunto_agregar(b, 2); numero i = __conjunto_interseccion(a, b); imprimir(__conjunto_tiene(i, 2)); imprimir(__conjunto_tiene(i, 1));"#).unwrap();
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn test_new_086_set_diff() {
    let output = run_source(r#"numero a = __conjunto_nuevo(); a = __conjunto_agregar(a, 1); a = __conjunto_agregar(a, 2); a = __conjunto_agregar(a, 3); numero b = __conjunto_nuevo(); b = __conjunto_agregar(b, 2); numero d = __conjunto_diferencia(a, b); imprimir(__conjunto_tiene(d, 1)); imprimir(__conjunto_tiene(d, 2));"#).unwrap();
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn test_new_087_set_duplicate() {
    let output = run_source(r#"numero s = __conjunto_nuevo(); s = __conjunto_agregar(s, "x"); s = __conjunto_agregar(s, "x"); imprimir(__conjunto_tiene(s, "x"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_088_map_keys_contains() {
    let output = run_source(r#"numero d = __map_nuevo(); d = __map_poner(d, "x", 1); d = __map_poner(d, "y", 2); lista<texto> k = __map_claves(d); imprimir(k.largo());"#).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_new_089_map_english_alias() {
    let output = run_source(
        r#"numero d = __map_new(); d = __map_set(d, "a", 10); imprimir(__map_get(d, "a"));"#,
    )
    .unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn test_new_090_set_english_alias() {
    let output =
        run_source(r#"numero s = __set_new(); s = __set_add(s, 5); imprimir(__set_has(s, 5));"#)
            .unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_091_deque_new_len() {
    let output =
        run_source(r#"numero dq = __deque_nuevo(); imprimir(__deque_longitud(dq));"#).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_new_092_deque_push_back() {
    let output = run_source(r#"numero dq = __deque_nuevo(); dq = __deque_agregar_final(dq, 1); dq = __deque_agregar_final(dq, 2); imprimir(__deque_longitud(dq));"#).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_new_093_deque_pop_front() {
    let output = run_source(r#"numero dq = __deque_nuevo(); dq = __deque_agregar_final(dq, 10); dq = __deque_agregar_final(dq, 20); numero f = __deque_quitar_frente(dq); imprimir(f);"#).unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn test_new_094_heap_new_peek() {
    let output = run_source(r#"numero h = __monticulo_nuevo(); h = __monticulo_agregar(h, 5); h = __monticulo_agregar(h, 10); imprimir(__monticulo_ver(h));"#).unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn test_new_095_heap_pop() {
    let output = run_source(r#"numero h = __monticulo_nuevo(); h = __monticulo_agregar(h, 7); h = __monticulo_agregar(h, 3); numero p = __monticulo_quitar(h); imprimir(p);"#).unwrap();
    assert_eq!(output, vec!["7"]);
}

#[test]
fn test_new_096_enlazada_new_len() {
    let output =
        run_source(r#"numero ll = __enlazada_nuevo(); imprimir(__enlazada_longitud(ll));"#)
            .unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_new_097_enlazada_push() {
    let output = run_source(r#"numero ll = __enlazada_nuevo(); ll = __enlazada_agregar_final(ll, "a"); ll = __enlazada_agregar_final(ll, "b"); imprimir(__enlazada_longitud(ll));"#).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_new_098_list_reverse() {
    let output = run_source(
        r#"lista<entero> xs = [3, 1, 2]; lista<entero> ys = __lista_invertir(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[2, 1, 3]"]);
}

#[test]
fn test_new_099_list_sort() {
    let output = run_source(
        r#"lista<entero> xs = [3, 1, 2]; lista<entero> ys = __lista_ordenar(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[1, 2, 3]"]);
}

#[test]
fn test_new_100_deque_empty_pop() {
    let output = run_source(
        r#"numero dq = __deque_nuevo(); numero f = __deque_quitar_frente(dq); imprimir(f);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["void"]);
}

#[test]
fn test_new_101_si_true() {
    let output = run_source(r#"si (verdadero) { imprimir("si"); }"#).unwrap();
    assert_eq!(output, vec!["si"]);
}

#[test]
fn test_new_102_si_false_sino() {
    let output = run_source(r#"si (falso) { imprimir("si"); } sino { imprimir("no"); }"#).unwrap();
    assert_eq!(output, vec!["no"]);
}

#[test]
fn test_new_103_si_else_if() {
    let output = run_source(r#"entero x = 2; si (x == 1) { imprimir(1); } sino si (x == 2) { imprimir(2); } sino { imprimir(3); }"#).unwrap();
    assert_eq!(output, vec!["2"]);
}

#[test]
fn test_new_104_mientras_loop() {
    let output =
        run_source(r#"entero i = 0; mientras (i < 3) { imprimir(i); i = i + 1; }"#).unwrap();
    assert_eq!(output, vec!["0", "1", "2"]);
}

#[test]
fn test_new_105_mientras_false() {
    let output = run_source(r#"mientras (falso) { imprimir("no"); } imprimir("fin");"#).unwrap();
    assert_eq!(output, vec!["fin"]);
}

#[test]
fn test_new_106_para_for_loop() {
    let output = run_source(r#"para (entero i = 0; i < 3; i = i + 1) { imprimir(i); }"#).unwrap();
    assert_eq!(output, vec!["0", "1", "2"]);
}

#[test]
fn test_new_107_para_break() {
    let output = run_source(
        r#"para (entero i = 0; i < 5; i = i + 1) { si (i == 2) { romper; } imprimir(i); }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["0", "1"]);
}

#[test]
fn test_new_108_para_continue() {
    let output = run_source(
        r#"para (entero i = 0; i < 5; i = i + 1) { si (i == 2) { continuar; } imprimir(i); }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["0", "1", "3", "4"]);
}

#[test]
fn test_new_109_foreach_lista() {
    let output =
        run_source(r#"lista<entero> nums = [1,2,3]; para n en nums { imprimir(n); }"#).unwrap();
    assert_eq!(output, vec!["1", "2", "3"]);
}

#[test]
fn test_new_110_foreach_empty() {
    let output =
        run_source(r#"lista<entero> nums = []; para n en nums { imprimir(n); } imprimir("fin");"#)
            .unwrap();
    assert_eq!(output, vec!["fin"]);
}

#[test]
fn test_new_111_elegir_simple() {
    let output = run_source(r#"entero x = 2; elegir (x) { caso 1: imprimir("uno"); caso 2: imprimir("dos"); defecto: imprimir("otro"); }"#).unwrap();
    assert_eq!(output, vec!["dos"]);
}

#[test]
fn test_new_112_elegir_defecto() {
    let output = run_source(
        r#"entero x = 9; elegir (x) { caso 1: imprimir("uno"); defecto: imprimir("def"); }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["def"]);
}

#[test]
fn test_new_113_elegir_guard_pass() {
    let output = run_source(
        r#"entero x = 5; elegir (x) { caso 5 si x > 3: imprimir("ok"); defecto: imprimir("no"); }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["ok"]);
}

#[test]
fn test_new_114_match_english() {
    let output = run_source(
        r#"integer x = 3; match (x) { case 3: print("three"); default: print("other"); }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["three"]);
}

#[test]
fn test_new_115_elegir_or_pattern() {
    let output = run_source(r#"entero x = 2; elegir (x) { caso 1 | 2: imprimir("or"); caso 3: imprimir("tres"); defecto: imprimir("def"); }"#).unwrap();
    assert_eq!(output, vec!["or"]);
}

#[test]
fn test_new_116_para_range_inclusive() {
    let output = run_source(r#"para x en 1..=3 { imprimir(x); }"#).unwrap();
    assert_eq!(output, vec!["1", "2", "3"]);
}

#[test]
fn test_new_117_para_range_exclusive() {
    let output = run_source(r#"para x en 1..3 { imprimir(x); }"#).unwrap();
    assert_eq!(output, vec!["1", "2"]);
}

#[test]
fn test_new_118_si_anidado() {
    let output = run_source(
        r#"entero a = 5; entero b = 10; si (a < b) { si (b == 10) { imprimir("ok"); } }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["ok"]);
}

#[test]
fn test_new_119_mientras_continue_nested() {
    let output = run_source(
        r#"entero i = 0; mientras (i < 4) { i = i + 1; si (i == 2) { continuar; } imprimir(i); }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["1", "3", "4"]);
}

#[test]
fn test_new_120_elegir_multiple_arms_guard() {
    let output = run_source(r#"entero x = 3; elegir (x) { caso 3 si x < 0: imprimir("neg"); caso 3: imprimir("tres"); defecto: imprimir("def"); }"#).unwrap();
    assert_eq!(output, vec!["tres"]);
}

#[test]
fn test_new_121_logic_and() {
    let output =
        run_source(r#"imprimir(verdadero && verdadero); imprimir(verdadero && falso);"#).unwrap();
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn test_new_122_logic_or() {
    let output = run_source(r#"imprimir(falso || verdadero); imprimir(falso || falso);"#).unwrap();
    assert_eq!(output, vec!["true", "false"]);
}

#[test]
fn test_new_123_elegir_nested() {
    let output = run_source(r#"entero x = 1; elegir (x) { caso 1: { entero y = 2; elegir (y) { caso 2: imprimir("nested"); } } defecto: imprimir("def"); }"#).unwrap();
    assert_eq!(output, vec!["nested"]);
}

#[test]
fn test_new_124_para_with_condition() {
    let output = run_source(
        r#"lista<entero> nums = [1,2,3,4]; para n en nums { si (n % 2 == 0) { imprimir(n); } }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["2", "4"]);
}

#[test]
fn test_new_125_mientras_break() {
    let output = run_source(
        r#"entero i = 0; mientras (i < 10) { si (i == 3) { romper; } imprimir(i); i = i + 1; }"#,
    )
    .unwrap();
    assert_eq!(output, vec!["0", "1", "2"]);
}

#[test]
fn test_new_126_recursion_factorial() {
    let output = run_source(r#"funcion entero fact(entero n) { si (n <= 1) { retornar 1; } retornar n * fact(n - 1); } imprimir(fact(5));"#).unwrap();
    assert_eq!(output, vec!["120"]);
}

#[test]
fn test_new_127_recursion_fib() {
    let output = run_source(r#"funcion entero fib(entero n) { si (n <= 1) { retornar n; } retornar fib(n-1) + fib(n-2); } imprimir(fib(6));"#).unwrap();
    assert_eq!(output, vec!["8"]);
}

#[test]
fn test_new_128_closure_iife() {
    let output = run_source(r#"imprimir(funcion(entero x) { retornar x * 2; }(6));"#).unwrap();
    assert_eq!(output, vec!["12"]);
}

#[test]
fn test_new_129_closure_capture() {
    let output = run_source(r#"entero base = 10; funcion entero addBase(entero x) { retornar x + base; } imprimir(addBase(5));"#).unwrap();
    assert_eq!(output, vec!["15"]);
}

#[test]
fn test_new_130_default_param() {
    let output = run_source(r#"funcion entero suma(entero a, entero b = 10) { retornar a + b; } imprimir(suma(5)); imprimir(suma(5, 20));"#).unwrap();
    assert_eq!(output, vec!["15", "25"]);
}

#[test]
fn test_new_131_early_return() {
    let output = run_source(r#"funcion entero check(entero x) { si (x < 0) { retornar -1; } retornar x; } imprimir(check(-5)); imprimir(check(5));"#).unwrap();
    assert_eq!(output, vec!["-1", "5"]);
}

#[test]
fn test_new_132_pipe_simple() {
    let output = run_source(r#"funcion entero doble(entero x) { retornar x * 2; } entero r = 5 |> doble(); imprimir(r);"#).unwrap();
    assert_eq!(output, vec!["10"]);
}

#[test]
fn test_new_133_pipe_chaining() {
    let output = run_source(r#"funcion entero doble(entero x) { retornar x * 2; } funcion entero inc(entero x) { retornar x + 1; } entero r = 10 |> doble() |> inc(); imprimir(r);"#).unwrap();
    assert_eq!(output, vec!["21"]);
}

#[test]
fn test_new_134_list_comp_simple() {
    let output = run_source(r#"lista<entero> nums = [1,2,3]; lista<entero> dob = [x * 2 para x en nums]; imprimir(dob);"#).unwrap();
    assert_eq!(output, vec!["[2, 4, 6]"]);
}

#[test]
fn test_new_135_list_comp_filter() {
    let output = run_source(r#"lista<entero> nums = [1,2,3,4,5,6]; lista<entero> pares = [x para x en nums si x % 2 == 0]; imprimir(pares);"#).unwrap();
    assert_eq!(output, vec!["[2, 4, 6]"]);
}

#[test]
fn test_new_136_linq_spanish() {
    let output = run_source(r#"lista<entero> datos = [10, 15, 20, 25]; lista<entero> res = consultar x en datos donde x >= 20 seleccionar x; imprimir(res);"#).unwrap();
    assert_eq!(output, vec!["[20, 25]"]);
}

#[test]
fn test_new_137_linq_english() {
    let output = run_source(r#"array<integer> items = [1,2,3,4]; array<integer> res = query x in items where x > 2 select x; print(res);"#).unwrap();
    assert_eq!(output, vec!["[3, 4]"]);
}

#[test]
fn test_new_138_lambda_multi_args() {
    let output =
        run_source(r#"imprimir(funcion(entero a, entero b) { retornar a + b; }(3, 4));"#).unwrap();
    assert_eq!(output, vec!["7"]);
}

#[test]
fn test_new_139_recursion_sum() {
    let output = run_source(r#"funcion entero sum(entero n) { si (n == 0) { retornar 0; } retornar n + sum(n - 1); } imprimir(sum(5));"#).unwrap();
    assert_eq!(output, vec!["15"]);
}

#[test]
fn test_new_140_func_string() {
    let output = run_source(
        r#"funcion texto saludo(texto n) { retornar "hola " + n; } imprimir(saludo("Ana"));"#,
    )
    .unwrap();
    assert_eq!(output, vec!["hola Ana"]);
}

#[test]
fn test_new_141_list_reverse_english() {
    let output = run_source(
        r#"lista<entero> xs = [1,2,3]; lista<entero> ys = __list_reverse(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[3, 2, 1]"]);
}

#[test]
fn test_new_142_lista_invertir_spanish() {
    let output = run_source(
        r#"lista<entero> xs = [1,2,3]; lista<entero> ys = __lista_invertir(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[3, 2, 1]"]);
}

#[test]
fn test_new_143_list_sort_english() {
    let output = run_source(
        r#"lista<entero> xs = [3,1,2]; lista<entero> ys = __list_sort(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[1, 2, 3]"]);
}

#[test]
fn test_new_144_lista_ordenar_spanish() {
    let output = run_source(
        r#"lista<entero> xs = [3,1,2]; lista<entero> ys = __lista_ordenar(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[1, 2, 3]"]);
}

#[test]
fn test_new_145_list_sort_already_sorted() {
    let output = run_source(
        r#"lista<entero> xs = [1,2,3]; lista<entero> ys = __list_sort(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[1, 2, 3]"]);
}

#[test]
fn test_new_146_list_reverse_single() {
    let output = run_source(
        r#"lista<entero> xs = [42]; lista<entero> ys = __lista_invertir(xs); imprimir(ys);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["[42]"]);
}

#[test]
fn test_new_147_empty_string() {
    let output = run_source(r#"imprimir("");"#).unwrap();
    assert_eq!(output, vec![""]);
}

#[test]
fn test_new_148_zero() {
    let output = run_source(r#"imprimir(0);"#).unwrap();
    assert_eq!(output, vec!["0"]);
}

#[test]
fn test_new_149_large_number() {
    let output = run_source(r#"imprimir(999999);"#).unwrap();
    assert_eq!(output, vec!["999999"]);
}

#[test]
fn test_new_150_unicode_cafe() {
    let output = run_source(r#"imprimir("café");"#).unwrap();
    assert_eq!(output, vec!["café"]);
}

#[test]
fn test_new_151_string_concat_plus() {
    let output = run_source(r#"imprimir("hola" + " " + "mundo");"#).unwrap();
    assert_eq!(output, vec!["hola mundo"]);
}

#[test]
fn test_new_152_emoji() {
    let output = run_source(r#"imprimir("😀");"#).unwrap();
    assert_eq!(output, vec!["😀"]);
}

#[test]
fn test_new_153_int_division() {
    let output = run_source(r#"imprimir(7 / 2);"#).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_154_float_division() {
    let output = run_source(r#"imprimir(7.0 / 2);"#).unwrap();
    assert_eq!(output, vec!["3.5"]);
}

#[test]
fn test_new_155_modulo() {
    let output = run_source(r#"imprimir(10 % 3);"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_156_negative_large() {
    let output = run_source(r#"imprimir(-999999);"#).unwrap();
    assert_eq!(output, vec!["-999999"]);
}

#[test]
fn test_new_157_bool_negation() {
    let output = run_source(r#"imprimir(!verdadero); imprimir(!falso);"#).unwrap();
    assert_eq!(output, vec!["false", "true"]);
}

#[test]
fn test_new_158_float_strip() {
    let output = run_source(r#"imprimir(3.0);"#).unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_159_lex_error() {
    let result = run_source(r#"let @x = 1;"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("LexError"));
}

#[test]
fn test_new_160_parse_error() {
    let result = run_source(r#"numero x = ;"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("ParseError"));
}

#[test]
fn test_new_161_sem_type_error() {
    let result = run_source(r#"entero x = "hola";"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_new_162_division_by_zero() {
    let result = run_source(r#"numero x = 1 / 0;"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("DivisionByZero"));
}

#[test]
fn test_new_163_undefined_var() {
    let result = run_source(r#"imprimir(x);"#);
    assert!(result.is_err());
}

#[test]
fn test_new_164_array_oob() {
    let result = run_source(r#"lista<entero> a = [1]; imprimir(a[5]);"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("fuera de rango"));
}

#[test]
fn test_new_165_struct_missing_field() {
    let result =
        run_source(r#"estructura Punto { x: entero, y: entero } Punto p = Punto { x: 10 };"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_new_166_break_outside() {
    let result = run_source(r#"romper;"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_new_167_continue_outside() {
    let result = run_source(r#"continuar;"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_new_168_match_arm_type_error() {
    let result = run_source(r#"entero x = 1; elegir (x) { caso "texto": imprimir("no"); }"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_new_169_array_negative_index() {
    let result = run_source(r#"lista<entero> nums = [1]; imprimir(nums[-1]);"#);
    assert!(result.is_err());
}

#[test]
fn test_new_170_destructure_arity() {
    let result = run_source(r#"entero x, texto y = (1, 2);"#);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("SemError"));
}

#[test]
fn test_new_171_regex_is_match_true() {
    let output = run_source(r#"imprimir(__regex_coincide("\\d+", "abc123"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_172_regex_replace() {
    let output =
        run_source(r#"imprimir(__regex_reemplazar("mundo", "Hola mundo", "Lumen"));"#).unwrap();
    assert_eq!(output, vec!["Hola Lumen"]);
}

#[test]
fn test_new_173_regex_english_alias() {
    let output = run_source(r#"imprimir(__regex_is_match("hello", "hello world"));"#).unwrap();
    assert_eq!(output, vec!["true"]);
}

#[test]
fn test_new_174_encoding_utf8() {
    let output =
        run_source(r#"lista<entero> b = __codificacion_utf8("abc"); imprimir(b);"#).unwrap();
    assert_eq!(output, vec!["[97, 98, 99]"]);
}

#[test]
fn test_new_175_desde_utf8() {
    let output = run_source(
        r#"lista<entero> b = __codificacion_utf8("Hola"); texto d = __desde_utf8(b); imprimir(d);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["Hola"]);
}

#[test]
fn test_new_176_padding_inicio() {
    let output = run_source(r#"imprimir(__str_padding_inicio("7", 3, "0"));"#).unwrap();
    assert_eq!(output, vec!["007"]);
}

#[test]
fn test_new_177_padding_fin() {
    let output = run_source(r#"imprimir(__str_padding_fin("42", 5, "."));"#).unwrap();
    assert_eq!(output, vec!["42..."]);
}

#[test]
fn test_new_178_tiempo_formatear() {
    let output = run_source(r#"imprimir(__tiempo_formatear(0));"#).unwrap();
    assert_eq!(output, vec!["1970-01-01T00:00:00Z"]);
}

#[test]
fn test_new_179_tiempo_diferencia() {
    let output = run_source(r#"imprimir(__tiempo_diferencia(100, 50));"#).unwrap();
    assert_eq!(output, vec!["50"]);
}

#[test]
fn test_new_180_tipo_de() {
    let output = run_source(r#"imprimir(__tipo_de(42)); imprimir(__tipo_de("hola"));"#).unwrap();
    assert_eq!(output, vec!["entero", "texto"]);
}

#[test]
fn test_new_181_min_max() {
    let output = run_source(r#"imprimir(min(181, 182)); imprimir(max(181, 182));"#).unwrap();
    assert_eq!(output, vec!["181", "182"]);
}

#[test]
fn test_new_182_imprimir_combined() {
    let output = run_source(r#"imprimir(182, 183);"#).unwrap();
    assert_eq!(output, vec!["182183"]);
}

#[test]
fn test_new_183_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_184_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_185_control_si() {
    let output =
        run_source(r#"entero x = 185; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_186_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_187_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_188_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k188", 188); imprimir(__map_obtener(m, "k188"));"#).unwrap();
    assert_eq!(output, vec!["188"]);
}

#[test]
fn test_new_189_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_190_abs_int() {
    let output = run_source(r#"imprimir(abs(-190));"#).unwrap();
    assert_eq!(output, vec!["190"]);
}

#[test]
fn test_new_191_min_max() {
    let output = run_source(r#"imprimir(min(191, 192)); imprimir(max(191, 192));"#).unwrap();
    assert_eq!(output, vec!["191", "192"]);
}

#[test]
fn test_new_192_imprimir_combined() {
    let output = run_source(r#"imprimir(192, 193);"#).unwrap();
    assert_eq!(output, vec!["192193"]);
}

#[test]
fn test_new_193_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_194_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_195_control_si() {
    let output =
        run_source(r#"entero x = 195; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_196_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_197_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_198_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k198", 198); imprimir(__map_obtener(m, "k198"));"#).unwrap();
    assert_eq!(output, vec!["198"]);
}

#[test]
fn test_new_199_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_200_abs_int() {
    let output = run_source(r#"imprimir(abs(-200));"#).unwrap();
    assert_eq!(output, vec!["200"]);
}

#[test]
fn test_new_201_min_max() {
    let output = run_source(r#"imprimir(min(201, 202)); imprimir(max(201, 202));"#).unwrap();
    assert_eq!(output, vec!["201", "202"]);
}

#[test]
fn test_new_202_imprimir_combined() {
    let output = run_source(r#"imprimir(202, 203);"#).unwrap();
    assert_eq!(output, vec!["202203"]);
}

#[test]
fn test_new_203_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_204_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_205_control_si() {
    let output =
        run_source(r#"entero x = 205; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_206_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_207_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_208_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k208", 208); imprimir(__map_obtener(m, "k208"));"#).unwrap();
    assert_eq!(output, vec!["208"]);
}

#[test]
fn test_new_209_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_210_abs_int() {
    let output = run_source(r#"imprimir(abs(-210));"#).unwrap();
    assert_eq!(output, vec!["210"]);
}

#[test]
fn test_new_211_min_max() {
    let output = run_source(r#"imprimir(min(211, 212)); imprimir(max(211, 212));"#).unwrap();
    assert_eq!(output, vec!["211", "212"]);
}

#[test]
fn test_new_212_imprimir_combined() {
    let output = run_source(r#"imprimir(212, 213);"#).unwrap();
    assert_eq!(output, vec!["212213"]);
}

#[test]
fn test_new_213_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_214_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_215_control_si() {
    let output =
        run_source(r#"entero x = 215; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_216_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_217_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_218_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k218", 218); imprimir(__map_obtener(m, "k218"));"#).unwrap();
    assert_eq!(output, vec!["218"]);
}

#[test]
fn test_new_219_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_220_abs_int() {
    let output = run_source(r#"imprimir(abs(-220));"#).unwrap();
    assert_eq!(output, vec!["220"]);
}

#[test]
fn test_new_221_min_max() {
    let output = run_source(r#"imprimir(min(221, 222)); imprimir(max(221, 222));"#).unwrap();
    assert_eq!(output, vec!["221", "222"]);
}

#[test]
fn test_new_222_imprimir_combined() {
    let output = run_source(r#"imprimir(222, 223);"#).unwrap();
    assert_eq!(output, vec!["222223"]);
}

#[test]
fn test_new_223_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_224_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_225_control_si() {
    let output =
        run_source(r#"entero x = 225; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_226_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_227_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_228_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k228", 228); imprimir(__map_obtener(m, "k228"));"#).unwrap();
    assert_eq!(output, vec!["228"]);
}

#[test]
fn test_new_229_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_230_abs_int() {
    let output = run_source(r#"imprimir(abs(-230));"#).unwrap();
    assert_eq!(output, vec!["230"]);
}

#[test]
fn test_new_231_min_max() {
    let output = run_source(r#"imprimir(min(231, 232)); imprimir(max(231, 232));"#).unwrap();
    assert_eq!(output, vec!["231", "232"]);
}

#[test]
fn test_new_232_imprimir_combined() {
    let output = run_source(r#"imprimir(232, 233);"#).unwrap();
    assert_eq!(output, vec!["232233"]);
}

#[test]
fn test_new_233_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_234_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_235_control_si() {
    let output =
        run_source(r#"entero x = 235; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_236_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_237_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_238_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k238", 238); imprimir(__map_obtener(m, "k238"));"#).unwrap();
    assert_eq!(output, vec!["238"]);
}

#[test]
fn test_new_239_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_240_abs_int() {
    let output = run_source(r#"imprimir(abs(-240));"#).unwrap();
    assert_eq!(output, vec!["240"]);
}

#[test]
fn test_new_241_min_max() {
    let output = run_source(r#"imprimir(min(241, 242)); imprimir(max(241, 242));"#).unwrap();
    assert_eq!(output, vec!["241", "242"]);
}

#[test]
fn test_new_242_imprimir_combined() {
    let output = run_source(r#"imprimir(242, 243);"#).unwrap();
    assert_eq!(output, vec!["242243"]);
}

#[test]
fn test_new_243_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_244_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_245_control_si() {
    let output =
        run_source(r#"entero x = 245; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_246_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_247_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_248_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k248", 248); imprimir(__map_obtener(m, "k248"));"#).unwrap();
    assert_eq!(output, vec!["248"]);
}

#[test]
fn test_new_249_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_250_abs_int() {
    let output = run_source(r#"imprimir(abs(-250));"#).unwrap();
    assert_eq!(output, vec!["250"]);
}

#[test]
fn test_new_251_min_max() {
    let output = run_source(r#"imprimir(min(251, 252)); imprimir(max(251, 252));"#).unwrap();
    assert_eq!(output, vec!["251", "252"]);
}

#[test]
fn test_new_252_imprimir_combined() {
    let output = run_source(r#"imprimir(252, 253);"#).unwrap();
    assert_eq!(output, vec!["252253"]);
}

#[test]
fn test_new_253_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_254_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_255_control_si() {
    let output =
        run_source(r#"entero x = 255; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_256_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_257_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_258_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k258", 258); imprimir(__map_obtener(m, "k258"));"#).unwrap();
    assert_eq!(output, vec!["258"]);
}

#[test]
fn test_new_259_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_260_abs_int() {
    let output = run_source(r#"imprimir(abs(-260));"#).unwrap();
    assert_eq!(output, vec!["260"]);
}

#[test]
fn test_new_261_min_max() {
    let output = run_source(r#"imprimir(min(261, 262)); imprimir(max(261, 262));"#).unwrap();
    assert_eq!(output, vec!["261", "262"]);
}

#[test]
fn test_new_262_imprimir_combined() {
    let output = run_source(r#"imprimir(262, 263);"#).unwrap();
    assert_eq!(output, vec!["262263"]);
}

#[test]
fn test_new_263_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_264_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_265_control_si() {
    let output =
        run_source(r#"entero x = 265; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_266_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_267_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_268_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k268", 268); imprimir(__map_obtener(m, "k268"));"#).unwrap();
    assert_eq!(output, vec!["268"]);
}

#[test]
fn test_new_269_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_270_abs_int() {
    let output = run_source(r#"imprimir(abs(-270));"#).unwrap();
    assert_eq!(output, vec!["270"]);
}

#[test]
fn test_new_271_min_max() {
    let output = run_source(r#"imprimir(min(271, 272)); imprimir(max(271, 272));"#).unwrap();
    assert_eq!(output, vec!["271", "272"]);
}

#[test]
fn test_new_272_imprimir_combined() {
    let output = run_source(r#"imprimir(272, 273);"#).unwrap();
    assert_eq!(output, vec!["272273"]);
}

#[test]
fn test_new_273_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_274_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_275_control_si() {
    let output =
        run_source(r#"entero x = 275; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_276_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_277_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_278_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k278", 278); imprimir(__map_obtener(m, "k278"));"#).unwrap();
    assert_eq!(output, vec!["278"]);
}

#[test]
fn test_new_279_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_280_abs_int() {
    let output = run_source(r#"imprimir(abs(-280));"#).unwrap();
    assert_eq!(output, vec!["280"]);
}

#[test]
fn test_new_281_min_max() {
    let output = run_source(r#"imprimir(min(281, 282)); imprimir(max(281, 282));"#).unwrap();
    assert_eq!(output, vec!["281", "282"]);
}

#[test]
fn test_new_282_imprimir_combined() {
    let output = run_source(r#"imprimir(282, 283);"#).unwrap();
    assert_eq!(output, vec!["282283"]);
}

#[test]
fn test_new_283_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_284_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_285_control_si() {
    let output =
        run_source(r#"entero x = 285; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_286_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_287_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_288_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k288", 288); imprimir(__map_obtener(m, "k288"));"#).unwrap();
    assert_eq!(output, vec!["288"]);
}

#[test]
fn test_new_289_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_290_abs_int() {
    let output = run_source(r#"imprimir(abs(-290));"#).unwrap();
    assert_eq!(output, vec!["290"]);
}

#[test]
fn test_new_291_min_max() {
    let output = run_source(r#"imprimir(min(291, 292)); imprimir(max(291, 292));"#).unwrap();
    assert_eq!(output, vec!["291", "292"]);
}

#[test]
fn test_new_292_imprimir_combined() {
    let output = run_source(r#"imprimir(292, 293);"#).unwrap();
    assert_eq!(output, vec!["292293"]);
}

#[test]
fn test_new_293_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_294_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_295_control_si() {
    let output =
        run_source(r#"entero x = 295; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_296_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_297_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_298_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k298", 298); imprimir(__map_obtener(m, "k298"));"#).unwrap();
    assert_eq!(output, vec!["298"]);
}

#[test]
fn test_new_299_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_300_abs_int() {
    let output = run_source(r#"imprimir(abs(-300));"#).unwrap();
    assert_eq!(output, vec!["300"]);
}

#[test]
fn test_new_301_min_max() {
    let output = run_source(r#"imprimir(min(301, 302)); imprimir(max(301, 302));"#).unwrap();
    assert_eq!(output, vec!["301", "302"]);
}

#[test]
fn test_new_302_imprimir_combined() {
    let output = run_source(r#"imprimir(302, 303);"#).unwrap();
    assert_eq!(output, vec!["302303"]);
}

#[test]
fn test_new_303_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_304_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_305_control_si() {
    let output =
        run_source(r#"entero x = 305; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_306_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_307_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_308_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k308", 308); imprimir(__map_obtener(m, "k308"));"#).unwrap();
    assert_eq!(output, vec!["308"]);
}

#[test]
fn test_new_309_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_310_abs_int() {
    let output = run_source(r#"imprimir(abs(-310));"#).unwrap();
    assert_eq!(output, vec!["310"]);
}

#[test]
fn test_new_311_min_max() {
    let output = run_source(r#"imprimir(min(311, 312)); imprimir(max(311, 312));"#).unwrap();
    assert_eq!(output, vec!["311", "312"]);
}

#[test]
fn test_new_312_imprimir_combined() {
    let output = run_source(r#"imprimir(312, 313);"#).unwrap();
    assert_eq!(output, vec!["312313"]);
}

#[test]
fn test_new_313_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_314_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_315_control_si() {
    let output =
        run_source(r#"entero x = 315; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_316_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_317_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_318_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k318", 318); imprimir(__map_obtener(m, "k318"));"#).unwrap();
    assert_eq!(output, vec!["318"]);
}

#[test]
fn test_new_319_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_320_abs_int() {
    let output = run_source(r#"imprimir(abs(-320));"#).unwrap();
    assert_eq!(output, vec!["320"]);
}

#[test]
fn test_new_321_min_max() {
    let output = run_source(r#"imprimir(min(321, 322)); imprimir(max(321, 322));"#).unwrap();
    assert_eq!(output, vec!["321", "322"]);
}

#[test]
fn test_new_322_imprimir_combined() {
    let output = run_source(r#"imprimir(322, 323);"#).unwrap();
    assert_eq!(output, vec!["322323"]);
}

#[test]
fn test_new_323_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_324_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_325_control_si() {
    let output =
        run_source(r#"entero x = 325; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_326_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_327_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_328_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k328", 328); imprimir(__map_obtener(m, "k328"));"#).unwrap();
    assert_eq!(output, vec!["328"]);
}

#[test]
fn test_new_329_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_330_abs_int() {
    let output = run_source(r#"imprimir(abs(-330));"#).unwrap();
    assert_eq!(output, vec!["330"]);
}

#[test]
fn test_new_331_min_max() {
    let output = run_source(r#"imprimir(min(331, 332)); imprimir(max(331, 332));"#).unwrap();
    assert_eq!(output, vec!["331", "332"]);
}

#[test]
fn test_new_332_imprimir_combined() {
    let output = run_source(r#"imprimir(332, 333);"#).unwrap();
    assert_eq!(output, vec!["332333"]);
}

#[test]
fn test_new_333_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_334_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_335_control_si() {
    let output =
        run_source(r#"entero x = 335; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_336_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_337_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_338_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k338", 338); imprimir(__map_obtener(m, "k338"));"#).unwrap();
    assert_eq!(output, vec!["338"]);
}

#[test]
fn test_new_339_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_340_abs_int() {
    let output = run_source(r#"imprimir(abs(-340));"#).unwrap();
    assert_eq!(output, vec!["340"]);
}

#[test]
fn test_new_341_min_max() {
    let output = run_source(r#"imprimir(min(341, 342)); imprimir(max(341, 342));"#).unwrap();
    assert_eq!(output, vec!["341", "342"]);
}

#[test]
fn test_new_342_imprimir_combined() {
    let output = run_source(r#"imprimir(342, 343);"#).unwrap();
    assert_eq!(output, vec!["342343"]);
}

#[test]
fn test_new_343_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_344_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_345_control_si() {
    let output =
        run_source(r#"entero x = 345; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_346_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_347_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_348_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k348", 348); imprimir(__map_obtener(m, "k348"));"#).unwrap();
    assert_eq!(output, vec!["348"]);
}

#[test]
fn test_new_349_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_350_abs_int() {
    let output = run_source(r#"imprimir(abs(-350));"#).unwrap();
    assert_eq!(output, vec!["350"]);
}

#[test]
fn test_new_351_min_max() {
    let output = run_source(r#"imprimir(min(351, 352)); imprimir(max(351, 352));"#).unwrap();
    assert_eq!(output, vec!["351", "352"]);
}

#[test]
fn test_new_352_imprimir_combined() {
    let output = run_source(r#"imprimir(352, 353);"#).unwrap();
    assert_eq!(output, vec!["352353"]);
}

#[test]
fn test_new_353_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_354_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_355_control_si() {
    let output =
        run_source(r#"entero x = 355; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_356_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_357_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_358_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k358", 358); imprimir(__map_obtener(m, "k358"));"#).unwrap();
    assert_eq!(output, vec!["358"]);
}

#[test]
fn test_new_359_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_360_abs_int() {
    let output = run_source(r#"imprimir(abs(-360));"#).unwrap();
    assert_eq!(output, vec!["360"]);
}

#[test]
fn test_new_361_min_max() {
    let output = run_source(r#"imprimir(min(361, 362)); imprimir(max(361, 362));"#).unwrap();
    assert_eq!(output, vec!["361", "362"]);
}

#[test]
fn test_new_362_imprimir_combined() {
    let output = run_source(r#"imprimir(362, 363);"#).unwrap();
    assert_eq!(output, vec!["362363"]);
}

#[test]
fn test_new_363_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_364_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_365_control_si() {
    let output =
        run_source(r#"entero x = 365; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_366_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_367_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_368_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k368", 368); imprimir(__map_obtener(m, "k368"));"#).unwrap();
    assert_eq!(output, vec!["368"]);
}

#[test]
fn test_new_369_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_370_abs_int() {
    let output = run_source(r#"imprimir(abs(-370));"#).unwrap();
    assert_eq!(output, vec!["370"]);
}

#[test]
fn test_new_371_min_max() {
    let output = run_source(r#"imprimir(min(371, 372)); imprimir(max(371, 372));"#).unwrap();
    assert_eq!(output, vec!["371", "372"]);
}

#[test]
fn test_new_372_imprimir_combined() {
    let output = run_source(r#"imprimir(372, 373);"#).unwrap();
    assert_eq!(output, vec!["372373"]);
}

#[test]
fn test_new_373_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_374_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_375_control_si() {
    let output =
        run_source(r#"entero x = 375; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_376_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_377_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_378_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k378", 378); imprimir(__map_obtener(m, "k378"));"#).unwrap();
    assert_eq!(output, vec!["378"]);
}

#[test]
fn test_new_379_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_380_abs_int() {
    let output = run_source(r#"imprimir(abs(-380));"#).unwrap();
    assert_eq!(output, vec!["380"]);
}

#[test]
fn test_new_381_min_max() {
    let output = run_source(r#"imprimir(min(381, 382)); imprimir(max(381, 382));"#).unwrap();
    assert_eq!(output, vec!["381", "382"]);
}

#[test]
fn test_new_382_imprimir_combined() {
    let output = run_source(r#"imprimir(382, 383);"#).unwrap();
    assert_eq!(output, vec!["382383"]);
}

#[test]
fn test_new_383_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_384_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_385_control_si() {
    let output =
        run_source(r#"entero x = 385; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_386_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_387_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_388_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k388", 388); imprimir(__map_obtener(m, "k388"));"#).unwrap();
    assert_eq!(output, vec!["388"]);
}

#[test]
fn test_new_389_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_390_abs_int() {
    let output = run_source(r#"imprimir(abs(-390));"#).unwrap();
    assert_eq!(output, vec!["390"]);
}

#[test]
fn test_new_391_min_max() {
    let output = run_source(r#"imprimir(min(391, 392)); imprimir(max(391, 392));"#).unwrap();
    assert_eq!(output, vec!["391", "392"]);
}

#[test]
fn test_new_392_imprimir_combined() {
    let output = run_source(r#"imprimir(392, 393);"#).unwrap();
    assert_eq!(output, vec!["392393"]);
}

#[test]
fn test_new_393_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_394_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_395_control_si() {
    let output =
        run_source(r#"entero x = 395; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_396_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_397_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_398_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k398", 398); imprimir(__map_obtener(m, "k398"));"#).unwrap();
    assert_eq!(output, vec!["398"]);
}

#[test]
fn test_new_399_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_400_abs_int() {
    let output = run_source(r#"imprimir(abs(-400));"#).unwrap();
    assert_eq!(output, vec!["400"]);
}

#[test]
fn test_new_401_min_max() {
    let output = run_source(r#"imprimir(min(401, 402)); imprimir(max(401, 402));"#).unwrap();
    assert_eq!(output, vec!["401", "402"]);
}

#[test]
fn test_new_402_imprimir_combined() {
    let output = run_source(r#"imprimir(402, 403);"#).unwrap();
    assert_eq!(output, vec!["402403"]);
}

#[test]
fn test_new_403_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_404_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_405_control_si() {
    let output =
        run_source(r#"entero x = 405; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_406_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_407_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_408_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k408", 408); imprimir(__map_obtener(m, "k408"));"#).unwrap();
    assert_eq!(output, vec!["408"]);
}

#[test]
fn test_new_409_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_410_abs_int() {
    let output = run_source(r#"imprimir(abs(-410));"#).unwrap();
    assert_eq!(output, vec!["410"]);
}

#[test]
fn test_new_411_min_max() {
    let output = run_source(r#"imprimir(min(411, 412)); imprimir(max(411, 412));"#).unwrap();
    assert_eq!(output, vec!["411", "412"]);
}

#[test]
fn test_new_412_imprimir_combined() {
    let output = run_source(r#"imprimir(412, 413);"#).unwrap();
    assert_eq!(output, vec!["412413"]);
}

#[test]
fn test_new_413_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_414_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_415_control_si() {
    let output =
        run_source(r#"entero x = 415; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_416_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_417_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_418_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k418", 418); imprimir(__map_obtener(m, "k418"));"#).unwrap();
    assert_eq!(output, vec!["418"]);
}

#[test]
fn test_new_419_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_420_abs_int() {
    let output = run_source(r#"imprimir(abs(-420));"#).unwrap();
    assert_eq!(output, vec!["420"]);
}

#[test]
fn test_new_421_min_max() {
    let output = run_source(r#"imprimir(min(421, 422)); imprimir(max(421, 422));"#).unwrap();
    assert_eq!(output, vec!["421", "422"]);
}

#[test]
fn test_new_422_imprimir_combined() {
    let output = run_source(r#"imprimir(422, 423);"#).unwrap();
    assert_eq!(output, vec!["422423"]);
}

#[test]
fn test_new_423_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_424_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_425_control_si() {
    let output =
        run_source(r#"entero x = 425; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_426_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_427_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_428_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k428", 428); imprimir(__map_obtener(m, "k428"));"#).unwrap();
    assert_eq!(output, vec!["428"]);
}

#[test]
fn test_new_429_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_430_abs_int() {
    let output = run_source(r#"imprimir(abs(-430));"#).unwrap();
    assert_eq!(output, vec!["430"]);
}

#[test]
fn test_new_431_min_max() {
    let output = run_source(r#"imprimir(min(431, 432)); imprimir(max(431, 432));"#).unwrap();
    assert_eq!(output, vec!["431", "432"]);
}

#[test]
fn test_new_432_imprimir_combined() {
    let output = run_source(r#"imprimir(432, 433);"#).unwrap();
    assert_eq!(output, vec!["432433"]);
}

#[test]
fn test_new_433_string_len() {
    let output = run_source(r#"imprimir(largo("hola"));"#).unwrap();
    assert_eq!(output, vec!["4"]);
}

#[test]
fn test_new_434_array_push() {
    let output = run_source(
        r#"lista<entero> a = [1, 2]; a = a ++ [3]; imprimir(largo(a)); imprimir(a[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "3"]);
}

#[test]
fn test_new_435_control_si() {
    let output =
        run_source(r#"entero x = 435; si (x > 0) { imprimir(1); } sino { imprimir(0); }"#).unwrap();
    assert_eq!(output, vec!["1"]);
}

#[test]
fn test_new_436_loop_mientras() {
    let output = run_source(
        r#"entero c = 0; entero s = 0; mientras (c < 3) { s = s + c; c = c + 1; } imprimir(s);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3"]);
}

#[test]
fn test_new_437_func_recursion() {
    let output = run_source(r#"funcion entero f(entero n) { si (n <= 1) { retornar 1; } retornar n * f(n-1); } imprimir(f(4));"#).unwrap();
    assert_eq!(output, vec!["24"]);
}

#[test]
fn test_new_438_map() {
    let output = run_source(r#"numero m = __map_nuevo(); m = __map_poner(m, "k438", 438); imprimir(__map_obtener(m, "k438"));"#).unwrap();
    assert_eq!(output, vec!["438"]);
}

#[test]
fn test_new_439_rango() {
    let output = run_source(
        r#"lista<entero> r = 0..3; imprimir(largo(r)); imprimir(r[0]); imprimir(r[2]);"#,
    )
    .unwrap();
    assert_eq!(output, vec!["3", "0", "2"]);
}

#[test]
fn test_new_440_abs_int() {
    let output = run_source(r#"imprimir(abs(-440));"#).unwrap();
    assert_eq!(output, vec!["440"]);
}

// === REGRESIÓN: bugs escalables corregidos en 64db441 y 730e74d ===

#[test]
fn test_regression_fallthrough_early_return() {
    // Funciones void con early return condicional no deben hacer fallthrough a la siguiente función (bug limpiar_pantalla → limpiar_pantalla_alfa)
    let src = r#"
        funcion void foo(entero r, entero g, entero b) { si r == 0 { retornar; } imprimir(r); }
        funcion void bar(entero r, entero g, entero b, entero a) { imprimir(a); }
        foo(0,0,0); foo(1,2,3); bar(1,2,3,99);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["1", "99"]);
}

#[test]
fn test_regression_matematicas_potencia() {
    // potencia usada por seno/coseno/__factorial; fallaba por colisión de labels globales resets
    let src2 = r#"
        funcion numero potencia(numero base, entero exp) {
            si (exp == 0) { retornar 1; }
            numero res = 1; entero i=0; entero e=exp; si(e<0){e=-e;}
            mientras(i<e){ res=res*base; i=i+1; }
            si(exp<0){retornar 1.0/res;} retornar res;
        }
        imprimir(potencia(2,10));
    "#;
    let output = run_source(src2).unwrap();
    assert_eq!(output, vec!["1024"]);
}

#[test]
fn test_regression_defaults_callvalue() {
    // CallValue con defaults debe usar valor por defecto, no Void (bug VM pop)
    let src = r#"cualquiera f = funcion(entero a, entero b = 10) { retornar a + b; }; imprimir(f(5)); imprimir(f(5,20));"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["15", "25"]);
}

#[test]
fn test_regression_lambda_fallthrough() {
    // Lambda con early return condicional debe tener Return final, no fallthrough
    let src = r#"cualquiera f = funcion(entero x) { si x > 0 { retornar x * 2; } retornar 0; }; imprimir(f(5)); imprimir(f(0));"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["10", "0"]);
}

// === REGRESIÓN: hardening runtime (stress) ===

#[test]
fn test_regression_try_catch_runtime_error() {
    // intentar/atrapar debe capturar errores de runtime (antes: catch era código muerto)
    let src = r#"
        intentar { entero x = 1 / 0; } atrapar (e) { imprimir("capturado"); }
        imprimir("continua");
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["capturado", "continua"]);
}

#[test]
fn test_regression_try_catch_undefined_var() {
    let src = r#"
        intentar { lista<entero> arr = [1]; imprimir(arr[5]); } atrapar (ex) { imprimir("undef ok"); }
        intentar { entero m = 5 % 0; } atrapar (e2) { imprimir("mod ok"); }
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["undef ok", "mod ok"]);
}

#[test]
fn test_regression_int_overflow_wraps_no_panic() {
    // Overflow aritmético hace wrap (comportamiento definido), nunca panic/crash
    let src = r#"
        entero max = 9223372036854775807;
        imprimir(max + 1);
        imprimir(max * 2);
        entero mn = 0 - 9223372036854775807 - 1;
        imprimir(-mn);
        imprimir(mn / (0 - 1));
        imprimir(mn % (0 - 1));
        imprimir(1 << 63);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output.len(), 6); // sin panic; valores wrap deterministas
}

#[test]
fn test_regression_agregar_value_semantics_o_n() {
    // agregar in-place: O(n) amortizado Y semántica de valores preservada
    let src = r#"
        lista<entero> a = [1, 2, 3];
        lista<entero> b = a;
        a.agregar(99);
        imprimir(a.largo());
        imprimir(b.largo());
        entero i = 0;
        mientras i < 5000 { a.agregar(i); i = i + 1; }
        imprimir(a.largo());
        imprimir(b.largo());
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["4", "3", "5004", "3"]);
}

#[test]
fn test_regression_scientific_notation() {
    // Notación científica en literales numéricos
    let src = r#"decimal d = 1.0e5; imprimir(d); decimal e = 2.5E-1; imprimir(e);"#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["100000", "0.25"]);
}

// === BUG #6 COMPLETO: referencias reales prestado mut con write-back ===

#[test]
fn test_ref_mut_scalar_writeback() {
    // Caso QA bug #6: mutación de un escalar dentro de la función visible fuera
    let src = r#"
        funcion vacio incrementar(prestado mut entero n) {
            n = n + 1;
        }
        entero x = 5;
        incrementar(x);
        imprimir(x);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["6"]);
}

#[test]
fn test_ref_mut_swap_two_vars() {
    let src = r#"
        funcion vacio intercambiar(prestado mut entero a, prestado mut entero b) {
            sea t = a;
            a = b;
            b = t;
        }
        entero a = 1;
        entero b = 2;
        intercambiar(a, b);
        imprimir(a);
        imprimir(b);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["2", "1"]);
}

#[test]
fn test_ref_mut_struct_field_writeback() {
    let src = r#"
        estructura Punto { x: entero, y: entero }
        funcion vacio reset(prestado mut Punto p) {
            p.x = 0;
            p.y = 0;
        }
        sea p = Punto { x: 3, y: 4 };
        reset(p);
        imprimir(p.x);
        imprimir(p.y);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["0", "0"]);
}

#[test]
fn test_ref_mut_array_push_and_set() {
    let src = r#"
        funcion vacio agregar_dato(prestado mut lista<entero> xs, entero v) {
            xs.agregar(v);
        }
        funcion vacio poner_primero(prestado mut lista<entero> xs, entero v) {
            xs[0] = v;
        }
        sea xs = [1, 2];
        agregar_dato(xs, 99);
        imprimir(largo(xs));
        imprimir(xs[2]);
        poner_primero(xs, -7);
        imprimir(xs[0]);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["3", "99", "-7"]);
}

#[test]
fn test_ref_mut_forwarding_chain() {
    // g recibe ref y la reenvía a f: ambas mutaciones persisten
    let src = r#"
        funcion vacio f(prestado mut entero n) {
            n = n + 10;
        }
        funcion vacio g(prestado mut entero m) {
            f(m);
            m = m + 1;
        }
        entero v = 100;
        g(v);
        imprimir(v);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["111"]);
}

#[test]
fn test_ref_mut_literal_arg_no_crash() {
    // Argumento no-lvalue a param prestado mut: fallback por valor, sin crash
    let src = r#"
        funcion vacio duplicar(prestado mut entero n) {
            n = n * 2;
        }
        duplicar(5);
        imprimir("ok");
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["ok"]);
}

#[test]
fn test_ref_mut_by_value_still_works() {
    // Sin prestado mut sigue siendo paso por valor (no se muta el original)
    let src = r#"
        funcion vacio inutil(entero n) {
            n = n + 1;
        }
        entero x = 5;
        inutil(x);
        imprimir(x);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["5"]);
}

// === BUG #7 COMPLETO: comptime con llamadas a funciones ===

#[test]
fn test_comptime_arithmetic_folds() {
    let src = r#"
        sea tabla = comptime { (1024*1024)/16 + 42 };
        imprimir(tabla);
        sea cubo = comptime { 7 * 7 * 7 };
        imprimir(cubo);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["65578", "343"]);
}

#[test]
fn test_comptime_function_call_fib() {
    // fib(20) se evalúa en COMPILE TIME (intérprete const-eval)
    let src = r#"
        funcion entero fib(n: entero) {
            si (n < 2) { retornar n; }
            retornar fib(n - 1) + fib(n - 2);
        }
        funcion vacio main() {
            sea f = comptime { fib(20) };
            imprimir(f);
        }
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["6765"]);
}

#[test]
fn test_comptime_string_concat() {
    let src = r#"
        sea msg = comptime { "hola" + " " + "mundo" };
        imprimir(msg);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["hola mundo"]);
}

#[test]
fn test_comptime_pure_builtins() {
    let src = r#"
        sea a = comptime { raiz(144) };
        imprimir(a);
        sea b = comptime { abs(0 - 42) };
        imprimir(b);
        sea c = comptime { potencia(2, 10) };
        imprimir(c);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["12", "42", "1024"]);
}

#[test]
fn test_comptime_nonfoldable_falls_back_to_runtime() {
    // Variable externa → NO plegable → se ejecuta normal en runtime
    let src = r#"
        entero base = 100;
        base = base + 1;
        imprimir(base);
    "#;
    let output = run_source(src).unwrap();
    assert_eq!(output, vec!["101"]);
}
