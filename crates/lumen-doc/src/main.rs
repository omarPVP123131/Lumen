// lumen-doc — Generador de documentación HTML
// Extrae comentarios /// y genera documentación estática

use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Uso: lumen-doc <archivo.nv> [output.html]");
        std::process::exit(1);
    }

    let input = &args[1];
    let output = if args.len() > 2 {
        args[2].clone()
    } else {
        input.replace(".nv", ".html")
    };

    let source = fs::read_to_string(input).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let html = generate_docs(&source, input);
    fs::write(&output, &html).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!("✓ Documentación generada: {}", output);
}

fn generate_docs(source: &str, name: &str) -> String {
    let mut html = String::from(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8">
<title>LÚMEN Docs</title><style>body{font-family:monospace;max-width:900px;margin:auto;padding:20px;background:#1e1e1e;color:#d4d4d4}
.fn{color:#569cd6;margin:10px 0}.comment{color:#6a9955}.keyword{color:#c586c0}.type{color:#4ec9b0}
code{background:#2d2d2d;padding:2px 6px;border-radius:3px}.section{margin:20px 0;padding:10px;border-left:3px solid #569cd6}
</style></head><body><h1>LÚMEN Docs</h1>"#,
    );

    html.push_str(&format!(
        "<p>Archivo: <code>{}</code></p><hr>\n",
        escape_html(name)
    ));

    let mut current_doc = String::new();

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("///") {
            let doc = trimmed.strip_prefix("///").unwrap_or("").trim();
            if !current_doc.is_empty() {
                current_doc.push('\n');
            }
            current_doc.push_str(doc);
        } else if trimmed.starts_with("funcion ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("estructura ")
            || trimmed.starts_with("struct ")
            || trimmed.starts_with("enum ")
            || trimmed.starts_with("rasgo ")
            || trimmed.starts_with("trait ")
        {
            if !current_doc.is_empty() {
                html.push_str(&format!(
                    "<div class=\"comment\">/// {}</div>\n",
                    escape_html(&current_doc)
                ));
            }
            html.push_str(&format!(
                "<div class=\"fn\"><code>{}</code></div>\n",
                escape_html(trimmed)
            ));
            current_doc.clear();
        }
    }

    html.push_str("</body></html>");
    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs() {
        let src =
            "/// Suma dos numeros\nfuncion entero suma(entero a, entero b) { retornar a + b; }";
        let html = generate_docs(src, "test.nv");
        assert!(html.contains("Suma dos numeros"));
        assert!(html.contains("funcion"));
    }
}
