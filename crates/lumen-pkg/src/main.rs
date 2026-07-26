// lumen-pkg — Package Manager
// lumen install <pkg>, registry, dependency resolution
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 || args[1] != "install" {
        eprintln!("LUMEN Package Manager v1.5.0");
        eprintln!("Uso: lumen install <paquete>");
        eprintln!("      lumen install --path <ruta>");
        return;
    }

    let pkg = &args[2];
    let cache_dir = dirs()
        .unwrap_or_else(|| PathBuf::from(".lumen"))
        .join("lumen_cache");
    fs::create_dir_all(&cache_dir).ok();

    if args.len() > 3 && args[2] == "--path" {
        install_from_path(&args[3], &cache_dir);
    } else {
        install_from_registry(pkg, &cache_dir);
    }
}

fn install_from_registry(pkg: &str, cache: &Path) {
    let pkg_dir = cache.join(pkg);
    if pkg_dir.exists() {
        println!("✓ {} ya instalado en {}", pkg, pkg_dir.display());
        return;
    }

    // Try to clone from a registry URL
    let url = format!("https://github.com/lumen-pkgs/{}", pkg);
    println!("Instalando {} desde {}...", pkg, url);

    let status = Command::new("git")
        .args(["clone", &url, pkg_dir.to_str().unwrap()])
        .status();

    match status {
        Ok(s) if s.success() => println!("✓ {} instalado en {}", pkg, pkg_dir.display()),
        Ok(s) => eprintln!("Error: git clone exit {}", s),
        Err(e) => eprintln!(
            "Error: git no encontrado ({}) — instala dependencias manualmente en {}",
            e,
            cache.display()
        ),
    }
}

fn install_from_path(path: &str, cache: &Path) {
    let src = PathBuf::from(path);
    let name = src.file_name().unwrap_or_default().to_string_lossy();
    let dest = cache.join(name.as_ref());
    if !src.exists() {
        eprintln!("Error: '{}' no existe", path);
        return;
    }
    fs::create_dir_all(&dest).ok();
    copy_dir(&src, &dest);
    println!("✓ {} instalado en {}", name, dest.display());
}

fn copy_dir(src: &Path, dest: &Path) {
    if let Ok(entries) = fs::read_dir(src) {
        for entry in entries.flatten() {
            let path = entry.path();
            let target = dest.join(path.file_name().unwrap());
            if path.is_dir() {
                fs::create_dir_all(&target).ok();
                copy_dir(&path, &target);
            } else {
                fs::copy(&path, &target).ok();
            }
        }
    }
}

fn dirs() -> Option<PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|d| PathBuf::from(d).join("lumen"))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|d| PathBuf::from(d).join(".lumen"))
        })
        .or_else(|| Some(PathBuf::from(".lumen")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dirs() {
        assert!(dirs().is_some());
    }
}
