use std::path::Path;

fn main() {
    let target = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
    let dll = Path::new(&target).parent().unwrap().parent().unwrap()
        .join("target")
        .join(std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string()))
        .join("sqlite3.dll");
    if !dll.exists() {
        println!("cargo:warning=sqlite3.dll not found at {}", dll.display());
        println!("cargo:warning=SQLite disabled. Download sqlite3.dll from https://www.sqlite.org/download.html");
        println!("cargo:warning=and place it in target/debug/ or target/release/");
    }
    println!("cargo:rerun-if-changed=build.rs");
}
