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
        Self::create_with_template(name, "default")
    }

    pub fn create_with_template(name: &str, template: &str) -> Result<PathBuf, String> {
        let project_dir = PathBuf::from(name);
        if project_dir.exists() {
            return Err(format!("El directorio '{}' ya existe", name));
        }

        fs::create_dir_all(project_dir.join("src")).map_err(|e| format!("{}", e))?;
        fs::create_dir_all(project_dir.join("tests")).map_err(|e| format!("{}", e))?;
        fs::create_dir_all(project_dir.join("stdlib")).map_err(|e| format!("{}", e))?;
        fs::create_dir_all(project_dir.join("pkgs")).map_err(|e| format!("{}", e))?;

        let manifest_content = format!(
            r#"[proyecto]
nombre = "{}"
version = "0.1.0"
descripcion = "Proyecto LÚMEN moderno y de alto rendimiento"
autores = ["Desarrollador <usuario@ejemplo.com>"]
licencia = "MIT"
principal = "src/main.nv"
lib_dirs = ["stdlib", "pkgs"]

# English aliases
[project]
name = "{}"
version = "0.1.0"
description = "High-performance modern LÚMEN project"
main = "src/main.nv"

[dependencias]
# Agrega paquetes con: lumen add <paquete>
# Add packages with: lumen add <package>
"#,
            name, name
        );

        let manifest_path = project_dir.join("lumen.toml");
        fs::write(&manifest_path, &manifest_content).map_err(|e| format!("{}", e))?;

        let (main_content, test_content) = match template {
            "ia" | "ai" | "ml" => (
                format!(
                    r#"// ============================================================================
// Proyecto de Inteligencia Artificial: {}
// Generado con: lumen new {} --template ia
// ============================================================================

importar ingles;
importar "tensor.nv";
importar "nn.nv";

imprimir("╔══════════════════════════════════════════════════════╗");
imprimir("║   🤖 Proyecto de IA & Redes Neuronales LÚMEN        ║");
imprimir("╚══════════════════════════════════════════════════════╝\n");

// 1. Grafo de Diferenciación Automática (Autograd)
tensor_GrafoAutograd grafo = tensor_autograd_nuevo();
grafo = grafo.variable(2.5); // x
grafo = grafo.variable(3.0); // w
grafo = grafo.multiplicar(0, 1); // x * w = 7.5
grafo = grafo.backward(2);

imprimir("• Forward pass (x * w): ", grafo.valor(2));
imprimir("• Gradiente dL/dx: ", grafo.gradiente(0));

// 2. Transformer Multi-Head Self-Attention
nn_BloqueTransformer transformer = nn_transformer_crear(8, 2);
lista<decimal> embeddings = [0.1, 0.4, 0.8, -0.2, 0.5, 0.9, -0.4, 0.2];
lista<decimal> salida = transformer.procesar(embeddings);
imprimir("• Inferencia Transformer (dim=8): ", salida);

imprimir("\n🚀 Para entrenar o compilar a binario nativo:");
imprimir("   lumen run src/main.nv");
imprimir("   lumen build --native src/main.nv\n");
"#,
                    name, name
                ),
                r#"importar "testing.nv";
importar "tensor.nv";

funcion void test_tensor_dot() {
    lista<decimal> a = [1.0, 2.0];
    lista<decimal> b = [3.0, 4.0];
    decimal d = tensor_producto_punto(a, b);
    testing_afirmar_igual(d, 11.0);
}
"#.to_string(),
            ),
            "web" | "api" | "servidor" => (
                format!(
                    r#"// ============================================================================
// Microservicio Web & REST API: {}
// Generado con: lumen new {} --template web
// ============================================================================

importar ingles;
importar "servidor.nv";

servidor_ServidorWeb app = servidor_crear(8080);

// Rutas REST, WebSockets y Documentación OpenAPI 3.0 Automática
app = app.ruta_get("/api/saludo", "handle_saludo");
app = app.ruta_ws("/ws/chat", "handle_chat");
app = app.habilitar_swagger("/docs", "API Oficial de {}", "1.0.0");

imprimir("🚀 Servidor iniciado en http://0.0.0.0:8080/");
imprimir("📖 Swagger UI disponible en http://0.0.0.0:8080/docs\n");

app.iniciar();
"#,
                    name, name, name
                ),
                r#"importar "testing.nv";
importar "servidor.nv";

funcion void test_json_resp() {
    servidor_RespuestaHTTP r = servidor_respuesta_json("{\"ok\": true}");
    testing_afirmar_igual(r.codigo, 200);
}
"#.to_string(),
            ),
            "uni" | "universidad" | "academico" => (
                format!(
                    r#"// ============================================================================
// Proyecto Académico / Universidad: {}
// Curso: Algoritmos y Estructuras de Datos en LÚMEN
// ============================================================================

importar ingles;
importar "matrices.nv";

imprimir("╔══════════════════════════════════════════════════════╗");
imprimir("║   🎓 LÚMEN — Algoritmos y Estructuras de Datos      ║");
imprimir("╚══════════════════════════════════════════════════════╝\n");

// 1. Álgebra Lineal y Matrices 2D
lista<lista<decimal>> matriz_a = [
    [3.0, 8.0],
    [4.0, 6.0]
];
decimal det = matrices_determinante_2x2(matriz_a);
imprimir("• Determinante det(A): ", det);

// 2. Consultas Declarativas LINQ/SQL
lista<entero> notas = [85, 92, 58, 74, 99, 62, 88];
lista<entero> aprobadas = consultar n en notas donde n >= 70 seleccionar n;
imprimir("• Calificaciones aprobadas: ", aprobadas);
"#,
                    name
                ),
                r#"importar "testing.nv";
importar "matrices.nv";

funcion void test_algebra_determinante() {
    lista<lista<decimal>> m = [[2.0, 1.0], [1.0, 2.0]];
    decimal det = matrices_determinante_2x2(m);
    testing_afirmar_igual(det, 3.0);
}
"#.to_string(),
            ),
            _ => (
                format!(
                    r#"// ============================================================================
// Proyecto: {}
// Generado con: lumen new {}
// ============================================================================

importar ingles;

// 1. Estructura con métodos inherentes
estructura Tarea {{
    id: entero,
    titulo: texto,
    completada: booleano,
}}

impl Tarea {{
    funcion texto describir(este) {{
        texto estado = este.completada ? "COMPLETADA" : "PENDIENTE";
        retornar f"[#{{este.id}}] {{este.titulo}} ({{estado}})";
    }}
}}

// 2. Punto de entrada principal
imprimir("╔══════════════════════════════════════════════════════╗");
imprimir(f"║   ¡Bienvenido a tu proyecto LÚMEN: {:<18} ║");
imprimir("╚══════════════════════════════════════════════════════╝\n");

Tarea mi_tarea = Tarea {{
    id: 1,
    titulo: "Aprender LÚMEN y crear software increíble",
    completada: verdadero,
}};

imprimir("• Tarea inicial : ", mi_tarea.describir());
imprimir("\n🚀 Para compilar a binario nativo ultra-rápido ejecuta:");
imprimir("   lumen build --native src/main.nv\n");
"#,
                    name, name, name
                ),
                r#"importar "testing.nv";

funcion void test_suma() {
    entero r = 2 + 2;
    testing_afirmar_igual(r, 4);
}

funcion void test_interpolacion() {
    texto nombre = "LÚMEN";
    texto saludo = f"Hola {nombre}";
    testing_afirmar_igual(saludo, "Hola LÚMEN");
}
"#.to_string(),
            ),
        };

        let main_path = project_dir.join("src/main.nv");
        fs::write(&main_path, &main_content).map_err(|e| format!("{}", e))?;

        let test_path = project_dir.join("tests/test_main.nv");
        fs::write(&test_path, &test_content).map_err(|e| format!("{}", e))?;

        let readme_content = format!(
            r#"# Proyecto `{}` — LÚMEN

Proyecto configurado con la plantilla **{}** de LÚMEN v2.4.4.

## 🚀 Comandos Rápidos

```bash
lumen run src/main.nv             # Ejecutar en desarrollo
lumen test tests/test_main.nv     # Ejecutar suite de pruebas
lumen check .                     # Comprobar todo el proyecto
lumen bench src/main.nv           # Benchmark de rendimiento
lumen build --native src/main.nv  # Compilar binario nativo optimizado
```
"#,
            name, template
        );
        let readme_path = project_dir.join("README.md");
        fs::write(&readme_path, &readme_content).map_err(|e| format!("{}", e))?;

        let gitignore_content = r#"*.nvc
*.o
*.obj
*.exe
target/
output/
"#;
        let gitignore_path = project_dir.join(".gitignore");
        fs::write(&gitignore_path, gitignore_content).map_err(|e| format!("{}", e))?;

        Ok(project_dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_create_and_load() {
        let tmp = env::temp_dir().join("test_lumen_project_std");
        let _ = fs::remove_dir_all(&tmp);
        let dir = ProjectManifest::create(&tmp.to_string_lossy()).unwrap();
        assert!(dir.join("lumen.toml").exists());
        assert!(dir.join("src/main.nv").exists());
        let _ = fs::remove_dir_all(&tmp);
    }

    #[test]
    fn test_create_template_ia() {
        let tmp = env::temp_dir().join("test_lumen_project_ia");
        let _ = fs::remove_dir_all(&tmp);
        let dir = ProjectManifest::create_with_template(&tmp.to_string_lossy(), "ia").unwrap();
        assert!(dir.join("lumen.toml").exists());
        assert!(dir.join("src/main.nv").exists());
        let src = fs::read_to_string(dir.join("src/main.nv")).unwrap();
        assert!(src.contains("tensor_autograd_nuevo"));
        let _ = fs::remove_dir_all(&tmp);
    }
}
