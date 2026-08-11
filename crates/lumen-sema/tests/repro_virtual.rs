#[test]
fn repro_virtual_flatten_stdlib() {
    // Simula exactamente el runtime WASM: stdlib leída de disco pero inyectada
    // como memory_files (misma fuente que build.rs). La feature de lumen-vm
    // aquí es "default-features=false, features=[]" (igual que lumen-wasm).
    let stdlib_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("stdlib");
    let mut files = std::collections::HashMap::new();
    for entry in std::fs::read_dir(&stdlib_dir).unwrap() {
        let entry = entry.unwrap();
        if entry.file_type().unwrap().is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".nv") {
                let content = std::fs::read_to_string(entry.path()).unwrap();
                files.insert(name, content);
            }
        }
    }
    assert!(files.contains_key("texto.nv"), "texto.nv embebido");
    assert!(files.contains_key("coleccion.nv"), "coleccion.nv embebido");
    assert!(files.contains_key("matematicas.nv"), "matematicas.nv embebido");

    let cases: Vec<(&str, &str, &str)> = vec![
        (
            "importar texto.nv",
            "importar \"texto.nv\";\nimprimir(texto_mayusculas(\"hola\"));",
            "HOLA",
        ),
        (
            "importar coleccion.nv",
            "importar \"coleccion.nv\";\nlista<numero> l = [1, 2, 3, 2];\nimprimir(coleccion_contar(l, 2));",
            "2",
        ),
        (
            "importar matematicas.nv",
            "importar \"matematicas.nv\";\nimprimir(matematicas_potencia(2, 10));",
            "1024",
        ),
        (
            "error sintáctico",
            "imprimir(1 +",
            "E020",
        ),
    ];

    for (name, src, expected) in cases {
        let mut loader = lumen_sema::ModuleLoader::with_memory_files(files.clone());
        let program = match loader.resolve_imports(src, std::path::Path::new("__lumen_mem__/main.nv"))
        {
            Ok(p) => p,
            Err(e) => {
                let msg = module_error_str(&e);
                assert!(
                    msg.contains(expected),
                    "[{}] resolve_imports error: {}",
                    name,
                    msg
                );
                eprintln!("[{}] ok (error de compile): {}", name, msg);
                continue;
            }
        };

        let sema = lumen_sema::SemanticAnalyzer::new();
        let mut prog = program;
        let sem_errors = sema.analyze(&mut prog);
        if !sem_errors.is_empty() {
            let msgs = sem_errors
                .iter()
                .map(|e| e.message.clone())
                .collect::<Vec<_>>();
            let joined = msgs.join("; ");
            assert!(
                joined.contains(expected),
                "[{}] errores semánticos: {}",
                name,
                joined
            );
            eprintln!("[{}] ok (error semántico): {}", name, joined);
            continue;
        }

        let ir = lumen_ir::IRBuilder::new().build(&prog);
        let (bc, _w) = lumen_codegen::Codegen::new().generate(&ir);
        let mut vm = lumen_vm::vm::VM::new(bc);
        match vm.run() {
            Ok(()) => {}
            Err(e) => {
                eprintln!("[{}] VM ERROR: {:?}", name, e);
                panic!("[{}] vm run failed: {}", name, e);
            }
        }
        let out = vm.output().join("\n");
        assert_eq!(out, expected, "[{}]", name);
        eprintln!("[{}] ok: {}", name, out);
    }
}

fn module_error_str(e: &lumen_sema::ModuleError) -> String {
    use lumen_sema::ModuleError;
    match e {
        ModuleError::Io { path, message } => format!("{}: {}", path.display(), message),
        ModuleError::Lex { details, .. } => details.join("\n"),
        ModuleError::Parse { details, .. } => details.join("\n"),
        ModuleError::Circular { path, span } => format!(
            "Import circular en {}:{}:{}",
            path.display(),
            span.start.line,
            span.start.col
        ),
    }
}
