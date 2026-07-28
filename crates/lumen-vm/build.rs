use std::path::Path;

fn main() {
    // Silently check for sqlite3.dll — no warning on CI
    let target = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let dll = Path::new(&target)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target")
        .join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()))
        .join("sqlite3.dll");
    if dll.exists() {
        println!("cargo:rustc-cfg=sqlite_available");
    }
}
