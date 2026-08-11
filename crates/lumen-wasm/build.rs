// Build script: embebe la stdlib de LÚMEN (stdlib/*.nv) dentro del wasm.
// Genera OUT_DIR/embedded_stdlib.rs con `STDLIB_FILES: &[(&str, &str)]`.
// Regenerar el wasm incluye automáticamente la stdlib vigente del repo.
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // crates/lumen-wasm → raíz del repo → stdlib/
    let stdlib_dir = manifest_dir.join("../../stdlib");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("embedded_stdlib.rs");

    let mut entries = Vec::new();
    if let Ok(read) = fs::read_dir(&stdlib_dir) {
        let mut files: Vec<_> = read
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |x| x == "nv"))
            .collect();
        files.sort_by_key(|e| e.file_name());
        for entry in files {
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            // Ruta del include_str! relativa al archivo generado (OUT_DIR).
            let rel = path
                .to_str()
                .unwrap()
                .replace('\\', "/");
            entries.push(format!(
                "    (\"{}\", include_str!(\"{}\")),",
                name, rel
            ));
            // Recompilar si un archivo de stdlib cambia
            println!("cargo:rerun-if-changed={}", rel);
        }
    }

    let generated = format!(
        "// GENERADO por build.rs — no editar.\n\
         pub const STDLIB_FILES: &[(&str, &str)] = &[\n{}\n];\n",
        entries.join("\n")
    );
    fs::write(&out_path, generated).expect("no se pudo escribir embedded_stdlib.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:warning=embedded_stdlib.rs: {} archivos de stdlib embebidos", entries.len());
}
