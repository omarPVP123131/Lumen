use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;
use lumen_ir::IRBuilder;
use lumen_codegen::Codegen;
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
numero resultado = suma(3, 7);
imprimir(resultado);";
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
