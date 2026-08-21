// Regresiones de las herramientas de la CLI encontradas al probarlas con casos
// reales de uso (v3.0):
//
//   BUG-073 `lumen bundle <src> <destino>` ignoraba el destino: dejaba el
//           binario junto al fuente pero anunciaba la ruta pedida, con tamaño
//           incluido, así que parecía haber funcionado.
//   BUG-074 `lumen lint` era un stub: imprimía "0 advertencias" para cualquier
//           entrada, incluidos archivos inexistentes o basura sintáctica.
//   BUG-076 `lumen fuzz` no ejecutaba nada: reportaba cifras fijas y declaraba
//           "100% seguro" un programa que aborta por división por cero.
//
// Son pruebas de extremo a extremo porque los tres fallos estaban en la CLI.

use std::process::Command;

fn lumen() -> &'static str {
    env!("CARGO_BIN_EXE_lumen")
}

fn dir_tmp(sub: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("lumen_tools_v3").join(sub);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// BUG-164: en Windows el enlazador anade `.exe`. Los tests que pedian un
/// binario por su nombre a secas no lo encontraban y culpaban al bundler.
fn exe_path(p: &std::path::Path) -> std::path::PathBuf {
    if cfg!(windows) && p.extension().is_none() {
        p.with_extension("exe")
    } else {
        p.to_path_buf()
    }
}

/// BUG-162: la URI se construia como `file://` + la ruta del sistema. En
/// Windows eso da `file://C:\\dir\\a.nv`, que NO es una URI valida: el host
/// queda como "C:" y las barras invertidas no se escapan. El LSP no encontraba
/// el fichero y no emitia diagnostico alguno, y el test culpaba al servidor.
fn uri_de(p: &std::path::Path) -> String {
    let s = p.display().to_string().replace('\\', "/");
    if cfg!(windows) {
        format!("file:///{}", s)
    } else {
        format!("file://{}", s)
    }
}

fn escribir(dir: &std::path::Path, nombre: &str, contenido: &str) -> std::path::PathBuf {
    let p = dir.join(nombre);
    std::fs::write(&p, contenido).unwrap();
    p
}

// ─────────────────────────── BUG-074: lint ───────────────────────────

#[test]
fn bug074_lint_falla_ante_codigo_con_errores_de_sintaxis() {
    let dir = dir_tmp("lint_malo");
    let f = escribir(&dir, "malo.nv", "esto no es lumen valido {{{ !!!\n");

    let out = Command::new(lumen()).arg("lint").arg(&f).output().unwrap();

    assert!(
        !out.status.success(),
        "lint debe salir con código != 0 ante errores de sintaxis; salió {:?}",
        out.status.code()
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("Error sintáctico"),
        "lint debe reportar el error de sintaxis. stderr: {}",
        err
    );
}

#[test]
fn bug074_lint_falla_si_el_archivo_no_existe() {
    let out = Command::new(lumen())
        .arg("lint")
        .arg("/no/existe/jamas_de_los_jamases.nv")
        .output()
        .unwrap();

    assert!(
        !out.status.success(),
        "lint sobre un archivo inexistente no puede reportar éxito"
    );
}

#[test]
fn bug074_lint_acepta_codigo_valido_y_detecta_avisos_de_estilo() {
    let dir = dir_tmp("lint_bueno");

    let limpio = escribir(
        &dir,
        "limpio.nv",
        "funcion void principal() {\n    imprimir(\"ok\");\n}\n",
    );
    let out = Command::new(lumen())
        .arg("lint")
        .arg(&limpio)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "lint debe aceptar código válido. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("0 advertencias"));

    // Tabulador y espacios finales: válido pero con avisos de estilo.
    let sucio = escribir(
        &dir,
        "sucio.nv",
        "funcion void principal() {\n\timprimir(\"ok\");   \n}\n",
    );
    let out = Command::new(lumen())
        .arg("lint")
        .arg(&sucio)
        .output()
        .unwrap();
    assert!(out.status.success(), "los avisos de estilo no deben fallar");
    let salida = String::from_utf8_lossy(&out.stdout);
    assert!(
        salida.contains("advertencia"),
        "se esperaban avisos de estilo. stdout: {}",
        salida
    );
}

// ─────────────────────────── BUG-076: fuzz ───────────────────────────

#[test]
fn bug076_fuzz_detecta_una_division_por_cero_real() {
    let dir = dir_tmp("fuzz_crash");
    let f = escribir(
        &dir,
        "crash.nv",
        "funcion entero div(entero a, entero b) { retornar a / b; }\n\
         funcion void principal() { imprimir(div(10, 0)); }\n",
    );

    let out = Command::new(lumen()).arg("fuzz").arg(&f).output().unwrap();

    assert!(
        !out.status.success(),
        "fuzz no puede declarar seguro un programa que aborta"
    );
    let salida = String::from_utf8_lossy(&out.stdout);
    assert!(
        salida.contains("División por cero"),
        "fuzz debe reportar el fallo concreto. stdout: {}",
        salida
    );
    assert!(
        !salida.contains("100% seguro"),
        "fuzz no debe afirmar seguridad con fallos detectados"
    );
}

#[test]
fn bug076_fuzz_ejecuta_mutaciones_de_verdad_en_codigo_sano() {
    let dir = dir_tmp("fuzz_sano");
    let f = escribir(
        &dir,
        "sano.nv",
        "funcion entero suma_hasta(entero n) {\n\
         \x20   entero s = 0;\n\
         \x20   para i en 1..=n { s = s + i; }\n\
         \x20   retornar s;\n\
         }\n\
         funcion void principal() { imprimir(suma_hasta(10)); }\n",
    );

    let out = Command::new(lumen()).arg("fuzz").arg(&f).output().unwrap();

    assert!(
        out.status.success(),
        "código sano no debe reportar fallos. stdout: {}",
        String::from_utf8_lossy(&out.stdout)
    );
    let salida = String::from_utf8_lossy(&out.stdout);
    // Antes se imprimía siempre "5000"; ahora la cuenta depende del programa.
    assert!(
        salida.contains("Ejecutadas en la VM"),
        "debe informar de las ejecuciones reales. stdout: {}",
        salida
    );
    let ejecutadas = salida
        .lines()
        .find(|l| l.contains("Ejecutadas en la VM"))
        .and_then(|l| l.rsplit(':').next().map(|s| s.trim().to_string()))
        .unwrap_or_default()
        .parse::<usize>()
        .unwrap_or(0);
    assert!(
        ejecutadas > 0,
        "el fuzzer debe ejecutar al menos una mutación, ejecutó {}",
        ejecutadas
    );
}

// ─────────────────────────── BUG-073: bundle ───────────────────────────
//
// `bundle` invoca al compilador de C del sistema; si no está disponible en el
// entorno de CI la prueba se salta en vez de fallar por una causa ajena.

fn hay_compilador_c() -> bool {
    Command::new("cc")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
fn bug073_bundle_respeta_la_ruta_de_salida_indicada() {
    if !hay_compilador_c() {
        eprintln!("aviso: sin compilador C disponible, se omite la prueba de bundle");
        return;
    }

    let dir = dir_tmp("bundle");
    let src = escribir(
        &dir,
        "app.nv",
        "funcion void principal() {\n    imprimir(\"bundle-ok\");\n}\n",
    );
    let destino = dir.join("salida").join("mi_binario");

    let out = Command::new(lumen())
        .arg("bundle")
        .arg(&src)
        .arg(&destino)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "bundle falló. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let destino = exe_path(&destino);
    assert!(
        destino.is_file(),
        "el binario debe existir en la ruta pedida ({}), no junto al fuente",
        destino.display()
    );

    // Y debe ser un ejecutable que realmente funciona.
    let run = Command::new(&destino).output().unwrap();
    assert!(run.status.success(), "el binario generado no se ejecuta");
    assert!(
        String::from_utf8_lossy(&run.stdout).contains("bundle-ok"),
        "salida inesperada del binario: {}",
        String::from_utf8_lossy(&run.stdout)
    );

    // No debe dejar el binario junto al fuente (comportamiento antiguo).
    assert!(
        !exe_path(&dir.join("app")).is_file(),
        "bundle no debe escribir el binario junto al fuente cuando se indica destino"
    );
}

// ─────────────────────── BUG-075: backend Cranelift ───────────────────────
//
// La pila de operandos del backend Cranelift guardaba valores SSA crudos que se
// consumían tras un cambio de bloque, así que el verificador rechazaba la
// función con "uses value vN from non-dominating inst" y el compilador
// PANICABA ("aot define_function fallo"). Pasaba con cualquier `elegir` sobre
// el resultado de una llamada. Antes del arreglo: 49 de 172 ejemplos panicaban.

#[test]
fn bug075_cranelift_no_panica_con_elegir_sobre_una_llamada() {
    if !hay_compilador_c() {
        eprintln!("aviso: sin toolchain nativa disponible, se omite la prueba de Cranelift");
        return;
    }

    let dir = dir_tmp("cranelift");
    let src = escribir(
        &dir,
        "opt.nv",
        "estructura C { nombre: texto, }\n\
         funcion opcion<C> buscar(texto n) {\n\
         \x20   si (n == \"x\") { retornar algun(C{nombre: n}); }\n\
         \x20   retornar ninguno;\n\
         }\n\
         elegir (buscar(\"Zoe\")) {\n\
         \x20   caso algun(c):\n\
         \x20       imprimir(\"encontrado: \", c.nombre);\n\
         \x20   caso ninguno:\n\
         \x20       imprimir(\"no está\");\n\
         }\n",
    );

    let out = Command::new(lumen())
        .arg("build")
        .arg("--native")
        .arg("--aot")
        .arg("rust")
        .arg(&src)
        .output()
        .unwrap();

    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("define_function"),
        "el backend Cranelift volvió a rechazar la función: {}",
        err
    );
    assert!(!err.contains("panicked"), "el compilador panicó: {}", err);
}

// ─────────────────────── BUG-077 / BUG-078: bindgen + FFI ───────────────────────
//
// BUG-077 `lumen bindgen` generaba un módulo que NO COMPILA: declaraba
//         `entero _lib_handle = __ffi_cargar(...)` cuando ese builtin devuelve
//         `texto` (E031), y fijaba el retorno a "entero" ignorando la cabecera.
// BUG-078 el binding omitía la cadena de tipos de `__ffi_llamar`, así que los
//         argumentos se desplazaban y la VM PANICABA con "index out of bounds".

#[test]
fn bug077_bindgen_genera_un_modulo_que_compila() {
    let dir = dir_tmp("bindgen");
    let h = escribir(
        &dir,
        "mini.h",
        "int suma(int a, int b);\ndouble media(double* xs, int n);\nvoid reset(void);\n",
    );

    let out = Command::new(lumen())
        .arg("bindgen")
        .arg(&h)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "bindgen falló: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let generado = dir.join("mini_bindings.nv");
    assert!(generado.is_file(), "no se generó el módulo de bindings");

    let texto = std::fs::read_to_string(&generado).unwrap();
    assert!(
        texto.contains("texto _lib_handle"),
        "el handle de __ffi_cargar es 'texto', no 'entero': {}",
        texto
    );
    // El retorno debe seguir a la cabecera, no ser siempre "entero".
    assert!(
        texto.contains("\"decimal\""),
        "media() devuelve double → decimal. Generado:\n{}",
        texto
    );
    // BUG-078: la cadena de tipos debe estar presente.
    assert!(
        texto.contains("\"entero,entero\""),
        "falta la cadena de tipos en __ffi_llamar. Generado:\n{}",
        texto
    );

    // Y sobre todo: el módulo generado debe pasar `lumen check`.
    let chk = Command::new(lumen())
        .arg("check")
        .arg(&generado)
        .output()
        .unwrap();
    assert!(
        chk.status.success(),
        "el módulo generado por bindgen no compila:\n{}{}",
        String::from_utf8_lossy(&chk.stdout),
        String::from_utf8_lossy(&chk.stderr)
    );
}

#[test]
fn bug078_ffi_con_mas_tipos_que_argumentos_no_panica() {
    let dir = dir_tmp("ffi_panic");
    // No hace falta que la biblioteca exista: el fallo saltaba al despachar los
    // argumentos, antes de resolver el símbolo.
    let f = escribir(
        &dir,
        "malffi.nv",
        "texto h = __ffi_cargar(\"/no/existe/lib.so\");\n\
         imprimir(__ffi_llamar(h, \"suma\", \"entero,entero\", [], \"entero\"));\n",
    );

    let out = Command::new(lumen()).arg("run").arg(&f).output().unwrap();
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("panicked"),
        "la VM no debe panicar ante una llamada FFI mal formada: {}",
        err
    );
    assert!(
        !err.contains("index out of bounds"),
        "regresión de BUG-078: {}",
        err
    );
}

// ─────────────────────── BUG-079: doble liberar en FFI ───────────────────────
//
// `__ffi_liberar(p)` dos veces sobre el mismo puntero —o sobre uno cualquiera—
// llamaba a `dealloc` con el layout que diese el usuario aunque el puntero no
// estuviera registrado. El doble free **abortaba el proceso** (SIGABRT): un
// error del programa mataba la VM sin error recuperable ni traza.

#[test]
fn bug079_doble_liberar_no_mata_la_vm() {
    let dir = dir_tmp("ffi_double_free");
    let f = escribir(
        &dir,
        "doble.nv",
        "entero p = __ffi_asignar(8);\n\
         __ffi_liberar(p);\n\
         __ffi_liberar(p);\n\
         imprimir(\"sobrevivio\");\n",
    );

    let out = Command::new(lumen()).arg("run").arg(&f).output().unwrap();

    // SIGABRT se ve como código 134 (o None si lo mató la señal).
    assert_ne!(
        out.status.code(),
        Some(134),
        "la VM abortó por doble free: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        !err.contains("double free"),
        "regresión de BUG-079: {}",
        err
    );
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("sobrevivio"),
        "el programa debe continuar tras el segundo liberar"
    );
}

#[test]
fn bug079_el_ciclo_ffi_legitimo_sigue_funcionando() {
    let dir = dir_tmp("ffi_ok");
    let f = escribir(
        &dir,
        "ok.nv",
        "entero p = __ffi_asignar(64);\n\
         __ffi_escribir(p, 0, \"hola\");\n\
         __ffi_liberar(p);\n\
         entero q = __ffi_asignar(32);\n\
         __ffi_liberar(q);\n\
         imprimir(\"ciclo ok\");\n",
    );

    let out = Command::new(lumen()).arg("run").arg(&f).output().unwrap();
    assert!(
        out.status.success() && String::from_utf8_lossy(&out.stdout).contains("ciclo ok"),
        "reservar/escribir/liberar debe seguir funcionando. stdout: {} stderr: {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

// ─────────────────── BUG-080..083: paridad del backend nativo ───────────────────
//
// BUG-080 regex: el backend C usa POSIX, que no entiende \d \w \s (sintaxis Perl
//         que sí acepta la VM): regcomp fallaba y todo daba `false`.
// BUG-081 corrutinas: la pila de operandos y el control de profundidad eran
//         globales compartidos entre hilos; el binario abortaba con
//         "pila agotada" al reanudar una corrutina.
// BUG-082 __str_subcadena cortaba por BYTES y no por caracteres: con acentos
//         partía un carácter por la mitad y emitía UTF-8 inválido.
// BUG-083 el paso de argumentos clonaba en profundidad sin liberar: acumular en
//         una lista dentro de un bucle era O(n^2) en memoria y moría por OOM.

/// BUG-162: en Windows el runtime C abre stdout en modo texto y traduce cada
/// `\n` a `\r\n`, mientras que la VM (que escribe por Rust) emite `\n` a secas.
/// Los tests de paridad comparaban el stdout crudo, asi que en Windows fallaban
/// SIETE por una diferencia que no es del lenguaje sino del sistema. Comparar
/// bytes de fin de linea entre dos backends no prueba nada util: se normaliza.
fn normalizar(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn nativo(dir: &std::path::Path, nombre: &str, fuente: &str) -> Option<String> {
    if !hay_compilador_c() {
        return None;
    }
    let src = escribir(dir, nombre, fuente);
    let out = Command::new(lumen())
        .arg("build")
        .arg("--native")
        .arg(&src)
        .current_dir(dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "no compiló a nativo: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let exe = src.with_extension("");
    let run = Command::new(&exe).output().unwrap();
    Some(normalizar(&String::from_utf8_lossy(&run.stdout)))
}

fn en_vm(dir: &std::path::Path, nombre: &str, fuente: &str) -> String {
    let src = escribir(dir, nombre, fuente);
    let out = Command::new(lumen()).arg("run").arg(&src).output().unwrap();
    normalizar(&String::from_utf8_lossy(&out.stdout))
}

#[test]
fn bug080_regex_perl_coincide_igual_en_vm_y_nativo() {
    let dir = dir_tmp("regex");
    let fuente = "imprimir(__regex_coincide(\"\\\\d+\", \"abc123\"));\n\
                  imprimir(__regex_coincide(\"\\\\w+\", \"hola\"));\n\
                  imprimir(__regex_coincide(\"\\\\d+\", \"sin numeros\"));\n";
    let vm = en_vm(&dir, "re_vm.nv", fuente);
    assert!(
        vm.contains("true"),
        "la VM debería casar \\d+ con abc123: {}",
        vm
    );
    if let Some(nat) = nativo(&dir, "re_nat.nv", fuente) {
        assert_eq!(vm, nat, "regex diverge entre VM y binario nativo");
    }
}

/// BUG-166: en Windows y macOS el `#else` de la guarda de regex tenia stubs que
/// devolvian siempre "no coincide", asi que el binario nativo contestaba
/// `false` a todo mientras la VM contestaba `true`. Se cubre aqui el mismo
/// juego de patrones que uso el fallo original, mas los cuantificadores
/// `{n,m}` que el motor nuevo tuvo que aprender para igualar a la VM.
#[test]
fn bug166_regex_completo_coincide_en_vm_y_nativo() {
    let dir = dir_tmp("regex166");
    let fuente = concat!(
        "imprimir(__regex_coincide(\"^abc\", \"abcdef\"));\n",
        "imprimir(__regex_coincide(\"abc$\", \"abcx\"));\n",
        "imprimir(__regex_coincide(\"[a-z]+@[a-z]+\", \"mail@web\"));\n",
        "imprimir(__regex_coincide(\"gato|perro\", \"un perro\"));\n",
        "imprimir(__regex_coincide(\"(ab)+c\", \"ababc\"));\n",
        "imprimir(__regex_coincide(\"colou?r\", \"colour\"));\n",
        "imprimir(__regex_coincide(\"^\\\\d{3}$\", \"123\"));\n",
        "imprimir(__regex_coincide(\"^\\\\d{3}$\", \"12\"));\n",
        "imprimir(__regex_coincide(\"a{2,3}b\", \"aab\"));\n",
        "imprimir(__regex_coincide(\"[^0-9]+\", \"4242\"));\n",
    );
    let vm = en_vm(&dir, "r166_vm.nv", fuente);
    assert!(
        vm.contains("true") && vm.contains("false"),
        "la VM debe distinguir coincidencias de no coincidencias: {vm}"
    );
    if let Some(nat) = nativo(&dir, "r166_nat.nv", fuente) {
        assert_eq!(
            vm, nat,
            "el regex nativo no coincide con la VM (¿stub de plataforma?)"
        );
    }
}

/// BUG-167: `__regex_reemplazar` con un patron que puede casar la cadena vacia
/// (`[a-z]?|a`) avanzaba el puntero sin comprobar el terminador y el binario
/// nativo moria con SIGSEGV. Ademas la coincidencia vacia pegada al final de
/// otra no debe sustituirse, para que "a?" sobre "bab" de "#b#b#".
#[test]
fn bug167_reemplazo_con_coincidencia_vacia_no_desborda() {
    let dir = dir_tmp("regex167");
    let fuente = concat!(
        "imprimir(__regex_reemplazar(\"[a-z]?|a\", \"x_y\", \"#\"));\n",
        "imprimir(__regex_reemplazar(\"a?\", \"bab\", \"#\"));\n",
        "imprimir(__regex_reemplazar(\"a*\", \"bab\", \"#\"));\n",
        "imprimir(__regex_reemplazar(\"\", \"abc\", \"#\"));\n",
        "imprimir(__regex_reemplazar(\"x?\", \"\", \"#\"));\n",
        "imprimir(__regex_reemplazar(\"\\\\d*\", \"a1b\", \"#\"));\n",
        "imprimir(__regex_reemplazar(\"\\\\d+\", \"a1b22c\", \"#\"));\n",
    );
    let vm = en_vm(&dir, "r167_vm.nv", fuente);
    assert!(
        vm.contains("#_#"),
        "la VM debe sustituir sin colgarse: {vm}"
    );
    if let Some(nat) = nativo(&dir, "r167_nat.nv", fuente) {
        assert_eq!(
            vm, nat,
            "el reemplazo con coincidencia vacia diverge entre VM y nativo"
        );
    }
}

#[test]
fn bug082_subcadena_corta_por_caracteres_no_por_bytes() {
    let dir = dir_tmp("utf8");
    let fuente = "texto s = \"áéíóú-ñÁÉ\";\n\
                  imprimir(largo(s));\n\
                  imprimir(__str_subcadena(s, 0, 5));\n\
                  imprimir(__str_subcadena(s, 0, 8));\n";
    let vm = en_vm(&dir, "u_vm.nv", fuente);
    assert!(
        vm.contains("áéíóú"),
        "la VM debe cortar por caracteres: {}",
        vm
    );
    if let Some(nat) = nativo(&dir, "u_nat.nv", fuente) {
        assert_eq!(vm, nat, "el corte UTF-8 diverge entre VM y nativo");
        assert!(
            !nat.contains('\u{fffd}'),
            "el binario emitió UTF-8 inválido: {}",
            nat
        );
    }
}

#[test]
fn bug083_la_semantica_de_valor_se_conserva_con_copy_on_write() {
    let dir = dir_tmp("cow");
    // El copy-on-write que arregla el consumo de memoria NO debe dejar que una
    // mutación se filtre al original, ni siquiera en structs anidados.
    let fuente = "estructura Interior { v: entero, }\n\
                  estructura Exterior { dentro: Interior, }\n\
                  funcion entero toca(Exterior e, entero d) {\n\
                  \x20   e.dentro.v = e.dentro.v + d;\n\
                  \x20   retornar e.dentro.v;\n\
                  }\n\
                  Exterior e = Exterior { dentro: Interior { v: 10 } };\n\
                  entero r = toca(e, 11);\n\
                  imprimir(r);\n\
                  imprimir(e.dentro.v);\n\
                  lista<entero> d = [5,6,7];\n\
                  lista<entero> f = d;\n\
                  f[0] = 100;\n\
                  imprimir(d[0]);\n\
                  imprimir(f[0]);\n";
    let vm = en_vm(&dir, "c_vm.nv", fuente);
    // 21 (retorno), 10 (original intacto), 5 (original), 100 (alias mutado)
    assert!(
        vm.contains("21") && vm.contains("10") && vm.contains("100"),
        "la VM no conserva la semántica esperada: {}",
        vm
    );
    if let Some(nat) = nativo(&dir, "c_nat.nv", fuente) {
        assert_eq!(
            vm, nat,
            "una mutación se filtró al original en el binario nativo (regresión de COW)"
        );
    }
}

#[test]
fn bug083_acumular_en_una_lista_no_agota_la_memoria() {
    let dir = dir_tmp("cow_mem");
    // Antes esto crecía O(n^2): con 800 elementos lo mataba el OOM killer.
    let fuente = "estructura P { a: entero, b: entero, }\n\
                  funcion lista<P> mete(lista<P> l, entero i) {\n\
                  \x20   agregar(l, P{a: i, b: i});\n\
                  \x20   retornar l;\n\
                  }\n\
                  lista<P> l = [];\n\
                  para i en 1..=800 { l = mete(l, i); }\n\
                  imprimir(largo(l));\n";
    if let Some(nat) = nativo(&dir, "m_nat.nv", fuente) {
        assert!(
            nat.contains("800"),
            "el binario debe completar el bucle (antes moría por OOM): {}",
            nat
        );
    }
}

// ---------------------------------------------------------------------------
// BUG-084: el backend Cranelift (`--aot rust`) no implementa varios builtins
// (`largo`, `agregar`, `a_texto`, mapas, `leer`...) y emitía `iconst 0` en
// SILENCIO, así que el binario devolvía resultados falsos sin ningún aviso.
// Ahora debe fallar la compilación indicando qué builtins faltan.
// ---------------------------------------------------------------------------
#[test]
fn bug084_cranelift_no_genera_binarios_que_mienten() {
    let dir = dir_tmp("cranelift_no_soportado");
    let src = escribir(&dir, "m.nv", "sea l = [1, 2, 3];\nimprimir(largo(l));\n");

    let out = Command::new(lumen())
        .args(["build", "--native", "--aot", "rust"])
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        !out.status.success(),
        "compilar con builtins no soportados debe FALLAR, no emitir un 0 silencioso"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("sin soporte") && err.contains("largo"),
        "debe nombrar el builtin que falta; stderr={err}"
    );
}

#[test]
fn bug084_cranelift_sigue_compilando_lo_que_si_soporta() {
    let dir = dir_tmp("cranelift_soportado");
    let src = escribir(
        &dir,
        "ok.nv",
        "funcion entero suma(entero a, entero b) { retornar a + b; }\nimprimir(suma(20, 22));\n",
    );

    let out = Command::new(lumen())
        .args(["build", "--native", "--aot", "rust"])
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "un programa soportado debe compilar");

    let exe = exe_path(&dir.join("ok"));
    assert!(exe.exists(), "debe generarse el binario");
    let run = std::process::Command::new(&exe).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&run.stdout).trim(), "42");
}

// ---------------------------------------------------------------------------
// BUG-085: `--permitir-no-soportados` se consultaba con env::args() pero no
// estaba en el parser, así que caía en el catch-all de `dest` y se usaba como
// NOMBRE del binario de salida ("✓ Binario nativo: --permitir-no-soportados").
// ---------------------------------------------------------------------------
#[test]
fn bug085_la_bandera_permitir_no_soportados_no_es_el_nombre_de_salida() {
    let dir = dir_tmp("cranelift_bandera");
    let src = escribir(&dir, "m.nv", "sea l = [1, 2, 3];\nimprimir(largo(l));\n");

    let out = Command::new(lumen())
        .args(["build", "--native", "--aot", "rust"])
        .arg(&src)
        .arg("--permitir-no-soportados")
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        out.status.success(),
        "con la bandera debe compilar igualmente"
    );
    assert!(
        exe_path(&dir.join("m")).exists(),
        "el binario debe llamarse 'm', no como la bandera"
    );
    assert!(
        !dir.join("--permitir-no-soportados").exists(),
        "la bandera nunca debe tomarse como nombre de salida"
    );
}

// ---------------------------------------------------------------------------
// ---------------------------------------------------------------------------
// BUG-087: `__str_codigo` devolvía un elemento POR BYTE en el binario nativo
// ("añb" → [97,195,177,98]) y puntos de código en la VM ([97,241,98]).
// ---------------------------------------------------------------------------
#[test]
fn bug087_los_codigos_de_caracter_son_puntos_de_codigo_no_bytes() {
    let dir = dir_tmp("codigos_utf8");
    let fuente = "imprimir(__str_codigo(\"añb\"));\n";
    assert_eq!(en_vm(&dir, "c_vm.nv", fuente).trim(), "[97, 241, 98]");
    if let Some(nat) = nativo(&dir, "c_nat.nv", fuente) {
        assert_eq!(
            nat.trim(),
            "[97, 241, 98]",
            "el nativo devolvía bytes crudos"
        );
    }
}

// ---------------------------------------------------------------------------
// BUG-088: el orden de `__map_claves` difería entre VM (hash) y nativo
// (inserción): el mismo programa imprimía las claves en distinto orden.
// ---------------------------------------------------------------------------
#[test]
fn bug088_las_claves_de_un_mapa_salen_en_el_mismo_orden_en_ambos_backends() {
    let dir = dir_tmp("orden_mapa");
    let fuente = "sea m = __map_nuevo();\n\
                  m = __map_poner(m, \"zeta\", 1);\n\
                  m = __map_poner(m, \"alfa\", 2);\n\
                  m = __map_poner(m, \"medio\", 3);\n\
                  imprimir(__map_claves(m));\n";
    let vm = en_vm(&dir, "m_vm.nv", fuente);
    assert_eq!(vm.trim(), "[alfa, medio, zeta]");
    if let Some(nat) = nativo(&dir, "m_nat.nv", fuente) {
        assert_eq!(nat.trim(), vm.trim(), "el orden debe coincidir");
    }
}

#[test]
fn bug088_las_claves_numericas_se_ordenan_por_valor_no_como_texto() {
    let dir = dir_tmp("orden_mapa_num");
    let fuente = "sea m = __map_nuevo();\n\
                  m = __map_poner(m, 10, \"a\");\n\
                  m = __map_poner(m, 2, \"b\");\n\
                  m = __map_poner(m, 33, \"c\");\n\
                  imprimir(__map_claves(m));\n";
    // Ordenado como texto daría [10, 2, 33].
    let vm = en_vm(&dir, "mn_vm.nv", fuente);
    assert_eq!(vm.trim(), "[2, 10, 33]");
    if let Some(nat) = nativo(&dir, "mn_nat.nv", fuente) {
        assert_eq!(nat.trim(), vm.trim());
    }
}

// ---------------------------------------------------------------------------
// BUG-089: `1.0 / 0.0` daba `inf` en el binario nativo y «División por cero»
// en la VM, así que el mismo programa terminaba bien compilado y fallaba
// interpretado.
// ---------------------------------------------------------------------------
#[test]
fn bug089_la_division_decimal_por_cero_falla_igual_en_vm_y_nativo() {
    let dir = dir_tmp("div_cero_decimal");
    let fuente = "sea a = 1.0;\nsea b = 0.0;\nimprimir(a / b);\n";
    let src = escribir(&dir, "d.nv", fuente);

    let vm = Command::new(lumen()).arg("run").arg(&src).output().unwrap();
    let salida_vm =
        String::from_utf8_lossy(&vm.stderr).to_string() + &String::from_utf8_lossy(&vm.stdout);
    assert!(
        salida_vm.contains("División por cero"),
        "la VM debe rechazarlo; salida={salida_vm}"
    );

    if !hay_compilador_c() {
        return;
    }
    let b = Command::new(lumen())
        .args(["build", "--native"])
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(b.status.success(), "debe compilar");
    let r = Command::new(src.with_extension("")).output().unwrap();
    let salida_nat =
        String::from_utf8_lossy(&r.stderr).to_string() + &String::from_utf8_lossy(&r.stdout);
    assert!(
        salida_nat.contains("División por cero"),
        "el nativo imprimía 'inf'; salida={salida_nat}"
    );
    assert!(!salida_nat.contains("inf"), "no debe colarse un inf");
}

// ---------------------------------------------------------------------------
// BUG-095: el `match` de instrucciones del backend Cranelift terminaba en un
// `_ => {}` mudo, así que las instrucciones que ese backend NO implementa
// —`intentar`/`atrapar` (Push/PopHandler) y el emparejado de patrones
// (MatchType/MatchPayload)— se compilaban sin una sola advertencia. Un
// `intentar { 10/0 } atrapar { -1 }` devolvía 10 compilado y -1 en la VM.
// Es el agujero de BUG-084, por la vía de las instrucciones sueltas.
// ---------------------------------------------------------------------------
#[test]
fn bug095_cranelift_rechaza_intentar_atrapar_en_vez_de_mentir() {
    let dir = dir_tmp("cranelift_intentar");
    let fuente = "funcion entero f() { intentar { retornar 10 / 0; } \
                  atrapar (e) { retornar -1; } }\nimprimir(f());\n";
    let src = escribir(&dir, "t.nv", fuente);

    let out = Command::new(lumen())
        .args(["build", "--native", "--aot", "rust"])
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        !out.status.success(),
        "debe fallar: este backend no implementa intentar/atrapar"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("intentar/atrapar"),
        "debe nombrar la construcción que falta; stderr={err}"
    );
}

#[test]
fn bug095_cranelift_rechaza_elegir_sobre_variantes() {
    let dir = dir_tmp("cranelift_elegir");
    let fuente = "funcion opcion<entero> f() { retornar algun(5); }\n\
                  elegir (f()) { caso algun(v): imprimir(v); \
                  caso ninguno: imprimir(\"n\"); }\n";
    let src = escribir(&dir, "e.nv", fuente);

    let out = Command::new(lumen())
        .args(["build", "--native", "--aot", "rust"])
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(!out.status.success(), "debe rechazarse, no emitir un 0");
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("elegir") || err.contains("patron"),
        "debe nombrar lo que falta; stderr={err}"
    );
}

// ---------------------------------------------------------------------------
// BUG-096: el backend LLVM implementa 12 de los 42 opcodes. El resto
// desaparecía del IR sin dejar rastro, y las llamadas a funciones inexistentes
// se emitían igualmente: `largo(l)` generaba `call i64 @largo(...)` sin ningún
// `declare`, es decir LLVM IR **inválido** que no pasa el verificador ni
// enlaza. La CLI lo anunciaba con «✓ Archivo LLVM IR generado».
// ---------------------------------------------------------------------------
#[test]
fn bug096_llvm_no_anuncia_un_ir_invalido_como_exito() {
    let dir = dir_tmp("llvm_ir_invalido");
    let src = escribir(&dir, "l.nv", "sea l = [1, 2, 3];\nimprimir(largo(l));\n");

    let out = Command::new(lumen())
        .args(["build", "--native", "--aot", "llvm"])
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();

    assert!(
        !out.status.success(),
        "debe fallar: el IR resultante no enlazaría"
    );
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("sin soporte"),
        "debe enumerar lo que falta; stderr={err}"
    );
}

#[test]
fn bug096_el_ir_que_si_genera_no_tiene_llamadas_colgantes() {
    // Un programa puramente escalar sí está cubierto: el IR debe ser coherente,
    // sin llamar a nada que no esté declarado o definido en el propio módulo.
    let dir = dir_tmp("llvm_ir_coherente");
    let src = escribir(
        &dir,
        "s.nv",
        "funcion entero suma(entero a, entero b) { retornar a + b; }\n\
         funcion entero principal() { retornar suma(20, 22); }\n",
    );

    let out = Command::new(lumen())
        .args(["build", "--native", "--aot", "llvm"])
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "un programa escalar debe generar IR: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let ll = std::fs::read_to_string(src.with_extension("ll")).unwrap();
    let llamadas: std::collections::HashSet<&str> = ll
        .lines()
        .filter_map(|l| l.split("call i64 @").nth(1))
        .filter_map(|r| r.split('(').next())
        .collect();
    for f in llamadas {
        assert!(
            ll.contains(&format!("declare i64 @{f}(")) || ll.contains(&format!("define i64 @{f}(")),
            "se llama a @{f} sin declararla: IR inválido"
        );
    }
}

/// BUG-104: un mismo error de runtime debe producir el MISMO código de salida
/// en la VM y en el binario nativo. Históricamente la VM salía con 1 y el
/// nativo con 3 para división por cero, índice fuera de rango y campo
/// inexistente, así que un script de CI veía dos lenguajes distintos según el
/// backend.
#[test]
fn bug104_codigo_de_salida_igual_en_vm_y_nativo() {
    if !hay_compilador_c() {
        return;
    }
    let dir = dir_tmp("bug104_rc");
    let casos: [(&str, &str); 3] = [
        ("divzero.nv", "sea a = 10;\nsea b = 0;\nimprimir(a / b);\n"),
        ("indice.nv", "sea l = [1, 2, 3];\nimprimir(l[10]);\n"),
        ("negativo.nv", "sea l = [1, 2, 3];\nimprimir(l[-1]);\n"),
    ];
    for (nombre, fuente) in casos {
        let src = escribir(&dir, nombre, fuente);
        let vm = Command::new(lumen())
            .arg("run")
            .arg(&src)
            .current_dir(&dir)
            .output()
            .unwrap();
        let rc_vm = vm.status.code();
        assert_eq!(rc_vm, Some(1), "{nombre}: la VM debe salir con 1");

        let build = Command::new(lumen())
            .arg("build")
            .arg("--native")
            .arg(&src)
            .current_dir(&dir)
            .output()
            .unwrap();
        assert!(
            build.status.success(),
            "{nombre}: no compiló a nativo: {}",
            String::from_utf8_lossy(&build.stderr)
        );
        let exe = src.with_extension("");
        let nat = Command::new(&exe).current_dir(&dir).output().unwrap();
        assert_eq!(
            nat.status.code(),
            rc_vm,
            "{nombre}: el nativo salió con {:?} y la VM con {:?}",
            nat.status.code(),
            rc_vm
        );
    }
}

/// BUG-104: unificar el código de salida no debe romper `atrapar`, que sigue
/// desenrollando la pila en vez de terminar el proceso.
#[test]
fn bug104_atrapar_sigue_capturando_tras_unificar_rc() {
    let dir = dir_tmp("bug104_atrapar");
    let fuente =
        "intentar { imprimir(1 / 0); } atrapar (e) { imprimir(\"cap\"); }\nimprimir(\"sigue\");\n";
    let vm = en_vm(&dir, "atr.nv", fuente);
    assert!(vm.contains("cap") && vm.contains("sigue"), "VM: {vm}");
    if let Some(nat) = nativo(&dir, "atr_nat.nv", fuente) {
        assert!(
            nat.contains("cap") && nat.contains("sigue"),
            "nativo: {nat}"
        );
    }
}

/// BUG-108: un literal entero fuera del rango de 64 bits debe rechazarse con
/// E083 y código de salida != 0, en vez de convertirse en 0 en silencio.
#[test]
fn bug108_literal_fuera_de_rango_se_rechaza_con_e083() {
    let dir = dir_tmp("bug108_e083");
    let f = escribir(&dir, "grande.nv", "imprimir(9223372036854775808);\n");
    let out = Command::new(lumen())
        .arg("run")
        .arg(&f)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(
        !out.status.success(),
        "debe fallar; salió {:?}",
        out.status.code()
    );
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(texto.contains("E083"), "debe reportar E083: {texto}");
    assert!(!texto.contains("panicked"), "no debe hacer pánico: {texto}");

    // El entero mínimo sí es válido y debe ejecutarse.
    let f2 = escribir(&dir, "min.nv", "imprimir(-9223372036854775808);\n");
    let out2 = Command::new(lumen())
        .arg("run")
        .arg(&f2)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out2.status.success(), "el mínimo debe ser válido");
    assert!(
        String::from_utf8_lossy(&out2.stdout).contains("-9223372036854775808"),
        "salida: {}",
        String::from_utf8_lossy(&out2.stdout)
    );
}

/// BUG-109: `i64::MIN / -1` desbordaba y abortaba el proceso con un pánico de
/// Rust; además el proceso salía con código 0 pese a no imprimir nada.
#[test]
fn bug109_division_desbordante_no_hace_panic_en_la_cli() {
    let dir = dir_tmp("bug109_panic");
    for (nombre, fuente, esperado) in [
        (
            "div.nv",
            "imprimir(-9223372036854775808 / -1);\n",
            "-9223372036854775808",
        ),
        ("mod.nv", "imprimir(-9223372036854775808 % -1);\n", "0"),
    ] {
        let f = escribir(&dir, nombre, fuente);
        let out = Command::new(lumen())
            .arg("run")
            .arg(&f)
            .current_dir(&dir)
            .output()
            .unwrap();
        let texto = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(!texto.contains("panicked"), "{nombre}: pánico: {texto}");
        assert!(
            String::from_utf8_lossy(&out.stdout).contains(esperado),
            "{nombre}: esperaba {esperado}, salida: {texto}"
        );
    }
}

/// BUG-112: `~` pasaba por `double` en el runtime C, y un `double` no
/// representa todos los int64: `~9223372036854775807` daba el propio valor en
/// vez de i64::MIN. Debe coincidir con la VM.
#[test]
fn bug112_complemento_a_uno_no_pierde_precision_en_nativo() {
    let dir = dir_tmp("bug112_bnot");
    for (nombre, fuente, esperado) in [
        (
            "max.nv",
            "sea a = 9223372036854775807;\nimprimir(~a);\n",
            "-9223372036854775808",
        ),
        (
            "min.nv",
            "sea a = -9223372036854775808;\nimprimir(~a);\n",
            "9223372036854775807",
        ),
        ("cinco.nv", "sea a = 5;\nimprimir(~a);\n", "-6"),
    ] {
        let vm = en_vm(&dir, nombre, fuente);
        assert!(vm.contains(esperado), "{nombre}: VM dio {vm}");
        if let Some(nat) = nativo(&dir, &format!("nat_{nombre}"), fuente) {
            assert!(
                nat.contains(esperado),
                "{nombre}: el nativo dio {nat}, esperaba {esperado}"
            );
        }
    }
}

/// BUG-110: el desplazamiento fuera de 0-63 debe fallar igual en los dos
/// backends; en C era comportamiento indefinido y devolvía basura.
#[test]
fn bug110_desplazamiento_fuera_de_rango_falla_en_ambos_backends() {
    if !hay_compilador_c() {
        return;
    }
    let dir = dir_tmp("bug110_shift");
    let fuente = "sea a = 1;\nsea b = 64;\nimprimir(a << b);\n";
    let src = escribir(&dir, "shift.nv", fuente);

    let vm = Command::new(lumen())
        .arg("run")
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(!vm.status.success(), "la VM debe rechazarlo");

    let build = Command::new(lumen())
        .arg("build")
        .arg("--native")
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(build.status.success(), "debe compilar");
    let exe = src.with_extension("");
    let nat = Command::new(&exe).current_dir(&dir).output().unwrap();
    assert!(
        !nat.status.success(),
        "el nativo debe rechazarlo igual que la VM, no devolver basura"
    );
    let err = String::from_utf8_lossy(&nat.stderr);
    assert!(
        err.contains("Desplazamiento"),
        "debe dar el mismo error que la VM: {err}"
    );
}

/// BUG-113: el NaN se imprimía como "NaN" en la VM y como "-nan" en el binario
/// nativo. `_fmt` contemplaba `isinf` pero no `isnan`.
#[test]
fn bug113_el_nan_se_imprime_igual_en_ambos_backends() {
    let dir = dir_tmp("bug113_nan");
    for (nombre, fuente) in [
        ("raiz.nv", "imprimir(raiz(-1.0));\n"),
        ("texto.nv", "imprimir(a_texto(raiz(-4.0)));\n"),
    ] {
        let vm = en_vm(&dir, nombre, fuente);
        assert!(vm.contains("NaN"), "{nombre}: la VM dio {vm}");
        if let Some(nat) = nativo(&dir, &format!("nat_{nombre}"), fuente) {
            assert_eq!(
                nat.trim(),
                vm.trim(),
                "{nombre}: el nativo imprimió {nat} y la VM {vm}"
            );
            assert!(
                !nat.contains("nan"),
                "no debe salir 'nan' en minúsculas: {nat}"
            );
        }
    }
}

/// BUG-114: `a_entero` de un decimal fuera del rango de i64 es comportamiento
/// indefinido en C. Rust satura a los extremos; el nativo debe hacer lo mismo.
#[test]
fn bug114_a_entero_satura_igual_que_la_vm() {
    let dir = dir_tmp("bug114_sat");
    for (nombre, fuente, esperado) in [
        (
            "grande.nv",
            "imprimir(a_entero(1000000000000000000000.0));\n",
            "9223372036854775807",
        ),
        (
            "max.nv",
            "imprimir(a_entero(9223372036854775807.0));\n",
            "9223372036854775807",
        ),
        ("nan.nv", "imprimir(a_entero(raiz(-1.0)));\n", "0"),
        ("trunca.nv", "imprimir(a_entero(3.7));\n", "3"),
        ("negativo.nv", "imprimir(a_entero(-3.7));\n", "-3"),
    ] {
        let vm = en_vm(&dir, nombre, fuente);
        assert!(vm.contains(esperado), "{nombre}: la VM dio {vm}");
        if let Some(nat) = nativo(&dir, &format!("nat_{nombre}"), fuente) {
            assert_eq!(nat.trim(), vm.trim(), "{nombre}: nativo={nat} vm={vm}");
        }
    }
}

/// BUG-119: `sema` no comprobaba la aridad de `__str_codigo`,
/// `__str_a_caracteres` ni `__str_empieza_con`. Sus ramas del backend C
/// desapilan un número FIJO de argumentos, así que con uno de más el C se
/// quedaba con el argumento equivocado —un entero usado como puntero a
/// texto— y el binario nativo SEGFAULTEABA, mientras la VM ignoraba el
/// sobrante. Ahora se rechaza en compilación, como ya hacía BUG-098 con
/// `__str_concat_list`.
#[test]
fn bug119_los_builtins_de_texto_validan_su_aridad() {
    let dir = dir_tmp("bug119_aridad");
    for (nombre, fuente) in [
        ("codigo.nv", "imprimir(__str_codigo(\"abc\", 0));\n"),
        ("chars.nv", "imprimir(__str_a_caracteres(\"ab\", 9));\n"),
        (
            "empieza.nv",
            "imprimir(__str_empieza_con(\"abc\", \"a\", 9));\n",
        ),
    ] {
        let src = escribir(&dir, nombre, fuente);
        let out = Command::new(lumen())
            .arg("check")
            .arg(&src)
            .output()
            .unwrap();
        let salida = format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            salida.contains("E040"),
            "{nombre}: se esperaba E040 por aridad, salió: {salida}"
        );
    }
    // Con la aridad correcta sigue funcionando en los dos backends.
    let fuente = "imprimir(__str_codigo(\"abc\"));\n";
    let vm = en_vm(&dir, "ok.nv", fuente);
    assert!(vm.contains("97"), "la VM dio {vm}");
    if let Some(nat) = nativo(&dir, "nat_ok.nv", fuente) {
        assert_eq!(nat.trim(), vm.trim(), "nativo={nat} vm={vm}");
    }
}

/// BUG-120 (`__str_subcadena` con índices negativos), BUG-121
/// (`__str_reemplazar` con patrón vacío) y BUG-122 (`__str_dividir` con
/// separador vacío partía por BYTES, no por caracteres: la familia de
/// BUG-087). En los tres el binario nativo daba un resultado distinto de la
/// VM.
#[test]
fn bug120_121_122_los_extremos_de_texto_coinciden_en_ambos_backends() {
    let dir = dir_tmp("bug120_texto");
    for (nombre, fuente, esperado) in [
        // Un inicio negativo NO cuenta desde el final: la VM lo convierte a
        // `usize` y lo recorta a la longitud => cadena vacía.
        (
            "neg.nv",
            "imprimir(__str_subcadena(\"hola\", -2, -1));\n",
            "",
        ),
        (
            "neg2.nv",
            "imprimir(__str_subcadena(\"hola\", -3, 2));\n",
            "",
        ),
        // Patrón vacío: `str::replace` inserta en cada frontera de carácter.
        (
            "vacio.nv",
            "imprimir(__str_reemplazar(\"aaa\", \"\", \"X\"));\n",
            "XaXaXaX",
        ),
        // Separador vacío: un elemento por CARÁCTER, no por byte.
        (
            "utf8.nv",
            "imprimir(__str_dividir(\"ñoño\", \"\"));\n",
            "[ñ, o, ñ, o]",
        ),
        (
            "utf8b.nv",
            "imprimir(__str_dividir(\"日本語\", \"\"));\n",
            "[日, 本, 語]",
        ),
    ] {
        let vm = en_vm(&dir, nombre, fuente);
        assert_eq!(vm.trim(), esperado, "{nombre}: la VM dio {vm}");
        if let Some(nat) = nativo(&dir, &format!("nat_{nombre}"), fuente) {
            assert_eq!(nat.trim(), vm.trim(), "{nombre}: nativo={nat} vm={vm}");
        }
    }
}

/// BUG-123: usar un struct mal escrito daba DOS errores. El primero (E062) es
/// el bueno y hasta sugiere el nombre correcto; el segundo era ruido en
/// cascada —«no puedes acceder a un campo de un valor de tipo 'vacio'»— por
/// tipar la expresión fallida como `vacio` en vez de «no lo sé».
#[test]
fn bug123_un_struct_no_definido_no_arrastra_errores_en_cascada() {
    let dir = dir_tmp("bug123_cascada");
    let src = escribir(
        &dir,
        "casc.nv",
        "estructura Caja { n: entero, }\nsea c = Caj{n:1};\nimprimir(c.n);\n",
    );
    let out = Command::new(lumen())
        .arg("check")
        .arg(&src)
        .output()
        .unwrap();
    let salida = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(salida.contains("E062"), "falta el error real: {salida}");
    assert!(
        salida.contains("Caja"),
        "el error debería sugerir 'Caja': {salida}"
    );
    assert!(
        !salida.contains("E060"),
        "E060 en cascada no debería aparecer: {salida}"
    );

    // Pero un acceso a campo realmente inválido SÍ se sigue detectando.
    let src2 = escribir(
        &dir,
        "malo.nv",
        "estructura Caja { n: entero, }\nsea c = Caja{n:1};\nimprimir(c.noexiste);\n",
    );
    let out2 = Command::new(lumen())
        .arg("check")
        .arg(&src2)
        .output()
        .unwrap();
    let salida2 = format!(
        "{}{}",
        String::from_utf8_lossy(&out2.stdout),
        String::from_utf8_lossy(&out2.stderr)
    );
    assert!(
        salida2.contains("E059"),
        "un campo inexistente debe seguir dando error: {salida2}"
    );
}

/// BUG-124: el backend Cranelift buscaba como punto de entrada `__main__`, que
/// sólo existe si el fichero tiene código de NIVEL SUPERIOR. Un programa que
/// sólo define `funcion vacio principal()` —la forma habitual— no lo tiene, así
/// que generaba un `main` que retornaba 0 sin llamar a nada: el binario
/// anunciaba «✓ Binario nativo», no imprimía NADA y salía con código 0. Es el
/// «binario que miente» que BUG-050 y BUG-084 ya habían prohibido en los otros
/// backends; el C y el de LLVM sí hacían la cascada a `principal`.
#[test]
fn bug124_cranelift_ejecuta_principal() {
    if !hay_compilador_c() {
        return;
    }
    let dir = dir_tmp("bug124_cranelift");
    let src = escribir(
        &dir,
        "pr.nv",
        "funcion vacio principal() {\n  imprimir(42);\n}\n",
    );
    let salida_bin = dir.join("pr_cranelift");
    let comp = Command::new(lumen())
        .arg("build")
        .arg(&src)
        .arg("--aot")
        .arg("cranelift")
        .arg("-o")
        .arg(&salida_bin)
        .current_dir(&dir)
        .output()
        .unwrap();
    if !comp.status.success() || !salida_bin.exists() {
        // Si este entorno no puede enlazar con Cranelift, no hay nada que
        // comprobar; lo que NO se admite es un binario que no hace nada.
        return;
    }
    let run = Command::new(&salida_bin).output().unwrap();
    let vista = String::from_utf8_lossy(&run.stdout);
    assert!(
        vista.contains("42"),
        "el binario de Cranelift debería imprimir 42, pero salió: {vista:?}"
    );
}

/// BUG-125 (`>>` era lógico y no aritmético: `-1 >> 1` daba i64::MAX),
/// BUG-126 (un decimal se compilaba a 0 EN SILENCIO) y BUG-127 (los booleanos
/// se imprimían como 1/0) en el backend Cranelift. Los tres son el patrón de
/// «binario que miente» que BUG-050 y BUG-084 ya habían corregido: o el
/// artefacto es correcto, o hay que negarse a producirlo.
#[test]
fn bug125_126_127_cranelift_no_miente() {
    if !hay_compilador_c() {
        return;
    }
    let dir = dir_tmp("bug125_cranelift");
    let compilar = |nombre: &str, fuente: &str| -> Option<String> {
        let src = escribir(&dir, nombre, fuente);
        let bin = dir.join(format!("{nombre}.out"));
        let comp = Command::new(lumen())
            .arg("build")
            .arg(&src)
            .arg("--aot")
            .arg("cranelift")
            .arg("-o")
            .arg(&bin)
            .current_dir(&dir)
            .output()
            .unwrap();
        if !comp.status.success() || !bin.exists() {
            return None; // negarse es una respuesta válida
        }
        let run = Command::new(&bin).output().unwrap();
        Some(
            normalizar(&String::from_utf8_lossy(&run.stdout))
                .trim()
                .to_string(),
        )
    };

    // BUG-125: el desplazamiento derecho conserva el signo.
    if let Some(salida) = compilar(
        "shr.nv",
        "funcion vacio principal() {\n  imprimir(-1 >> 1);\n}\n",
    ) {
        assert_eq!(salida, "-1", "`-1 >> 1` debe dar -1, no {salida}");
    }
    // BUG-127: booleanos como true/false.
    if let Some(salida) = compilar(
        "bool.nv",
        "funcion vacio principal() {\n  sea a = 5;\n  sea b = 0;\n  imprimir(a > b);\n  imprimir(a < b);\n}\n",
    ) {
        assert_eq!(salida, "true\nfalse", "booleanos mal formateados: {salida}");
    }
    // BUG-126: un decimal ya no puede compilarse a «0» calladamente. O sale
    // bien, o el compilador se niega (que es lo que hace hoy).
    if let Some(salida) = compilar(
        "flt.nv",
        "funcion vacio principal() {\n  imprimir(1.5 + 2.5);\n}\n",
    ) {
        assert_ne!(
            salida, "0",
            "un decimal no puede compilarse a 0 en silencio"
        );
    }
}

/// BUG-128 y BUG-129: `build` (a bytecode) y `doc` ignoraban
/// `-o/--output/--salida` y escribían siempre junto al fuente, mientras que
/// `build --native` sí lo respetaba. Lo peor no es dónde escribían, sino que el
/// mensaje anunciaba la ruta real: `build a.nv -o /tmp/x.nvc` respondía
/// «Bytecode generado: a.nv c». El usuario pedía una ruta, el compilador usaba
/// otra y lo contaba sin avisar.
#[test]
fn bug128_129_build_y_doc_respetan_la_ruta_de_salida() {
    let dir = dir_tmp("bug128_salida");
    let src = escribir(
        &dir,
        "prog.nv",
        "funcion vacio principal() {\n  imprimir(7);\n}\n",
    );

    // BUG-128: bytecode en la ruta pedida, y ejecutable desde ahí.
    let destino = dir.join("sub").join("elegido.nvc");
    std::fs::create_dir_all(destino.parent().unwrap()).unwrap();
    let out = Command::new(lumen())
        .arg("build")
        .arg(&src)
        .arg("-o")
        .arg(&destino)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "build falló");
    assert!(destino.exists(), "el .nvc no se escribió en la ruta pedida");
    assert!(
        !src.with_extension("nvc").exists(),
        "no debería dejar también un .nvc junto al fuente"
    );
    let eco = String::from_utf8_lossy(&out.stdout);
    assert!(
        eco.contains("elegido.nvc"),
        "el mensaje debe nombrar la ruta real: {eco}"
    );
    let corrida = Command::new(lumen())
        .arg("run")
        .arg(&destino)
        .output()
        .unwrap();
    assert!(
        String::from_utf8_lossy(&corrida.stdout).contains('7'),
        "el bytecode de la ruta elegida debería ejecutarse"
    );

    // Sin -o, el comportamiento por defecto no cambia.
    let out2 = Command::new(lumen())
        .arg("build")
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out2.status.success());
    assert!(
        src.with_extension("nvc").exists(),
        "sin -o debe seguir escribiendo junto al fuente"
    );

    // BUG-129: lo mismo para `doc`.
    let doc_dest = dir.join("sub").join("manual.html");
    let out3 = Command::new(lumen())
        .arg("doc")
        .arg(&src)
        .arg("-o")
        .arg(&doc_dest)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(out3.status.success(), "doc falló");
    assert!(doc_dest.exists(), "la documentación ignoró -o");
}

// ───────── BUG-133/134/135: módulos, patrones y contrato de ficheros ─────────

/// BUG-133: en una cadena `app -> medio -> base`, los literales de struct
/// escritos dentro del módulo más profundo recibían el prefijo DOS veces
/// (`medio_base_Caja`), un tipo inexistente. Era el único sitio del aplanador
/// que prefijaba sin comprobar `is_known_prefixed`, así que la declaración y
/// el uso dejaban de coincidir y cualquier jerarquía de dos niveles fallaba.
/// Rompía cuatro módulos de la stdlib (`bpe`, `nn` y sus dependientes).
#[test]
fn bug133_importacion_transitiva_no_duplica_el_prefijo() {
    let dir = dir_tmp("mod_transitivo");
    let lib = dir.join("lib");
    std::fs::create_dir_all(&lib).unwrap();

    escribir(
        &lib,
        "base.nv",
        "estructura Caja { v: entero }\nfuncion entero base_doble(entero x) { retornar x * 2; }\n",
    );
    escribir(
        &lib,
        "medio.nv",
        "importar \"base.nv\";\n\
         funcion entero medio_usa(entero x) {\n\
             base_Caja c = base_Caja { v: x };\n\
             retornar c.v + base_doble(x);\n\
         }\n",
    );
    let app = escribir(
        &dir,
        "app.nv",
        "importar \"medio.nv\";\nimprimir(medio_usa(5));\n",
    );

    let out = Command::new(lumen())
        .arg("run")
        .arg(&app)
        .arg("-L")
        .arg(&lib)
        .output()
        .unwrap();
    let txt =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);

    assert!(
        !txt.contains("medio_base_Caja"),
        "BUG-133: prefijo aplicado dos veces:\n{txt}"
    );
    assert!(txt.contains("15"), "esperaba 15, se obtuvo:\n{txt}");
}

/// BUG-134: las variables que liga un patrón `si sea exito(v) = ...` no se
/// registraban como locales al aplanar módulos, así que se las tomaba por
/// globales y se les ponía el prefijo del módulo (`m_datos`). El cuerpo
/// referenciaba entonces una variable inexistente: E033 en cuanto alguien
/// importaba el módulo. `stdlib/logging.nv` era una de las víctimas.
#[test]
fn bug134_los_bindings_de_si_sea_no_reciben_prefijo_de_modulo() {
    let dir = dir_tmp("mod_ifsea");
    let lib = dir.join("lib");
    std::fs::create_dir_all(&lib).unwrap();

    escribir(
        &lib,
        "m.nv",
        "funcion texto m_leer(texto p) {\n\
             cualquiera r = __leer_archivo(p);\n\
             texto salida = \"vacio\";\n\
             si sea exito(datos) = r { salida = datos; }\n\
             retornar salida;\n\
         }\n",
    );
    let datos = escribir(&dir, "datos.txt", "hola");
    let app = escribir(
        &dir,
        "app.nv",
        &format!(
            "importar \"m.nv\";\nimprimir(m_leer(\"{}\"));\n",
            datos.display().to_string().replace('\\', "/")
        ),
    );

    let out = Command::new(lumen())
        .arg("run")
        .arg(&app)
        .arg("-L")
        .arg(&lib)
        .output()
        .unwrap();
    let txt =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);

    assert!(
        !txt.contains("m_datos"),
        "BUG-134: binding prefijado:\n{txt}"
    );
    assert!(txt.contains("hola"), "esperaba 'hola', se obtuvo:\n{txt}");
}

/// BUG-135: `__tamano_archivo` devolvía el tamaño como entero pelado pero el
/// fallo como `Error(...)`. Con el contrato partido, `si sea exito(t) = ...`
/// no casaba nunca y `logging_tamano_archivo` respondía -1 sobre ficheros que
/// existían. El resto de builtins de fichero sí envuelven en `Exito`.
#[test]
fn bug135_tamano_archivo_devuelve_exito_como_los_demas() {
    let dir = dir_tmp("tam_archivo");
    let datos = escribir(&dir, "d.txt", "12345");
    let app = escribir(
        &dir,
        "app.nv",
        &format!(
            "sea r = __tamano_archivo(\"{}\");\n\
             imprimir(__tipo_de(r));\n\
             si sea exito(t) = r {{ imprimir(t); }} sino {{ imprimir(\"no casa\"); }}\n",
            datos.display().to_string().replace('\\', "/")
        ),
    );

    let out = Command::new(lumen()).arg("run").arg(&app).output().unwrap();
    let txt = String::from_utf8_lossy(&out.stdout).to_string();

    assert!(txt.contains("exito"), "debe envolver en Exito:\n{txt}");
    assert!(txt.contains('5'), "esperaba tamaño 5:\n{txt}");
    assert!(
        !txt.contains("no casa"),
        "el patrón exito debe casar:\n{txt}"
    );
}

/// Guardián de distribución: TODOS los módulos de la stdlib deben poder
/// importarse. `lumen check` sobre el fichero suelto no basta —`bpe` y `nn`
/// lo pasaban y aun así reventaban al importarlos (BUG-133)—, que es
/// justamente lo que hace un usuario con una instalación recién bajada.
#[test]
fn stdlib_todos_los_modulos_se_pueden_importar() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("stdlib");
    if !raiz.is_dir() {
        return;
    }
    let dir = dir_tmp("stdlib_import");
    let mut rotos = Vec::new();

    for entrada in std::fs::read_dir(&raiz).unwrap() {
        let ruta = entrada.unwrap().path();
        if ruta.extension().and_then(|e| e.to_str()) != Some("nv") {
            continue;
        }
        let modulo = ruta.file_stem().unwrap().to_string_lossy().to_string();
        let app = escribir(
            &dir,
            "probe.nv",
            &format!("importar \"{modulo}\";\nimprimir(\"listo\");\n"),
        );
        let out = Command::new(lumen())
            .arg("run")
            .arg(&app)
            .arg("-L")
            .arg(&raiz)
            .output()
            .unwrap();
        let txt = String::from_utf8_lossy(&out.stdout).to_string()
            + &String::from_utf8_lossy(&out.stderr);
        if !txt.contains("listo") {
            let primera = txt
                .lines()
                .find(|l| l.contains('E') && l.contains(char::is_numeric))
                .unwrap_or("(sin diagnóstico)")
                .trim()
                .to_string();
            rotos.push(format!("{modulo}: {primera}"));
        }
    }

    assert!(
        rotos.is_empty(),
        "módulos de stdlib que no se pueden importar:\n{}",
        rotos.join("\n")
    );
}

// ───────── BUG-137/138: diagnóstico de FFI y salida en directo ─────────

/// BUG-137: al llamar `__ffi_llamar` con un handle que ya era `error(...)`
/// —lo que devuelve `__ffi_cargar` cuando la biblioteca no existe— el handle
/// se formateaba DENTRO del mensaje: `Biblioteca 'error(msvcrt.dll: cannot
/// open shared object file)' no encontrada`. Un error envuelto en otro que
/// culpa a una biblioteca cuyo nombre es, literalmente, el fallo anterior; la
/// causa real quedaba enterrada. Ahora se propaga el error original.
#[test]
fn bug137_el_error_de_ffi_no_se_anida_dentro_de_otro() {
    let dir = dir_tmp("ffi_error");
    let app = escribir(
        &dir,
        "app.nv",
        "sea l = __ffi_cargar(\"biblioteca_que_no_existe.so\");\n\
         sea r = __ffi_llamar(l, \"foo\", \"\", [], \"entero\");\n\
         imprimir(r);\n",
    );

    let out = Command::new(lumen()).arg("run").arg(&app).output().unwrap();
    let txt = String::from_utf8_lossy(&out.stdout).to_string();

    assert!(
        !txt.contains("Biblioteca 'error("),
        "BUG-137: error anidado dentro de otro:\n{txt}"
    );
    assert!(
        txt.contains("biblioteca_que_no_existe.so"),
        "el mensaje debe nombrar la biblioteca que falta:\n{txt}"
    );
}

/// BUG-138: `imprimir` sólo acumulaba en un buffer que `lumen run` volcaba al
/// terminar. Un programa que NO termina —un servidor, un bucle de eventos, un
/// TUI o simplemente un cuelgue— no mostraba absolutamente nada, ni siquiera
/// las líneas impresas antes de bloquearse. Aquí se arranca un programa con un
/// bucle infinito, se le da tiempo, se le mata y se comprueba que lo impreso
/// ANTES del bucle llegó a stdout.
#[test]
fn bug138_la_salida_se_emite_antes_de_que_el_programa_termine() {
    use std::process::Stdio;

    let dir = dir_tmp("salida_directo");
    let app = escribir(
        &dir,
        "app.nv",
        "imprimir(\"marca-antes-del-bucle\");\n\
         entero i = 0;\n\
         mientras i < 1 { i = 0; }\n",
    );
    let salida = dir.join("salida.txt");
    let fichero = std::fs::File::create(&salida).unwrap();

    let mut hijo = Command::new(lumen())
        .arg("run")
        .arg(&app)
        .stdout(Stdio::from(fichero))
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    std::thread::sleep(std::time::Duration::from_secs(2));
    let seguia_vivo = hijo.try_wait().unwrap().is_none();
    let _ = hijo.kill();
    let _ = hijo.wait();

    assert!(
        seguia_vivo,
        "el programa debía seguir en el bucle; si terminó, la prueba no mide nada"
    );

    let txt = std::fs::read_to_string(&salida).unwrap_or_default();
    assert!(
        txt.contains("marca-antes-del-bucle"),
        "BUG-138: la salida previa al bucle se perdió (capturado: {} bytes)",
        txt.len()
    );
}

/// La contrapartida de BUG-138: emitir en directo NO debe duplicar la salida
/// de un programa normal (el buffer sigue existiendo para el depurador y los
/// tests, y era fácil acabar imprimiéndolo dos veces).
#[test]
fn bug138_un_programa_normal_no_duplica_su_salida() {
    let dir = dir_tmp("salida_sin_duplicar");
    let app = escribir(&dir, "app.nv", "imprimir(\"uno\");\nimprimir(\"dos\");\n");

    let out = Command::new(lumen()).arg("run").arg(&app).output().unwrap();
    let txt = String::from_utf8_lossy(&out.stdout).to_string();

    assert_eq!(
        txt.matches("uno").count(),
        1,
        "la línea salió duplicada:\n{txt}"
    );
    assert_eq!(txt.matches("dos").count(), 1, "línea duplicada:\n{txt}");
}

// ───────── BUG-139: el servidor LSP giraba para siempre tras EOF ─────────

/// BUG-139: `read_line` devuelve `Ok(0)` —no un error— cuando stdin llega a
/// EOF. El servidor LSP salía del bucle de cabeceras con `content_length == 0`
/// y el bucle externo hacía `continue`, así que se quedaba girando a tope de
/// CPU sobre un stdin ya cerrado. Le ocurre a cualquier editor que cierre la
/// tubería sin mandar `exit` (un cierre brusco, un crash del cliente), y deja
/// un proceso `lumen lsp` quemando un núcleo hasta que alguien lo mata.
#[test]
fn bug139_el_lsp_termina_cuando_se_cierra_stdin() {
    use std::io::Write;
    use std::process::Stdio;

    let peticion = "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\
                    \"params\":{\"processId\":null,\"rootUri\":null,\"capabilities\":{}}}";
    let mut hijo = Command::new(lumen())
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();

    {
        let mut si = hijo.stdin.take().unwrap();
        write!(si, "Content-Length: {}\r\n\r\n{}", peticion.len(), peticion).unwrap();
        // `si` se cierra aquí: EOF sin haber enviado `exit`.
    }

    // Debe terminar por su cuenta y en seguida.
    let limite = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let mut termino = false;
    while std::time::Instant::now() < limite {
        if hijo.try_wait().unwrap().is_some() {
            termino = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    }
    if !termino {
        let _ = hijo.kill();
    }
    let _ = hijo.wait();
    assert!(
        termino,
        "BUG-139: el servidor LSP siguió vivo tras cerrarse stdin (bucle infinito)"
    );
}

/// El LSP debe coincidir con `lumen check`: si `check` acepta el programa, el
/// servidor no puede publicar diagnósticos sobre él, y viceversa. Dos motores
/// distintos que opinan cosas distintas sobre el mismo fichero es lo que hace
/// que un editor marque en rojo código que compila.
#[test]
fn el_lsp_no_contradice_a_check() {
    use std::io::Write;
    use std::process::Stdio;

    let dir = dir_tmp("lsp_acuerdo");
    let valido = escribir(
        &dir,
        "ok.nv",
        "funcion entero doble(entero x) { retornar x * 2; }\nimprimir(doble(21));\n",
    );
    let roto = escribir(&dir, "malo.nv", "entero x = \"soy texto\";\n");

    // JSON construido a mano: `lumen-cli` no depende de serde_json y no vale
    // la pena añadir la dependencia sólo para un test.
    fn escapar_json(s: &str) -> String {
        let mut out = String::new();
        for c in s.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
                c => out.push(c),
            }
        }
        out
    }

    let mut cuerpo = String::new();
    let anadir = |o: String, c: &mut String| {
        c.push_str(&format!("Content-Length: {}\r\n\r\n{}", o.len(), o));
    };
    anadir(
        "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"initialize\",\"params\":{\"processId\":null,\"rootUri\":null,\"capabilities\":{}}}".to_string(),
        &mut cuerpo,
    );
    for f in [&valido, &roto] {
        let texto = std::fs::read_to_string(f).unwrap();
        let m = format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/didOpen\",\"params\":{{\"textDocument\":{{\"uri\":\"{}\",\"languageId\":\"lumen\",\"version\":1,\"text\":\"{}\"}}}}}}",
            uri_de(f),
            escapar_json(&texto)
        );
        anadir(m, &mut cuerpo);
    }
    anadir(
        "{\"jsonrpc\":\"2.0\",\"method\":\"exit\",\"params\":{}}".to_string(),
        &mut cuerpo,
    );

    let mut hijo = Command::new(lumen())
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    hijo.stdin
        .take()
        .unwrap()
        .write_all(cuerpo.as_bytes())
        .unwrap();
    let salida = hijo.wait_with_output().unwrap();
    let txt = String::from_utf8_lossy(&salida.stdout).to_string();

    // El fichero roto debe generar al menos un diagnóstico…
    assert!(
        txt.contains("E031")
            || txt.contains("no puedes asignar")
            || txt.contains("No puedes asignar"),
        "el LSP no señaló el error de tipos:\n{txt}"
    );
    // …y el válido no debe aparecer con diagnósticos no vacíos.
    let marca_valida = uri_de(&valido);
    for trozo in txt.split("publishDiagnostics") {
        if trozo.contains(&marca_valida) {
            assert!(
                trozo.contains("\"diagnostics\":[]"),
                "el LSP marcó como erróneo un programa que `check` acepta:\n{trozo}"
            );
        }
    }
}

// ───────────────── BUG-140: doctor no sondeaba el toolchain LLVM ─────────────

/// `doctor` imprimía "LLVM IR Directo: ✓ Habilitado" como texto fijo, sin
/// comprobar nada, en una máquina sin `clang`/`llc`/`llvm-as` donde
/// `build --aot llvm` falla. Un diagnóstico no puede afirmar capacidades que
/// dependen de herramientas externas sin sondearlas.
#[test]
fn doctor_no_afirma_llvm_sin_toolchain() {
    let salida = Command::new(lumen()).arg("doctor").output().unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout).to_string();

    // BUG-162: esto partia el PATH por ':' y buscaba el binario sin extension.
    // En Windows el separador es ';' y los ejecutables llevan '.exe', asi que
    // la sonda concluia «no hay LLVM» aunque estuviera instalado, y entonces
    // exigia que `doctor` NO dijera «Habilitado» — fallando en la maquina que
    // si lo tiene. `split_paths` y `PATHEXT` son la forma portable.
    let hay_llvm = ["llc", "clang", "llvm-as"].iter().any(|b| {
        let exts: Vec<String> = if cfg!(windows) {
            std::env::var("PATHEXT")
                .unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_string())
                .split(';')
                .map(|e| e.to_lowercase())
                .collect()
        } else {
            vec![String::new()]
        };
        std::env::var_os("PATH")
            .map(|p| {
                std::env::split_paths(&p)
                    .any(|d| exts.iter().any(|e| d.join(format!("{}{}", b, e)).is_file()))
            })
            .unwrap_or(false)
    });

    let linea_llvm = texto
        .lines()
        .find(|l| l.contains("LLVM"))
        .unwrap_or("")
        .to_string();
    assert!(!linea_llvm.is_empty(), "doctor no menciona LLVM:\n{texto}");

    if !hay_llvm {
        assert!(
            !linea_llvm.contains("Habilitado"),
            "doctor anuncia LLVM habilitado sin toolchain instalado: {linea_llvm}"
        );
    }
}

// ───────────── BUG-141: unpack extraía fuera del directorio actual ───────────

/// El destino por defecto se derivaba de la ruta completa del `.lmp`, así que
/// desempaquetar un paquete de otro directorio extraía junto al paquete y
/// dejaba vacío el directorio de trabajo del usuario.
#[test]
fn unpack_extrae_en_el_directorio_actual() {
    let base = dir_tmp("unpack_cwd");
    let origen = base.join("origen");
    let trabajo = base.join("trabajo");
    std::fs::create_dir_all(&origen).unwrap();
    std::fs::create_dir_all(&trabajo).unwrap();

    let proy = origen.join("paqx");
    std::fs::create_dir_all(proy.join("src")).unwrap();
    escribir(
        &proy,
        "lumen.toml",
        "[paquete]\nnombre = \"paqx\"\nversion = \"0.1.0\"\nprincipal = \"src/main.nv\"\n",
    );
    escribir(&proy.join("src"), "main.nv", "imprimir(1);\n");

    let pack = Command::new(lumen())
        .arg("pack")
        .current_dir(&proy)
        .output()
        .unwrap();
    assert!(pack.status.success(), "pack falló");

    let lmp = std::fs::read_dir(&origen)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "lmp"))
        .expect("pack no generó .lmp");

    let salida = Command::new(lumen())
        .arg("unpack")
        .arg(&lmp)
        .current_dir(&trabajo)
        .output()
        .unwrap();
    assert!(
        salida.status.success(),
        "unpack falló: {}",
        String::from_utf8_lossy(&salida.stderr)
    );

    let extraido: Vec<_> = std::fs::read_dir(&trabajo)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert!(
        !extraido.is_empty(),
        "unpack dejó vacío el directorio de trabajo: extrajo fuera del cwd"
    );
}

// ──────────── BUG-142: install no aceptaba un .lmp local ─────────────────────

/// `pack` genera un `.lmp` e invita a instalarlo, pero `install` no tenía rama
/// para ficheros: la ruta caía al fallback de git y se concatenaba a
/// `https://github.com/`, fallando con "repository not found".
#[test]
fn install_acepta_un_paquete_lmp_local() {
    let base = dir_tmp("install_lmp");
    let lib = base.join("libx");
    std::fs::create_dir_all(lib.join("src")).unwrap();
    escribir(
        &lib,
        "lumen.toml",
        "[paquete]\nnombre = \"libx\"\nversion = \"0.1.0\"\nprincipal = \"src/main.nv\"\n",
    );
    escribir(
        &lib.join("src"),
        "main.nv",
        "funcion entero libx_doble(entero x) { retornar x * 2; }\n",
    );

    assert!(
        Command::new(lumen())
            .arg("pack")
            .current_dir(&lib)
            .output()
            .unwrap()
            .status
            .success(),
        "pack falló"
    );
    let lmp = std::fs::read_dir(&base)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .find(|p| p.extension().is_some_and(|x| x == "lmp"))
        .expect("pack no generó .lmp");

    let app = base.join("app");
    std::fs::create_dir_all(&app).unwrap();
    escribir(
        &app,
        "lumen.toml",
        "[paquete]\nnombre = \"app\"\nversion = \"0.1.0\"\nprincipal = \"src/main.nv\"\n",
    );

    let salida = Command::new(lumen())
        .arg("install")
        .arg(&lmp)
        .current_dir(&app)
        .output()
        .unwrap();
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );

    assert!(
        !texto.contains("github.com"),
        "install trató un fichero local como repositorio git:\n{texto}"
    );
    assert!(
        app.join("pkgs")
            .join("libx")
            .join("src")
            .join("main.nv")
            .is_file()
            || app.join("pkgs").join("libx").join("main.nv").is_file(),
        "install no dejó el paquete en pkgs/:\n{texto}"
    );
}

// ──────────── BUG-143: bindgen generaba módulos que no compilan ──────────────

/// El heurístico tomaba cualquier línea con paréntesis terminada en ';' por
/// una declaración, así que las *llamadas* del programa se emitían como
/// funciones: un fuente con dos `imprimir(...)` generaba la misma función dos
/// veces y redefinía un builtin, produciendo un módulo que no compila (E082).
#[test]
fn bindgen_genera_un_modulo_valido() {
    let dir = dir_tmp("bindgen_valido");
    let src = escribir(&dir, "prog.nv", "imprimir(\"hola\");\nimprimir(6 * 7);\n");

    let salida = Command::new(lumen())
        .arg("bindgen")
        .arg(&src)
        .current_dir(&dir)
        .output()
        .unwrap();
    assert!(salida.status.success(), "bindgen falló");

    let generado = dir.join("prog_bindings.nv");
    assert!(generado.is_file(), "bindgen no generó el módulo");

    let contenido = std::fs::read_to_string(&generado).unwrap();
    assert!(
        !contenido.contains("funcion cualquiera imprimir("),
        "bindgen redefine el builtin 'imprimir':\n{contenido}"
    );

    // Lo generado tiene que ser código válido.
    let check = Command::new(lumen())
        .arg("check")
        .arg(&generado)
        .output()
        .unwrap();
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    assert!(
        check.status.success() && !texto.contains("E082"),
        "el módulo generado por bindgen no compila:\n{texto}"
    );
}

/// Con una cabecera C real debe enlazar cada firma una sola vez.
#[test]
fn bindgen_enlaza_cada_firma_de_una_cabecera_c() {
    let dir = dir_tmp("bindgen_cabecera");
    let h = escribir(
        &dir,
        "mat.h",
        "double mat_raiz(double x);\nint mat_sumar(int a, int b);\nvoid mat_reset(void);\n",
    );

    let salida = Command::new(lumen())
        .arg("bindgen")
        .arg(&h)
        .arg("-o")
        .arg(dir.join("mat_b.nv"))
        .output()
        .unwrap();
    assert!(salida.status.success(), "bindgen falló");

    let contenido = std::fs::read_to_string(dir.join("mat_b.nv")).unwrap();
    for f in ["mat_raiz", "mat_sumar", "mat_reset"] {
        let n = contenido
            .matches(&format!("funcion cualquiera {f}("))
            .count();
        assert_eq!(n, 1, "'{f}' aparece {n} veces en el módulo generado");
    }

    let check = Command::new(lumen())
        .arg("check")
        .arg(dir.join("mat_b.nv"))
        .output()
        .unwrap();
    assert!(
        check.status.success(),
        "el módulo de la cabecera no compila"
    );
}

// ─────────── BUG-144: `config set` era imposible de invocar ──────────────────

/// La ayuda de `lumen config` anuncia `lumen config set <clave> <valor>`, pero
/// el parser de argumentos sólo aceptaba tres posicionales, así que el cuarto
/// moría con «Argumento desconocido» y rc=1: el comando que la herramienta
/// documenta no se podía ejecutar.
#[test]
fn config_set_es_invocable() {
    let salida = Command::new(lumen())
        .args(["config", "set", "backend", "llvm"])
        .output()
        .unwrap();
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );
    assert!(
        !texto.contains("Argumento desconocido"),
        "config set rechaza su propio argumento:\n{texto}"
    );
    assert!(salida.status.success(), "config set falló:\n{texto}");
    // Y debe orientar hacia la bandera real, ya que no persiste nada.
    assert!(
        texto.contains("--aot"),
        "config set no indica la bandera equivalente:\n{texto}"
    );
}

/// Una clave inventada tiene que rechazarse, no aceptarse en silencio.
#[test]
fn config_set_rechaza_claves_desconocidas() {
    let salida = Command::new(lumen())
        .args(["config", "set", "clave_que_no_existe", "1"])
        .output()
        .unwrap();
    assert!(
        !salida.status.success(),
        "config set aceptó una clave inexistente"
    );
}

// ────────── BUG-145: publish decía haber publicado sin subir nada ────────────

/// `publish` imprimía «¡publicado con éxito en el registro público!» sin
/// realizar ninguna petición de red, con credenciales del autor codificadas en
/// el binario y un «Checksum SHA-256» que era en realidad un `DefaultHasher`.
#[test]
fn publish_no_afirma_haber_subido_el_paquete() {
    let dir = dir_tmp("publish_honesto");
    std::fs::create_dir_all(dir.join("src")).unwrap();
    escribir(
        &dir,
        "lumen.toml",
        "[paquete]\nnombre = \"pubx\"\nversion = \"0.1.0\"\nprincipal = \"src/main.nv\"\n",
    );
    escribir(&dir.join("src"), "main.nv", "imprimir(1);\n");

    let salida = Command::new(lumen())
        .arg("publish")
        .current_dir(&dir)
        .output()
        .unwrap();
    let texto = format!(
        "{}{}",
        String::from_utf8_lossy(&salida.stdout),
        String::from_utf8_lossy(&salida.stderr)
    );

    assert!(
        !texto.contains("publicado con éxito"),
        "publish afirma haber publicado sin subir nada:\n{texto}"
    );
    assert!(
        !texto.contains("omar_dev"),
        "publish usa credenciales codificadas en el binario:\n{texto}"
    );
}

// ────── BUG-146: el tutorial enseñaba sintaxis que no compila ────────────────

/// `lumen tutor` es lo primero que ejecuta quien aprende el lenguaje. El código
/// que muestra tiene que compilar: enseñaba `funcion entero suma(Punto este)`
/// —el receptor va sin tipo— y `div(a,b)` —los parámetros exigen tipo—, y
/// ninguno de los dos pasa el parser.
///
/// El test **extrae el código de la salida real de `tutor`**: si copiara los
/// fragmentos aquí, seguiría pasando aunque el tutorial volviese a enseñar
/// sintaxis inválida.
#[test]
fn el_codigo_del_tutorial_compila() {
    let dir = dir_tmp("tutorial_valido");

    // Fragmentos que el tutorial promete, con el contexto mínimo para
    // ejecutarlos y el resultado que el propio tutorial anuncia en su
    // comentario.
    let casos: [(&str, &str, &str, &str); 2] = [
        (
            "data",
            "funcion entero suma(",
            "estructura Punto { x: entero, y: entero }\n\
             impl Punto {\n{LINEA}\n    retornar este.x + este.y;\n    }\n}\n\
             Punto p = Punto { x: 3, y: 4 };\n\
             imprimir(p.suma());\n",
            "7",
        ),
        (
            "advanced",
            "funcion resultado<entero,texto> div(",
            "{LINEA}\n\
             \x20   si b == 0 { retornar error(\"no\"); }\n\
             \x20   retornar exito(a / b);\n\
             }\n\
             si sea exito(v) = div(10, 2) { imprimir(v); } sino { imprimir(0); }\n",
            "5",
        ),
    ];

    for (tema, marca, plantilla, esperado) in casos {
        let salida = Command::new(lumen())
            .args(["tutor", tema])
            .output()
            .unwrap();
        let texto = String::from_utf8_lossy(&salida.stdout).to_string();

        // La línea tal y como el tutorial se la enseña al alumno.
        let linea = texto
            .lines()
            .find(|l| l.trim_start().starts_with(marca))
            .unwrap_or_else(|| panic!("'tutor {tema}' ya no muestra '{marca}'"))
            .trim()
            .to_string();
        let linea = if linea.ends_with('{') {
            linea
        } else {
            format!("{linea} {{")
        };

        let fuente = plantilla.replace("{LINEA}", &linea);
        let ruta = escribir(&dir, &format!("{tema}.nv"), &fuente);

        let r = Command::new(lumen())
            .arg("run")
            .arg(&ruta)
            .output()
            .unwrap();
        let obtenido = String::from_utf8_lossy(&r.stdout).trim().to_string();
        assert!(
            r.status.success() && obtenido == esperado,
            "el código que enseña 'tutor {tema}' no funciona.\n\
             línea del tutorial: {linea}\n\
             esperado: {esperado}, obtenido: {obtenido}\n{}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
}

/// Los temas que `tutor` ofrece tienen que existir todos.
#[test]
fn todos_los_temas_del_tutorial_responden() {
    for tema in ["basics", "functions", "data", "advanced", "stdlib", "pro"] {
        let salida = Command::new(lumen())
            .args(["tutor", tema])
            .output()
            .unwrap();
        let texto = String::from_utf8_lossy(&salida.stdout).to_string();
        assert!(
            !texto.contains("Temas disponibles"),
            "el tema '{tema}' no existe: tutor devolvió la lista de temas"
        );
        assert!(salida.status.success(), "tutor {tema} falló");
    }
}

/// Los ejemplos que el plan de aprendizaje manda ejecutar deben existir.
#[test]
fn learn_solo_cita_ejemplos_existentes() {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .unwrap()
        .to_path_buf();

    let salida = Command::new(lumen()).arg("learn").output().unwrap();
    let texto = String::from_utf8_lossy(&salida.stdout).to_string();

    for palabra in texto.split_whitespace() {
        let limpio = palabra.trim_matches(|c: char| !c.is_ascii_graphic());
        if let Some(resto) = limpio.strip_prefix("examples/") {
            // `mi_programa.nv` es el archivo que el alumno debe crear.
            if resto.starts_with("mi_programa") {
                continue;
            }
            if resto.ends_with(".nv") {
                assert!(
                    raiz.join("examples").join(resto).is_file(),
                    "learn manda ejecutar 'examples/{resto}', que no existe"
                );
            }
        }
    }
}

// ── BUG-147: `prestado mut` descartaba la mutación si el argumento no era
//            una variable simple ─────────────────────────────────────────────

/// El copy-back que implementa `prestado mut` sólo reconocía `Expr::Ident`, así
/// que pasar `s.campo` o `l[i]` a un parámetro prestado compilaba, pasaba
/// `check` y **no hacía nada**: la mutación se descartaba en silencio. Es el
/// mismo modo de fallo del BUG-008 —aceptar el código y no cumplirlo— en la
/// otra mitad de la expresión.
#[test]
fn prestado_mut_acepta_campos_y_elementos() {
    let dir = dir_tmp("prestamo_lvalue");

    let casos: [(&str, &str, &str); 4] = [
        (
            "campo",
            "estructura S { l: lista<entero>, }\n\
             funcion vacio toca(prestado mut lista<entero> l) { l[0] = 99; }\n\
             sea s = S{l: [1, 2]};\n\
             toca(s.l);\n\
             imprimir(s.l);\n",
            "[99, 2]",
        ),
        (
            "elemento",
            "estructura P { x: entero, }\n\
             funcion vacio toca(prestado mut P p) { p.x = 99; }\n\
             lista<P> l = [P{x: 1}, P{x: 2}];\n\
             toca(l[0]);\n\
             imprimir(l[0].x);\n",
            "99",
        ),
        (
            "lista_anidada",
            "funcion vacio ap(prestado mut lista<entero> l) { agregar(l, 9); }\n\
             lista<lista<entero>> m = [[1], [2]];\n\
             ap(m[0]);\n\
             imprimir(m[0]);\n",
            "[1, 9]",
        ),
        (
            "campo_anidado",
            "estructura I { v: entero, }\n\
             estructura E { i: I, }\n\
             funcion vacio toca(prestado mut I x) { x.v = 42; }\n\
             sea e = E{i: I{v: 1}};\n\
             toca(e.i);\n\
             imprimir(e.i.v);\n",
            "42",
        ),
    ];

    for (nombre, fuente, esperado) in casos {
        let ruta = escribir(&dir, &format!("{nombre}.nv"), fuente);
        let r = Command::new(lumen())
            .arg("run")
            .arg(&ruta)
            .output()
            .unwrap();
        let obtenido = String::from_utf8_lossy(&r.stdout).trim().to_string();
        assert!(
            r.status.success() && obtenido == esperado,
            "'{nombre}': la mutación a través de un préstamo se perdió.\n\
             esperado: {esperado}, obtenido: {obtenido}\n{}",
            String::from_utf8_lossy(&r.stderr)
        );
    }
}

/// Y pasar por valor tiene que seguir copiando: el préstamo no puede volverse
/// el comportamiento por defecto (sería reintroducir el BUG-008).
#[test]
fn pasar_por_valor_sigue_copiando() {
    let dir = dir_tmp("paso_por_valor");
    let ruta = escribir(
        &dir,
        "valor.nv",
        "estructura S { l: lista<entero>, }\n\
         funcion vacio toca(S s) { s.l[0] = 99; }\n\
         funcion vacio toca_lista(lista<entero> l) { l[0] = 99; }\n\
         sea s = S{l: [1, 2]};\n\
         toca(s);\n\
         imprimir(s.l);\n\
         lista<entero> a = [1, 2];\n\
         toca_lista(a);\n\
         imprimir(a);\n",
    );
    let r = Command::new(lumen())
        .arg("run")
        .arg(&ruta)
        .output()
        .unwrap();
    let salida = normalizar(&String::from_utf8_lossy(&r.stdout))
        .trim()
        .to_string();
    assert_eq!(
        salida, "[1, 2]\n[1, 2]",
        "pasar por valor dejó de copiar: la mutación se escapó (BUG-008)"
    );
}

/// BUG-149: la mutación que una closure hace sobre una variable capturada
/// tiene que verse en la variable original. La closure avanzaba (5, 10, 15)
/// mientras la variable de la función envolvente seguía en 0, porque el
/// copy-back llegaba a la celda compartida pero no al marco que la declara.
#[test]
fn closure_mutacion_visible_en_el_declarante() {
    let dir = dir_tmp("clo_mut_vm");
    let ruta = escribir(
        &dir,
        "m.nv",
        "funcion vacio p() {\n\
         \x20   entero x = 0;\n\
         \x20   sea inc = funcion(entero n) { x = x + n; retornar x; };\n\
         \x20   imprimir(inc(5));\n\
         \x20   imprimir(inc(5));\n\
         \x20   imprimir(inc(5));\n\
         \x20   imprimir(x);\n\
         }\n\
         p();\n",
    );
    let r = Command::new(lumen())
        .arg("run")
        .arg(&ruta)
        .output()
        .unwrap();
    let salida = normalizar(&String::from_utf8_lossy(&r.stdout))
        .trim()
        .to_string();
    assert_eq!(
        salida, "5\n10\n15\n15",
        "la mutación de la captura no volvió a la variable original (BUG-149)"
    );
}

/// BUG-150: el mismo caso, pero compilado a binario nativo. El backend C
/// restauraba las variables del llamador al volver de la closure (protección
/// de BUG-061 para las lambdas recursivas) y con ello deshacía la mutación
/// recién hecha sobre la captura: la VM daba 15 y el nativo 0, en silencio.
#[test]
fn closure_mutacion_visible_en_nativo() {
    let dir = dir_tmp("clo_mut_nat");
    let ruta = escribir(
        &dir,
        "m.nv",
        "funcion vacio p() {\n\
         \x20   entero x = 0;\n\
         \x20   sea inc = funcion(entero n) { x = x + n; retornar x; };\n\
         \x20   imprimir(inc(5));\n\
         \x20   imprimir(inc(5));\n\
         \x20   imprimir(inc(5));\n\
         \x20   imprimir(x);\n\
         }\n\
         p();\n",
    );
    let bin = dir.join("m_bin");
    let c = Command::new(lumen())
        .arg("build")
        .arg(&ruta)
        .arg("--native")
        .arg("-o")
        .arg(&bin)
        .output()
        .unwrap();
    assert!(c.status.success(), "la compilación nativa falló");
    let r = Command::new(&bin).output().unwrap();
    let salida = normalizar(&String::from_utf8_lossy(&r.stdout))
        .trim()
        .to_string();
    assert_eq!(
        salida, "5\n10\n15\n15",
        "el binario nativo perdió la mutación de la captura (BUG-150)"
    );
}

/// BUG-150 no debe reabrir BUG-061: una lambda recursiva sigue necesitando
/// que el llamador restaure sus variables, porque los parámetros viven en
/// slots globales y `fib(n-1)` machacaba `n` antes de evaluar `fib(n-2)`.
#[test]
fn lambda_recursiva_nativa_sigue_bien() {
    let dir = dir_tmp("clo_rec_nat");
    let ruta = escribir(
        &dir,
        "r.nv",
        "sea fib = funcion(entero n) {\n\
         \x20   si n <= 1 { retornar n; }\n\
         \x20   retornar fib(n - 1) + fib(n - 2);\n\
         };\n\
         imprimir(fib(10));\n",
    );
    let bin = dir.join("r_bin");
    let c = Command::new(lumen())
        .arg("build")
        .arg(&ruta)
        .arg("--native")
        .arg("-o")
        .arg(&bin)
        .output()
        .unwrap();
    assert!(c.status.success(), "la compilación nativa falló");
    let r = Command::new(&bin).output().unwrap();
    let salida = normalizar(&String::from_utf8_lossy(&r.stdout))
        .trim()
        .to_string();
    assert_eq!(salida, "55", "regresión de BUG-061 al arreglar BUG-150");
}

/// BUG-151: si falta el `{` de un bloque, el parser descartaba la sentencia
/// ENTERA sin emitir ningún error, y el bloque se reejecutaba luego como
/// bloque suelto — es decir, sin su condición. `si (1 == 2) basura { ... }`
/// imprimía el cuerpo y `lumen check` daba el programa por válido.
#[test]
fn bloque_sin_llave_no_se_ejecuta_en_silencio() {
    let dir = dir_tmp("blq_mudo");
    let ruta = escribir(
        &dir,
        "b.nv",
        "funcion vacio main() {\n\
         \x20   si (1 == 2) basura { imprimir(\"NO_DEBE_SALIR\"); }\n\
         \x20   imprimir(\"fin\");\n\
         }\n",
    );
    let r = Command::new(lumen())
        .arg("run")
        .arg(&ruta)
        .output()
        .unwrap();
    let salida = String::from_utf8_lossy(&r.stdout);
    assert!(
        !salida.contains("NO_DEBE_SALIR"),
        "la condición se ignoró y el bloque se ejecutó igual (BUG-151)"
    );
    assert!(!r.status.success(), "debería fallar, no salir con éxito");

    // `check` tampoco puede dar esto por bueno.
    let c = Command::new(lumen())
        .arg("check")
        .arg(&ruta)
        .output()
        .unwrap();
    assert!(
        !c.status.success(),
        "`lumen check` dio por válido un programa con un bloque sin '{{' (BUG-151)"
    );
}

/// BUG-154: al importar un PAQUETE (directorio con `lumen.toml`), el prefijo
/// salía del fichero de entrada y no del nombre del paquete. `lumen install`
/// terminaba diciendo «ya puedes importar sus módulos con: importar "X";» y
/// las funciones aparecían como `main_f` en vez de `X_f`.
#[test]
fn paquete_usa_su_nombre_como_prefijo() {
    let dir = dir_tmp("pkg_prefijo");
    let pkg = dir.join("libreria");
    std::fs::create_dir_all(pkg.join("src")).unwrap();
    std::fs::write(
        pkg.join("lumen.toml"),
        "[proyecto]\nnombre = \"libreria\"\nversion = \"0.1.0\"\nprincipal = \"src/main.nv\"\n",
    )
    .unwrap();
    std::fs::write(
        pkg.join("src/main.nv"),
        "funcion entero sumar(entero a, entero b) { retornar a + b; }\n",
    )
    .unwrap();

    let ruta = escribir(
        &dir,
        "uso.nv",
        "importar \"libreria\";\n\
         funcion vacio main() { imprimir(libreria_sumar(2, 3)); }\n",
    );
    let r = Command::new(lumen())
        .arg("run")
        .arg("-L")
        .arg(&dir)
        .arg(&ruta)
        .output()
        .unwrap();
    let salida = normalizar(&String::from_utf8_lossy(&r.stdout))
        .trim()
        .to_string();
    assert_eq!(
        salida,
        "5",
        "el paquete no expuso `libreria_sumar` (BUG-154); stderr: {}",
        String::from_utf8_lossy(&r.stderr)
    );
}

/// BUG-156: el REPL nunca invocaba al `ModuleLoader`, así que `importar` era
/// un no-op silencioso: la línea se aceptaba con `=> ()`, las funciones del
/// módulo seguían sin existir, y `importar "no_existe";` tampoco daba error.
/// Un REPL que no puede cargar un módulo no sirve para probarlo antes de
/// usarlo, que es justo para lo que se usa un REPL.
#[test]
fn repl_resuelve_imports() {
    use std::io::Write;
    use std::process::Stdio;

    let mut hijo = Command::new(lumen())
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    hijo.stdin
        .as_mut()
        .unwrap()
        .write_all(b"importar \"texto\";\nimprimir(texto_longitud(\"hola\"));\nsalir\n")
        .unwrap();
    let r = hijo.wait_with_output().unwrap();
    let salida = String::from_utf8_lossy(&r.stdout);
    assert!(
        salida.contains('4'),
        "el REPL no resolvió `importar \"texto\"` (BUG-156). Salida: {}",
        salida
    );
}

/// BUG-156, la otra mitad: importar algo que no existe tiene que doler.
#[test]
fn repl_rechaza_import_inexistente() {
    use std::io::Write;
    use std::process::Stdio;

    let mut hijo = Command::new(lumen())
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    hijo.stdin
        .as_mut()
        .unwrap()
        .write_all(b"importar \"modulo_que_no_existe_xyz\";\nsalir\n")
        .unwrap();
    let r = hijo.wait_with_output().unwrap();
    let todo = format!(
        "{}{}",
        String::from_utf8_lossy(&r.stdout),
        String::from_utf8_lossy(&r.stderr)
    );
    assert!(
        todo.contains("no encontrado") || todo.to_lowercase().contains("error"),
        "el REPL aceptó en silencio un módulo inexistente (BUG-156). Salida: {}",
        todo
    );
}

/// BUG-161: las declaraciones adelantadas (`funcion numero f(numero x);`), el
/// patron para recursion mutua, dejaron de parsear al convertir en error el
/// `None` mudo de BUG-151. Ocho ejemplos del repo llevaban anios usandolas y
/// nadie lo vio porque la baseline recorria `examples/*.nv` sin recursion,
/// dejando fuera `examples/compiler/` entero.
#[test]
fn bug161_declaracion_adelantada_es_valida() {
    let dir = dir_tmp("fwd_decl");
    let f = escribir(
        &dir,
        "fw.nv",
        "funcion entero par(entero n);\n\
         funcion entero impar(entero n) { si (n == 0) { retornar 0; } retornar par(n - 1); }\n\
         funcion entero par(entero n) { si (n == 0) { retornar 1; } retornar impar(n - 1); }\n\
         funcion vacio principal() { imprimir(par(10)); }\n",
    );

    let out = Command::new(lumen()).arg("run").arg(&f).output().unwrap();
    let sal = String::from_utf8_lossy(&out.stdout);
    assert!(
        out.status.success(),
        "la declaracion adelantada debe aceptarse. stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(sal.trim(), "1", "recursion mutua rota. Salida: {}", sal);
}

/// BUG-161 (cara B): aceptar el prototipo no puede volver a abrir el agujero de
/// BUG-151. Un prototipo SIN definicion debe ser un error explicito, no
/// desaparecer en silencio del AST.
#[test]
fn bug161_prototipo_sin_definicion_es_error() {
    let dir = dir_tmp("fwd_huerfano");
    let f = escribir(
        &dir,
        "h.nv",
        "funcion entero perdida(entero x);\n\
         funcion vacio principal() { imprimir(1); }\n",
    );

    let out = Command::new(lumen()).arg("check").arg(&f).output().unwrap();
    let todo = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !out.status.success(),
        "un prototipo huerfano debe fallar. Salida: {}",
        todo
    );
    assert!(
        todo.contains("E084") && todo.contains("perdida"),
        "debe nombrar el codigo y la funcion. Salida: {}",
        todo
    );
}

/// BUG-161 (cara C): el arreglo NO debe resucitar BUG-151 — un bloque colgado
/// tras una condicion invalida tiene que seguir siendo un error.
#[test]
fn bug161_no_resucita_el_bloque_mudo_de_bug151() {
    let dir = dir_tmp("fwd_no_151");
    let f = escribir(
        &dir,
        "b.nv",
        "funcion vacio principal() {\n    si (1 == 2) basura { imprimir(\"NO_DEBE_SALIR\"); }\n}\n",
    );

    let out = Command::new(lumen()).arg("run").arg(&f).output().unwrap();
    let todo = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !todo.contains("NO_DEBE_SALIR"),
        "el bloque se ejecuto sin su condicion (regresion de BUG-151): {}",
        todo
    );
}

/// BUG-159: el mismo valor se llamaba de dos maneras. `__tipo_de(nada())`
/// devolvia "nulo" pero `imprimir(nada())` escribia "void": el usuario no puede
/// escribir `si (__tipo_de(x) == ...)` fiandose de lo que ve impreso. Ademas
/// "void" era el unico anglicismo entre nombres de tipo en espanol. Se
/// comprueba la coherencia Y la paridad VM/nativo, que es donde estos arreglos
/// se suelen quedar a medias.
#[test]
fn bug159_nulo_se_imprime_igual_que_su_tipo() {
    let dir = dir_tmp("nulo_coherente");
    let f = escribir(
        &dir,
        "n.nv",
        "funcion vacio nada() { }\n\
         funcion vacio principal() {\n\
             imprimir(__tipo_de(nada()));\n\
             imprimir(nada());\n\
             imprimir(a_texto(nada()));\n\
         }\n",
    );

    let out = Command::new(lumen()).arg("run").arg(&f).output().unwrap();
    let vm = String::from_utf8_lossy(&out.stdout).trim().to_string();
    assert_eq!(
        vm, "nulo\nnulo\nnulo",
        "el nulo debe imprimirse igual que lo nombra __tipo_de (BUG-159). Salida: {}",
        vm
    );

    // Paridad con el backend nativo: el runtime C tenia su propia cadena.
    let bin = dir.join("n_nat");
    let comp = Command::new(lumen())
        .arg("build")
        .arg(&f)
        .arg("--native")
        .arg("-o")
        .arg(&bin)
        .output()
        .unwrap();
    if comp.status.success() && bin.exists() {
        let nat = Command::new(&bin).output().unwrap();
        let nat = normalizar(&String::from_utf8_lossy(&nat.stdout))
            .trim()
            .to_string();
        assert_eq!(
            nat, vm,
            "el binario nativo imprime el nulo distinto que la VM (BUG-159)"
        );
    }
}

/// BUG-157: el LSP analizaba cada fichero AISLADO, sin resolver sus imports.
/// Un archivo que `lumen check` da por válido salía subrayado en rojo en el
/// editor: todo lo que viniera de un módulo se marcaba «no definida». Un
/// servidor de lenguaje que contradice al compilador enseña a ignorar los
/// avisos, que es el peor resultado posible.
#[test]
fn lsp_resuelve_imports_sin_falsos_positivos() {
    use std::io::Write;
    use std::process::Stdio;

    let marco = |cuerpo: &str| format!("Content-Length: {}\r\n\r\n{}", cuerpo.len(), cuerpo);
    let init = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"processId":null,"rootUri":null,"capabilities":{}}}"#;
    let abrir = r#"{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{"textDocument":{"uri":"file:///tmp/lsp_bug157.nv","languageId":"lumen","version":1,"text":"importar \"texto\";\nfuncion vacio main() { imprimir(texto_longitud(\"hola\")); }\n"}}}"#;

    let mut hijo = Command::new(lumen())
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let e = hijo.stdin.as_mut().unwrap();
        e.write_all(marco(init).as_bytes()).unwrap();
        e.write_all(marco(abrir).as_bytes()).unwrap();
    }
    let r = hijo.wait_with_output().unwrap();
    let salida = String::from_utf8_lossy(&r.stdout);
    assert!(
        !salida.contains("texto_longitud' no está definida"),
        "el LSP marcó como error una función importada (BUG-157): {}",
        salida
    );
}
