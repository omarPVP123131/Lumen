//! Lexer nativo (v3.5.10) — puerto Rust de `stdlib/compiler/lexer.nv`.
//!
//! Produce EXACTAMENTE la misma estructura de tokens que el lexer LÚMEN:
//! un `Value::Map` con claves "0".."cnt-1" → token `{t,v,linea,col}`,
//! el token EOF en la clave `a_texto(cnt)` y `"cnt"` → Entero(cnt).
//!
//! Tipos: 1=Ident 2=Numero 3=String 4=Oper 5=Punt 6=Kw 99=EOF.
//!
//! Nota: el lexer LÚMEN tiene un bug latente (crash) si un literal decimal
//! aparece en la posición 0 del fuente (`chars[st-1]` con st=0 → índice -1).
//! Aquí se guarda con `st==0 → true`, lo cual es idéntico para toda entrada
//! que no crashee al lexer original y corrige ese borde.

use crate::value::{FixHasher, Value};
use im::HashMap as ImMap;

const KEYWORDS: &[&str] = &[
    "si",
    "if",
    "sino",
    "else",
    "mientras",
    "while",
    "para",
    "for",
    "funcion",
    "function",
    "retornar",
    "return",
    "entero",
    "texto",
    "numero",
    "booleano",
    "decimal",
    "lista",
    "diccionario",
    "void",
    "importar",
    "romper",
    "continuar",
    "verdadero",
    "falso",
    "imprimir",
    "intentar",
    "try",
    "exito",
    "error",
    "const",
    "true",
    "false",
    "estructura",
    "struct",
    "enum",
    "elegir",
    "opcion",
    "resultado",
    "sea",
    "let",
    "en",
    "in",
    "algun",
    "some",
    "ninguno",
    "none",
    "rasgo",
    "trait",
    "impl",
    "como",
    "posponer",
    "defer",
];

fn is_ident_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_'
}
fn is_ident_cont(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}
fn is_digit(c: char) -> bool {
    c.is_ascii_digit()
}
fn is_hex(c: char) -> bool {
    c.is_ascii_digit() || ('a'..='f').contains(&c) || ('A'..='F').contains(&c)
}
fn is_punct(c: char) -> bool {
    matches!(c, ';' | '(' | ')' | '{' | '}' | ',' | '.' | '[' | ']' | ':')
}

fn token_map(t: i64, v: &str, ln: i64, cl: i64) -> Value {
    let mut m: ImMap<Value, Value, FixHasher> = ImMap::with_hasher(FixHasher::default());
    m.insert(Value::str("t"), Value::Int(t));
    m.insert(Value::str("v"), Value::str(v));
    m.insert(Value::str("linea"), Value::Int(ln));
    m.insert(Value::str("col"), Value::Int(cl));
    Value::Map(m)
}

/// Un token crudo antes del post-proceso de fusión oper+ident→keyword.
#[derive(Clone)]
struct Tok {
    t: i64,
    v: String,
    ln: i64,
    cl: i64,
}

/// Tokeniza `s`. Equivalente byte-a-byte a `lexer_tokenizar` en LÚMEN.
pub fn native_lex(s: &str) -> Value {
    // Normaliza CRLF → LF y elimina CR sueltos (igual que el lexer LÚMEN).
    let norm = s.replace("\r\n", "\n").replace('\r', "");
    let chars: Vec<char> = norm.chars().collect();
    let n = chars.len();

    let mut toks: Vec<Tok> = Vec::new();
    let mut i: usize = 0;
    let mut ln: i64 = 1;
    let mut cl: i64 = 1;

    while i < n {
        let c = chars[i];
        if c == ' ' || c == '\t' {
            i += 1;
            cl += 1;
            continue;
        }
        if c == '\n' {
            i += 1;
            ln += 1;
            cl = 1;
            continue;
        }
        if c == '\r' {
            i += 1;
            continue;
        }

        // Identificador / palabra clave
        if is_ident_start(c) {
            let st = i;
            while i < n && is_ident_cont(chars[i]) {
                i += 1;
            }
            let id: String = chars[st..i].iter().collect();
            let t = if KEYWORDS.contains(&id.as_str()) {
                6
            } else {
                1
            };
            let adv = (i - st) as i64;
            toks.push(Tok { t, v: id, ln, cl });
            cl += adv;
            continue;
        }

        // Número (decimal o hexadecimal)
        if is_digit(c) {
            let st = i;
            // Hex: 0x...
            if chars[st] == '0' && st + 1 < n && (chars[st + 1] == 'x' || chars[st + 1] == 'X') {
                i = st + 2;
                let mut hv: i64 = 0;
                while i < n && is_hex(chars[i]) {
                    let ch = chars[i];
                    let dv = if ch.is_ascii_digit() {
                        (ch as i64) - ('0' as i64)
                    } else if ('a'..='f').contains(&ch) {
                        (ch as i64) - ('a' as i64) + 10
                    } else {
                        (ch as i64) - ('A' as i64) + 10
                    };
                    hv = hv.wrapping_mul(16).wrapping_add(dv);
                    i += 1;
                }
                let adv = (i - st) as i64;
                toks.push(Tok {
                    t: 2,
                    v: hv.to_string(),
                    ln,
                    cl,
                });
                cl += adv;
                continue;
            }
            // Decimal: dígitos y un único punto interior.
            // Guarda equivalente a `chars[st-1] != "."` con st==0 → true.
            let prev_not_dot = if st == 0 { true } else { chars[st - 1] != '.' };
            let mut dot = false;
            while i < n {
                let cc = chars[i];
                if is_digit(cc) {
                    i += 1;
                    continue;
                }
                if cc == '.'
                    && i + 1 < n
                    && is_digit(chars[i + 1])
                    && !dot
                    && i > st
                    && prev_not_dot
                {
                    dot = true;
                    i += 1;
                    continue;
                }
                break;
            }
            let num: String = chars[st..i].iter().collect();
            let adv = (i - st) as i64;
            toks.push(Tok {
                t: 2,
                v: num,
                ln,
                cl,
            });
            cl += adv;
            continue;
        }

        // Rango .. o ..= (antes que punct)
        if c == '.' && i + 1 < n && chars[i + 1] == '.' {
            let mut rng = String::from("..");
            i += 2;
            cl += 2;
            if i < n && chars[i] == '=' {
                rng.push('=');
                i += 1;
                cl += 1;
            }
            toks.push(Tok {
                t: 4,
                v: rng,
                ln,
                cl,
            });
            continue;
        }

        // Puntuación
        if is_punct(c) {
            toks.push(Tok {
                t: 5,
                v: c.to_string(),
                ln,
                cl,
            });
            i += 1;
            cl += 1;
            continue;
        }

        // Comentario de línea
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }
        // Comentario de bloque
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < n && !(chars[i] == '*' && chars[i + 1] == '/') {
                i += 1;
            }
            if i + 1 < n {
                i += 2;
            }
            continue;
        }

        // Cadena con escapes
        if c == '"' {
            i += 1;
            cl += 1;
            let mut sb = String::new();
            while i < n && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < n {
                    let nxt = chars[i + 1];
                    match nxt {
                        'n' => sb.push('\n'),
                        't' => sb.push('\t'),
                        'r' => sb.push('\r'),
                        '"' => sb.push('"'),
                        '\\' => sb.push('\\'),
                        other => sb.push(other),
                    }
                    i += 2;
                    cl += 2;
                } else {
                    sb.push(chars[i]);
                    i += 1;
                    cl += 1;
                }
            }
            if i < n {
                i += 1;
                cl += 1;
            }
            toks.push(Tok {
                t: 3,
                v: sb,
                ln,
                cl,
            });
            continue;
        }

        // Operadores multi-carácter
        if i + 1 < n {
            let two: String = chars[i..i + 2].iter().collect();
            if matches!(
                two.as_str(),
                "||" | "&&" | "==" | "!=" | "<=" | ">=" | "<<" | ">>" | "++"
            ) {
                toks.push(Tok {
                    t: 4,
                    v: two,
                    ln,
                    cl,
                });
                i += 2;
                cl += 2;
                continue;
            }
        }

        // Operador de un carácter
        toks.push(Tok {
            t: 4,
            v: c.to_string(),
            ln,
            cl,
        });
        i += 1;
        cl += 1;
    }

    // Post-proceso: fusiona oper(1 car) + ident en palabra clave.
    let mut out: Vec<Tok> = Vec::with_capacity(toks.len());
    let cnt = toks.len();
    let mut ii = 0usize;
    while ii < cnt {
        let tk = &toks[ii];
        if tk.t == 4 && tk.v.chars().count() == 1 && ii + 1 < cnt {
            let nxt = &toks[ii + 1];
            if nxt.t == 1 {
                let comb = format!("{}{}", tk.v, nxt.v);
                if KEYWORDS.contains(&comb.as_str()) {
                    out.push(Tok {
                        t: 6,
                        v: comb,
                        ln: tk.ln,
                        cl: tk.cl,
                    });
                    ii += 2;
                    continue;
                }
            }
        }
        out.push(tk.clone());
        ii += 1;
    }

    // Construir el mapa final.
    let mut map: ImMap<Value, Value, FixHasher> = ImMap::with_hasher(FixHasher::default());
    let ni = out.len() as i64;
    for (idx, tk) in out.iter().enumerate() {
        map.insert(
            Value::str(idx.to_string()),
            token_map(tk.t, &tk.v, tk.ln, tk.cl),
        );
    }
    // EOF
    map.insert(Value::str(ni.to_string()), token_map(99, "", ln, cl));
    map.insert(Value::str("cnt"), Value::Int(ni));
    Value::Map(map)
}
