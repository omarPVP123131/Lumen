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
    let result = run_source("numero x = \"hola\";");
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
