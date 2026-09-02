// lumen-pkg — Package Manager
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "install" {
        eprintln!("LUMEN Package Manager v1.5.0");
        eprintln!("Uso: lumen install <paquete>");
        eprintln!("      lumen install --path <ruta>");
        return;
    }

    let cache_dir = lumen_pkg::cache_dir();
    std::fs::create_dir_all(&cache_dir).ok();

    if args.len() > 3 && args[2] == "--path" {
        lumen_pkg::install_from_path(&args[3], &cache_dir);
    } else {
        lumen_pkg::install_package(&args[2], &cache_dir);
    }
}
