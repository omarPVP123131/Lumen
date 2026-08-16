// ============================================================================
// LÚMEN Language Server Protocol (LSP Pro) — v2.4.6
// Soporte Completo: Semantic Tokens, Inlay Hints, Signature Help, Code Actions,
// Diagnóstico en Tiempo Real, Hover, Definición y Autocompletado Inteligente
// ============================================================================

use std::collections::HashMap;
use std::io::{self, BufRead, Read, Write};
use lumen_lexer::token::{TokenKind};
use lumen_lexer::Lexer;
use lumen_parser::Parser;
use lumen_sema::SemanticAnalyzer;

pub fn run_lsp() {
    eprintln!("LÚMEN LSP Server Pro v2.4.6 — Semantic Tokens, Inlay Hints & Code Actions");

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    let mut doc_cache: HashMap<String, String> = HashMap::new();

    loop {
        let mut header = String::new();
        let mut content_length = 0usize;

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
                            "textDocumentSync": 1,
                            "completionProvider": {
                                "resolveProvider": false,
                                "triggerCharacters": [".", ":", ">", "|", "f", "<"]
                            },
                            "definitionProvider": true,
                            "hoverProvider": true,
                            "renameProvider": true,
                            "signatureHelpProvider": {
                                "triggerCharacters": ["(", ","]
                            },
                            "inlayHintProvider": true,
                            "codeActionProvider": {
                                "codeActionKinds": ["quickfix", "refactor.extract", "source.fixAll"]
                            },
                            "semanticTokensProvider": {
                                "legend": {
                                    "tokenTypes": [
                                        "keyword", "type", "function", "variable", 
                                        "parameter", "struct", "string", "number", 
                                        "operator", "comment", "macro"
                                    ],
                                    "tokenModifiers": ["declaration", "readonly", "static", "defaultLibrary"]
                                },
                                "full": true
                            },
                            "diagnosticProvider": {
                                "interFileDependencies": false,
                                "workspaceDiagnostics": false
                            }
                        },
                        "serverInfo": {"name": "lumen-lsp-pro", "version": "2.4.6"}
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

                doc_cache.insert(uri.to_string(), text.to_string());
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
            "textDocument/semanticTokens/full" => {
                let uri = request["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("");
                let doc = doc_cache.get(uri).cloned().unwrap_or_default();
                let data = compute_semantic_tokens(&doc);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "data": data
                    }
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/inlayHint" => {
                let uri = request["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("");
                let doc = doc_cache.get(uri).cloned().unwrap_or_default();
                let hints = compute_inlay_hints(&doc);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": hints
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/signatureHelp" => {
                let uri = request["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("");
                let line = request["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
                let col = request["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

                let doc = doc_cache.get(uri).cloned().unwrap_or_default();
                let sig = compute_signature_help(&doc, line, col);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": sig
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/codeAction" => {
                let uri = request["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("");
                let doc = doc_cache.get(uri).cloned().unwrap_or_default();
                let range = &request["params"]["range"];
                let diagnostics = request["params"]["context"]["diagnostics"]
                    .as_array()
                    .cloned()
                    .unwrap_or_default();
                let actions = compute_code_actions(&doc, uri, range, &diagnostics);
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": actions
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/completion" => {
                let completions = get_smart_completions();
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": completions
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/hover" => {
                let uri = request["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("");
                let line = request["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
                let col = request["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

                let doc = doc_cache.get(uri).cloned().unwrap_or_default();
                let hover_info = get_hover_info(&doc, line, col);

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "contents": {
                            "kind": "markdown",
                            "value": hover_info
                        }
                    }
                });
                send_response(&mut stdout, &response);
            }
            "textDocument/definition" => {
                let uri = request["params"]["textDocument"]["uri"]
                    .as_str()
                    .unwrap_or("");
                let line = request["params"]["position"]["line"].as_u64().unwrap_or(0) as usize;
                let col = request["params"]["position"]["character"].as_u64().unwrap_or(0) as usize;

                let doc = doc_cache.get(uri).cloned().unwrap_or_default();
                let loc = find_definition(&doc, uri, line, col);

                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": loc
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

// ── 1. Semantic Tokens Engine (LSP 3.17 Relative Deltas) ─────────────

pub fn compute_semantic_tokens(source: &str) -> Vec<u32> {
    let mut tokens_data = Vec::new();
    let lexer = Lexer::new(source);
    let (tokens, _) = lexer.tokenize();

    let mut last_line = 0usize;
    let mut last_col = 0usize;

    for tok in tokens {
        let (token_type_idx, modifier_mask) = match &tok.kind {
            TokenKind::Si | TokenKind::If | TokenKind::Sino | TokenKind::Else
            | TokenKind::Mientras | TokenKind::While | TokenKind::Para | TokenKind::For
            | TokenKind::Funcion | TokenKind::Function | TokenKind::Retornar | TokenKind::Return
            | TokenKind::Elegir | TokenKind::Match | TokenKind::Caso | TokenKind::Case
            | TokenKind::Defecto | TokenKind::Default | TokenKind::Estructura | TokenKind::Struct
            | TokenKind::Importar | TokenKind::Import | TokenKind::Como | TokenKind::As
            | TokenKind::Posponer | TokenKind::Defer | TokenKind::Intentar | TokenKind::Try
            | TokenKind::Atrapar | TokenKind::Catch | TokenKind::Prestado | TokenKind::Borrowed
            | TokenKind::Dueno | TokenKind::Owner | TokenKind::Mut | TokenKind::Mutable
            | TokenKind::EnTiempoCompilacion | TokenKind::Comptime
            | TokenKind::Ensamblador | TokenKind::Asm
            | TokenKind::BloqueC | TokenKind::BloqueRust => (0, 0), // keyword

            TokenKind::Entero | TokenKind::Integer | TokenKind::Decimal | TokenKind::Float
            | TokenKind::Texto | TokenKind::String | TokenKind::Booleano | TokenKind::Boolean
            | TokenKind::Numero | TokenKind::Number | TokenKind::Lista | TokenKind::Array
            | TokenKind::Resultado | TokenKind::Result | TokenKind::Opcion | TokenKind::Option => (1, 1), // type

            TokenKind::Ident(name) => {
                if name.starts_with("fn_") || name.contains('_') && !name.starts_with("var_") {
                    (2, 0) // function
                } else if name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    (5, 0) // struct
                } else {
                    (3, 0) // variable
                }
            }

            TokenKind::StrLiteral(_) | TokenKind::FStrLiteral(_) => (6, 0), // string
            TokenKind::NumLiteral(_) => (7, 0), // number
            TokenKind::Equal | TokenKind::Plus | TokenKind::Minus | TokenKind::Star 
            | TokenKind::Slash | TokenKind::PipeGreater | TokenKind::Ampersand => (8, 0), // operator
            TokenKind::Comment(_) => (9, 0), // comment
            _ => continue,
        };

        let current_line = tok.span.start.line.saturating_sub(1);
        let current_col = tok.span.start.col.saturating_sub(1);
        let len = (tok.span.end.col.saturating_sub(tok.span.start.col)).max(1);

        let delta_line = current_line.saturating_sub(last_line);
        let delta_col = if delta_line == 0 {
            current_col.saturating_sub(last_col)
        } else {
            current_col
        };

        tokens_data.push(delta_line as u32);
        tokens_data.push(delta_col as u32);
        tokens_data.push(len as u32);
        tokens_data.push(token_type_idx);
        tokens_data.push(modifier_mask);

        last_line = current_line;
        last_col = current_col;
    }

    tokens_data
}

// ── 2. Inlay Hints Engine (Tipos Deducidos y Parámetros) ─────────────

pub fn compute_inlay_hints(source: &str) -> Vec<serde_json::Value> {
    let mut hints = Vec::new();
    for (line_idx, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Deducir tipos en 'sea x = 42;' o 'let x = "hola";'
        if trimmed.starts_with("sea ") || trimmed.starts_with("let ") {
            if let Some(eq_pos) = line.find('=') {
                let ident_part = line[..eq_pos].trim();
                let var_name = ident_part.split_whitespace().last().unwrap_or("");
                let value_part = line[eq_pos + 1..].trim().trim_end_matches(';');
                
                let deduced_type = if value_part.starts_with('"') {
                    ": texto"
                } else if value_part.contains('.') {
                    ": decimal"
                } else if value_part.starts_with('[') {
                    ": lista"
                } else if value_part == "verdadero" || value_part == "falso" {
                    ": booleano"
                } else {
                    ": entero"
                };

                let char_idx = line.find(var_name).unwrap_or(0) + var_name.len();
                hints.push(serde_json::json!({
                    "position": {"line": line_idx as u32, "character": char_idx as u32},
                    "label": deduced_type,
                    "kind": 1, // Type hint
                    "paddingLeft": true
                }));
            }
        }
    }
    hints
}

// ── 3. Signature Help Engine ─────────────────────────────────────────

pub fn compute_signature_help(source: &str, line_idx: usize, _col_idx: usize) -> serde_json::Value {
    let lines: Vec<&str> = source.lines().collect();
    if line_idx >= lines.len() {
        return serde_json::Value::Null;
    }
    let current_line = lines[line_idx];

    // Detectar llamadas comunes de la stdlib
    if current_line.contains("imprimir(") || current_line.contains("print(") {
        return serde_json::json!({
            "signatures": [{
                "label": "funcion vacio imprimir(cualquiera... valores)",
                "documentation": "Imprime valores a la consola formateados con saltos de línea.",
                "parameters": [
                    {"label": "cualquiera... valores", "documentation": "Valores a imprimir"}
                ]
            }],
            "activeSignature": 0,
            "activeParameter": 0
        });
    }

    if current_line.contains("vector_db_buscar(") {
        return serde_json::json!({
            "signatures": [{
                "label": "funcion lista<ResultadoBusqueda> vector_db_buscar(BaseVectores db, lista<decimal> consulta, entero top_k, texto metrica)",
                "documentation": "Búsqueda semántica de los vecinos más cercanos por similitud coseno o euclidiana.",
                "parameters": [
                    {"label": "BaseVectores db"},
                    {"label": "lista<decimal> consulta"},
                    {"label": "entero top_k"},
                    {"label": "texto metrica"}
                ]
            }],
            "activeSignature": 0,
            "activeParameter": 1
        });
    }

    serde_json::Value::Null
}

// ── 4. Code Actions Engine (Corrección Rápida y Refactors) ────────────

pub fn compute_code_actions(
    _source: &str,
    uri: &str,
    range: &serde_json::Value,
    diagnostics: &[serde_json::Value]
) -> Vec<serde_json::Value> {
    let mut actions = Vec::new();

    for diag in diagnostics {
        let msg = diag["message"].as_str().unwrap_or("");
        let code = diag["code"].as_str().unwrap_or("");

        if code == "E012" || msg.contains(';') {
            actions.push(serde_json::json!({
                "title": "💡 Corregir: Agregar ';' al final de la sentencia",
                "kind": "quickfix",
                "isPreferred": true,
                "diagnostics": [diag],
                "edit": {
                    "changes": {
                        uri: [{
                            "range": range,
                            "newText": ";\n"
                        }]
                    }
                }
            }));
        }

        if code == "E042" || msg.contains("no está definida") {
            actions.push(serde_json::json!({
                "title": "💡 Importar módulo de la biblioteca estándar (stdlib)",
                "kind": "quickfix",
                "isPreferred": true,
                "diagnostics": [diag],
                "edit": {
                    "changes": {
                        uri: [{
                            "range": {
                                "start": {"line": 0, "character": 0},
                                "end": {"line": 0, "character": 0}
                            },
                            "newText": "importar \"matematicas.nv\";\nimportar \"vector_db.nv\";\n"
                        }]
                    }
                }
            }));
        }
    }

    // Refactor general
    actions.push(serde_json::json!({
        "title": "⚡ Formatear documento con LÚMEN Formatter (lumen fmt)",
        "kind": "source.fixAll"
    }));

    actions
}

// ── 5. Diagnósticos y Análisis Semántico ──────────────────────────────

fn analyze(source: &str, _uri: &str) -> Vec<serde_json::Value> {
    let mut diagnostics = Vec::new();

    let lexer = Lexer::new(source);
    let (tokens, lex_errors) = lexer.tokenize();
    for e in &lex_errors {
        diagnostics.push(serde_json::json!({
            "range": {
                "start": {"line": e.pos.line.saturating_sub(1) as u32, "character": e.pos.col.saturating_sub(1) as u32},
                "end": {"line": e.pos.line.saturating_sub(1) as u32, "character": e.pos.col as u32}
            },
            "severity": 1,
            "code": e.code,
            "source": "lumen-lexer",
            "message": format!("{} — {}", e.message, e.suggestion)
        }));
    }

    if !lex_errors.is_empty() {
        return diagnostics;
    }

    let parser = Parser::new(tokens);
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

    let sema = SemanticAnalyzer::new();
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

fn get_word_at(doc: &str, line_idx: usize, col_idx: usize) -> String {
    let lines: Vec<&str> = doc.lines().collect();
    if line_idx >= lines.len() {
        return String::new();
    }
    let line = lines[line_idx];
    let chars: Vec<char> = line.chars().collect();
    if col_idx >= chars.len() {
        return String::new();
    }

    let mut start = col_idx;
    while start > 0 && (chars[start - 1].is_alphanumeric() || chars[start - 1] == '_') {
        start -= 1;
    }

    let mut end = col_idx;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_') {
        end += 1;
    }

    chars[start..end].iter().collect()
}

fn get_hover_info(doc: &str, line_idx: usize, col_idx: usize) -> String {
    let word = get_word_at(doc, line_idx, col_idx);
    if word.is_empty() {
        return "### LÚMEN\nLenguaje de programación nativo bilingüe".to_string();
    }

    match word.as_str() {
        "funcion" | "function" => "```lumen\nfuncion <tipo> nombre(<parametros>) { ... }\n```\n**Función LÚMEN** — Define un bloque ejecutable con parámetros tipados y valor de retorno.".to_string(),
        "estructura" | "struct" => "```lumen\nestructura Nombre { campo1: tipo, campo2: tipo }\n```\n**Estructura de Datos** — Define un tipo compuesto con semántica de valor.".to_string(),
        "entero" | "integer" => "```lumen\nentero (i64)\n```\nTipo primitivo de número entero con signo de 64 bits.".to_string(),
        "decimal" | "float" => "```lumen\ndecimal (f64)\n```\nTipo primitivo numérico de punto flotante de precisión doble (64 bits).".to_string(),
        "texto" | "string" => "```lumen\ntexto (UTF-8)\n```\nCadena de caracteres inmutable codificada en UTF-8 con interpolación `f\"...\"` y slicing.".to_string(),
        "booleano" | "boolean" => "```lumen\nbooleano (verdadero / falso)\n```\nTipo booleano lógico para control de flujo y expresiones de predicado.".to_string(),
        "posponer" | "defer" => "```lumen\nposponer { ... }\n```\n**Gestor de Recursos RAII** — Garantiza la ejecución de limpieza al salir del ámbito actual.".to_string(),
        "imprimir" | "print" => "```lumen\nimprimir(valor1, valor2, ...)\n```\nFunción integrada para imprimir valores formateados a la salida estándar.".to_string(),
        "elegir" | "match" => "```lumen\nelegir (expresion) {\n    caso Patron: ...\n    defecto: ...\n}\n```\nPattern matching exhaustivo con soporte para rangos, enums y or-patterns.".to_string(),
        "prestado" | "borrowed" => "```lumen\nprestado T / borrowed T\n```\n**Borrow Checker Zero-GC** — Referencia inmutable de cero-copia verificada estáticamente en compilación.".to_string(),
        "dueno" | "owner" => "```lumen\ndueno T / owner T\n```\n**Propiedad Lineal** — Valor con transferencia de titularidad única y control estricto de vida.".to_string(),
        "en_tiempo_compilacion" | "comptime" => "```lumen\nen_tiempo_compilacion { expr }\n```\n**Metaprogramación Comptime** — Evalúa cálculos y tipos durante la compilación sin costo en runtime.".to_string(),
        other => format!("### Símbolo LÚMEN `{}`\nElemento de código analizado por el servidor LSP.", other),
    }
}

fn find_definition(doc: &str, uri: &str, line_idx: usize, col_idx: usize) -> serde_json::Value {
    let word = get_word_at(doc, line_idx, col_idx);
    if word.is_empty() {
        return serde_json::Value::Null;
    }

    for (i, line) in doc.lines().enumerate() {
        if line.contains(&format!("funcion {}", word))
            || line.contains(&format!("function {}", word))
            || line.contains(&format!("estructura {}", word))
            || line.contains(&format!("struct {}", word))
            || line.contains(&format!("enum {}", word))
        {
            return serde_json::json!({
                "uri": uri,
                "range": {
                    "start": {"line": i as u32, "character": 0},
                    "end": {"line": i as u32, "character": line.len() as u32}
                }
            });
        }
    }

    serde_json::Value::Null
}

fn get_smart_completions() -> Vec<serde_json::Value> {
    let mut items = Vec::new();

    let keywords = [
        ("funcion", "Define una nueva función", "funcion entero $1($2) {\n    retornar $0;\n}"),
        ("estructura", "Define una estructura de datos", "estructura $1 {\n    $0\n}"),
        ("impl", "Implementa métodos o rasgos", "impl $1 {\n    $0\n}"),
        ("prestado", "Declara una referencia prestada (Zero-GC)", "prestado $0"),
        ("dueno", "Declara propiedad lineal única", "dueno $0"),
        ("en_tiempo_compilacion", "Evalúa en tiempo de compilación", "en_tiempo_compilacion { $0 }"),
        ("ensamblador", "Bloque de ensamblador nativo inline", "ensamblador {\n    \"$0\"\n}"),
        ("bloque_c", "Bloque C99 inline", "bloque_c {\n    \"$0\"\n}"),
        ("bloque_rust", "Bloque Rust inline", "bloque_rust {\n    \"$0\"\n}"),
        ("posponer", "Ejecuta bloque de limpieza al salir del scope", "posponer {\n    $0\n}"),
        ("elegir", "Pattern matching de casos", "elegir ($1) {\n    caso $2: $0\n    defecto: \n}"),
        ("si", "Condicional si verdadero", "si $1 {\n    $0\n}"),
        ("sino", "Rama alternativa", "sino {\n    $0\n}"),
        ("mientras", "Bucle mientras condición sea verdadera", "mientras $1 {\n    $0\n}"),
        ("para", "Bucle para cada elemento", "para ($1) {\n    $0\n}"),
        ("importar", "Importa un módulo stdlib o paquete", "importar \"$1.nv\";"),
        ("retornar", "Retorna un valor de la función", "retornar $0;"),
    ];

    for (k, doc, snippet) in keywords {
        items.push(serde_json::json!({
            "label": k,
            "kind": 14,
            "detail": doc,
            "insertText": snippet,
            "insertTextFormat": 2
        }));
    }

    let types = [
        ("entero", "Tipo entero de 64 bits (i64)"),
        ("decimal", "Tipo decimal de 64 bits (f64)"),
        ("texto", "Tipo cadena de texto UTF-8"),
        ("booleano", "Tipo lógico verdadero/falso"),
        ("lista<entero>", "Lista dinámica de enteros"),
        ("opcion<texto>", "Valor opcional presente o ausente"),
        ("resultado<entero, texto>", "Resultado de éxito o error"),
    ];

    for (t, doc) in types {
        items.push(serde_json::json!({
            "label": t,
            "kind": 7,
            "detail": doc
        }));
    }

    let builtins = [
        ("imprimir", "Imprime valores a la consola"),
        ("leer", "Lee una línea desde la entrada estándar"),
        ("largo", "Retorna la longitud de una lista o texto"),
        ("agregar", "Agrega un elemento a una lista"),
        ("a_texto", "Convierte cualquier valor a texto"),
        ("a_entero", "Convierte texto a entero con resultado"),
        ("a_decimal", "Convierte texto a decimal con resultado"),
        ("vector_db_crear", "Crea una base de datos vectorial nativa"),
        ("vector_db_buscar", "Búsqueda por similitud coseno RAG"),
        ("ia_cuantizar_int8", "Cuantiza matriz de pesos a INT8 W8A16"),
        ("nexus_crear_app", "Crea una aplicación web Nexus con OpenAPI 3.0"),
        ("postgres_conectar", "Conecta a PostgreSQL vía Wire Protocol 3.0"),
        ("redis_conectar", "Conecta a Redis con protocolo RESP3"),
        ("ui_estado_crear", "Crea un hook de estado reactivo Virtual DOM"),
    ];

    for (b, doc) in builtins {
        items.push(serde_json::json!({
            "label": b,
            "kind": 3,
            "detail": doc
        }));
    }

    items
}

fn send_response(stdout: &mut impl Write, response: &serde_json::Value) {
    let body = serde_json::to_string(response).unwrap_or_default();
    let header = format!("Content-Length: {}\r\n\r\n", body.len());
    let _ = stdout.write_all(header.as_bytes());
    let _ = stdout.write_all(body.as_bytes());
    let _ = stdout.flush();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_semantic_tokens_generation() {
        let code = "entero x = 42;\nfuncion entero suma(entero a) { retornar a + 1; }";
        let tokens = compute_semantic_tokens(code);
        assert!(!tokens.is_empty());
        assert_eq!(tokens.len() % 5, 0); // Formato LSP de 5 enteros por token
    }

    #[test]
    fn test_inlay_hints() {
        let code = "sea total = 100;\nlet saludo = \"hola\";";
        let hints = compute_inlay_hints(code);
        assert_eq!(hints.len(), 2);
    }

    #[test]
    fn test_signature_help() {
        let code = "imprimir(42);";
        let sig = compute_signature_help(code, 0, 9);
        assert!(!sig.is_null());
    }

    #[test]
    fn test_code_actions_quickfix() {
        let diag = serde_json::json!({
            "code": "E012",
            "message": "Se esperaba ';'"
        });
        let range = serde_json::json!({"start": {"line": 0, "character": 5}, "end": {"line": 0, "character": 5}});
        let actions = compute_code_actions("numero x = 10", "file:///test.nv", &range, &[diag]);
        assert!(!actions.is_empty());
    }
}
