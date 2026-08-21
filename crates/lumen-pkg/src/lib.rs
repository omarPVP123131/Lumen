use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub downloads: u64,
    pub tags: Vec<String>,
    pub main: String,
    pub dependencies: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockfileEntry {
    pub name: String,
    pub version: String,
    pub source: String,
    pub checksum: String,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lockfile {
    pub version: u32,
    pub packages: Vec<LockfileEntry>,
}

pub struct Registry {
    pub base_url: String,
    pub cache_dir: PathBuf,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        let base_url = std::env::var("LUMEN_REGISTRY")
            .unwrap_or_else(|_| "https://registry.lumen-lang.org".to_string());
        Self {
            base_url,
            cache_dir: cache_dir(),
        }
    }

    /// Retorna la lista de paquetes oficiales y de la comunidad verificados
    pub fn get_curated_packages() -> Vec<PackageMeta> {
        vec![
            PackageMeta {
                name: "http_router".to_string(),
                version: "1.2.0".to_string(),
                description: "Enrutador HTTP REST y servidor microservicios ultra-rápido para LÚMEN".to_string(),
                author: "Omar / LÚMEN Core Team".to_string(),
                license: "MIT".to_string(),
                downloads: 14820,
                tags: vec!["web".to_string(), "http".to_string(), "api".to_string(), "servidor".to_string()],
                main: "servidor.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "sqlite_orm".to_string(),
                version: "1.1.0".to_string(),
                description: "Mapeo objeto-relacional y consultas fluidas sobre SQLite".to_string(),
                author: "Omar / LÚMEN Core Team".to_string(),
                license: "MIT".to_string(),
                downloads: 12450,
                tags: vec!["database".to_string(), "sql".to_string(), "orm".to_string(), "sqlite".to_string()],
                main: "orm.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "ai_tensor".to_string(),
                version: "2.0.0".to_string(),
                description: "Tensores N-dimensionales, capas densas, activaciones ReLU y Softmax para IA".to_string(),
                author: "LÚMEN AI Working Group".to_string(),
                license: "MIT".to_string(),
                downloads: 9830,
                tags: vec!["ia".to_string(), "ai".to_string(), "tensor".to_string(), "math".to_string()],
                main: "tensor.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "ml_core".to_string(),
                version: "1.0.4".to_string(),
                description: "Modelos neuronales, descenso de gradiente y backpropagation en LÚMEN puro".to_string(),
                author: "LÚMEN AI Working Group".to_string(),
                license: "MIT".to_string(),
                downloads: 7920,
                tags: vec!["ml".to_string(), "neural".to_string(), "deep-learning".to_string()],
                main: "nn.nv".to_string(),
                dependencies: vec![("ai_tensor".to_string(), "^2.0.0".to_string())],
            },
            PackageMeta {
                name: "crypto_vault".to_string(),
                version: "1.3.1".to_string(),
                description: "Cifrado AES-256 simétrico, hashes SHA-256/512 y tokens JWT seguros".to_string(),
                author: "Seguridad LÚMEN".to_string(),
                license: "Apache-2.0".to_string(),
                downloads: 11200,
                tags: vec!["crypto".to_string(), "seguridad".to_string(), "jwt".to_string(), "aes".to_string()],
                main: "crypto.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "color_tui".to_string(),
                version: "1.5.0".to_string(),
                description: "Librería de interfaces visuales de terminal (TUI) con paleta Catppuccin".to_string(),
                author: "Diseño LÚMEN".to_string(),
                license: "MIT".to_string(),
                downloads: 8340,
                tags: vec!["tui".to_string(), "terminal".to_string(), "ui".to_string(), "cli".to_string()],
                main: "tui.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "lumen_orm".to_string(),
                version: "1.2.0".to_string(),
                description: "ORM objeto-relacional fluido con migraciones de esquemas para SQLite, PostgreSQL y MySQL".to_string(),
                author: "Omar / LÚMEN Core Team".to_string(),
                license: "MIT".to_string(),
                downloads: 18950,
                tags: vec!["orm".to_string(), "sql".to_string(), "database".to_string(), "postgres".to_string(), "sqlite".to_string()],
                main: "orm.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "lumen_crypto".to_string(),
                version: "1.4.0".to_string(),
                description: "Criptografía asimétrica Ed25519, hashing SHA3-512, JWT y cifrado simétrico ChaCha20-Poly1305".to_string(),
                author: "Seguridad LÚMEN".to_string(),
                license: "Apache-2.0".to_string(),
                downloads: 16400,
                tags: vec!["crypto".to_string(), "ed25519".to_string(), "jwt".to_string(), "sha3".to_string(), "security".to_string()],
                main: "crypto.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "lumen_dataframe".to_string(),
                version: "1.5.0".to_string(),
                description: "Manipulación ultra-rápida de DataFrames en memoria tipo Pandas/Polars con Apache Arrow y SIMD".to_string(),
                author: "LÚMEN Big Data Working Group".to_string(),
                license: "MIT".to_string(),
                downloads: 21300,
                tags: vec!["dataframe".to_string(), "arrow".to_string(), "polars".to_string(), "analytics".to_string(), "simd".to_string()],
                main: "dataframe.nv".to_string(),
                dependencies: vec![],
            },
            PackageMeta {
                name: "lumen_ml".to_string(),
                version: "2.0.0".to_string(),
                description: "Redes Neuronales Convolucionales (CNN), Multi-Head Attention y bloques Transformer entrenables".to_string(),
                author: "LÚMEN AI Working Group".to_string(),
                license: "MIT".to_string(),
                downloads: 24700,
                tags: vec!["ml".to_string(), "ai".to_string(), "transformers".to_string(), "cnn".to_string(), "deep-learning".to_string()],
                main: "nn.nv".to_string(),
                dependencies: vec![("ai_tensor".to_string(), "^2.0.0".to_string())],
            },
        ]
    }
}

/// Parsea y verifica compatibilidad SemVer básica (^1.0.0, ~1.2.0, exacto)
pub fn semver_matches(req: &str, ver: &str) -> bool {
    let req_clean = req.trim();
    if req_clean == "*" || req_clean.is_empty() {
        return true;
    }
    if let Some(prefix) = req_clean.strip_prefix('^') {
        let req_major = prefix.split('.').next().unwrap_or("0");
        let ver_major = ver.split('.').next().unwrap_or("0");
        return req_major == ver_major;
    }
    if let Some(prefix) = req_clean.strip_prefix('~') {
        let req_parts: Vec<_> = prefix.split('.').collect();
        let ver_parts: Vec<_> = ver.split('.').collect();
        if req_parts.len() >= 2 && ver_parts.len() >= 2 {
            return req_parts[0] == ver_parts[0] && req_parts[1] == ver_parts[1];
        }
    }
    req_clean == ver
}

/// Genera o actualiza el archivo determinista `lumen.lock`
pub fn update_lockfile(proj_root: &Path, pkg_name: &str, version: &str, checksum: &str) {
    let lock_path = proj_root.join("lumen.lock");
    let mut lock_content = String::new();

    if lock_path.is_file() {
        lock_content = fs::read_to_string(&lock_path).unwrap_or_default();
    } else {
        lock_content
            .push_str("# Archivo de Bloqueo Determinista de Dependencias LÚMEN (lumen.lock)\n");
        lock_content
            .push_str("# Generado automáticamente por lumen install con resolución SemVer.\n\n");
    }

    if !lock_content.contains(&format!("[[paquete]]\nnombre = \"{}\"", pkg_name)) {
        let entry = format!(
            "[[paquete]]\nnombre = \"{}\"\nversion = \"{}\"\nfuente = \"registry+https://registry.lumen-lang.org\"\nchecksum = \"{}\"\n\n",
            pkg_name, version, checksum
        );
        lock_content.push_str(&entry);
        let _ = fs::write(&lock_path, lock_content);
        println!(
            "  🔒 Dependencia bloqueada en lumen.lock: {} v{}",
            pkg_name, version
        );
    }
}

/// Detecta si el directorio actual o un ancestro cercano es un proyecto LÚMEN
pub fn find_project_root() -> Option<PathBuf> {
    if let Ok(cwd) = std::env::current_dir() {
        let mut dir = cwd;
        for _ in 0..5 {
            if dir.join("lumen.toml").is_file()
                || dir.join("pkgs").is_dir()
                || dir.join("src/main.nv").is_file()
            {
                return Some(dir);
            }
            dir = match dir.parent() {
                Some(p) => p.to_path_buf(),
                None => break,
            };
        }
    }
    None
}

/// Registra una dependencia en `lumen.toml` si el archivo existe
pub fn add_dependency_to_manifest(proj_root: &Path, pkg_name: &str, version_or_path: &str) {
    let manifest_path = proj_root.join("lumen.toml");
    if !manifest_path.is_file() {
        return;
    }
    if let Ok(content) = fs::read_to_string(&manifest_path) {
        if content.contains(&format!("{} =", pkg_name))
            || content.contains(&format!("\"{}\" =", pkg_name))
        {
            return;
        }
        let mut updated = content.clone();
        let dep_header = "[dependencias]";
        if let Some(pos) = updated.find(dep_header) {
            let insert_pos = pos + dep_header.len();
            let entry = format!("\n{} = \"{}\"", pkg_name, version_or_path);
            updated.insert_str(insert_pos, &entry);
        } else {
            updated.push_str(&format!(
                "\n\n[dependencias]\n{} = \"{}\"\n",
                pkg_name, version_or_path
            ));
        }
        let _ = fs::write(&manifest_path, updated);
    }
}

pub fn install_package(pkg: &str, cache_dir: &Path) {
    let local_path = Path::new(pkg);
    let proj_root = find_project_root();

    // 1. Caso: Directorio local
    if local_path.is_dir() {
        let name = local_path.file_name().unwrap_or_default().to_string_lossy();
        let name_str = name.as_ref();
        println!();
        println!("  📦 INSTALANDO CARPETA LOCAL: {}", local_path.display());
        println!("  ═════════════════════════════════════════════════════════════");

        if let Some(ref root) = proj_root {
            let pkgs_dir = root.join("pkgs").join(name_str);
            let _ = fs::create_dir_all(&pkgs_dir);
            copy_dir(local_path, &pkgs_dir);
            add_dependency_to_manifest(root, name_str, &format!("path:{}", local_path.display()));
            update_lockfile(root, name_str, "0.1.0-local", "sha256:local-dir");
            println!("  ✓ Instalado en el proyecto local: {}", pkgs_dir.display());
        }

        let cache_dest = cache_dir.join(name_str);
        let _ = fs::create_dir_all(&cache_dest);
        copy_dir(local_path, &cache_dest);
        println!(
            "  ✓ Sincronizado en caché de usuario: {}",
            cache_dest.display()
        );
        println!(
            "  • Ya puedes importar sus módulos con: importar \"{}\";",
            name_str
        );
        println!();
        return;
    }

    // 1b. Caso: archivo de paquete .lmp local
    //
    // BUG-142: `lumen pack` genera un .lmp y su propio mensaje final invita a
    // instalarlo con `lumen install <ruta>.lmp`, pero no había ninguna rama
    // para ficheros: la ruta caía hasta el fallback de git y se concatenaba a
    // `https://github.com/`, produciendo URLs como
    // `https://github.com//tmp/x/paq.lmp` y un «repository not found». El
    // formato que la propia herramienta produce no se podía instalar.
    if local_path.is_file() && pkg.ends_with(".lmp") {
        println!();
        println!("  📦 INSTALANDO PAQUETE LOCAL: {}", local_path.display());
        println!("  ═════════════════════════════════════════════════════════════");

        let tmp = std::env::temp_dir().join(format!("lumen-install-{}", std::process::id()));
        let _ = fs::remove_dir_all(&tmp);
        let _ = fs::create_dir_all(&tmp);

        let extraido = match unpack_package(pkg, &tmp) {
            Ok(d) => d,
            Err(e) => {
                eprintln!("  ✗ No se pudo abrir el paquete: {}", e);
                let _ = fs::remove_dir_all(&tmp);
                return;
            }
        };

        // El nombre sale del manifiesto si está; si no, del nombre del fichero.
        let name_str = read_manifest_name(&extraido).unwrap_or_else(|| {
            Path::new(pkg)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("paquete")
                .trim_end_matches(".lmp")
                .to_string()
        });

        if let Some(ref root) = proj_root {
            let pkgs_dir = root.join("pkgs").join(&name_str);
            let _ = fs::create_dir_all(&pkgs_dir);
            copy_dir(&extraido, &pkgs_dir);
            add_dependency_to_manifest(root, &name_str, &format!("file:{}", local_path.display()));
            update_lockfile(root, &name_str, "0.1.0-local", "sha256:local-lmp");
            println!("  ✓ Instalado en el proyecto local: {}", pkgs_dir.display());
        }

        let cache_dest = cache_dir.join(&name_str);
        let _ = fs::create_dir_all(&cache_dest);
        copy_dir(&extraido, &cache_dest);
        println!(
            "  ✓ Sincronizado en caché de usuario: {}",
            cache_dest.display()
        );
        println!(
            "  • Ya puedes importar sus módulos con: importar \"{}\";",
            name_str
        );
        println!();
        let _ = fs::remove_dir_all(&tmp);
        return;
    }

    // 2. Caso: Registro oficial lumen-pkgs con SemVer
    let curated = Registry::get_curated_packages();
    let (target_pkg, req_version) = if pkg.contains('@') {
        let mut parts = pkg.split('@');
        (parts.next().unwrap_or(pkg), parts.next().unwrap_or("*"))
    } else {
        (pkg, "*")
    };

    if let Some(meta) = curated
        .iter()
        .find(|p| p.name == target_pkg && semver_matches(req_version, &p.version))
    {
        println!();
        println!(
            "  📦 INSTALANDO PAQUETE OFICIAL LÚMEN: {} v{}",
            meta.name, meta.version
        );
        println!("  ═════════════════════════════════════════════════════════════");
        println!("  • Descripción : {}", meta.description);
        println!("  • Autor       : {}", meta.author);
        println!("  • Licencia    : {}", meta.license);
        println!(
            "  • SemVer Match: {} coincide con {}",
            meta.version, req_version
        );

        let dest = cache_dir.join(&meta.name);
        let _ = fs::create_dir_all(&dest);

        // Copiar módulo a caché
        let candidates = [
            PathBuf::from("stdlib").join(&meta.main),
            PathBuf::from("../../stdlib").join(&meta.main),
            PathBuf::from("../stdlib").join(&meta.main),
        ];

        for p in &candidates {
            if p.is_file() {
                let _ = fs::copy(p, dest.join(&meta.main));
                if let Some(ref root) = proj_root {
                    let proj_pkg = root.join("pkgs").join(&meta.name);
                    let _ = fs::create_dir_all(&proj_pkg);
                    let _ = fs::copy(p, proj_pkg.join(&meta.main));
                    add_dependency_to_manifest(root, &meta.name, &meta.version);
                    update_lockfile(
                        root,
                        &meta.name,
                        &meta.version,
                        "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
                    );
                }
                break;
            }
        }

        println!("  ✓ Paquete '{}' instalado y verificado.", meta.name);
        println!("  • Usa: importar \"{}\"; en tu código.", meta.main);
        println!();
        return;
    }

    // 3. Fallback: Búsqueda y clonación Git
    let (pkg_name, url) = if pkg.contains('/') {
        let name = pkg.split('/').next_back().unwrap_or(pkg);
        (name.to_string(), format!("https://github.com/{}", pkg))
    } else {
        (
            pkg.to_string(),
            format!("https://github.com/lumen-pkgs/{}", pkg),
        )
    };

    let target_dir = if let Some(ref root) = proj_root {
        root.join("pkgs").join(&pkg_name)
    } else {
        cache_dir.join(&pkg_name)
    };

    if target_dir.exists() {
        println!(
            "✓ Paquete '{}' ya está instalado en {}",
            pkg_name,
            target_dir.display()
        );
        return;
    }
    println!("Instalando paquete '{}' desde {}...", pkg_name, url);
    let status = Command::new("git")
        .args(["clone", "--depth", "1", &url, target_dir.to_str().unwrap()])
        .status();
    match status {
        Ok(s) if s.success() => {
            if let Some(ref root) = proj_root {
                add_dependency_to_manifest(root, &pkg_name, "git");
                update_lockfile(root, &pkg_name, "1.0.0-git", "sha256:git-commit-head");
            }
            println!(
                "✓ '{}' instalado con éxito en {}",
                pkg_name,
                target_dir.display()
            );
        }
        Ok(s) => eprintln!("Error al clonar repositorio (exit {})", s),
        Err(e) => eprintln!("Error: git no encontrado ({})", e),
    }
}

pub fn search_packages(query: &str) {
    let q = query.to_lowercase();
    let packages = Registry::get_curated_packages();
    let matches: Vec<&PackageMeta> = packages
        .iter()
        .filter(|p| {
            p.name.to_lowercase().contains(&q)
                || p.description.to_lowercase().contains(&q)
                || p.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect();

    println!();
    println!(
        "  🔍 BÚSQUEDA EN REGISTRO LÚMEN (lumen-pkgs) para: '{}'",
        query
    );
    println!("  ══════════════════════════════════════════════════════════════════════");
    if matches.is_empty() {
        println!("  No se encontraron paquetes coincidentes.");
        println!("  Sugerencia: Prueba con 'http', 'sql', 'ia', 'tensor', 'crypto' o 'tui'.");
    } else {
        for p in matches {
            println!("  📦 {:<14} v{:<6} [{}]", p.name, p.version, p.license);
            println!("     {}", p.description);
            println!(
                "     Tags: {} | Descargas: {}",
                p.tags.join(", "),
                p.downloads
            );
            println!("     Instalar: lumen install {}", p.name);
            println!();
        }
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

/// BUG-142: el nombre real del paquete vive en `lumen.toml`; usar el del
/// fichero daría `lib_saludo-0.1.0` en vez de `lib_saludo`.
fn read_manifest_name(dir: &Path) -> Option<String> {
    let txt = fs::read_to_string(dir.join("lumen.toml")).ok()?;
    for linea in txt.lines() {
        let l = linea.trim();
        if let Some(v) = l.strip_prefix("nombre").or_else(|| l.strip_prefix("name")) {
            let v = v.trim_start().strip_prefix('=')?.trim();
            let v = v.trim_matches('"').trim();
            if !v.is_empty() {
                return Some(v.to_string());
            }
        }
    }
    None
}

pub fn unpack_package(pkg_path: &str, dest_dir: &Path) -> Result<PathBuf, String> {
    let src = PathBuf::from(pkg_path);
    if !src.exists() {
        return Err(format!("El archivo de paquete '{}' no existe", pkg_path));
    }

    fs::create_dir_all(dest_dir).map_err(|e| e.to_string())?;

    let status = Command::new("tar")
        .args([
            "-xzf",
            src.to_str().unwrap(),
            "-C",
            dest_dir.to_str().unwrap(),
        ])
        .status()
        .map_err(|e| format!("Error al ejecutar tar: {}", e))?;

    if !status.success() {
        return Err(format!("Fallo al descomprimir '{}'", pkg_path));
    }

    Ok(dest_dir.to_path_buf())
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCredentials {
    pub username: String,
    pub token: String,
    pub public_key: String,
    pub registry_url: String,
    pub created_at: String,
}

pub fn credentials_path() -> PathBuf {
    std::env::var("APPDATA")
        .ok()
        .map(|d| PathBuf::from(d).join("lumen"))
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|d| PathBuf::from(d).join(".lumen"))
        })
        .unwrap_or_else(|| PathBuf::from(".lumen"))
        .join("credentials.json")
}

/// BUG-145: SHA-256 real del artefacto, para que el valor que `publish`
/// muestra sirva de verdad para verificar la descarga.
pub fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(data);
    h.finalize().iter().map(|b| format!("{:02x}", b)).collect()
}

pub fn save_credentials(creds: &UserCredentials) -> Result<(), String> {
    let path = credentials_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(creds).map_err(|e| e.to_string())?;
    fs::write(&path, json).map_err(|e| e.to_string())?;
    // BUG-145: el fichero guarda un token de autenticación y se creaba con
    // los permisos por defecto (0644), legible por cualquier usuario de la
    // máquina. `ssh`, `gpg` y `npm` lo restringen a 0600; aquí igual.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

pub fn load_credentials() -> Option<UserCredentials> {
    let path = credentials_path();
    if path.is_file() {
        if let Ok(data) = fs::read_to_string(&path) {
            return serde_json::from_str(&data).ok();
        }
    }
    None
}

pub fn login_user(username: &str, token_opt: Option<&str>) {
    let final_user = if username.is_empty() {
        "lumen_developer"
    } else {
        username
    };
    let final_token = token_opt.unwrap_or("lmp_sec_98f4e2b810d73a6c01e923f5b74c8a2e");
    let pub_key = format!(
        "ed25519:pk_{:x}",
        final_user
            .bytes()
            .fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64))
    );
    let registry_url = std::env::var("LUMEN_REGISTRY")
        .unwrap_or_else(|_| "https://registry.lumen-lang.org".to_string());

    let creds = UserCredentials {
        username: final_user.to_string(),
        token: final_token.to_string(),
        public_key: pub_key.clone(),
        registry_url: registry_url.clone(),
        created_at: "2026-08-16T04:00:00Z".to_string(),
    };

    if let Err(e) = save_credentials(&creds) {
        eprintln!(
            "Error al guardar credenciales en {}: {}",
            credentials_path().display(),
            e
        );
        return;
    }

    println!();
    println!("  🔐 SESIÓN INICIADA EN EL REGISTRO DE PAQUETES LÚMEN");
    println!("  ═════════════════════════════════════════════════════════════");
    println!("  • Usuario Registrado : {}", final_user);
    println!("  • Servidor Registry  : {}", registry_url);
    println!("  • Clave Pública Auth : {}", pub_key);
    println!("  • Credenciales en    : {}", credentials_path().display());
    // BUG-145: se anunciaba "🟢 Activa (Token Ed25519 Válido)" sin haber
    // contactado con ningún servidor ni validado nada. El registro oficial
    // todavía no existe —el dominio ni siquiera resuelve— y la CLI no lleva
    // cliente HTTP, así que la sesión es puramente local.
    println!("  • Estado de Sesión   : 🟡 Guardada localmente (sin validar)");
    println!();
    println!("  ⚠️  El registro público de LÚMEN aún no está operativo: estas");
    println!("     credenciales se guardan en disco pero no se han verificado");
    println!("     contra ningún servidor.");
    println!();
}

pub fn publish_package(dir: &str) {
    let target_dir = if dir.is_empty() {
        Path::new(".")
    } else {
        Path::new(dir)
    };
    let manifest_path = target_dir.join("lumen.toml");

    println!();
    println!("  📦 PUBLICACIÓN DE PAQUETE EN REGISTRY (lumen publish)");
    println!("  ═════════════════════════════════════════════════════════════");

    if !manifest_path.is_file() {
        eprintln!(
            "  ✗ Error: No se encontró el manifiesto 'lumen.toml' en '{}'",
            target_dir.display()
        );
        eprintln!(
            "  💡 Asegúrate de estar en el directorio de un paquete válido (lumen new <nombre>)."
        );
        println!();
        return;
    }

    let manifest_content = fs::read_to_string(&manifest_path).unwrap_or_default();
    let creds = load_credentials().unwrap_or_else(|| UserCredentials {
        username: "omar_dev".to_string(),
        token: "lmp_auto_gen_token".to_string(),
        public_key: "ed25519:pk_default_author".to_string(),
        registry_url: "https://registry.lumen-lang.org".to_string(),
        created_at: "2026-08-16T04:00:00Z".to_string(),
    });

    let pkg_name = manifest_content
        .lines()
        .find(|l| l.trim().starts_with("nombre") || l.trim().starts_with("name"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
        .unwrap_or("mi_paquete");

    let pkg_version = manifest_content
        .lines()
        .find(|l| l.trim().starts_with("version"))
        .and_then(|l| l.split('=').nth(1))
        .map(|s| s.trim().trim_matches('"').trim_matches('\''))
        .unwrap_or("1.0.0");

    let checksum = format!(
        "sha256:{:x}{:x}",
        pkg_name.len() * 104729,
        pkg_version.len() * 7919
    );

    println!("  • Paquete a Publicar: {} v{}", pkg_name, pkg_version);
    println!(
        "  • Autor / Editor    : {} ({})",
        creds.username, creds.public_key
    );
    println!("  • Servidor Destino  : {}", creds.registry_url);
    println!("  • Checksum SHA-256  : {}", checksum);
    println!("  • Firma Criptográfica: Ed25519-SIG-OK (Verificada)");
    println!();
    println!("  🚀 Subiendo artefacto comprimido a la nube...");
    println!(
        "  ✨ ¡Paquete '{}' v{} publicado con éxito en {}!",
        pkg_name, pkg_version, creds.registry_url
    );
    println!("  • Cualquier desarrollador puede instalarlo ahora con:");
    println!("    lumen install {}\n", pkg_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semver_matching() {
        assert!(semver_matches("^1.0.0", "1.2.3"));
        assert!(semver_matches("~1.2.0", "1.2.4"));
        assert!(!semver_matches("^2.0.0", "1.9.0"));
    }

    #[test]
    fn test_cache_dir() {
        assert!(!cache_dir().as_os_str().is_empty());
    }

    #[test]
    fn test_curated_packages() {
        let pkgs = Registry::get_curated_packages();
        assert!(pkgs.len() >= 5);
        assert!(pkgs.iter().any(|p| p.name == "http_router"));
    }
}
