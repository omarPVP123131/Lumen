use crate::value::Value;
use im::HashMap;
use crate::value::FixHasher;

pub fn parse_json(s: &str) -> Result<Value, String> {
    let mut p = Parser { s, i: 0 };
    p.skip_ws();
    p.parse_value()
}

fn json_to_lumen(j: JValue) -> Value {
    match j {
        JValue::Null => Value::Void,
        JValue::Bool(b) => Value::Bool(b),
        JValue::Int(n) => Value::Int(n),
        JValue::Float(f) => Value::Float(f),
        JValue::Str(s) => Value::Str(s),
        JValue::Array(arr) => Value::arr(arr.into_iter().map(json_to_lumen).collect()),
        JValue::Object(map) => {
            let mut m = HashMap::with_hasher(FixHasher::default());
            for (k, v) in map {
                m.insert(Value::Str(k), json_to_lumen(v));
            }
            Value::Map(m)
        }
    }
}

fn lumen_to_json(v: &Value) -> JValue {
    match v {
        Value::Void | Value::Opcion(None) => JValue::Null,
        Value::Bool(b) => JValue::Bool(*b),
        Value::Int(n) => JValue::Int(*n),
        Value::Float(f) => JValue::Float(*f),
        Value::Str(s) => JValue::str(s.clone()),
        Value::Array(arr) => JValue::arr(arr.iter().map(lumen_to_json).collect()),
        Value::Map(map) => {
            let mut entries = Vec::new();
            for (k, v) in map {
                let ks = match k {
                    Value::Str(s) => s.clone(),
                    other => format!("{other}"),
                };
                entries.push((ks, lumen_to_json(v)));
            }
            JValue::Object(entries)
        }
        _ => JValue::Null,
    }
}

#[derive(Clone, Debug)]
enum JValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<JValue>),
    Object(Vec<(String, JValue)>),
}

pub fn json_stringify(j: &JValue) -> String {
    match j {
        JValue::Null => "null".to_string(),
        JValue::Bool(b) => (if *b { "true" } else { "false" }).to_string(),
        JValue::Int(n) => n.to_string(),
        JValue::Float(f) => f.to_string(),
        JValue::Str(s) => format!("\"{}\"", escape_str(s)),
        JValue::Array(arr) => {
            let items: Vec<String> = arr.iter().map(json_stringify).collect();
            format!("[{}]", items.join(","))
        }
        JValue::Object(map) => {
            let items: Vec<String> = map.iter()
                .map(|(k, v)| format!("\"{}\":{}", escape_str(k), json_stringify(v)))
                .collect();
            format!("{{{}}}", items.join(","))
        }
    }
}

fn escape_str(s: &str) -> String {
    let mut res = String::new();
    for c in s.chars() {
        match c {
            '"' => res.push_str("\\\""),
            '\\' => res.push_str("\\\\"),
            '\n' => res.push_str("\\n"),
            '\t' => res.push_str("\\t"),
            '\r' => res.push_str("\\r"),
            c if (c as u32) < 0x20 => res.push_str(&format!("\\u{:04x}", c as u32)),
            c => res.push(c),
        }
    }
    res
}

struct Parser<'a> {
    s: &'a str,
    i: usize,
}

impl Parser<'_> {
    fn skip_ws(&mut self) {
        while self.i < self.s.len() {
            let c = self.s.as_bytes()[self.i] as char;
            if c == ' ' || c == '\t' || c == '\n' || c == '\r' { self.i += 1; } else { break; }
        }
    }

    fn peek(&self) -> Option<char> {
        self.s[self.i..].chars().next()
    }

    fn advance(&mut self) -> char {
        let c = self.s[self.i..].chars().next().unwrap();
        self.i += c.len_utf8();
        c
    }

    fn expect(&mut self, expected: char) -> Result<(), String> {
        self.skip_ws();
        if self.peek() == Some(expected) { self.advance(); Ok(()) }
        else { Err(format!("Expected '{expected}' at position {}", self.i)) }
    }

    fn parse_value(&mut self) -> Result<JValue, String> {
        self.skip_ws();
        match self.peek() {
            Some('"') => self.parse_string().map(JValue::Str),
            Some('{') => self.parse_object(),
            Some('[') => self.parse_array(),
            Some('t') => { self.i += 4; Ok(JValue::Bool(true)) }
            Some('f') => { self.i += 5; Ok(JValue::Bool(false)) }
            Some('n') => { self.i += 4; Ok(JValue::Null) }
            Some(c) if c == '-' || c.is_ascii_digit() => self.parse_number(),
            Some(c) => Err(format!("Unexpected character '{c}' at position {}", self.i)),
            None => Err("Unexpected end of input".to_string()),
        }
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect('"')?;
        let mut res = String::new();
        loop {
            match self.peek() {
                Some('"') => { self.advance(); return Ok(res); }
                Some('\\') => {
                    self.advance();
                    match self.advance() {
                        '"' => res.push('"'),
                        '\\' => res.push('\\'),
                        '/' => res.push('/'),
                        'n' => res.push('\n'),
                        't' => res.push('\t'),
                        'r' => res.push('\r'),
                        'u' => {
                            let hex: String = (0..4).map(|_| self.advance()).collect();
                            let code = u32::from_str_radix(&hex, 16).map_err(|e| e.to_string())?;
                            res.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                        }
                        c => res.push(c),
                    }
                }
                Some(c) => { res.push(c); self.advance(); }
                None => return Err("Unterminated string".to_string()),
            }
        }
    }

    fn parse_number(&mut self) -> Result<JValue, String> {
        let start = self.i;
        if self.peek() == Some('-') { self.advance(); }
        while self.peek().map_or(false, |c| c.is_ascii_digit()) { self.advance(); }
        let mut is_float = false;
        if self.peek() == Some('.') {
            is_float = true;
            self.advance();
            while self.peek().map_or(false, |c| c.is_ascii_digit()) { self.advance(); }
        }
        let s = &self.s[start..self.i];
        if is_float {
            let f: f64 = s.parse().map_err(|e| format!("Invalid number '{s}': {e}"))?;
            Ok(JValue::Float(f))
        } else {
            let n: i64 = s.parse().map_err(|e| format!("Invalid number '{s}': {e}"))?;
            Ok(JValue::Int(n))
        }
    }

    fn parse_array(&mut self) -> Result<JValue, String> {
        self.expect('[')?;
        self.skip_ws();
        if self.peek() == Some(']') { self.advance(); return Ok(JValue::arr(vec![])); }
        let mut arr = Vec::new();
        loop {
            arr.push(self.parse_value()?);
            self.skip_ws();
            match self.peek() {
                Some(',') => { self.advance(); }
                Some(']') => { self.advance(); return Ok(JValue::Array(arr)); }
                _ => return Err(format!("Expected ',' or ']' at position {}", self.i)),
            }
        }
    }

    fn parse_object(&mut self) -> Result<JValue, String> {
        self.expect('{')?;
        self.skip_ws();
        if self.peek() == Some('}') { self.advance(); return Ok(JValue::Object(Vec::new())); }
        let mut map = Vec::new();
        loop {
            let key = self.parse_string()?;
            self.expect(':')?;
            let val = self.parse_value()?;
            map.push((key, val));
            self.skip_ws();
            match self.peek() {
                Some(',') => { self.advance(); }
                Some('}') => { self.advance(); return Ok(JValue::Object(map)); }
                _ => return Err(format!("Expected ',' or '}}' at position {}", self.i)),
            }
        }
    }
}
