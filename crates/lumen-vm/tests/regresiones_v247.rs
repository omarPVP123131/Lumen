//! Regresiones corregidas sobre LÚMEN v2.4.6.
//!
//! Cada test referencia el identificador del bug del reporte original para que
//! una regresión futura sea fácil de rastrear.

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
        return Err(format!(
            "SemError: [{}] {}",
            sem_errors[0].code, sem_errors[0].message
        ));
    }

    let builder = IRBuilder::new();
    let ir_program = builder.build(&program);
    let codegen = Codegen::new();
    let (bytecode, _warnings) = codegen.generate(&ir_program);

    let mut vm = VM::new(bytecode);
    vm.run().map_err(|e| format!("RuntimeError: {:?}", e))?;
    Ok(vm.output().to_vec())
}

fn out(source: &str) -> Vec<String> {
    run_source(source).expect("el programa debería ejecutarse sin errores")
}

// ── BUG-001: abs() y utilidades matemáticas como builtins ──────────────────

#[test]
fn bug001_abs_preserva_el_tipo() {
    assert_eq!(out("imprimir(abs(-5));"), vec!["5"]);
    assert_eq!(out("imprimir(abs(-2.5));"), vec!["2.5"]);
    assert_eq!(out("imprimir(abs(7));"), vec!["7"]);
}

#[test]
fn bug001_min_max_raiz_potencia() {
    assert_eq!(out("imprimir(minimo(3, 9));"), vec!["3"]);
    assert_eq!(out("imprimir(maximo(3, 9));"), vec!["9"]);
    assert_eq!(out("imprimir(raiz(16.0));"), vec!["4"]);
    assert_eq!(out("imprimir(potencia(2.0, 10.0));"), vec!["1024"]);
}

#[test]
fn bug001_funcion_del_usuario_tiene_prioridad_sobre_el_builtin() {
    // Un programa que define `abs` no debe quedar ensombrecido por el builtin.
    let src = r#"
funcion entero abs(entero x) { retornar 12345; }
imprimir(abs(-3));
"#;
    assert_eq!(out(src), vec!["12345"]);
}

// ── BUG-007 / BUG-002: conversiones texto → número ─────────────────────────

#[test]
fn bug007_a_entero_es_builtin() {
    assert_eq!(out(r#"imprimir(a_entero("42"));"#), vec!["42"]);
    assert_eq!(out(r#"imprimir(a_entero("  -17 "));"#), vec!["-17"]);
    // "3.9" se trunca hacia cero, como `as i64`.
    assert_eq!(out(r#"imprimir(a_entero("3.9"));"#), vec!["3"]);
}

#[test]
fn bug007_a_decimal_y_es_numero() {
    assert_eq!(out(r#"imprimir(a_decimal("3.5"));"#), vec!["3.5"]);
    assert_eq!(out(r#"imprimir(es_numero("12"));"#), vec!["true"]);
    assert_eq!(out(r#"imprimir(es_numero("abc"));"#), vec!["false"]);
}

#[test]
fn bug007_variantes_seguras_devuelven_resultado() {
    let ok = r#"
resultado<entero, texto> r = a_entero_seguro("100");
elegir (r) {
    caso exito(v): imprimir(v);
    caso error(e): imprimir("err");
}
"#;
    assert_eq!(out(ok), vec!["100"]);

    let err = r#"
resultado<entero, texto> r = a_entero_seguro("xyz");
elegir (r) {
    caso exito(v): imprimir("no");
    caso error(e): imprimir("err");
}
"#;
    assert_eq!(out(err), vec!["err"]);
}

#[test]
fn bug002_sugerencia_de_conversion_con_prefijo_a_() {
    // `texto(42)` no existe; el error debe orientar hacia `a_texto`.
    let err = run_source("imprimir(texto(42));").unwrap_err();
    assert!(err.contains("E042"), "código inesperado: {}", err);
}

// ── BUG-003: destructuring de datos de enum en elegir/caso ─────────────────

#[test]
fn bug003_captura_dato_de_variante() {
    let src = r#"
enum Figura { Circulo(decimal), Cuadrado(decimal) }
funcion decimal area(Figura f) {
    elegir (f) {
        caso Figura::Circulo(r): retornar 3.0 * r * r;
        caso Figura::Cuadrado(l): retornar l * l;
    }
    retornar 0.0;
}
imprimir(area(Figura::Cuadrado(4.0)));
"#;
    assert_eq!(out(src), vec!["16"]);
}

#[test]
fn bug003_variante_con_varios_datos() {
    let src = r#"
enum F { Rect(decimal, decimal) }
funcion decimal area(F f) {
    elegir (f) {
        caso F::Rect(w, h): retornar w * h;
    }
    retornar 0.0;
}
imprimir(area(F::Rect(3.0, 4.0)));
"#;
    assert_eq!(out(src), vec!["12"]);
}

#[test]
fn bug003_patron_literal_tiene_prioridad_sobre_la_captura() {
    let src = r#"
enum Msg { Codigo(entero) }
funcion texto d(Msg m) {
    elegir (m) {
        caso Msg::Codigo(404): retornar "no encontrado";
        caso Msg::Codigo(c): retornar a_texto(c);
    }
    retornar "?";
}
imprimir(d(Msg::Codigo(404)));
imprimir(d(Msg::Codigo(200)));
"#;
    assert_eq!(out(src), vec!["no encontrado", "200"]);
}

#[test]
fn bug003_no_rompe_los_patrones_or() {
    // Regresión detectada al implementar BUG-003: `caso A | B` debe seguir
    // cayendo al siguiente patrón cuando el primero no coincide.
    let src = r#"
enum Color { Rojo, Verde, Azul, Amarillo }
funcion texto t(Color c) {
    elegir (c) {
        caso Color::Rojo | Color::Amarillo: retornar "calido";
        caso Color::Verde | Color::Azul: retornar "frio";
        defecto: retornar "desconocido";
    }
}
imprimir(t(Color::Azul));
imprimir(t(Color::Amarillo));
"#;
    assert_eq!(out(src), vec!["frio", "calido"]);
}

#[test]
fn bug003_aridad_incorrecta_es_error() {
    let src = r#"
enum F { Circulo(decimal) }
funcion decimal a(F f) {
    elegir (f) {
        caso F::Circulo(x, y): retornar 0.0;
    }
    retornar 0.0;
}
imprimir(a(F::Circulo(1.0)));
"#;
    let err = run_source(src).unwrap_err();
    assert!(err.contains("E067"), "código inesperado: {}", err);
}

// ── BUG-006: `resultado` / `opcion` como nombres de variable ───────────────

#[test]
fn bug006_resultado_como_nombre_de_variable() {
    let src = r#"
entero resultado = 0;
resultado = resultado + 42;
imprimir(resultado);
"#;
    assert_eq!(out(src), vec!["42"]);
}

#[test]
fn bug006_opcion_como_nombre_y_tipo_generico_conviven() {
    let src = r#"
entero opcion = 7;
resultado<entero, texto> r = exito(5);
elegir (r) {
    caso exito(v): imprimir(opcion * v);
    caso error(e): imprimir("err");
}
"#;
    assert_eq!(out(src), vec!["35"]);
}

#[test]
fn bug006_resultado_como_nombre_de_parametro() {
    let src = r#"
funcion entero doblar(entero resultado) { retornar resultado * 2; }
imprimir(doblar(21));
"#;
    assert_eq!(out(src), vec!["42"]);
}

#[test]
fn bug006_palabra_reservada_real_da_mensaje_claro() {
    let err = run_source("entero mientras = 1;").unwrap_err();
    assert!(
        err.contains("palabra reservada"),
        "mensaje inesperado: {}",
        err
    );
}

// ── BUG-008: paso por referencia con `prestado mut` ────────────────────────

#[test]
fn bug008_struct_por_referencia_muta_al_llamador() {
    let src = r#"
estructura Caja { valor: entero }
funcion vacio cambiar(prestado mut Caja c) { c.valor = 999; }
Caja a = Caja { valor: 1 };
cambiar(a);
imprimir(a.valor);
"#;
    assert_eq!(out(src), vec!["999"]);
}

#[test]
fn bug008_lista_por_referencia_conserva_agregar() {
    let src = r#"
funcion vacio agregar_algo(prestado mut lista<texto> l) { l.agregar("hola"); }
lista<texto> mi = [];
agregar_algo(mi);
agregar_algo(mi);
imprimir(mi.largo());
"#;
    assert_eq!(out(src), vec!["2"]);
}

#[test]
fn bug008_asignacion_por_indice_por_referencia() {
    let src = r#"
funcion vacio set0(prestado mut lista<entero> l) { l[0] = 77; }
lista<entero> nums = [1, 2, 3];
set0(nums);
imprimir(nums[0]);
"#;
    assert_eq!(out(src), vec!["77"]);
}

#[test]
fn bug008_acumulacion_recursiva_caso_laberinto() {
    let src = r#"
funcion vacio excavar(prestado mut lista<entero> camino, entero n) {
    si n <= 0 { retornar; }
    camino.agregar(n);
    excavar(camino, n - 1);
}
lista<entero> camino = [];
excavar(camino, 5);
imprimir(camino.largo());
"#;
    assert_eq!(out(src), vec!["5"]);
}

#[test]
fn bug008_mutacion_a_traves_de_dos_niveles() {
    let src = r#"
estructura C { v: entero }
funcion vacio inc(prestado mut C c) { c.v = c.v + 1; }
funcion vacio inc2(prestado mut C c) { inc(c); inc(c); }
C x = C { v: 0 };
inc2(x);
imprimir(x.v);
"#;
    assert_eq!(out(src), vec!["2"]);
}

#[test]
fn bug008_sin_prestado_sigue_siendo_por_valor() {
    // La semántica por defecto no cambia: sólo `prestado mut` es por referencia.
    let src = r#"
estructura Caja { valor: entero }
funcion vacio no_muta(Caja c) { c.valor = 123; }
Caja a = Caja { valor: 7 };
no_muta(a);
imprimir(a.valor);
"#;
    assert_eq!(out(src), vec!["7"]);
}

#[test]
fn bug008_prestado_inmutable_no_permite_mutar() {
    let src = r#"
estructura Caja { valor: entero }
funcion vacio malo(prestado Caja c) { c.valor = 1; }
imprimir("x");
"#;
    let err = run_source(src).unwrap_err();
    assert!(err.contains("E061"), "código inesperado: {}", err);
}

#[test]
fn bug008_lectura_a_traves_de_prestado_inmutable() {
    let src = r#"
estructura Caja { valor: entero }
funcion entero leer_v(prestado Caja c) { retornar c.valor; }
Caja a = Caja { valor: 7 };
imprimir(leer_v(a));
"#;
    assert_eq!(out(src), vec!["7"]);
}

// ── BUG-009: imprimir con varios argumentos produce UNA línea ──────────────

#[test]
fn bug009_imprimir_multiargumento_es_una_sola_linea() {
    let output = out(r#"imprimir("a: ", 1, " b: ", 2);"#);
    assert_eq!(output, vec!["a: 1 b: 2"]);
}

#[test]
fn bug009_imprimir_de_un_argumento_no_cambia() {
    assert_eq!(out(r#"imprimir("solo");"#), vec!["solo"]);
}

// ── BUG-010: retorno temprano no debe truncar la ejecución ─────────────────

#[test]
fn bug010_retorno_temprano_no_corta_el_programa() {
    let src = r#"
funcion vacio f(entero n) {
    si n <= 0 { retornar; }
    imprimir("A");
}
imprimir("1");
f(5);
imprimir("2");
imprimir("3");
"#;
    assert_eq!(out(src), vec!["1", "A", "2", "3"]);
}

#[test]
fn bug010_retorno_temprano_tomado() {
    let src = r#"
funcion vacio f(entero n) {
    imprimir("inicio");
    si n <= 0 { retornar; }
    imprimir("tarde");
}
f(0);
imprimir("DESPUES");
"#;
    assert_eq!(out(src), vec!["inicio", "DESPUES"]);
}

// ── BUG-011: `lista[i].campo = v` ──────────────────────────────────────────

#[test]
fn bug011_asignacion_a_campo_de_struct_en_lista() {
    let src = r#"
estructura Cr { vida: entero }
lista<Cr> equipo = [Cr { vida: 100 }, Cr { vida: 50 }];
equipo[0].vida = 5;
equipo[1].vida = equipo[1].vida - 10;
imprimir(equipo[0].vida);
imprimir(equipo[1].vida);
"#;
    assert_eq!(out(src), vec!["5", "40"]);
}

// ── BUG-014: `main` se ejecutaba dos veces ─────────────────────────────────

#[test]
fn bug014_main_llamada_explicita_no_se_duplica() {
    // Con una llamada explícita en el nivel superior, `main` debe correr UNA vez.
    let src = r#"
funcion vacio main() {
    imprimir("dentro");
}
main();
"#;
    assert_eq!(out(src), vec!["dentro"]);
}

#[test]
fn bug014_main_se_autoinvoca_si_no_hay_llamada() {
    // Sin llamada explícita se conserva la auto-invocación tras el top-level.
    let src = r#"
funcion vacio main() {
    imprimir("main");
}
imprimir("toplevel");
"#;
    assert_eq!(out(src), vec!["toplevel", "main"]);
}

// ── BUG-015: `romper` / `continuar` dentro de `para ... en` ────────────────

#[test]
fn bug015_continuar_y_romper_en_para_cada() {
    let src = r#"
para n en [1,2,3,4,5] {
    si (n == 3) { continuar; }
    si (n == 5) { romper; }
    imprimir(n);
}
"#;
    assert_eq!(out(src), vec!["1", "2", "4"]);
}

#[test]
fn bug015_romper_solo_afecta_al_bucle_interno() {
    let src = r#"
para i en [1,2] {
    para j en [10,20,30] {
        si (j == 20) { romper; }
        imprimir(j);
    }
    imprimir(i);
}
"#;
    assert_eq!(out(src), vec!["10", "1", "10", "2"]);
}

#[test]
fn bug015_continuar_directo_no_cuelga_el_bucle() {
    // `continuar` debe saltar al incremento del índice, no al inicio del bucle.
    let src = r#"
para n en [1,2,3] { continuar; }
imprimir("fin");
"#;
    assert_eq!(out(src), vec!["fin"]);
}

// ── BUG-016: tipo declarado inexistente ────────────────────────────────────

#[test]
fn bug016_tipo_no_definido_da_error_claro() {
    // Antes: E031 filtrando `Struct { name: "X", fields: [] }`.
    let src = r#"
TipoQueNoExiste x = 5;
imprimir(x);
"#;
    let err = run_source(src).expect_err("debería fallar el análisis semántico");
    assert!(err.contains("E062"), "esperaba E062, salió: {err}");
    assert!(
        err.contains("no está definido"),
        "mensaje poco claro: {err}"
    );
    assert!(
        !err.contains("fields: []"),
        "no debe filtrar la representación interna: {err}"
    );
}

// ── BUG-017: lambdas que capturan el entorno ───────────────────────────────

#[test]
fn bug017_lambda_captura_variable_externa() {
    let src = r#"
entero base = 100;
f = funcion() { retornar base; };
imprimir(f());
"#;
    assert_eq!(out(src), vec!["100"]);
}

#[test]
fn bug017_lambda_captura_dentro_de_funcion() {
    let src = r#"
funcion entero externa() {
    entero local = 7;
    g = funcion() { retornar local; };
    retornar g();
}
imprimir(externa());
"#;
    assert_eq!(out(src), vec!["7"]);
}

#[test]
fn bug017_parametro_sombrea_la_captura() {
    let src = r#"
entero x = 50;
f = funcion(entero x) { retornar x; };
imprimir(f(9));
"#;
    assert_eq!(out(src), vec!["9"]);
}

// ── BUG-018: `leer` / `read` como nombre de función del usuario ────────────

#[test]
fn bug018_funcion_de_usuario_llamada_leer() {
    // El builtin `leer` (stdin) ensombrecía la función del programa y
    // devolvía "" en silencio.
    let src = r#"
entero base = 100;
funcion entero leer() { retornar base; }
imprimir(leer());
"#;
    assert_eq!(out(src), vec!["100"]);
}

// ── BUG-020: `prestado mut self` en métodos de `impl` ──────────────────────

#[test]
fn bug020_metodo_con_prestado_mut_self_muta() {
    let src = r#"
estructura Contador { valor: entero }
impl Contador {
    funcion vacio incrementar(prestado mut Contador self) { self.valor = self.valor + 1; }
    funcion entero obtener(prestado Contador self) { retornar self.valor; }
}
Contador c = Contador { valor: 0 };
c.incrementar();
c.incrementar();
imprimir(c.obtener());
"#;
    assert_eq!(out(src), vec!["2"]);
}

#[test]
fn bug020_struct_sigue_pasandose_por_valor() {
    // La copia de vuelta no debe convertir la asignación en aliasing.
    let src = r#"
estructura P { x: entero }
P a = P { x: 1 };
P b = a;
b.x = 99;
imprimir(a.x);
imprimir(b.x);
"#;
    assert_eq!(out(src), vec!["1", "99"]);
}

#[test]
fn bug021_lambda_declarada_dentro_de_otra_lambda() {
    // `collect_variable_refs` metía los nombres ASIGNADOS en la lista de
    // capturas, así que la lambda interna se renombraba a `__cap_N_interna`
    // mientras el Store escribía en `interna`: la lectura no hallaba el slot.
    let src = r#"
sea externa = funcion() {
    sea interna = funcion() { retornar 7; };
    retornar interna();
};
imprimir(externa());
"#;
    assert_eq!(out(src), vec!["7"]);
}

#[test]
fn bug021_lambda_anidada_no_rompe_las_capturas() {
    // La contrapartida de BUG-017: lo que la lambda sí captura debe seguir
    // resolviéndose contra la variable de fuera.
    let src = r#"
entero base = 10;
sea externa = funcion() {
    sea interna = funcion() { retornar base + 5; };
    retornar interna();
};
imprimir(externa());
"#;
    assert_eq!(out(src), vec!["15"]);
}

#[test]
fn bug023_local_no_pisa_la_global_homonima() {
    // `Store` escribía en el marco global si el nombre ya existía allí, de modo
    // que declarar una local homónima destruía la global. Ahora las
    // declaraciones emiten `StoreLocal`, que liga en el marco actual.
    let src = r#"
entero total = 100;
funcion vacio f() { entero total = 7; imprimir("local:", total); }
f();
imprimir("global:", total);
"#;
    assert_eq!(out(src), vec!["local:7", "global:100"]);
}

#[test]
fn bug023_la_asignacion_sigue_alcanzando_la_global() {
    // Sólo las DECLARACIONES ligan localmente; una asignación a secas debe
    // seguir modificando la global, o romperíamos el resto del lenguaje.
    let src = r#"
entero total = 1;
funcion vacio f() { total = 42; }
f();
imprimir(total);
"#;
    assert_eq!(out(src), vec!["42"]);
}

#[test]
fn bug026_lambda_que_captura_la_variable_del_bucle_no_cuelga() {
    // La captura movía `k` al slot `__cap_N_k`, pero la condición del
    // `mientras` ya emitida seguía leyendo `k`: nunca cambiaba y el bucle
    // se volvía infinito. Ahora ambos nombres se mantienen sincronizados.
    let src = r#"
entero k = 0;
mientras (k < 3) {
    sea f = funcion() { retornar k * 2; };
    imprimir(f());
    k = k + 1;
}
imprimir("fin");
"#;
    assert_eq!(out(src), vec!["0", "2", "4", "fin"]);
}

#[test]
fn bug027_imprimir_dentro_de_funcion_no_corrompe_la_pila() {
    // `imprimir(...)` como sentencia dejaba su `void` en la pila; al retornar,
    // esa basura se mezclaba con los argumentos que el llamador estaba
    // montando y `imprimir("total=", h(2))` salía como `void2`.
    let src = r#"
funcion entero h(entero n) {
    imprimir("dentro n=", n);
    retornar n;
}
imprimir("total=", h(2));
"#;
    assert_eq!(out(src), vec!["dentro n=2", "total=2"]);
}

#[test]
fn bug027_recursion_con_imprimir_intermedio() {
    let src = r#"
funcion entero h(entero n) {
    si (n <= 0) { retornar 0; }
    entero r = h(n - 1);
    imprimir("n=", n);
    retornar r + n;
}
imprimir("total=", h(2));
"#;
    assert_eq!(out(src), vec!["n=1", "n=2", "total=3"]);
}

#[test]
fn bug028_posponer_se_ejecuta_al_salir_no_donde_esta_escrito() {
    // `posponer` es un `defer`: antes se emitía en línea, así que la limpieza
    // corría ANTES del código que usaba el recurso.
    let src = r#"
funcion vacio f() {
    imprimir("1 abrir");
    posponer { imprimir("3 cerrar"); }
    imprimir("2 usar");
}
f();
"#;
    assert_eq!(out(src), vec!["1 abrir", "2 usar", "3 cerrar"]);
}

#[test]
fn bug028_posponer_corre_tambien_en_retorno_temprano() {
    let src = r#"
funcion entero f(entero n) {
    posponer { imprimir("limpieza"); }
    si (n < 0) { retornar 0; }
    retornar n;
}
imprimir("r=", f(-1));
"#;
    assert_eq!(out(src), vec!["limpieza", "r=0"]);
}

#[test]
fn bug028_varios_posponer_en_orden_inverso() {
    // LIFO, como un `defer` de verdad.
    let src = r#"
funcion vacio f() {
    posponer { imprimir("A"); }
    posponer { imprimir("B"); }
    imprimir("cuerpo");
}
f();
"#;
    assert_eq!(out(src), vec!["cuerpo", "B", "A"]);
}

#[test]
fn bug028_posponer_a_nivel_global() {
    // El código de nivel superior acaba en `Halt`, no en `Return`: el bloque
    // diferido se quedaba sin emitir y no se ejecutaba nunca.
    let src = r#"
posponer { imprimir("al final"); }
imprimir("main");
"#;
    assert_eq!(out(src), vec!["main", "al final"]);
}

#[test]
fn bug029_lambda_puede_llamar_a_funciones_y_builtins() {
    // El destino de una llamada es un nombre de función, no una variable
    // capturada: apuntarlo como captura hacía que la lambda muriera con
    // "Variable 'imprimir' no definida".
    let src = r#"
funcion entero doblar(entero x) { retornar x * 2; }
sea g = funcion() { imprimir("dentro"); retornar doblar(21); };
imprimir(g());
"#;
    assert_eq!(out(src), vec!["dentro", "42"]);
}

#[test]
fn bug030_listas_con_el_mismo_contenido_son_iguales() {
    // El `match` de `Opcode::Eq` no tenía rama para listas y caía en
    // `_ => false`: dos listas iguales —o incluso una lista consigo misma—
    // se comparaban como distintas, y `!=` devolvía `true` por simetría.
    let src = r#"
lista<entero> a = [1,2,3];
lista<entero> b = [1,2,3];
lista<entero> c = [9,9];
imprimir(a == b);
imprimir(a == c);
imprimir(a != b);
"#;
    assert_eq!(out(src), vec!["true", "false", "false"]);
}

#[test]
fn bug030_tuplas_y_resultados_tambien_se_comparan() {
    let src = r#"
sea t1 = (1, "a");
sea t2 = (1, "a");
imprimir(t1 == t2);
imprimir(exito(5) == exito(5));
lista<lista<entero>> m1 = [[1,2]];
lista<lista<entero>> m2 = [[1,2]];
imprimir(m1 == m2);
"#;
    assert_eq!(out(src), vec!["true", "true", "true"]);
}

#[test]
fn bug030_la_comparacion_numerica_mixta_sigue_funcionando() {
    // El fix delega en `PartialEq`, pero entero-vs-decimal necesita
    // tolerancia en coma flotante y se conserva aparte.
    let src = r#"
imprimir(1 == 1.0);
imprimir(1.0 == 1);
imprimir(2.5 == 2.5);
imprimir(1 == 2);
imprimir("a" == "a");
"#;
    assert_eq!(out(src), vec!["true", "true", "true", "false", "true"]);
}

#[test]
fn bug031_patron_comodin_de_nivel_superior() {
    // `caso _:` pasaba `lumen check` y reventaba en runtime con
    // "Variable '_' no definida": el comodín sólo se manejaba como
    // subpatrón de un destructuring, no como patrón de nivel superior.
    let src = r#"
funcion texto f(entero n) {
  elegir (n) {
    caso 1: retornar "uno";
    caso 2: retornar "dos";
    caso _: retornar "otro";
  }
  retornar "?";
}
imprimir(f(1));
imprimir(f(2));
imprimir(f(99));
"#;
    assert_eq!(out(src), vec!["uno", "dos", "otro"]);
}

#[test]
fn bug031_comodin_respeta_el_orden_de_los_casos() {
    // El comodín no debe atrapar antes de tiempo: los casos previos ganan.
    let src = r#"
funcion texto f(entero n) {
  elegir (n) {
    caso 1: retornar "uno";
    caso _: retornar "comodin";
  }
  retornar "?";
}
imprimir(f(1));
imprimir(f(2));
"#;
    assert_eq!(out(src), vec!["uno", "comodin"]);
}

#[test]
fn bug031_comodin_con_texto_y_defecto_sin_regresion() {
    // El comodín funciona sobre texto y `defecto` sigue intacto.
    let src = r#"
funcion texto g(texto s) {
  elegir (s) {
    caso "A": retornar "es A";
    caso _: retornar "otro";
  }
  retornar "?";
}
funcion texto h(entero n) {
  elegir (n) {
    caso 1: retornar "uno";
    defecto: retornar "otro";
  }
  retornar "?";
}
imprimir(g("A"));
imprimir(g("Z"));
imprimir(h(1));
imprimir(h(5));
"#;
    assert_eq!(out(src), vec!["es A", "otro", "uno", "otro"]);
}

#[test]
fn bug033_agregar_sobre_campo_de_struct() {
    // `c.items.agregar(x)` hacía el push y tiraba el resultado: el receptor no
    // era un identificador simple, así que nunca se escribía de vuelta y la
    // mutación se perdía SIN ERROR.
    let src = r#"
estructura Caja { items: lista<entero> }
Caja c = Caja{items: [1, 2]};
c.items.agregar(3);
imprimir(c.items);
imprimir(largo(c.items));
"#;
    assert_eq!(out(src), vec!["[1, 2, 3]", "3"]);
}

#[test]
fn bug033_agregar_sobre_elemento_indexado() {
    // Mismo fallo con `m[i].agregar(x)`.
    let src = r#"
lista<lista<entero>> m = [[1], [2]];
m[0].agregar(9);
imprimir(m);
"#;
    assert_eq!(out(src), vec!["[[1, 9], [2]]"]);
}

#[test]
fn bug033_writeback_anidado_dos_niveles() {
    // El write-back sube recursivamente hasta la variable con nombre.
    let src = r#"
estructura Interna { xs: lista<entero> }
estructura Externa { dentro: Interna }
Externa e = Externa{dentro: Interna{xs: [1]}};
e.dentro.xs.agregar(2);
e.dentro.xs.agregar(3);
imprimir(e.dentro.xs);
"#;
    assert_eq!(out(src), vec!["[1, 2, 3]"]);
}

#[test]
fn bug033_no_afecta_a_listas_sueltas_ni_a_bucles() {
    // El camino con receptor `Ident` sigue igual que antes.
    let src = r#"
lista<entero> l = [1];
l.agregar(2);
lista<entero> acc = [];
para x en 1..=3 {
  acc.agregar(x * 10);
}
imprimir(l);
imprimir(acc);
"#;
    assert_eq!(out(src), vec!["[1, 2]", "[10, 20, 30]"]);
}

#[test]
fn bug038_igualdad_de_structs_por_contenido() {
    // El backend C comparaba structs por un campo basura (`default: a.i == b.i`)
    // y daba `true` para structs distintos. La VM ya era correcta; este test
    // fija la semántica que ambos backends deben compartir.
    let src = r#"
estructura Punto { x: entero, y: entero }
Punto p1 = Punto { x: 3, y: 4 };
Punto p2 = Punto { x: 3, y: 4 };
Punto p3 = Punto { x: 0, y: 0 };
imprimir(p1 == p2);
imprimir(p1 == p3);
imprimir(p1 != p3);
"#;
    assert_eq!(out(src), vec!["true", "false", "true"]);
}

#[test]
fn bug035_largo_de_texto_cuenta_caracteres() {
    // `largo` sobre texto cuenta caracteres, no bytes: el AOT usaba `strlen` y
    // decía 13 donde la VM decía 7.
    let src = r#"
imprimir(largo("áéíóú ñ"));
imprimir(largo("日本語"));
imprimir(largo("café"));
imprimir(largo("hola"));
"#;
    assert_eq!(out(src), vec!["7", "3", "4", "4"]);
}

#[test]
fn bug040_reemplazar_clave_de_mapa() {
    // Los mapas son persistentes: `__map_poner` devuelve uno nuevo. Volver a
    // poner una clave debe REEMPLAZARLA, no duplicarla (el backend C añadía
    // siempre al final y `__map_obtener` seguía devolviendo el valor viejo).
    let src = r#"
sea m = __map_nuevo();
m = __map_poner(m, "a", 1);
m = __map_poner(m, "b", 2);
m = __map_poner(m, "a", 10);
imprimir(__map_obtener(m, "a"));
imprimir(__map_longitud(m));
imprimir(__map_contiene(m, "z"));
"#;
    assert_eq!(out(src), vec!["10", "2", "false"]);
}

#[test]
fn bug039_conversiones_seguras() {
    // `a_entero_seguro` devuelve `exito`/`error` y el `elegir` debe casar.
    let src = r#"
resultado<entero, texto> ok = a_entero_seguro("100");
elegir (ok) {
    caso exito(v): imprimir("ok ", v);
    caso error(e): imprimir("err ", e);
}
resultado<entero, texto> mal = a_entero_seguro("no-soy-numero");
elegir (mal) {
    caso exito(v): imprimir("ok ", v);
    caso error(e): imprimir("err");
}
"#;
    assert_eq!(out(src), vec!["ok 100", "err"]);
}

#[test]
fn bug042_str_longitud_cuenta_caracteres() {
    // `__str_longitud` usaba bytes (`s.len()`) y `largo` caracteres: el mismo
    // texto medía 6 o 5 según qué builtin se usara.
    let src = r#"
imprimir(__str_longitud("Lúmen"));
imprimir(largo("Lúmen"));
imprimir(__str_longitud("abc"));
"#;
    assert_eq!(out(src), vec!["5", "5", "3"]);
}

#[test]
fn bug046_funcion_modifica_variable_global() {
    // El backend C guardaba y restauraba las variables del llamador alrededor
    // de cada llamada, revirtiendo también las globales que la función acababa
    // de modificar. Este test fija la semántica correcta.
    let src = r#"
entero g = 1;
funcion vacio subir() { g = g + 1; }
subir();
subir();
imprimir(g);
"#;
    assert_eq!(out(src), vec!["3"]);
}

#[test]
fn bug046_parametro_que_sombrea_una_global() {
    // El caso límite del arreglo anterior: un parámetro homónimo de una global
    // es local y NO debe sobrevivir a la llamada.
    let src = r#"
entero g = 100;
funcion vacio sombra(entero g) { imprimir(g); }
sombra(7);
imprimir(g);
"#;
    assert_eq!(out(src), vec!["7", "100"]);
}

#[test]
fn bug048_largo_como_metodo_sobre_texto() {
    // `s.largo()` devolvía 0 compilado, así que los bucles que dependían de él
    // no se ejecutaban ni una vez.
    let src = r#"
texto s = "Hola";
imprimir(s.largo());
lista<entero> l = [1, 2, 3];
imprimir(l.largo());
"#;
    assert_eq!(out(src), vec!["4", "3"]);
}

#[test]
fn bug041_indexar_texto_por_caracter() {
    // `s[0]` reventaba en los binarios nativos con "fuera de rango (largo: 0)".
    let src = r#"
texto s = "Lumen";
imprimir(s[0]);
imprimir(s[4]);
texto a = "áéí";
imprimir(a[1]);
"#;
    assert_eq!(out(src), vec!["L", "n", "é"]);
}

#[test]
fn bug049_recursion_infinita_da_error_no_mata_el_proceso() {
    // La VM no tenía límite de profundidad: una recursión infinita hacía
    // crecer `call_stack` hasta que el SO mataba el proceso por memoria. Ahora
    // se aborta con un error normal del programa.
    let src = r#"
funcion entero infinita(entero n) { retornar infinita(n + 1); }
imprimir(infinita(0));
"#;
    let err = run_source(src).expect_err("debería fallar con error de profundidad");
    let msg = err;
    assert!(
        msg.contains("Profundidad máxima de llamadas"),
        "mensaje inesperado: {}",
        msg
    );
}

#[test]
fn bug049_recursion_mutua_infinita_tambien_se_corta() {
    let src = r#"
funcion entero a(entero n) { retornar b(n + 1); }
funcion entero b(entero n) { retornar a(n + 1); }
imprimir(a(0));
"#;
    let err = run_source(src).expect_err("debería fallar con error de profundidad");
    assert!(err.contains("Profundidad máxima de llamadas"));
}

#[test]
fn bug049_la_recursion_legitima_sigue_funcionando() {
    // El límite no debe estorbar a la recursión normal, ni siquiera profunda.
    let src = r#"
funcion entero fact(entero n) { si (n <= 1) { retornar 1; } retornar n * fact(n - 1); }
funcion entero mil(entero n) { si (n <= 0) { retornar 0; } retornar 1 + mil(n - 1); }
funcion booleano par(entero n) { si (n == 0) { retornar verdadero; } retornar impar(n-1); }
funcion booleano impar(entero n) { si (n == 0) { retornar falso; } retornar par(n-1); }
imprimir(fact(15));
imprimir(mil(1000));
imprimir(par(500));
"#;
    assert_eq!(out(src), vec!["1307674368000", "1000", "true"]);
}

fn compilar_ir(source: &str) -> lumen_ir::Program {
    let lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();
    let parser = Parser::new(tokens);
    let (mut program, _) = parser.parse();
    let sema = SemanticAnalyzer::new();
    let _ = sema.analyze(&mut program);
    IRBuilder::new().build(&program)
}

// ---------------------------------------------------------------------------
// BUG-050: el backend C generaba un stub `return _v_void()` para todo builtin
// que no implementaba, produciendo binarios que devolvían valores falsos en
// silencio. Ahora esos nombres quedan registrados para que el CLI aborte.
// ---------------------------------------------------------------------------

#[test]
fn bug050_builtin_no_soportado_queda_registrado() {
    let src = r#"imprimir(__builtin_inexistente_xyz(1));"#;
    let ir = compilar_ir(src);
    let _ = lumen_aot::compile_to_c(&ir);
    let faltantes = lumen_aot::take_unsupported_builtins();
    assert!(
        faltantes.iter().any(|f| f == "__builtin_inexistente_xyz"),
        "el builtin sin soporte debe registrarse, se obtuvo: {:?}",
        faltantes
    );
}

#[test]
fn bug050_programa_soportado_no_reporta_faltantes() {
    let src = r#"
funcion entero doble(entero x) { retornar x * 2; }
imprimir(doble(21));
imprimir(largo("hola"));
"#;
    let ir = compilar_ir(src);
    let _ = lumen_aot::compile_to_c(&ir);
    let faltantes = lumen_aot::take_unsupported_builtins();
    assert!(
        faltantes.is_empty(),
        "un programa totalmente soportado no debe reportar faltantes: {:?}",
        faltantes
    );
}

// ---------------------------------------------------------------------------
// BUG-051: el runtime C no vigilaba ni el tope de la pila de valores ni la
// profundidad de llamadas. Como las funciones LUMEN se emiten como funciones C
// recursivas, una recursión infinita desbordaba la pila del proceso y el
// binario moría por SEGFAULT sin imprimir nada (rc=139). Ahora el C generado
// llama a `_ckdepth()` al entrar en cada función y la pila de valores crece
// bajo demanda, de modo que el nativo aborta con el mismo mensaje que la VM.
// ---------------------------------------------------------------------------

#[test]
fn bug051_c_generado_comprueba_profundidad() {
    let src = r#"
funcion entero f(entero n) { retornar f(n + 1); }
imprimir(f(1));
"#;
    let ir = compilar_ir(src);
    let c = lumen_aot::compile_to_c(&ir);
    assert!(
        c.contains("_ckdepth();"),
        "cada función emitida debe comprobar la profundidad de llamadas"
    );
    assert!(
        c.contains("_stack_init();"),
        "_init debe fijar la referencia de pila"
    );
    assert!(
        c.contains("_depth--"),
        "el retorno debe liberar el nivel de profundidad"
    );
}

#[test]
fn bug051_pila_de_valores_crece_bajo_demanda() {
    // Un tope fijo de 16384 rompía la recursión legítima profunda que la VM sí
    // resuelve (paridad VM/AOT).
    let ir = compilar_ir("imprimir(1);");
    let c = lumen_aot::compile_to_c(&ir);
    assert!(
        c.contains("_st_grow"),
        "la pila de valores debe crecer en vez de tener un tope fijo"
    );
}

#[test]
fn bug051_recursion_profunda_legitima_sigue_funcionando_en_la_vm() {
    let src = r#"
funcion entero suma(entero n) { si (n <= 0) { retornar 0; } retornar n + suma(n - 1); }
imprimir(suma(10000));
"#;
    assert_eq!(out(src), vec!["50005000"]);
}

// ---------------------------------------------------------------------------
// BUG-032: una lambda creada dentro de otra función perdía sus capturas al ser
// DEVUELTA: se leían del marco del padre, que ya había muerto ("Variable 'n' no
// definida"). Los slots globales `__cap_*` no servían porque dos closures de la
// misma factoría los compartirían. Ahora la lambda anota qué nombres captura y
// `FuncRef` los resuelve en un entorno propio de cada instancia.
// ---------------------------------------------------------------------------

#[test]
fn bug032_closure_devuelta_conserva_su_captura() {
    let src = r#"
sea hacer_sumador = funcion(entero n) {
  sea f = funcion(entero x) { retornar x + n; };
  retornar f;
};
sea suma5 = hacer_sumador(5);
imprimir(suma5(10));
"#;
    assert_eq!(out(src), vec!["15"]);
}

#[test]
fn bug032_dos_instancias_no_comparten_entorno() {
    // El caso que descartó los intentos anteriores: con slots globales ambas
    // closures devolvían 101.
    let src = r#"
funcion entero llamar(entero n, entero x) { retornar n + x; }
sea mk = funcion(entero n) {
  entero cap = n;
  sea f = funcion(entero x) { retornar llamar(cap, x); };
  retornar f;
};
sea s5 = mk(5);
sea s100 = mk(100);
imprimir(s5(1));
imprimir(s100(1));
"#;
    assert_eq!(out(src), vec!["6", "101"]);
}

#[test]
fn bug032_varias_capturas_y_reuso() {
    let src = r#"
sea mk = funcion(entero a, entero b) {
  sea f = funcion(entero x) { retornar (a * 100) + (b * 10) + x; };
  retornar f;
};
sea g = mk(1, 2);
sea h = mk(3, 4);
imprimir(g(3));
imprimir(h(5));
imprimir(g(3));
"#;
    assert_eq!(out(src), vec!["123", "345", "123"]);
}

#[test]
fn bug032_triple_anidamiento_propaga_la_captura() {
    // La lambda intermedia no menciona `n`, pero debe capturarlo para poder
    // pasárselo a la más interna.
    let src = r#"
sea externa = funcion(entero n) {
  sea media = funcion(entero m) {
    sea interna = funcion(entero x) { retornar n + m + x; };
    retornar interna;
  };
  retornar media;
};
sea f = externa(100);
sea g = f(20);
imprimir(g(3));
"#;
    assert_eq!(out(src), vec!["123"]);
}

#[test]
fn bug032_captura_de_nivel_superior_sigue_funcionando() {
    // No debe romperse la captura clásica (slots `__cap_*`) ni la mutación de
    // una variable del entorno global.
    let src = r#"
entero base = 100;
sea sumar_base = funcion(entero x) { retornar x + base; };
imprimir(sumar_base(20));
entero cuenta = 0;
sea incrementar = funcion() { cuenta = cuenta + 1; retornar cuenta; };
imprimir(incrementar());
imprimir(incrementar());
imprimir(cuenta);
"#;
    assert_eq!(out(src), vec!["120", "1", "2", "2"]);
}

// ---------------------------------------------------------------------------
// BUG-052: con BUG-032 la captura era por VALOR, así que el idioma del contador
// (una closure que muta lo capturado y se devuelve) seguía roto: la VM fallaba
// con "Variable 'n' no definida" y el binario nativo respondía sin aislar las
// instancias. Causa: `collect_assigned_names` mezclaba declaraciones con
// asignaciones, de modo que `n = n + 1` parecía declarar una local. Ahora las
// capturas mutadas viajan en celdas COMPARTIDAS por invocación.
// ---------------------------------------------------------------------------

#[test]
fn bug052_contador_conserva_su_estado_entre_llamadas() {
    let src = r#"
sea mk = funcion(entero n) {
  sea inc = funcion() { n = n + 1; retornar n; };
  retornar inc;
};
sea c = mk(10);
imprimir(c());
imprimir(c());
"#;
    assert_eq!(out(src), vec!["11", "12"]);
}

#[test]
fn bug052_contadores_independientes_no_comparten_celda() {
    // Dos invocaciones de la misma factoría ocupan la misma profundidad de
    // marco: si la celda se indexara por profundidad, `b` heredaría el estado
    // de `a`.
    let src = r#"
sea mk = funcion(entero n) {
  entero c = n;
  sea inc = funcion() { c = c + 1; retornar c; };
  retornar inc;
};
sea a = mk(10);
sea b = mk(100);
imprimir(a());
imprimir(a());
imprimir(b());
"#;
    assert_eq!(out(src), vec!["11", "12", "101"]);
}

#[test]
fn bug052_declaracion_interna_sigue_siendo_local() {
    // Una variable DECLARADA dentro de la lambda no es una captura: no debe
    // arrastrar nada del entorno homónimo.
    let src = r#"
entero x = 5;
sea f = funcion() {
  entero x = 100;
  retornar x;
};
imprimir(f());
imprimir(x);
"#;
    assert_eq!(out(src), vec!["100", "5"]);
}

// ---------------------------------------------------------------------------
// BUG-053: `lumen fmt` DESTRUÍA el código fuente. Las construcciones que el
// formateador no cubría caían en brazos `_ => {}` y desaparecían del archivo:
// las lambdas se reescribían como `Infer f = ;`, `l[j] = x;` se borraba (la
// ordenación por burbuja compilaba pero dejaba de ordenar), `exito(...)` se
// perdía y `10.0` pasaba a `10`, convirtiendo divisiones reales en enteras.
// Además el formateo no era idempotente: los paréntesis se acumulaban.
// ---------------------------------------------------------------------------

fn fmt_ok(src: &str) -> String {
    lumen_fmt::format_source(src).expect("el formateador no debe fallar")
}

#[test]
fn bug053_fmt_conserva_lambdas() {
    let src = "sea f = funcion(entero x) { retornar x + 1; };\nimprimir(f(1));\n";
    let out = fmt_ok(src);
    assert!(
        out.contains("funcion(entero x)"),
        "la lambda debe conservarse, se obtuvo:\n{}",
        out
    );
    assert!(!out.contains("Infer"), "`sea` no debe salir como `Infer`");
}

#[test]
fn bug053_fmt_conserva_asignacion_a_indice() {
    let src = "lista<entero> l = [1, 2];\nl[0] = 9;\nimprimir(l[0]);\n";
    let out = fmt_ok(src);
    assert!(
        out.contains("l[0] = 9;"),
        "la asignación a índice debe conservarse:\n{}",
        out
    );
}

#[test]
fn bug053_fmt_conserva_decimales_y_resultado() {
    let src = "imprimir(10.0 / 4.0);\n";
    let out = fmt_ok(src);
    assert!(
        out.contains("10.0") && out.contains("4.0"),
        "los decimales deben conservar el .0 o la división cambia de tipo:\n{}",
        out
    );
}

#[test]
fn bug053_fmt_es_idempotente() {
    let src = "entero i = 1;\nmientras (i <= 3) { i = i + 1; }\nimprimir(i);\n";
    let una = fmt_ok(src);
    let dos = fmt_ok(&una);
    assert_eq!(una, dos, "formatear dos veces debe dar lo mismo");
}

// ---------------------------------------------------------------------------
// BUG-054: `__ffi_escribir` desbordaba el búfer con texto UTF-8.
//
// `_tc_write` (stdlib/tui_core.nv) reservaba `s.largo()` bytes, pero `largo()`
// cuenta CARACTERES. Los marcos de `tui_ventana` usan box-drawing (`╭─│`), de
// 3 bytes cada uno, así que la copia pisaba el heap y el proceso abortaba con
// `realloc(): invalid next size` (rc=134). Ahora la VM conoce el tamaño real
// de cada reserva y rechaza la escritura en vez de corromper la memoria.
// ---------------------------------------------------------------------------

#[test]
fn bug054_ffi_escribir_rechaza_desbordar_el_bufer() {
    let src = r#"
texto s = "╭─────╮";
entero buf = __ffi_asignar(s.largo());
__ffi_escribir(buf, 0, s);
imprimir("no deberia llegar aqui");
"#;
    let src = format!("{}\n", src.trim());
    let msg = match run_source(&src) {
        Ok(o) => panic!("debía fallar en vez de corromper el heap, salió: {:?}", o),
        Err(e) => e,
    };
    assert!(
        msg.contains("__ffi_escribir") && msg.contains("bytes"),
        "el error debe explicar el desbordamiento en bytes:\n{}",
        msg
    );
}

#[test]
fn bug054_ffi_escribir_devuelve_los_bytes_escritos() {
    // 7 caracteres, 21 bytes en UTF-8: quien llama necesita la cifra en bytes
    // para pasársela a `write(2)`, si no el texto sale truncado.
    let src = "texto s = \"╭─────╮\";\n\
               entero buf = __ffi_asignar(s.largo() * 4 + 1);\n\
               imprimir(__ffi_escribir(buf, 0, s));\n";
    assert_eq!(out(src), vec!["21"]);
}

#[test]
fn bug054_ffi_ascii_sigue_funcionando() {
    let src = "texto s = \"hola\";\n\
               entero buf = __ffi_asignar(s.largo() * 4 + 1);\n\
               imprimir(__ffi_escribir(buf, 0, s));\n\
               __ffi_liberar(buf, s.largo() * 4 + 1);\n";
    assert_eq!(out(src), vec!["4"]);
}

// ---------------------------------------------------------------------------
// BUG-022: `intentar/atrapar` no capturaba nada.
//
// El generador emitía la etiqueta del `atrapar` pero NADIE saltaba a ella, y
// `err_var` se ignoraba: el bloque era código muerto y cualquier error abortaba
// el programa entero. Ahora `PushHandler`/`PopHandler` instalan manejadores
// reales y la VM desenrolla la pila hasta el `atrapar` más interno.
// ---------------------------------------------------------------------------

#[test]
fn bug022_atrapa_division_por_cero_y_continua() {
    let src = "intentar { imprimir(1 / 0); } atrapar (e) { imprimir(\"capturado\"); }\n\
               imprimir(\"sigue\");\n";
    assert_eq!(out(src), vec!["capturado", "sigue"]);
}

#[test]
fn bug022_liga_el_mensaje_a_la_variable_del_atrapar() {
    let src = "intentar { imprimir(1 / 0); } atrapar (e) { imprimir(e); }\n";
    assert_eq!(out(src), vec!["División por cero"]);
}

#[test]
fn bug022_sin_error_el_catch_no_se_ejecuta() {
    let src = "intentar { imprimir(\"ok\"); } atrapar (e) { imprimir(\"no\"); }\n\
               imprimir(\"fin\");\n";
    assert_eq!(out(src), vec!["ok", "fin"]);
}

#[test]
fn bug022_desenrolla_la_pila_desde_una_funcion() {
    // El error ocurre dos marcos más adentro: hay que tirar esos marcos para
    // llegar al `atrapar` en un estado coherente.
    let src = "funcion entero explota(entero x) { retornar x / 0; }\n\
               intentar { imprimir(explota(5)); } atrapar (e) { imprimir(\"capturado\"); }\n\
               imprimir(\"vivo\");\n";
    assert_eq!(out(src), vec!["capturado", "vivo"]);
}

#[test]
fn bug022_anidados_atrapa_el_mas_interno() {
    let src = "intentar {\n\
                 intentar { imprimir(1 / 0); } atrapar (a) { imprimir(\"interno\"); }\n\
                 imprimir(\"sigue externo\");\n\
               } atrapar (b) { imprimir(\"externo\"); }\n";
    assert_eq!(out(src), vec!["interno", "sigue externo"]);
}

#[test]
fn bug022_el_manejador_no_sobrevive_a_su_bloque() {
    // Tras cerrarse el `intentar`, su manejador debe desinstalarse: si no, un
    // error posterior saltaría a un `atrapar` que ya no aplica.
    let src = "intentar { imprimir(\"ok\"); } atrapar (e) { imprimir(\"no\"); }\n\
               imprimir(1 / 0);\n";
    let msg = match run_source(src) {
        Ok(o) => panic!("el error de fuera del try no debía capturarse: {:?}", o),
        Err(e) => e,
    };
    // `run_source` formatea el error con `{:?}`, de ahí `DivisionByZero`.
    assert!(msg.contains("DivisionByZero"), "error inesperado: {}", msg);
}

#[test]
fn bug022_no_fuga_manejadores_en_un_bucle() {
    // 200 vueltas atrapando: los manejadores deben instalarse y quitarse en
    // cada iteración, no acumularse.
    let src = "entero i = 0;\n\
               entero fallos = 0;\n\
               mientras (i < 200) {\n\
                 intentar { imprimir(1 / (i - i)); } atrapar (e) { fallos = fallos + 1; }\n\
                 i = i + 1;\n\
               }\n\
               imprimir(fallos);\n";
    assert_eq!(out(src), vec!["200"]);
}

#[test]
fn bug022_la_pila_de_operandos_queda_limpia() {
    // El error se produce evaluando un argumento: lo ya apilado debe
    // descartarse o la siguiente llamada leería basura.
    let src = "funcion entero suma(entero a, entero b) { retornar a + b; }\n\
               intentar { imprimir(suma(1, 2 / 0)); } atrapar (e) { imprimir(\"atrapado\"); }\n\
               imprimir(suma(20, 22));\n";
    assert_eq!(out(src), vec!["atrapado", "42"]);
}

#[test]
fn bug022_retornar_desde_el_catch() {
    let src = "funcion entero seguro(entero a, entero b) {\n\
                 intentar { retornar a / b; } atrapar (e) { retornar -1; }\n\
               }\n\
               imprimir(seguro(10, 2));\n\
               imprimir(seguro(10, 0));\n";
    assert_eq!(out(src), vec!["5", "-1"]);
}

#[test]
fn bug022_atrapa_indice_fuera_de_rango() {
    let src = "lista<entero> l = [1, 2, 3];\n\
               intentar { imprimir(l[99]); } atrapar (e) { imprimir(\"capturado\"); }\n\
               imprimir(\"vivo\");\n";
    assert_eq!(out(src), vec!["capturado", "vivo"]);
}

// ---------------------------------------------------------------------------
// BUG-056: `fmt` partía `} sino {` y `} atrapar (e) {` en dos líneas.
//
// `fmt_block` termina con un salto, así que al encadenar la cláusula siguiente
// salía `}\n sino {`: sintácticamente válido, pero feo y con un espacio suelto.
// ---------------------------------------------------------------------------

#[test]
fn bug056_fmt_mantiene_sino_en_la_misma_linea() {
    let src = "si (1 > 0) { imprimir(\"a\"); } sino { imprimir(\"b\"); }\n";
    let out = fmt_ok(src);
    assert!(
        out.contains("} sino {"),
        "el `sino` debe seguir a la llave de cierre:\n{}",
        out
    );
    assert!(
        !out.contains("\n sino"),
        "salto espurio antes de `sino`:\n{}",
        out
    );
}

#[test]
fn bug056_fmt_mantiene_atrapar_en_la_misma_linea() {
    let src = "intentar { imprimir(1); } atrapar (e) { imprimir(e); }\n";
    let out = fmt_ok(src);
    assert!(
        out.contains("} atrapar (e) {"),
        "el `atrapar` debe seguir a la llave de cierre:\n{}",
        out
    );
}

// ---------------------------------------------------------------------------
// BUG-058: los parámetros de tipo dentro de tipos compuestos no se resolvían.
//
// `resolve_type` sólo trataba `Type::Struct` y `GenericStruct`; el resto caía en
// `type_to_info`, que no conoce los `type_params`. Así, la `T` de `lista<T>` se
// resolvía como un struct vacío llamado "T" en vez de `TypeVar("T")`, y pasar
// una `lista<entero>` a `funcion entero cuantos<T>(lista<T> l)` daba E041. Con
// `T` a secas sí funcionaba, que es lo que lo hacía desconcertante.
// ---------------------------------------------------------------------------

#[test]
fn bug058_generico_infiere_dentro_de_lista() {
    let src = "funcion entero cuantos<T>(lista<T> l) { retornar l.largo(); }\n\
               lista<entero> a = [1, 2, 3];\n\
               imprimir(cuantos(a));\n";
    assert_eq!(out(src), vec!["3"]);
}

#[test]
fn bug058_generico_en_lista_anidada() {
    let src = "funcion entero total<T>(lista<lista<T>> m) { retornar m.largo(); }\n\
               lista<lista<entero>> m = [[1, 2], [3]];\n\
               imprimir(total(m));\n";
    assert_eq!(out(src), vec!["2"]);
}

#[test]
fn bug058_sigue_rechazando_un_tipo_incompatible_de_verdad() {
    // El arreglo no debe volver permisivo al comprobador: un entero no es una
    // lista, con genéricos o sin ellos.
    let src = "funcion entero cuantos<T>(lista<T> l) { retornar l.largo(); }\n\
               imprimir(cuantos(42));\n";
    assert!(
        run_source(src).is_err(),
        "pasar un entero donde se espera lista<T> debe seguir siendo un error"
    );
}

#[test]
fn bug058_tipo_explicito_sigue_funcionando() {
    let src = "funcion T identidad<T>(T x) { retornar x; }\n\
               imprimir(identidad<entero>(42));\n";
    assert_eq!(out(src), vec!["42"]);
}

// ---------------------------------------------------------------------------
// BUG-059: los errores de tipos filtraban la representación interna de Rust.
//
// Los mensajes usaban `{:?}` sobre `TypeInfo`, así que el usuario leía
// `Lista(Texto)` o, peor, `Struct { name: "P", fields: [("x", Entero)] }` en
// lugar de la sintaxis del propio lenguaje.
// ---------------------------------------------------------------------------

#[test]
fn bug059_el_error_usa_la_sintaxis_del_lenguaje_no_la_de_rust() {
    let src = "funcion entero f(lista<texto> l) { retornar 1; }\n\
               lista<entero> a = [1];\n\
               imprimir(f(a));\n";
    let msg = match run_source(src) {
        Ok(o) => panic!("debía fallar: {:?}", o),
        Err(e) => e,
    };
    assert!(
        msg.contains("lista<texto>") && msg.contains("lista<entero>"),
        "el error debe hablar en LÚMEN:\n{}",
        msg
    );
    assert!(
        !msg.contains("Lista(") && !msg.contains("Texto)"),
        "no debe filtrar el Debug de Rust:\n{}",
        msg
    );
}

#[test]
fn bug059_un_struct_se_nombra_por_su_nombre() {
    let src = "estructura P { x: entero }\n\
               funcion entero g(P p) { retornar p.x; }\n\
               imprimir(g(5));\n";
    let msg = match run_source(src) {
        Ok(o) => panic!("debía fallar: {:?}", o),
        Err(e) => e,
    };
    assert!(
        !msg.contains("fields:"),
        "no debe volcar los campos del struct:\n{}",
        msg
    );
    assert!(msg.contains('P'), "debe nombrar el struct:\n{}", msg);
}

// ---------------------------------------------------------------------------
// BUG-060: una lambda recursiva no se veía a sí misma.
//
// `sea fact = funcion(entero n) { ... fact(n - 1) ... };` fallaba con E042
// «La función 'fact' no está definida»: sema analizaba el cuerpo ANTES de
// declarar el nombre. La VM sí lo soportaba (asignar la lambda a una variable
// ya declarada daba 120), así que era sólo un problema de orden. Además, el
// generador capturaba la autorreferencia por valor cuando aún no tenía valor.
// ---------------------------------------------------------------------------

#[test]
fn bug060_lambda_recursiva_factorial() {
    let src = "sea fact = funcion(entero n) {\n\
                 si (n <= 1) { retornar 1; }\n\
                 retornar n * fact(n - 1);\n\
               };\n\
               imprimir(fact(5));\n";
    assert_eq!(out(src), vec!["120"]);
}

#[test]
fn bug060_lambda_recursiva_con_dos_llamadas() {
    // `fib` se llama dos veces por nivel: si el parámetro no se restaura entre
    // llamadas, el resultado sale mal (así se descubrió BUG-061 en el AOT).
    let src = "sea fib = funcion(entero n) {\n\
                 si (n < 2) { retornar n; }\n\
                 retornar fib(n - 1) + fib(n - 2);\n\
               };\n\
               imprimir(fib(10));\n";
    assert_eq!(out(src), vec!["55"]);
}

#[test]
fn bug060_lambda_recursiva_dentro_de_una_funcion() {
    let src = "funcion entero calcular(entero n) {\n\
                 sea f = funcion(entero k) {\n\
                   si (k <= 1) { retornar 1; }\n\
                   retornar k * f(k - 1);\n\
                 };\n\
                 retornar f(n);\n\
               }\n\
               imprimir(calcular(5));\n";
    assert_eq!(out(src), vec!["120"]);
}

#[test]
fn bug060_no_rompe_la_captura_normal_del_entorno() {
    // La lambda sigue capturando variables que NO son ella misma.
    let src = "entero base = 7;\n\
               sea suma = funcion(entero x) { retornar x + base; };\n\
               imprimir(suma(3));\n";
    assert_eq!(out(src), vec!["10"]);
}

#[test]
fn bug060_no_rompe_las_closures_con_captura_mutable() {
    // Regresión de BUG-052: dos contadores de la misma factoría siguen
    // aislados y conservando su estado.
    let src = "sea mk = funcion(entero n) {\n\
                 sea inc = funcion() { n = n + 1; retornar n; };\n\
                 retornar inc;\n\
               };\n\
               sea a = mk(10);\n\
               sea b = mk(100);\n\
               imprimir(a());\n\
               imprimir(a());\n\
               imprimir(b());\n";
    assert_eq!(out(src), vec!["11", "12", "101"]);
}

// ---------------------------------------------------------------------------
// BUG-062: las etiquetas de las lambdas colisionaban con las del programa.
//
// `codegen` resuelve los saltos con un ÚNICO mapa global `etiqueta -> posición`,
// pero `compile_lambda` reiniciaba el contador de etiquetas a 0 en cada lambda.
// Así, el `L0` de la lambda sobrescribía el `L0` de la función envolvente y los
// saltos aterrizaban en otra función. El síntoma era desconcertante: un
// `si/sino` seguido de una lambda hacía que se ejecutaran LAS DOS ramas, y una
// lambda recursiva no terminaba nunca. Ya estaba en v2.4.6, donde el programa
// de `bug062_las_ramas_no_se_mezclan` se cuelga para siempre.
// ---------------------------------------------------------------------------

#[test]
fn bug062_las_ramas_no_se_mezclan() {
    let src = "si (1 > 0) { imprimir(\"A\"); } sino { imprimir(\"B\"); }\n\
               sea f = funcion() { si (falso) { imprimir(\"X\"); } sino { imprimir(\"Y\"); } retornar 0; };\n\
               f();\n\
               si (1 > 0) { imprimir(\"C\"); } sino { imprimir(\"D\"); }\n";
    assert_eq!(out(src), vec!["A", "Y", "C"]);
}

#[test]
fn bug062_lambda_recursiva_despues_de_un_condicional() {
    let src = "si (1 > 0) { imprimir(\"antes\"); } sino { imprimir(\"no\"); }\n\
               sea fact = funcion(entero n) {\n\
                 si (n <= 1) { retornar 1; }\n\
                 retornar n * fact(n - 1);\n\
               };\n\
               imprimir(fact(5));\n";
    assert_eq!(out(src), vec!["antes", "120"]);
}

#[test]
fn bug062_lambda_recursiva_despues_de_un_bucle() {
    let src = "mientras (falso) { imprimir(\"nunca\"); }\n\
               sea fact = funcion(entero n) {\n\
                 si (n <= 1) { retornar 1; }\n\
                 retornar n * fact(n - 1);\n\
               };\n\
               imprimir(fact(5));\n";
    assert_eq!(out(src), vec!["120"]);
}

#[test]
fn bug062_lambda_recursiva_despues_de_un_elegir() {
    let src = "elegir (5) {\n\
                 caso 5: imprimir(\"cinco\");\n\
                 defecto: imprimir(\"otro\");\n\
               }\n\
               sea fact = funcion(entero n) {\n\
                 si (n <= 1) { retornar 1; }\n\
                 retornar n * fact(n - 1);\n\
               };\n\
               imprimir(fact(5));\n";
    assert_eq!(out(src), vec!["cinco", "120"]);
}

// ---------------------------------------------------------------------------
// BUG-063: las variables de bloque machacaban las de fuera en vez de
// sombrearlas.
//
// `sema` empuja un ámbito por bloque, pero las variables del runtime son planas
// por marco (una tabla por nombre), así que `si (...) { entero x = 2; }` pisaba
// la `x` exterior. Peor: como `sema` seguía creyendo que la de fuera era la
// suya, se podía declarar `texto x` dentro de un bloque y luego hacer `x + 10`
// fuera — `check` daba el programa por válido y el resultado era "hola10".
// Presente también en v2.4.6 y en los dos backends.
// ---------------------------------------------------------------------------

#[test]
fn bug063_bloque_si_no_pisa_la_variable_de_fuera() {
    let src = "entero x = 1;\n\
               si (1 > 0) { entero x = 2; imprimir(x); }\n\
               imprimir(x);\n";
    assert_eq!(out(src), vec!["2", "1"]);
}

#[test]
fn bug063_bloque_mientras_no_pisa_la_variable_de_fuera() {
    let src = "texto s = \"fuera\";\n\
               mientras (verdadero) { texto s = \"dentro\"; romper; }\n\
               imprimir(s);\n";
    assert_eq!(out(src), vec!["fuera"]);
}

#[test]
fn bug063_bloque_para_no_pisa_la_variable_de_fuera() {
    let src = "entero t = 10;\n\
               para i en 1..3 { entero t = i; }\n\
               imprimir(t);\n";
    assert_eq!(out(src), vec!["10"]);
}

#[test]
fn bug063_el_sombreado_no_cambia_el_tipo_de_la_de_fuera() {
    // Antes imprimía "hola" y luego "hola10": la `x` exterior (entero) se
    // quedaba con el texto del bloque y `x + 10` concatenaba.
    let src = "entero x = 1;\n\
               si (1 > 0) { texto x = \"hola\"; imprimir(x); }\n\
               imprimir(x + 10);\n";
    assert_eq!(out(src), vec!["hola", "11"]);
}

#[test]
fn bug063_el_bloque_sigue_viendo_y_mutando_lo_de_fuera() {
    // El sombreado sólo debe activarse cuando hay una declaración nueva; una
    // asignación normal desde dentro del bloque tiene que seguir funcionando.
    let src = "entero a = 1;\n\
               si (1 > 0) { a = 5; }\n\
               imprimir(a);\n\
               entero b = 0;\n\
               para i en 1..4 { b = b + i; }\n\
               imprimir(b);\n";
    assert_eq!(out(src), vec!["5", "6"]);
}

// ---------------------------------------------------------------------------
// BUG-064: `agregar(l, x)` en forma de función no mutaba nada, en silencio.
//
// El builtin es funcional: apila la lista nueva y, como sentencia, ese valor se
// descartaba con un `Drop`. El elemento se perdía sin error alguno y
// `lumen check` daba el programa por válido. La forma método `l.agregar(x)` sí
// escribía de vuelta desde BUG-033, así que dos sintaxis equivalentes hacían
// cosas distintas. Ya estaba en v2.4.6. Lo encontró el fuzzer diferencial de
// structs/listas/`prestado mut`.
// ---------------------------------------------------------------------------

#[test]
fn bug064_agregar_como_funcion_muta_la_lista() {
    let src = "lista<entero> l = [1, 2];\n\
               agregar(l, 9);\n\
               imprimir(largo(l));\n\
               imprimir(l[2]);\n";
    assert_eq!(out(src), vec!["3", "9"]);
}

#[test]
fn bug064_agregar_funcion_y_metodo_coinciden() {
    let src = "lista<entero> a = [1];\n\
               lista<entero> b = [1];\n\
               agregar(a, 2);\n\
               b.agregar(2);\n\
               imprimir(largo(a), \" \", largo(b));\n";
    assert_eq!(out(src), vec!["2 2"]);
}

#[test]
fn bug064_agregar_sobre_campo_de_struct() {
    let src = "estructura C { items: lista<entero>, }\n\
               C c = C { items: [1] };\n\
               agregar(c.items, 5);\n\
               imprimir(largo(c.items));\n";
    assert_eq!(out(src), vec!["2"]);
}

#[test]
fn bug064_la_semantica_por_valor_se_mantiene() {
    // `agregar` dentro de una función NO debe verse fuera: las listas se pasan
    // por valor salvo `prestado mut`.
    let src = "funcion entero mete(lista<entero> l) {\n\
                 agregar(l, 9);\n\
                 retornar largo(l);\n\
               }\n\
               lista<entero> xs = [1, 2];\n\
               imprimir(mete(xs));\n\
               imprimir(largo(xs));\n";
    assert_eq!(out(src), vec!["3", "2"]);
}

#[test]
fn bug064_prestado_mut_sigue_propagando() {
    let src = "funcion entero mete(prestado mut lista<entero> l) {\n\
                 agregar(l, 9);\n\
                 retornar largo(l);\n\
               }\n\
               lista<entero> xs = [1, 2];\n\
               imprimir(mete(xs));\n\
               imprimir(largo(xs));\n";
    assert_eq!(out(src), vec!["3", "3"]);
}

#[test]
fn bug090_agregar_devuelve_la_lista_no_vacio() {
    // BUG-090: sema declaraba `agregar` como `vacio` aunque el runtime SIEMPRE
    // devolvió la lista nueva. La contradicción sólo se notaba al asignar el
    // resultado a un campo o a un elemento —`c.items = agregar(c.items, x)`,
    // que es la forma documentada de usarlo con structs—: saltaba un E031 «no
    // puedes asignar un valor de tipo 'vacio'». Con `sea` colaba porque no se
    // comprueba el tipo declarado. Antes este test fijaba el comportamiento
    // antiguo; ahora fija el correcto: el tipo estático coincide con lo que el
    // runtime devuelve de verdad.
    let src = "lista<entero> l = [1];\n\
               sea otra = agregar(l, 2);\n\
               imprimir(largo(otra));\n";
    let lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let parser = Parser::new(tokens);
    let (mut program, _) = parser.parse();
    let errores = SemanticAnalyzer::new().analyze(&mut program);
    assert!(
        errores.is_empty(),
        "usar el retorno de `agregar` es válido, salió: {errores:?}"
    );
    assert_eq!(out(src), vec!["2"]);
}

#[test]
fn bug090_asignar_el_retorno_de_agregar_a_un_campo_es_valido() {
    // El caso que destapó el fallo: la única forma de hacer crecer una lista
    // dentro de un struct es reasignar el campo.
    let src = "estructura C { items: lista<entero>, }\n\
               sea c = C{items: [1]};\n\
               c.items = agregar(c.items, 2);\n\
               imprimir(largo(c.items));\n";
    assert_eq!(out(src), vec!["2"]);
}

// ---------------------------------------------------------------------------
// BUG-065: el dato capturado en un patrón se ligaba SIEMPRE como `numero`.
//
// `bind_pattern_vars` definía todo identificador del patrón con
// `TypeInfo::Numero` sin mirar el tipo del valor examinado. En la práctica
// `opcion<T>` y `resultado<T,E>` sólo servían con números: al hacer
// `elegir (o) { caso algun(p): p.campo }` sobre un `opcion<Contacto>` saltaba
// «E060 No puedes acceder a un campo de un valor de tipo 'numero'». Ya estaba
// en v2.4.6. Lo encontré escribiendo una agenda de contactos de verdad.
// ---------------------------------------------------------------------------

#[test]
fn bug065_opcion_de_struct_conserva_el_tipo() {
    let src = "estructura P { x: entero, }\n\
               funcion opcion<P> dame() { retornar algun(P { x: 7 }); }\n\
               elegir (dame()) {\n\
                 caso algun(p): imprimir(p.x);\n\
                 caso ninguno: imprimir(\"nada\");\n\
               }\n";
    assert_eq!(out(src), vec!["7"]);
}

#[test]
fn bug065_resultado_de_struct_conserva_el_tipo() {
    let src = "estructura P { x: entero, }\n\
               funcion resultado<P, texto> dame() { retornar exito(P { x: 9 }); }\n\
               elegir (dame()) {\n\
                 caso exito(p): imprimir(p.x);\n\
                 caso error(e): imprimir(e);\n\
               }\n";
    assert_eq!(out(src), vec!["9"]);
}

#[test]
fn bug065_el_error_conserva_su_tipo_texto() {
    let src = "estructura P { x: entero, }\n\
               funcion resultado<P, texto> dame() { retornar error(\"vaya\"); }\n\
               elegir (dame()) {\n\
                 caso exito(p): imprimir(p.x);\n\
                 caso error(e): imprimir(largo(e));\n\
               }\n";
    assert_eq!(out(src), vec!["4"]);
}

#[test]
fn bug065_opcion_de_lista_conserva_el_tipo() {
    let src = "funcion opcion<lista<entero>> dame() { retornar algun([1, 2, 3]); }\n\
               elegir (dame()) {\n\
                 caso algun(l): imprimir(largo(l));\n\
                 caso ninguno: imprimir(\"nada\");\n\
               }\n";
    assert_eq!(out(src), vec!["3"]);
}

#[test]
fn bug065_opcion_de_texto_conserva_el_tipo() {
    let src = "funcion opcion<texto> dame() { retornar algun(\"hola\"); }\n\
               elegir (dame()) {\n\
                 caso algun(s): imprimir(largo(s));\n\
                 caso ninguno: imprimir(\"nada\");\n\
               }\n";
    assert_eq!(out(src), vec!["4"]);
}

#[test]
fn bug065_los_numeros_siguen_funcionando() {
    let src = "opcion<entero> o = algun(5);\n\
               elegir (o) {\n\
                 caso algun(n): imprimir(n + 1);\n\
                 caso ninguno: imprimir(\"nada\");\n\
               }\n";
    assert_eq!(out(src), vec!["6"]);
}

// ---------------------------------------------------------------------------
// BUG-091: builtins cuyo tipo estático no coincidía con lo que devuelven.
//
//   · `__map_longitud` no estaba registrado en sema y caía al tipo por defecto
//     `decimal`, pese a devolver siempre un entero.
//   · `piso`, `techo` y `redondear` se declaraban `decimal` aunque su cometido
//     es justamente dar el entero más cercano.
//
// En ambos casos el runtime devolvía un entero, así que el uso natural
// —guardar el resultado en un campo `entero`— fallaba con un E031 inventado.
// ---------------------------------------------------------------------------
#[test]
fn bug091_map_longitud_es_entero_y_cabe_en_un_campo_entero() {
    let src = "estructura S { n: entero, }\n\
               sea s = S{n: 0};\n\
               sea m = __map_nuevo();\n\
               m = __map_poner(m, \"a\", 1);\n\
               s.n = __map_longitud(m);\n\
               imprimir(s.n);\n";
    assert_eq!(out(src), vec!["1"]);
}

#[test]
fn bug091_piso_techo_y_redondear_son_enteros() {
    let src = "estructura S { n: entero, }\n\
               sea s = S{n: 0};\n\
               s.n = piso(3.7);\n\
               imprimir(s.n);\n\
               s.n = techo(3.2);\n\
               imprimir(s.n);\n\
               s.n = redondear(3.5);\n\
               imprimir(s.n);\n";
    assert_eq!(out(src), vec!["3", "4", "4"]);
}

#[test]
fn bug091_raiz_sigue_siendo_decimal() {
    // `raiz(2.0)` no es entero: este NO debe cambiar.
    let src = "sea r = raiz(16.0);\nimprimir(r);\n";
    assert_eq!(out(src), vec!["4"]);

    let malo = "estructura S { n: entero, }\n\
                sea s = S{n: 0};\n\
                s.n = raiz(16.0);\n";
    let lexer = Lexer::new(malo);
    let (tokens, _) = lexer.tokenize();
    let parser = Parser::new(tokens);
    let (mut program, _) = parser.parse();
    let errores = SemanticAnalyzer::new().analyze(&mut program);
    assert!(
        !errores.is_empty(),
        "asignar un decimal a un campo entero debe seguir siendo error"
    );
}

// ---------------------------------------------------------------------------
// BUG-092: los CUERPOS de los métodos de un `impl <Rasgo> para <Tipo>` no se
// analizaban nunca; sólo se comprobaba que las firmas encajaran con el rasgo.
// Dentro de un impl pasaba cualquier disparate: `lumen check` decía «es
// válido», la VM fallaba en runtime con «Variable no definida» y el binario
// nativo imprimía 0 en silencio. La rama de impl inherente sí los analizaba,
// así que las dos formas de `impl` no coincidían.
// ---------------------------------------------------------------------------
#[test]
fn bug092_una_variable_inexistente_en_un_metodo_de_rasgo_es_error() {
    let src = "rasgo R { funcion entero m(este); }\n\
               estructura S { c: entero, }\n\
               impl R para S { funcion entero m(este) { retornar zzz * 2; } }\n";
    let lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let parser = Parser::new(tokens);
    let (mut program, _) = parser.parse();
    let errores = SemanticAnalyzer::new().analyze(&mut program);
    assert!(
        errores.iter().any(|e| format!("{e:?}").contains("E033")),
        "esperaba E033 por la variable inexistente, salió: {errores:?}"
    );
}

#[test]
fn bug092_usar_un_campo_sin_este_es_error_dentro_del_impl() {
    // `c` es un campo, no una variable en ámbito: sin `este.` no existe.
    let src = "rasgo R { funcion entero m(este); }\n\
               estructura S { c: entero, }\n\
               impl R para S { funcion entero m(este) { retornar c * 2; } }\n";
    let lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let parser = Parser::new(tokens);
    let (mut program, _) = parser.parse();
    let errores = SemanticAnalyzer::new().analyze(&mut program);
    assert!(
        errores.iter().any(|e| format!("{e:?}").contains("E033")),
        "esperaba E033, salió: {errores:?}"
    );
}

#[test]
fn bug092_un_impl_correcto_sigue_siendo_valido() {
    let src = "rasgo Area { funcion entero area(este); }\n\
               estructura Cua { l: entero, }\n\
               impl Area para Cua { funcion entero area(este) { retornar este.l * este.l; } }\n\
               sea c = Cua{l: 3};\n\
               imprimir(c.area());\n";
    assert_eq!(out(src), vec!["9"]);
}

// ---------------------------------------------------------------------------
// BUG-093: al inicializar un struct genérico SIN argumentos de tipo explícitos
// no se sustituían los parámetros, así que los campos conservaban la variable
// de tipo (`T`). Con un nivel colaba, pero anidarlos era imposible: leer
// `a.v.v` daba «E060 No puedes acceder a un campo de un valor de tipo 'T'».
// ---------------------------------------------------------------------------
#[test]
fn bug093_los_structs_genericos_se_pueden_anidar() {
    let src = "estructura Caja<T> { v: T, }\n\
               sea anidada = Caja{v: Caja{v: 7}};\n\
               imprimir(anidada.v.v);\n";
    assert_eq!(out(src), vec!["7"]);
}

#[test]
fn bug093_generico_de_dos_parametros_anidado() {
    let src = "estructura Par<T,U> { a: T, b: U, }\n\
               sea p = Par{a: Par{a: 1, b: 2}, b: \"x\"};\n\
               imprimir(p.a.b);\n\
               imprimir(p.b);\n";
    assert_eq!(out(src), vec!["2", "x"]);
}

#[test]
fn bug093_un_solo_nivel_sigue_funcionando_con_varios_tipos() {
    let src = "estructura Caja<T> { v: T, }\n\
               sea a = Caja{v: 5};\n\
               sea b = Caja{v: \"texto\"};\n\
               imprimir(a.v);\n\
               imprimir(b.v);\n";
    assert_eq!(out(src), vec!["5", "texto"]);
}

// ---------------------------------------------------------------------------
// BUG-094: la asignación indexada sólo escribía de vuelta cuando el contenedor
// era una VARIABLE suelta. Si la base era un campo de struct —`m.g[i][j] = v`,
// `a.b.l[i] = v`, `a.l[0].campo = v`— el contenedor modificado se quedaba en la
// pila y se descartaba: la asignación no hacía nada, en silencio y sin error.
// `m.g[i] = v` (un solo índice) sí funcionaba, así que el fallo aparecía justo
// al anidar. Preexistente en v2.4.6.
// ---------------------------------------------------------------------------
#[test]
fn bug094_asignar_en_una_matriz_dentro_de_un_struct() {
    let src = "estructura M { g: lista<lista<entero>>, }\n\
               sea m = M{g: [[0,0],[0,0]]};\n\
               para i en 0..2 { para j en 0..2 { m.g[i][j] = i + j; } }\n\
               imprimir(m.g[0]);\n\
               imprimir(m.g[1]);\n";
    assert_eq!(out(src), vec!["[0, 1]", "[1, 2]"]);
}

#[test]
fn bug094_asignar_por_indice_en_un_campo_de_struct_anidado() {
    let src = "estructura A { l: lista<entero>, }\n\
               estructura B { a: A, }\n\
               sea b = B{a: A{l: [0, 0]}};\n\
               b.a.l[1] = 42;\n\
               imprimir(b.a.l);\n";
    assert_eq!(out(src), vec!["[0, 42]"]);
}

#[test]
fn bug094_asignar_a_un_campo_de_un_elemento_dentro_de_un_struct() {
    let src = "estructura P { x: entero, }\n\
               estructura A { l: lista<P>, }\n\
               sea a = A{l: [P{x: 1}]};\n\
               a.l[0].x = 99;\n\
               imprimir(a.l[0].x);\n";
    assert_eq!(out(src), vec!["99"]);
}

#[test]
fn bug094_los_casos_de_un_solo_nivel_siguen_funcionando() {
    // Estos ya funcionaban: fijarlos para no romperlos al generalizar.
    let simple = "sea l = [0, 0];\nl[0] = 5;\nimprimir(l);\n";
    assert_eq!(out(simple), vec!["[5, 0]"]);

    let matriz = "sea m = [[0,0],[0,0]];\nm[0][1] = 7;\nimprimir(m[0]);\n";
    assert_eq!(out(matriz), vec!["[0, 7]"]);

    let campo = "estructura M { g: lista<entero>, }\n\
                 sea m = M{g: [0, 0]};\n\
                 m.g[1] = 9;\n\
                 imprimir(m.g);\n";
    assert_eq!(out(campo), vec!["[0, 9]"]);

    let anidado = "estructura A { v: entero, }\n\
                   estructura B { a: A, }\n\
                   estructura C { b: B, }\n\
                   sea c = C{b: B{a: A{v: 1}}};\n\
                   c.b.a.v = 7;\n\
                   imprimir(c.b.a.v);\n";
    assert_eq!(out(anidado), vec!["7"]);
}

// ---------------------------------------------------------------------------
// BUG-097: una lista declarada con el literal vacío (`sea l = []`) tiene el
// tipo de elemento genérico `numero`. Al hacerla crecer con `agregar` ese tipo
// no se refinaba, así que iterarla devolvía elementos `numero` —que al operar
// se vuelven `decimal`— y acumular en un `entero` fallaba con un E031
// imposible de evitar sin anotar el tipo a mano. Es el patrón más natural para
// construir una lista, y no compilaba.
// ---------------------------------------------------------------------------
#[test]
fn bug097_acumular_enteros_de_una_lista_construida_con_agregar() {
    let src = "sea l = [];\n\
               para i en 1..=10 { l = agregar(l, i); }\n\
               sea t = 0;\n\
               para x en l { t = t + x; }\n\
               imprimir(largo(l));\n\
               imprimir(t);\n";
    assert_eq!(out(src), vec!["10", "55"]);
}

#[test]
fn bug097_el_refinamiento_funciona_con_textos_structs_y_listas() {
    let textos = "sea l = [];\n\
                  l = agregar(l, \"a\");\n\
                  sea s = \"\";\n\
                  para x en l { s = s + x; }\n\
                  imprimir(s);\n";
    assert_eq!(out(textos), vec!["a"]);

    let structs = "estructura P { x: entero, }\n\
                   sea l = [];\n\
                   l = agregar(l, P{x: 7});\n\
                   para p en l { imprimir(p.x); }\n";
    assert_eq!(out(structs), vec!["7"]);

    let listas = "sea l = [];\n\
                  l = agregar(l, [1, 2]);\n\
                  para f en l { imprimir(largo(f)); }\n";
    assert_eq!(out(listas), vec!["2"]);
}

#[test]
fn bug097_los_decimales_siguen_siendo_decimales() {
    let src = "sea l = [];\n\
               l = agregar(l, 1.5);\n\
               sea t = 0.0;\n\
               para x en l { t = t + x; }\n\
               imprimir(t);\n";
    assert_eq!(out(src), vec!["1.5"]);
}

#[test]
fn bug097_no_afloja_la_comprobacion_de_tipos() {
    // Sumar un texto a un entero debe seguir siendo un error: el refinamiento
    // ajusta el tipo del elemento, no permite mezclarlos.
    let src = "sea l = [];\n\
               l = agregar(l, \"a\");\n\
               sea t = 0;\n\
               para x en l { t = t + x; }\n";
    let lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let parser = Parser::new(tokens);
    let (mut program, _) = parser.parse();
    let errores = SemanticAnalyzer::new().analyze(&mut program);
    assert!(
        !errores.is_empty(),
        "sumar un texto a un entero debe seguir siendo error"
    );
}

// ---------------------------------------------------------------------------
// BUG-098: `__str_concat_list` recibe un único argumento (la lista). Nadie
// comprobaba la aridad, así que llamarlo con un separador extra —un error
// natural, porque el nombre sugiere un `join`— pasaba la validación: la VM
// ignoraba el sobrante y devolvía "abc", mientras que el backend C desapilaba
// un solo valor y se quedaba con el separador en lugar de con la lista,
// devolviendo la cadena vacía. El mismo programa daba resultados distintos
// según se interpretara o se compilara. Ahora se rechaza al analizar.
// ---------------------------------------------------------------------------
#[test]
fn bug098_concat_list_con_un_argumento_sigue_funcionando() {
    let src = "imprimir(__str_concat_list([\"a\", \"b\", \"c\"]));\n";
    assert_eq!(out(src), vec!["abc"]);
}

#[test]
fn bug098_concat_list_rechaza_aridades_incorrectas() {
    for src in [
        "imprimir(__str_concat_list());\n",
        "imprimir(__str_concat_list([\"a\", \"b\"], \"-\"));\n",
        "imprimir(__str_concat_list([\"a\"], \"-\", 9));\n",
    ] {
        let lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (mut program, _) = parser.parse();
        let errores = SemanticAnalyzer::new().analyze(&mut program);
        assert!(
            errores.iter().any(|e| e.code == "E040"),
            "se esperaba E040 por aridad en: {src}"
        );
    }
}

// ---------------------------------------------------------------------------
// BUG-099: `numero` es el tipo *dinámico* del analizador —lo que devuelven los
// builtins que no pueden saber estáticamente qué guardó el usuario, como
// `__map_obtener`—, pero se trataba literalmente como "un número". Consecuencia:
// todo lo que no fuera escalar y pasara por un mapa quedaba inutilizable
// (`p.x` → E060, `para x en xs` → E044) aunque el runtime lo maneja sin
// problema, y operar con un valor dinámico daba `decimal`, así que acumularlo
// en un `entero` fallaba con E031. Ahora el tipo dinámico se propaga como
// dinámico en los tres sitios.
// ---------------------------------------------------------------------------
#[test]
fn bug099_struct_guardado_en_un_mapa_permite_leer_sus_campos() {
    let src = "estructura P { x: entero, }\n\
               sea m = __map_nuevo();\n\
               m = __map_poner(m, \"p\", P{x: 5});\n\
               sea p = __map_obtener(m, \"p\");\n\
               imprimir(p.x);\n";
    assert_eq!(out(src), vec!["5"]);
}

#[test]
fn bug099_lista_guardada_en_un_mapa_se_puede_recorrer() {
    let src = "sea m = __map_nuevo();\n\
               m = __map_poner(m, \"xs\", [1, 2, 3]);\n\
               sea xs = __map_obtener(m, \"xs\");\n\
               sea t = 0;\n\
               para x en xs { t = t + x; }\n\
               imprimir(t);\n";
    assert_eq!(out(src), vec!["6"]);
}

#[test]
fn bug099_lista_vacia_de_un_mapa_se_puede_llenar_y_sumar() {
    // Cruce con BUG-097: lista vacía, guardada en un mapa, recuperada,
    // rellenada con `agregar` y acumulada en un entero.
    let src = "sea m = __map_nuevo();\n\
               m = __map_poner(m, \"xs\", []);\n\
               sea xs = __map_obtener(m, \"xs\");\n\
               xs = agregar(xs, 4);\n\
               sea t = 0;\n\
               para x en xs { t = t + x; }\n\
               imprimir(t);\n";
    assert_eq!(out(src), vec!["4"]);
}

#[test]
fn bug099_mapas_anidados_siguen_funcionando() {
    let src = "sea inner = __map_nuevo();\n\
               inner = __map_poner(inner, \"k\", 7);\n\
               sea outer = __map_nuevo();\n\
               outer = __map_poner(outer, \"i\", inner);\n\
               sea got = __map_obtener(outer, \"i\");\n\
               imprimir(__map_obtener(got, \"k\"));\n";
    assert_eq!(out(src), vec!["7"]);
}

#[test]
fn bug099_la_aritmetica_normal_no_cambia() {
    // Un decimal de verdad sigue dando decimal; sólo el tipo dinámico se
    // propaga como dinámico.
    assert_eq!(
        out("sea a = 1;\nsea b = 2.5;\nimprimir(a + b);\n"),
        vec!["3.5"]
    );
    assert_eq!(out("imprimir(2 + 3);\n"), vec!["5"]);
    assert_eq!(out("imprimir(\"a\" + 1);\n"), vec!["a1"]);
}

#[test]
fn bug099_no_afloja_las_comprobaciones_de_tipo() {
    // Recorrer un entero, leer un campo de un texto y asignar un decimal a un
    // entero deben seguir siendo errores.
    for (src, code) in [
        ("sea x = 5;\npara i en x { imprimir(i); }\n", "E044"),
        ("sea s = \"hola\";\nimprimir(s.campo);\n", "E060"),
        ("sea t = 0;\nt = 1 + 2.5;\n", "E031"),
    ] {
        let lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (mut program, _) = parser.parse();
        let errores = SemanticAnalyzer::new().analyze(&mut program);
        assert!(
            errores.iter().any(|e| e.code == code),
            "se esperaba {code} en: {src}"
        );
    }
}

#[test]
fn bug099_los_valores_dinamicos_tambien_son_asignables() {
    // El tipo dinámico llega también al lado izquierdo: un struct o una lista
    // sacados de un mapa se podían leer pero no modificar, pese a que el
    // runtime lo permite.
    let campo = "estructura P { x: entero, }\n\
                 sea m = __map_nuevo();\n\
                 m = __map_poner(m, \"p\", P{x: 1});\n\
                 sea p = __map_obtener(m, \"p\");\n\
                 p.x = 9;\n\
                 imprimir(p.x);\n";
    assert_eq!(out(campo), vec!["9"]);

    let indice = "sea m = __map_nuevo();\n\
                  m = __map_poner(m, \"xs\", [1, 2]);\n\
                  sea xs = __map_obtener(m, \"xs\");\n\
                  xs[0] = 7;\n\
                  imprimir(xs[0]);\n";
    assert_eq!(out(indice), vec!["7"]);
}

#[test]
fn bug099_asignaciones_invalidas_siguen_rechazandose() {
    for (src, code) in [
        ("sea s = \"x\";\ns.campo = 1;\n", "E060"),
        ("sea n = 5;\nn[0] = 1;\n", "E060"),
        (
            "estructura P { x: entero, }\nsea p = P{x: 1};\np.noexiste = 2;\n",
            "E059",
        ),
    ] {
        let lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (mut program, _) = parser.parse();
        let errores = SemanticAnalyzer::new().analyze(&mut program);
        assert!(
            errores.iter().any(|e| e.code == code),
            "se esperaba {code} en: {src}"
        );
    }
}

// ---------------------------------------------------------------------------
// BUG-100: la causa de fondo del BUG-099. El analizador usaba `TypeInfo::Numero`
// para dos cosas incompatibles: el tipo de los números y el tipo *dinámico* de
// los valores cuyo tipo real sólo se conoce en ejecución (lo que devuelve
// `__map_obtener`). Parchear caso por caso obligaba a aceptar `numero` en
// contextos donde no debía valer —`numero x = 1; si (x)` tiene que seguir
// fallando—, así que el tipo dinámico pasó a ser una variante propia,
// `TypeInfo::Dinamico`, compatible con todo y sin significado numérico.
// ---------------------------------------------------------------------------
#[test]
fn bug100_una_lambda_guardada_en_un_mapa_se_puede_llamar() {
    // Desde una lista sí funcionaba; desde un mapa daba E058.
    let src = "sea f = funcion(entero a) { retornar a * 2; };\n\
               sea m = __map_nuevo();\n\
               m = __map_poner(m, \"f\", f);\n\
               sea g = __map_obtener(m, \"f\");\n\
               imprimir(g(21));\n";
    assert_eq!(out(src), vec!["42"]);
}

#[test]
fn bug100_un_booleano_guardado_en_un_mapa_sirve_de_condicion() {
    let si = "sea m = __map_nuevo();\n\
              m = __map_poner(m, \"b\", verdadero);\n\
              sea b = __map_obtener(m, \"b\");\n\
              si b { imprimir(\"si\"); }\n";
    assert_eq!(out(si), vec!["si"]);

    let negado = "sea m = __map_nuevo();\n\
                  m = __map_poner(m, \"b\", verdadero);\n\
                  sea b = __map_obtener(m, \"b\");\n\
                  si !b { imprimir(\"no\"); } sino { imprimir(\"si\"); }\n";
    assert_eq!(out(negado), vec!["si"]);

    let bucle = "sea m = __map_nuevo();\n\
                 m = __map_poner(m, \"b\", verdadero);\n\
                 sea b = __map_obtener(m, \"b\");\n\
                 sea i = 0;\n\
                 mientras b { i = i + 1; si i > 2 { b = falso; } }\n\
                 imprimir(i);\n";
    assert_eq!(out(bucle), vec!["3"]);
}

#[test]
fn bug100_los_builtins_aceptan_valores_dinamicos() {
    let casos = [
        ("[3, 1, 2]", "imprimir(largo(d));", "3"),
        ("\"hola\"", "imprimir(__str_mayusculas(d));", "HOLA"),
        ("\"hola\"", "imprimir(__str_longitud(d));", "4"),
        ("5", "imprimir(abs(d));", "5"),
        ("5", "imprimir(d + 1);", "6"),
        ("[1, 2]", "imprimir(largo(agregar(d, 4)));", "3"),
        ("\"7\"", "imprimir(a_entero(d));", "7"),
        ("[1, 2]", "imprimir(d[1]);", "2"),
    ];
    for (valor, uso, esperado) in casos {
        let src = format!(
            "sea m = __map_nuevo();\n\
             m = __map_poner(m, \"v\", {valor});\n\
             sea d = __map_obtener(m, \"v\");\n\
             {uso}\n"
        );
        assert_eq!(out(&src), vec![esperado], "falló con {valor} / {uso}");
    }
}

#[test]
fn bug100_numero_explicito_no_es_un_valor_dinamico() {
    // El límite del arreglo: `numero` declarado a mano es un número, no un
    // valor de tipo desconocido, así que no vale como condición ni como
    // función. Este test es el que delató que el primer intento —aceptar
    // `Numero` en todas partes— era demasiado amplio.
    for (src, code) in [
        ("numero x = 1;\nsi (x) { }\n", "E034"),
        ("numero x = 1;\nmientras (x) { }\n", "E034"),
        ("numero x = 1;\nimprimir(!x);\n", "E039"),
    ] {
        let lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (mut program, _) = parser.parse();
        let errores = SemanticAnalyzer::new().analyze(&mut program);
        assert!(
            errores.iter().any(|e| e.code == code),
            "se esperaba {code} en: {src}"
        );
    }
}

#[test]
fn bug100_llamar_lo_que_no_es_funcion_sigue_fallando() {
    for src in [
        "sea x = 5;\nimprimir(x(1));\n",
        "sea s = \"h\";\nimprimir(s(1));\n",
        "sea l = [1, 2];\nimprimir(l(0));\n",
    ] {
        let lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (mut program, _) = parser.parse();
        let errores = SemanticAnalyzer::new().analyze(&mut program);
        assert!(
            errores.iter().any(|e| e.code == "E058"),
            "se esperaba E058 en: {src}"
        );
    }
}

// ---------------------------------------------------------------------------
// BUG-101: dos fallos encadenados en `elegir` sobre un `resultado` sin anotar,
// ambos REGRESIONES introducidas por mis propios arreglos anteriores (la v2.4.6
// oficial ejecuta estos programas sin protestar):
//
//   1. `caso exito(v)` / `caso error(e)` son *patrones* que descomponen el
//      sujeto, pero se analizaban como expresiones. `error(e)` se leía entonces
//      como una construcción `error(...)` con argumento desconocido y disparaba
//      un E064 («no puedes crear un resultado de error con un valor vacío»)
//      sobre un `elegir` correcto.
//   2. `sea r = exito(1)` marcaba el tipo del error como `vacio` en vez de
//      «desconocido», así que `caso error(e)` ligaba `e` a un valor vacío y
//      usarlo (`"e=" + e`) daba un E035 absurdo.
//
// Sólo ocurría sin anotar el tipo — que es justo la forma corta y habitual.
// ---------------------------------------------------------------------------
#[test]
fn bug101_elegir_sobre_un_resultado_sin_anotar() {
    let src = "sea r = exito(21);\n\
               elegir (r) {\n\
                 caso exito(v): imprimir(v * 2);\n\
                 caso error(e): imprimir(\"e=\" + e);\n\
               }\n";
    assert_eq!(out(src), vec!["42"]);
}

#[test]
fn bug101_la_rama_de_error_se_ejecuta_y_liga_su_valor() {
    let src = "sea r = error(\"roto\");\n\
               elegir (r) {\n\
                 caso exito(v): imprimir(v);\n\
                 caso error(e): imprimir(\"e=\" + e);\n\
               }\n";
    assert_eq!(out(src), vec!["e=roto"]);
}

#[test]
fn bug101_funciona_con_valores_dinamicos_dentro_del_resultado() {
    let listas = "sea m = __map_nuevo();\n\
                  m = __map_poner(m, \"xs\", [1, 2, 3]);\n\
                  sea r = exito(__map_obtener(m, \"xs\"));\n\
                  elegir (r) {\n\
                    caso exito(v): imprimir(largo(v));\n\
                    caso error(e): imprimir(\"err\");\n\
                  }\n";
    assert_eq!(out(listas), vec!["3"]);

    let enteros = "sea m = __map_nuevo();\n\
                   m = __map_poner(m, \"n\", 5);\n\
                   sea r = exito(__map_obtener(m, \"n\"));\n\
                   elegir (r) {\n\
                     caso exito(v): imprimir(v + 1);\n\
                     caso error(e): imprimir(\"err\");\n\
                   }\n";
    assert_eq!(out(enteros), vec!["6"]);
}

#[test]
fn bug101_el_resultado_anotado_sigue_funcionando() {
    let src = "resultado<entero, texto> r = exito(21);\n\
               elegir (r) {\n\
                 caso exito(v): imprimir(v * 2);\n\
                 caso error(e): imprimir(\"e=\" + e);\n\
               }\n";
    assert_eq!(out(src), vec!["42"]);

    let funcion = "funcion resultado<entero, texto> dividir(entero a, entero b) {\n\
                     si b == 0 { retornar error(\"div0\"); }\n\
                     retornar exito(a / b);\n\
                   }\n\
                   elegir (dividir(10, 2)) {\n\
                     caso exito(v): imprimir(v);\n\
                     caso error(e): imprimir(e);\n\
                   }\n";
    assert_eq!(out(funcion), vec!["5"]);
}

#[test]
fn bug101_un_caso_mal_tipado_sigue_dando_e056() {
    let src = "sea n = 5;\n\
               elegir (n) {\n\
                 caso \"x\": imprimir(1);\n\
                 defecto: imprimir(2);\n\
               }\n";
    let lexer = Lexer::new(src);
    let (tokens, _) = lexer.tokenize();
    let parser = Parser::new(tokens);
    let (mut program, _) = parser.parse();
    let errores = SemanticAnalyzer::new().analyze(&mut program);
    assert!(errores.iter().any(|e| e.code == "E056"));
}

#[test]
fn bug101_opcion_con_valores_dinamicos() {
    // La otra mitad de la veta: `opcion` conteniendo lo que sale de un mapa.
    let lambda = "sea m = __map_nuevo();\n\
                  m = __map_poner(m, \"f\", funcion(entero a) { retornar a + 1; });\n\
                  sea o = algun(__map_obtener(m, \"f\"));\n\
                  si sea algun(g) = o { imprimir(g(41)); }\n";
    assert_eq!(out(lambda), vec!["42"]);

    let booleano = "sea m = __map_nuevo();\n\
                    m = __map_poner(m, \"b\", verdadero);\n\
                    sea o = algun(__map_obtener(m, \"b\"));\n\
                    si sea algun(b) = o { si b { imprimir(\"dentro\"); } }\n";
    assert_eq!(out(booleano), vec!["dentro"]);
}

// ---------------------------------------------------------------------------
// BUG-102: al introducir el tipo dinámico (BUG-100) metí en el mismo saco a
// `__map_obtener` —cuyo resultado sí es desconocido— y a `__map_nuevo` /
// `__map_poner`, que devuelven algo perfectamente conocido: un mapa. Como
// «dinámico» es compatible con todo, eso hizo que operaciones sin sentido sobre
// un mapa, que la v2.4.6 rechazaba, pasaran el analizador: `m[0]` se ejecutaba
// devolviendo un 0 inventado en la VM mientras el binario nativo abortaba con
// «Índice 0 fuera de rango», y `m[0] = 9` divergía igual. Un mapa tiene ahora
// su propio tipo.
// ---------------------------------------------------------------------------
#[test]
fn bug102_no_se_puede_indexar_un_mapa() {
    for (src, code) in [
        ("sea m = __map_nuevo();\nimprimir(m[0]);\n", "E044"),
        ("sea m = __map_nuevo();\nm[0] = 9;\nimprimir(m[0]);\n", "E060"),
        (
            "sea m = __map_nuevo();\nelegir (m) {\n  caso 5: imprimir(\"cinco\");\n  defecto: imprimir(\"otro\");\n}\n",
            "E056",
        ),
    ] {
        let lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (mut program, _) = parser.parse();
        let errores = SemanticAnalyzer::new().analyze(&mut program);
        assert!(
            errores.iter().any(|e| e.code == code),
            "se esperaba {code} en: {src}"
        );
    }
}

#[test]
fn bug102_el_uso_normal_de_los_mapas_no_se_toca() {
    let src = "sea m = __map_nuevo();\n\
               m = __map_poner(m, \"a\", 1);\n\
               m = __map_poner(m, \"b\", 2);\n\
               imprimir(__map_longitud(m));\n\
               imprimir(__map_obtener(m, \"a\"));\n\
               imprimir(__map_contiene(m, \"b\"));\n\
               imprimir(largo(__map_claves(m)));\n";
    assert_eq!(out(src), vec!["2", "1", "true", "2"]);

    // Y lo que se saca del mapa sigue siendo dinámico y utilizable (BUG-099).
    let dinamico = "estructura P { x: entero, }\n\
                    sea m = __map_nuevo();\n\
                    m = __map_poner(m, \"p\", P{x: 5});\n\
                    sea p = __map_obtener(m, \"p\");\n\
                    imprimir(p.x);\n";
    assert_eq!(out(dinamico), vec!["5"]);

    // Un mapa anidado dentro de otro sigue funcionando.
    let anidado = "sea i = __map_poner(__map_nuevo(), \"k\", 7);\n\
                   sea o = __map_poner(__map_nuevo(), \"i\", i);\n\
                   imprimir(__map_obtener(__map_obtener(o, \"i\"), \"k\"));\n";
    assert_eq!(out(anidado), vec!["7"]);
}

// ---------------------------------------------------------------------------
// BUG-103: definir una función con el nombre de un builtin que el runtime
// intercepta se aceptaba en silencio, y luego el builtin la suplantaba. El caso
// que lo destapó:
//
//   funcion vacio push(prestado mut lista<entero> l) { l = agregar(l, 9); }
//   sea l = [1];  push(l);  imprimir(largo(l));
//
// `push` es alias de `agregar`, así que la llamada iba al builtin con un
// argumento de menos, la variable quedaba en `vacio` y el programa moría con
// «'largo' espera lista o texto, no Void» — un mensaje que no menciona `push`
// por ninguna parte. La v2.4.6 al menos lo rechazaba al analizar (por otro
// motivo), así que era además una regresión de comportamiento.
// ---------------------------------------------------------------------------
#[test]
fn bug103_redefinir_un_builtin_interceptado_se_avisa() {
    for src in [
        "funcion vacio push(prestado mut lista<entero> l) { l = agregar(l, 9); }\nsea l = [1];\npush(l);\n",
        "funcion entero largo(entero x) { retornar 42; }\nimprimir(largo(3));\n",
        "funcion vacio imprimir(entero x) { }\n",
    ] {
        let lexer = Lexer::new(src);
        let (tokens, _) = lexer.tokenize();
        let parser = Parser::new(tokens);
        let (mut program, _) = parser.parse();
        let errores = SemanticAnalyzer::new().analyze(&mut program);
        assert!(
            errores.iter().any(|e| e.code == "E082"),
            "se esperaba E082 en: {src}"
        );
    }
}

#[test]
fn bug103_los_builtins_sombreables_siguen_permitidos() {
    // BUG-018 decidió que nombres tan naturales como `abs` o `leer` se pueden
    // redefinir, y la VM da prioridad a la función del usuario. Eso no cambia.
    assert_eq!(
        out("funcion entero abs(entero x) { retornar 42; }\nimprimir(abs(-3));\n"),
        vec!["42"]
    );
    assert_eq!(
        out("funcion texto leer() { retornar \"mio\"; }\nimprimir(leer());\n"),
        vec!["mio"]
    );
    assert_eq!(
        out(
            "funcion entero minimo(entero a, entero b) { retornar 99; }\nimprimir(minimo(1, 2));\n"
        ),
        vec!["99"]
    );
}

#[test]
fn bug103_el_prefijo_de_privado_no_es_un_builtin() {
    // La stdlib usa `__factorial`, `__render_mes`… como convención de «privado
    // del módulo». Son funciones suyas y deben poder declararse.
    let src = "funcion entero __ayuda(entero x) { retornar x * 2; }\nimprimir(__ayuda(21));\n";
    assert_eq!(out(src), vec!["42"]);
}

#[test]
fn bug103_una_funcion_con_nombre_propio_funciona() {
    // El mismo programa del repro, con un nombre que no choca.
    let src = "funcion vacio agrega(prestado mut lista<entero> l) { l = agregar(l, 9); }\n\
               sea l = [1];\n\
               agrega(l);\n\
               imprimir(largo(l));\n";
    assert_eq!(out(src), vec!["2"]);
}

// ── BUG-106: la reducción de fuerza `x*2 -> x<<1` ignoraba los decimales ────

#[test]
fn bug106_decimal_por_potencia_de_dos() {
    // El IR convertía `* 2|4|8` en desplazamientos de bits sin mirar el tipo:
    // en la VM reventaba con «ShiftLeft requires integers» y en el binario
    // nativo truncaba el decimal (2.5 * 2 imprimía 4).
    assert_eq!(out("sea x = 2.5;\nimprimir(x * 2);"), vec!["5"]);
    assert_eq!(out("sea x = 2.5;\nimprimir(x * 4);"), vec!["10"]);
    assert_eq!(out("sea x = 1.5;\nimprimir(x * 8);"), vec!["12"]);
    assert_eq!(out("decimal x = 3.5;\nimprimir(x * 2);"), vec!["7"]);
}

#[test]
fn bug106_entero_por_potencia_de_dos_sigue_bien() {
    assert_eq!(out("sea x = 3;\nimprimir(x * 2);"), vec!["6"]);
    assert_eq!(out("sea x = 3;\nimprimir(x * 4);"), vec!["12"]);
    assert_eq!(out("sea x = 5;\nimprimir(x * 8);"), vec!["40"]);
    assert_eq!(out("sea x = 2.5;\nimprimir(x * 3);"), vec!["7.5"]);
}

#[test]
fn bug106_decimal_por_dos_dentro_de_funcion() {
    assert_eq!(
        out("funcion decimal doble(decimal v) { retornar v * 2; }\nimprimir(doble(1.5));"),
        vec!["3"]
    );
}

#[test]
fn bug106_el_desplazamiento_explicito_sigue_funcionando() {
    assert_eq!(out("sea x = 3;\nimprimir(x << 1);"), vec!["6"]);
    assert_eq!(out("sea x = 16;\nimprimir(x >> 2);"), vec!["4"]);
}

// ── BUG-107: los errores de operador salían en inglés ───────────────────────

#[test]
fn bug107_errores_de_operador_en_espanol() {
    // Se llega al error en runtime a través de un valor dinámico (un mapa),
    // que es lo que esquiva la comprobación estática.
    let base = "sea m = __map_nuevo();\nm = __map_poner(m, \"l\", [1, 2]);\nsea v = __map_obtener(m, \"l\");\n";
    for (expr, operador) in [
        ("imprimir(v + 1);", "'+'"),
        ("imprimir(v - 1);", "'-'"),
        ("imprimir(v * 2);", "'*'"),
        ("imprimir(v / 2);", "'/'"),
    ] {
        let err = run_source(&format!("{base}{expr}")).expect_err("debería fallar en runtime");
        let err = format!("{err:?}");
        assert!(
            err.contains("El operador") && err.contains(operador),
            "el error debe nombrar el operador {operador} en español: {err}"
        );
        assert!(
            !err.contains("requires"),
            "el mensaje no debe quedar en inglés: {err}"
        );
    }
}

#[test]
fn bug107_el_error_nombra_el_operador_escrito_no_el_opcode() {
    // Antes de BUG-106 este caso decía «ShiftLeft requires integers» pese a que
    // en el código fuente no hay ningún `<<`.
    let src = "sea m = __map_nuevo();\nm = __map_poner(m, \"l\", [1, 2]);\nsea v = __map_obtener(m, \"l\");\nimprimir(v * 2);";
    let err = format!("{:?}", run_source(src).expect_err("debería fallar"));
    assert!(err.contains("'*'"), "debe hablar de '*': {err}");
    assert!(
        !err.contains("ShiftLeft"),
        "no debe mencionar ShiftLeft: {err}"
    );
}

// ── BUG-108: literales enteros fuera del rango de 64 bits ──────────────────

#[test]
fn bug108_el_entero_minimo_es_valido() {
    // -9223372036854775808 cabe en i64, pero su valor absoluto no: el signo es
    // un operador unario aparte, así que hay que reconocerlo como una unidad.
    assert_eq!(
        out("imprimir(-9223372036854775808);"),
        vec!["-9223372036854775808"]
    );
    assert_eq!(
        out("sea x = -9223372036854775808;\nimprimir(x);"),
        vec!["-9223372036854775808"]
    );
    assert_eq!(
        out("imprimir(9223372036854775807);"),
        vec!["9223372036854775807"]
    );
}

#[test]
fn bug108_un_literal_demasiado_grande_se_rechaza() {
    // Antes `unwrap_or(0)` lo convertía en 0 en silencio y el programa se
    // ejecutaba con un número que nadie escribió.
    // `run_source` traduce el fallo a un ParseError con el mensaje del
    // diagnóstico; el código E083 se comprueba a través de la CLI en
    // `herramientas_v3.rs`.
    let err = run_source("imprimir(9223372036854775808);")
        .expect_err("un literal fuera de rango debe rechazarse");
    let err = format!("{err:?}");
    assert!(
        err.contains("no cabe en un 'entero'"),
        "debe avisar del desbordamiento: {err}"
    );

    let err = format!(
        "{:?}",
        run_source("imprimir(-9223372036854775809);").expect_err("fuera de rango")
    );
    assert!(
        err.contains("no cabe en un 'entero'"),
        "debe avisar del desbordamiento: {err}"
    );
}

#[test]
fn bug108_la_resta_y_la_negacion_siguen_funcionando() {
    assert_eq!(out("imprimir(-42);"), vec!["-42"]);
    assert_eq!(out("sea a = 5;\nimprimir(-a);"), vec!["-5"]);
    assert_eq!(out("imprimir(10 - 3);"), vec!["7"]);
    assert_eq!(out("sea a = 5;\nsea b = 3;\nimprimir(a - -b);"), vec!["8"]);
    assert_eq!(out("imprimir(-1.5);"), vec!["-1.5"]);
}

// ── BUG-109: `i64::MIN / -1` hacía pánico de Rust ──────────────────────────

#[test]
fn bug109_division_del_minimo_entre_menos_uno_no_hace_panic() {
    // Desbordaba tanto al plegar constantes (al compilar) como en la VM.
    assert_eq!(
        out("imprimir(-9223372036854775808 / -1);"),
        vec!["-9223372036854775808"]
    );
    assert_eq!(
        out("sea x = -9223372036854775808;\nsea y = -1;\nimprimir(x / y);"),
        vec!["-9223372036854775808"]
    );
}

#[test]
fn bug109_modulo_del_minimo_entre_menos_uno_no_hace_panic() {
    assert_eq!(out("imprimir(-9223372036854775808 % -1);"), vec!["0"]);
    assert_eq!(
        out("sea x = -9223372036854775808;\nsea y = -1;\nimprimir(x % y);"),
        vec!["0"]
    );
}

#[test]
fn bug109_la_division_entera_normal_no_cambia() {
    assert_eq!(out("imprimir(7 / 2);"), vec!["3"]);
    assert_eq!(out("imprimir(-7 / 2);"), vec!["-3"]);
    assert_eq!(out("imprimir(7 % 3);"), vec!["1"]);
    assert_eq!(out("imprimir(-7 % 3);"), vec!["-1"]);
    // La división por cero sigue siendo un error, no un desbordamiento.
    assert!(run_source("imprimir(7 / 0);").is_err());
}

// ── BUG-110: desplazamientos fuera de rango en el backend nativo ───────────

#[test]
fn bug110_desplazamiento_fuera_de_rango_es_error() {
    // La VM ya lo validaba; el nativo hacía `x << 64` (comportamiento
    // indefinido en C) y devolvía basura. Ahora ambos dan el mismo error.
    assert!(run_source("sea a = 1;\nsea b = 64;\nimprimir(a << b);").is_err());
    assert!(run_source("sea a = 1;\nsea b = -1;\nimprimir(a << b);").is_err());
    assert!(run_source("sea a = 1;\nsea b = 64;\nimprimir(a >> b);").is_err());
}

#[test]
fn bug110_desplazamiento_valido_sigue_funcionando() {
    assert_eq!(out("imprimir(1 << 10);"), vec!["1024"]);
    assert_eq!(out("imprimir(1024 >> 3);"), vec!["128"]);
    assert_eq!(out("imprimir(5 << 0);"), vec!["5"]);
    assert_eq!(out("imprimir(1 << 63);"), vec!["-9223372036854775808"]);
}

// ── BUG-111: el `%` con dividendo negativo daba tres resultados distintos ──

#[test]
fn bug111_el_modulo_negativo_es_coherente() {
    // La VM usaba el resto euclídeo (siempre positivo) mientras el plegado de
    // constantes, el backend en C y el `%` de decimales usaban el resto
    // truncado: `-7 % 3` valía 2 por variables y -1 con literales.
    assert_eq!(out("sea a = -7;\nsea b = 3;\nimprimir(a % b);"), vec!["-1"]);
    assert_eq!(out("imprimir(-7 % 3);"), vec!["-1"]);
    assert_eq!(out("sea a = -7;\nimprimir(a % 3);"), vec!["-1"]);
}

#[test]
fn bug111_todas_las_combinaciones_de_signo() {
    assert_eq!(out("sea a = 7;\nsea b = 3;\nimprimir(a % b);"), vec!["1"]);
    assert_eq!(out("sea a = -7;\nsea b = 3;\nimprimir(a % b);"), vec!["-1"]);
    assert_eq!(out("sea a = 7;\nsea b = -3;\nimprimir(a % b);"), vec!["1"]);
    assert_eq!(
        out("sea a = -7;\nsea b = -3;\nimprimir(a % b);"),
        vec!["-1"]
    );
    assert_eq!(
        out("sea a = -10;\nsea b = 4;\nimprimir(a % b);"),
        vec!["-2"]
    );
}

// ── BUG-115 / BUG-116: formateo de decimales grandes y pequeños ────────────

#[test]
fn bug116_los_decimales_no_usan_notacion_cientifica() {
    // El binario nativo caía al `%g` de C para valores fuera de [1e-5, 1e16] y
    // pasaba a notación científica ("1e+18", "1e-06"); la VM nunca lo hace.
    assert_eq!(
        out("sea a = 1000000000000000000.0;\nimprimir(a);"),
        vec!["1000000000000000000"]
    );
    assert_eq!(out("sea a = 0.000001;\nimprimir(a);"), vec!["0.000001"]);
    assert_eq!(out("sea a = 1000000.0;\nimprimir(a);"), vec!["1000000"]);
    assert_eq!(
        out("sea a = 123456789.5;\nimprimir(a);"),
        vec!["123456789.5"]
    );
}

#[test]
fn bug116_un_decimal_mayor_que_i64_no_se_satura_al_imprimir() {
    // `*n as i64` satura en Rust: 2^63 se imprimía como 9223372036854775807
    // aunque su valor real es 9223372036854775808.
    assert_eq!(
        out("sea a = 9223372036854775807.0;\nimprimir(a);"),
        vec!["9223372036854775808"]
    );
    assert_eq!(
        out("sea a = 9223372036854775807.0;\nimprimir(a + 0.5);"),
        vec!["9223372036854775808"]
    );
}

#[test]
fn bug116_los_decimales_normales_no_cambian() {
    assert_eq!(out("imprimir(0.1 + 0.2);"), vec!["0.30000000000000004"]);
    assert_eq!(out("imprimir(1.0 / 3.0);"), vec!["0.3333333333333333"]);
    assert_eq!(out("imprimir(2.5);"), vec!["2.5"]);
    assert_eq!(out("imprimir(-0.0);"), vec!["0"]);
    assert_eq!(out("imprimir(3.0);"), vec!["3"]);
}

/// BUG-117: dieciocho mensajes de error del intérprete seguían en inglés en un
/// lenguaje cuyas palabras clave y diagnósticos son en español. No eran texto
/// muerto: cualquiera de ellos es alcanzable desde código de usuario, por
/// ejemplo pidiendo un canal o un actor que no existe.
#[test]
fn bug117_los_errores_de_concurrencia_estan_en_espanol() {
    for (fuente, esperado) in [
        (
            "imprimir(__canal_enviar(\"chan_99\", 5));",
            "Canal no encontrado",
        ),
        (
            "imprimir(__actor_enviar(\"actor_99\", 5));",
            "Actor no encontrado",
        ),
        (
            "imprimir(__generador_siguiente(\"gen_99\", 1));",
            "Generador no encontrado",
        ),
        ("imprimir(__tarea_esperar(\"t99\"));", "Tarea no encontrada"),
        ("imprimir(__hilo_esperar(\"h99\"));", "Hilo no encontrado"),
    ] {
        let salida = out(fuente).join("\n");
        assert!(
            salida.contains(esperado),
            "se esperaba «{esperado}», salió: {salida}"
        );
    }
}

/// BUG-130: un fichero guardado con BOM UTF-8 —lo que hacen por defecto el
/// Bloc de notas y varios editores de Windows— fallaba con «E001: Caracter
/// inesperado» en 1:1. El código era válido; el carácter culpable es invisible,
/// así que el mensaje no ayudaba en absoluto. El BOM sólo indica la
/// codificación y debe descartarse.
#[test]
fn bug130_se_acepta_un_fichero_con_bom_utf8() {
    let con_bom = "\u{FEFF}imprimir(1 + 1);";
    assert_eq!(out(con_bom), vec!["2"]);

    // También cuando lo primero es un comentario.
    let con_bom_y_comentario = "\u{FEFF}// nota\nimprimir(\"hola\");";
    assert_eq!(out(con_bom_y_comentario), vec!["hola"]);

    // Y sin BOM sigue funcionando igual.
    assert_eq!(out("imprimir(1 + 1);"), vec!["2"]);
}

/// BUG-131: `lumen fmt` reescribía un `impl` inherente (`impl C { ... }`, sin
/// rasgo) como `impl  para C`, que no es sintaxis válida. El propio formateador
/// detectaba que su salida no recompilaría y abandonaba, dejando el archivo sin
/// formatear con un aviso desconcertante sobre código perfectamente correcto.
/// BUG-132: además perdía el `prestado mut` del receptor, así que el método
/// pasaba a recibir el struct por valor y dejaba de mutarlo: el formateador
/// cambiaba en silencio lo que hacía el programa.
#[test]
fn bug131_132_fmt_conserva_impl_inherente_y_prestado_mut() {
    let src = "estructura C { v: entero }\n\
               impl C {\n\
               funcion vacio poner(prestado mut C self, entero n) { self.v = n; }\n\
               funcion entero leer2(prestado C self) { retornar self.v; }\n\
               }\n\
               C c = C { v: 0 };\n\
               c.poner(42);\n\
               imprimir(c.leer2());\n";

    let formateado = lumen_fmt::format_source(src).expect("el fuente es válido");

    assert!(
        formateado.contains("impl C {"),
        "BUG-131: el impl inherente debe seguir siendo `impl C`, se obtuvo:\n{formateado}"
    );
    assert!(
        !formateado.contains(" para "),
        "BUG-131: no debe aparecer `para` en un impl sin rasgo:\n{formateado}"
    );
    assert!(
        formateado.contains("prestado mut C self"),
        "BUG-132: el receptor debe conservar `prestado mut`:\n{formateado}"
    );

    // La prueba de fuego: el resultado recompila y hace exactamente lo mismo.
    assert_eq!(out(src), vec!["42"]);
    assert_eq!(
        out(&formateado),
        vec!["42"],
        "el formateo cambió la semántica"
    );

    // Y formatear dos veces no vuelve a cambiar nada.
    let otra_vez = lumen_fmt::format_source(&formateado).expect("la salida debe reparsear");
    assert_eq!(otra_vez, formateado, "fmt no es idempotente");
}
