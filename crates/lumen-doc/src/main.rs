// lumen-doc — Generador de documentación HTML
use std::fs;

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

    let html = lumen_doc::generate_docs(&source, input);
    fs::write(&output, &html).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    println!("✓ Documentación generada: {}", output);
}
