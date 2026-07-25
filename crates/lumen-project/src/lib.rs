use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub authors: Vec<String>,
    pub main: String,
    pub lib_dirs: Vec<String>,
}

impl ProjectManifest {
    pub fn load(path: &str) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("No se pudo leer '{}': {}", path, e))?;
        toml::from_str(&content).map_err(|e| format!("TOML inválido: {}", e))
    }

    pub fn create(name: &str) -> Result<PathBuf, String> {
        let project_dir = PathBuf::from(name);
        if project_dir.exists() {
            return Err(format!("El directorio '{}' ya existe", name));
        }

        fs::create_dir_all(project_dir.join("src")).map_err(|e| format!("{}", e))?;
        fs::create_dir_all(project_dir.join("stdlib")).map_err(|e| format!("{}", e))?;

        let manifest = ProjectManifest {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            description: format!("Proyecto {}", name),
            authors: vec![],
            main: "src/main.nv".to_string(),
            lib_dirs: vec!["stdlib".to_string()],
        };

        let toml_str = format!(
            r#"[project]
name = "{}"
version = "{}"
description = "{}"
authors = []
main = "src/main.nv"
lib_dirs = ["stdlib"]
"#,
            manifest.name, manifest.version, manifest.description
        );

        let manifest_path = project_dir.join("lumen.toml");
        fs::write(&manifest_path, &toml_str).map_err(|e| format!("{}", e))?;

        let main_path = project_dir.join("src/main.nv");
        fs::write(
            &main_path,
            "importar ingles;\n\nimprimir(\"¡Hola desde LÚMEN!\");\n",
        )
        .map_err(|e| format!("{}", e))?;

        Ok(project_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_create_and_load() {
        let tmp = env::temp_dir().join("test_lumen_project");
        let _ = fs::remove_dir_all(&tmp);
        let dir = ProjectManifest::create(&tmp.to_string_lossy()).unwrap();
        assert!(dir.join("lumen.toml").exists());
        assert!(dir.join("src/main.nv").exists());
        let _ = fs::remove_dir_all(&tmp);
    }
}
