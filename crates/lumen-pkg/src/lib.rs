use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn install_package(pkg: &str, cache_dir: &Path) {
    let pkg_dir = cache_dir.join(pkg);
    if pkg_dir.exists() {
        println!("✓ {} ya instalado en {}", pkg, pkg_dir.display());
        return;
    }
    let url = format!("https://github.com/lumen-pkgs/{}", pkg);
    println!("Instalando {} desde {}...", pkg, url);
    let status = Command::new("git")
        .args(["clone", &url, pkg_dir.to_str().unwrap()])
        .status();
    match status {
        Ok(s) if s.success() => println!("✓ {} instalado en {}", pkg, pkg_dir.display()),
        Ok(s) => eprintln!("Error: git clone exit {}", s),
        Err(e) => eprintln!(
            "Error: git no encontrado ({}) — instala dependencias manualmente",
            e
        ),
    }
}

pub fn install_from_path(path: &str, cache_dir: &Path) {
    let src = PathBuf::from(path);
    let name = src.file_name().unwrap_or_default().to_string_lossy();
    let dest = cache_dir.join(name.as_ref());
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

pub fn cache_dir() -> PathBuf {
    std::env::var("APPDATA")
        .ok()
        .map(|d| PathBuf::from(d).join("lumen"))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|d| PathBuf::from(d).join(".lumen"))
        })
        .unwrap_or_else(|| PathBuf::from(".lumen"))
        .join("lumen_cache")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_dir() {
        assert!(!cache_dir().as_os_str().is_empty());
    }
}
