// BUG-072: las funciones `test_*` nunca se ejecutaban y `lumen test` daba por
// buenas las suites que fallaban. Se comprueba de extremo a extremo, invocando
// el binario, porque el fallo estaba en el runner de la CLI.

use std::process::Command;

fn lumen() -> &'static str {
    env!("CARGO_BIN_EXE_lumen")
}

fn escribir(nombre: &str, contenido: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("lumen_test_runner");
    std::fs::create_dir_all(&dir).unwrap();
    let p = dir.join(nombre);
    std::fs::write(&p, contenido).unwrap();
    p
}

#[test]
fn bug072_una_prueba_que_falla_se_reporta_y_sale_con_codigo_1() {
    let f = escribir(
        "falla.nv",
        "importar \"testing.nv\";\n\
         funcion void test_que_falla() {\n\
             testing_afirmar_igual(2 + 2, 5);\n\
         }\n",
    );
    let salida = Command::new(lumen())
        .arg("test")
        .arg(&f)
        .output()
        .expect("no se pudo ejecutar lumen");
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(
        texto.contains("1 fallaron") || texto.contains("FALLÓ"),
        "no detectó el fallo:\n{texto}"
    );
    assert_eq!(
        salida.status.code(),
        Some(1),
        "debe salir con código 1 para que CI lo detecte:\n{texto}"
    );
}

#[test]
fn bug072_el_cuerpo_de_la_prueba_se_ejecuta_de_verdad() {
    // La huella se comprueba a través de una ASERCIÓN que sólo puede fallar si
    // el cuerpo corre: antes del arreglo la función ni siquiera se invocaba, se
    // reportaba ✓ OK y esta prueba habría pasado por la vía equivocada. (La VM
    // captura la salida del programa en un buffer, así que no vale con buscar
    // un `imprimir` en la stdout del proceso.)
    let f = escribir(
        "corre.nv",
        "importar \"testing.nv\";\n\
         funcion void test_con_efecto() {\n\
             lista<entero> l = [1];\n\
             agregar(l, 2);\n\
             testing_afirmar_igual(largo(l), 99);\n\
         }\n",
    );
    let salida = Command::new(lumen()).arg("test").arg(&f).output().unwrap();
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(
        texto.contains("FALLÓ") && texto.contains("!= 99"),
        "el cuerpo de la prueba no llegó a ejecutarse:\n{texto}"
    );
}

#[test]
fn bug072_una_suite_correcta_sigue_pasando_con_codigo_0() {
    let f = escribir(
        "pasa.nv",
        "importar \"testing.nv\";\n\
         funcion void test_suma() {\n\
             testing_afirmar_igual(2 + 2, 4);\n\
         }\n",
    );
    let salida = Command::new(lumen()).arg("test").arg(&f).output().unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout).to_string();
    assert!(texto.contains("1 pasaron"), "no pasó:\n{texto}");
    assert_eq!(salida.status.code(), Some(0));
}
