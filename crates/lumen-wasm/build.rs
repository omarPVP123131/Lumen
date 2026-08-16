// Build script: embebe la stdlib de LÚMEN (stdlib/*.nv) dentro del wasm.
// Genera OUT_DIR/embedded_stdlib.rs con `STDLIB_FILES: &[(&str, &str)]`.
// Regenerar el wasm incluye automáticamente la stdlib vigente del repo.
use std::env;
use std::fs;
use std::path::PathBuf;

fn collect_nv(dir: &std::path::Path, base_dir: &std::path::Path, entries: &mut Vec<(String, String)>) {
    if let Ok(read) = fs::read_dir(dir) {
        let mut items: Vec<_> = read.filter_map(|e| e.ok()).collect();
        items.sort_by_key(|e| e.file_name());
        for entry in items {
            let path = entry.path();
            if path.is_dir() {
                collect_nv(&path, base_dir, entries);
            } else if path.extension().is_some_and(|x| x == "nv") {
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let rel = path.to_str().unwrap().replace('\\', "/");
                entries.push((name.clone(), rel.clone()));
                if let Ok(rel_sub) = path.strip_prefix(base_dir) {
                    let sub_str = rel_sub.to_string_lossy().to_string().replace('\\', "/");
                    if sub_str != name {
                        entries.push((sub_str, rel.clone()));
                    }
                }
                println!("cargo:rerun-if-changed={}", rel);
            }
        }
    }
}

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // crates/lumen-wasm → raíz del repo → stdlib/
    let stdlib_dir = manifest_dir.join("../../stdlib");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap()).join("embedded_stdlib.rs");

    let mut raw_entries = Vec::new();
    collect_nv(&stdlib_dir, &stdlib_dir, &mut raw_entries);

    let mut entries = Vec::new();
    for (name, rel) in raw_entries {
        entries.push(format!("    (\"{}\", include_str!(\"{}\")),", name, rel));
    }

    let generated = format!(
        "// GENERADO por build.rs — no editar.\n\
         pub const STDLIB_FILES: &[(&str, &str)] = &[\n{}\n];\n",
        entries.join("\n")
    );
    fs::write(&out_path, generated).expect("no se pudo escribir embedded_stdlib.rs");
    println!("cargo:rerun-if-changed=build.rs");
    println!(
        "cargo:warning=embedded_stdlib.rs: {} entradas de stdlib embebidas",
        entries.len()
    );
}
