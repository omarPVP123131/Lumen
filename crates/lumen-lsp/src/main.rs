// LUMEN LSP Server — Language Server Protocol
// Proporciona diagnósticos (errores) en vivo para editores como VS Code

use std::io::{self, BufRead, Read, Write};

fn main() {
    eprintln!("LUMEN LSP Server v1.5.0 — iniciado");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Read Content-Length header + JSON body
    loop {
        let mut header = String::new();
        let mut content_length = 0usize;

        // Read headers
        loop {
            header.clear();
            if stdin.lock().read_line(&mut header).is_err() {
                break;
            }
            let trimmed = header.trim();
            if trimmed.is_empty() {
                break;
            }
            if let Some(len) = trimmed.strip_prefix("Content-Length: ") {
                content_length = len.trim().parse().unwrap_or(0);
            }
        }

        if content_length == 0 {
            continue;
        }

        // Read JSON body
        let mut body = vec![0u8; content_length];
        if stdin.lock().read_exact(&mut body).is_err() {
            break;
        }

        let request: serde_json::Value = match serde_json::from_slice(&body) {
            Ok(v) => v,
            Err(_) => continue,
        };

        let method = request["method"].as_str().unwrap_or("");
        let id = request["id"].clone();

        match method {
            "initialize" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "capabilities": {
                            "textDocumentSync": 1, // Full sync
                            "completionProvider": {
                                "resolveProvider": false,
                                "triggerCharacters": [".", ":"]
                            },
                            "definitionProvider": true,
                            "hoverProvider": true,
                            "diagnosticProvider": {
                                "interFileDependencies": false,
                                "workspaceDiagnostics": false
                            }
                        },
                        "serverInfo": {"name": "lumen-lsp", "version": "1.5.0"}
                    }
                });
                send_response(&mut stdout, &response);
            }
            "initialized" => {}
            "textDocument/didOpen" | "textDocument/didChange" => {
                let uri = request["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("");
                let text = if method == "textDocument/didOpen" {
                    request["params"]["textDocument"]["text"]
                        .as_str()
                        .unwrap_or("")
                } else {
                    request["params"]["contentChanges"][0]["text"]
                        .as_str()
                        .unwrap_or("")
                };

                let diagnostics = analyze(text, uri);

                let notification = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "textDocument/publishDiagnostics",
                    "params": {
                        "uri": uri,
                        "diagnostics": diagnostics
                    }
                });
                send_response(&mut stdout, &notification);
            }
            "textDocument/completion" => {
                let completions = get_completions();
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": completions
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/definition" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": serde_json::Value::Null
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/hover" => {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "contents": {
                            "kind": "markdown",
                            "value": "### LÚMEN Symbol\nElemento de código LÚMEN"
                        }
                    }
                });
                send_response(&mut stdout, &response);
            }
            "shutdown" => {
                let response = serde_json::json!({"jsonrpc": "2.0", "id": id, "result": null});
                send_response(&mut stdout, &response);
            }
            "exit" => {
                break;
            }
            _ => {}
        }
    }
}

fn analyze(source: &str, _uri: &str) -> Vec<serde_json::Value> {
    let mut diagnostics = Vec::new();

    // Lexer
    let lexer = lumen_lexer::Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    for e in &lex_errors {
        diagnostics.push(serde_json::json!({
            "range": {
                "start": {"line": e.pos.line.saturating_sub(1) as u32, "character": e.pos.col.saturating_sub(1) as u32},
                "end": {"line": e.pos.line.saturating_sub(1) as u32, "character": e.pos.col as u32}
            },
            "severity": 1, // Error
            "code": e.code,
            "source": "lumen-lexer",
            "message": format!("{} — {}", e.message, e.suggestion)
        }));
    }

    if !lex_errors.is_empty() {
        return diagnostics;
    }

    // Parser
    let parser = lumen_parser::Parser::new(tokens);
    let (mut program, parse_errors) = parser.parse();
    for e in &parse_errors {
        diagnostics.push(serde_json::json!({
            "range": {
                "start": {"line": e.span.start.line.saturating_sub(1) as u32, "character": e.span.start.col.saturating_sub(1) as u32},
                "end": {"line": e.span.end.line.saturating_sub(1) as u32, "character": e.span.end.col as u32}
            },
            "severity": 1,
            "code": e.code,
            "source": "lumen-parser",
            "message": format!("{} — {}", e.message, e.suggestion)
        }));
    }

    if !parse_errors.is_empty() {
        return diagnostics;
    }

    // Semantic analysis
    let sema = lumen_sema::SemanticAnalyzer::new();
    let sem_errors = sema.analyze(&mut program);
    for e in &sem_errors {
        diagnostics.push(serde_json::json!({
            "range": {
                "start": {"line": e.span.start.line.saturating_sub(1) as u32, "character": e.span.start.col.saturating_sub(1) as u32},
                "end": {"line": e.span.end.line.saturating_sub(1) as u32, "character": e.span.end.col as u32}
            },
            "severity": 1,
            "code": e.code,
            "source": "lumen-sema",
            "message": format!("{} — {}", e.message, e.suggestion)
        }));
    }

    diagnostics
}

fn get_completions() -> Vec<serde_json::Value> {
    let keywords = [
        "funcion",
        "function",
        "entero",
        "integer",
        "decimal",
        "float",
        "texto",
        "string",
        "booleano",
        "boolean",
        "si",
        "if",
        "sino",
        "else",
        "mientras",
        "while",
        "para",
        "for",
        "en",
        "in",
        "elegir",
        "match",
        "caso",
        "case",
        "defecto",
        "default",
        "romper",
        "break",
        "continuar",
        "continue",
        "retornar",
        "return",
        "estructura",
        "struct",
        "rasgo",
        "trait",
        "impl",
        "para",
        "for",
        "importar",
        "import",
        "tipo",
        "const",
        "sea",
        "let",
        "algun",
        "ninguno",
        "exito",
        "error",
        "verdadero",
        "falso",
    ];

    keywords
        .iter()
        .map(|k| {
            serde_json::json!({
                "label": k,
                "kind": 14, // Keyword
                "detail": "Palabra clave de LÚMEN"
            })
        })
        .collect()
}

fn send_response(stdout: &mut impl Write, response: &serde_json::Value) {
    let body = serde_json::to_string(response).unwrap_or_default();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let _ = stdout.write_all(header.as_bytes());
    let _ = stdout.write_all(body.as_bytes());
    let _ = stdout.flush();
}
