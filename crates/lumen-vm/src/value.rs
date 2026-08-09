use std::fmt;
use std::hash::{BuildHasher, Hash, Hasher};
use std::sync::Arc;
use im::HashMap;

#[derive(Debug, Clone)]
pub struct FixHasher {
    hash: u64,
}

impl Default for FixHasher {
    fn default() -> Self {
        Self {
            hash: 0xcbf29ce484222325,
        }
    }
}

impl Hasher for FixHasher {
    fn finish(&self) -> u64 {
        self.hash
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.hash ^= b as u64;
            self.hash = self.hash.wrapping_mul(0x100000001b3);
        }
    }
}

impl BuildHasher for FixHasher {
    type Hasher = FixHasher;
    fn build_hasher(&self) -> Self::Hasher {
        FixHasher::default()
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(Arc<str>),
    Bool(bool),
    Array(Arc<Vec<Value>>),
    Func(String),
    Struct {
        name: String,
        fields: Vec<(String, Value)>,
    },
    Enum {
        name: String,
        variant: String,
        fields: Vec<Value>,
    },
    Exito(Box<Value>),
    Error(Box<Value>),
    Opcion(Option<Box<Value>>),
    Tuple(Vec<Value>),
    Map(HashMap<Value, Value, FixHasher>),
    Void,
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a.to_bits() == b.to_bits(),
            (Value::Str(a), Value::Str(b)) => a.as_ref() == b.as_ref(),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Array(a), Value::Array(b)) => a.as_ref() == b.as_ref(),
            (Value::Func(a), Value::Func(b)) => a == b,
            (Value::Struct { name: na, fields: fa }, Value::Struct { name: nb, fields: fb }) => {
                na == nb && fa == fb
            }
            (
                Value::Enum {
                    name: na,
                    variant: va,
                    fields: fa,
                },
                Value::Enum {
                    name: nb,
                    variant: vb,
                    fields: fb,
                },
            ) => na == nb && va == vb && fa == fb,
            (Value::Exito(a), Value::Exito(b)) => a == b,
            (Value::Error(a), Value::Error(b)) => a == b,
            (Value::Opcion(a), Value::Opcion(b)) => a == b,
            (Value::Tuple(a), Value::Tuple(b)) => a == b,
            (Value::Map(a), Value::Map(b)) => {
                a.len() == b.len() && a.iter().all(|(k, v)| b.get(k) == Some(v))
            }
            (Value::Void, Value::Void) => true,
            _ => false,
        }
    }
}

impl Eq for Value {}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Void => 0u8.hash(state),
            Value::Int(n) => {
                1u8.hash(state);
                n.hash(state);
            }
            Value::Float(f) => {
                2u8.hash(state);
                f.to_bits().hash(state);
            }
            Value::Str(s) => {
                3u8.hash(state);
                s.hash(state);
            }
            Value::Bool(b) => {
                4u8.hash(state);
                b.hash(state);
            }
            Value::Array(arr) => {
                5u8.hash(state);
                for item in arr.iter() {
                    item.hash(state);
                }
            }
            Value::Func(name) => {
                6u8.hash(state);
                name.hash(state);
            }
            Value::Struct { name, fields } => {
                7u8.hash(state);
                name.hash(state);
                for (k, v) in fields {
                    k.hash(state);
                    v.hash(state);
                }
            }
            Value::Enum {
                name,
                variant,
                fields,
            } => {
                8u8.hash(state);
                name.hash(state);
                variant.hash(state);
                for v in fields {
                    v.hash(state);
                }
            }
            Value::Exito(v) => {
                9u8.hash(state);
                v.hash(state);
            }
            Value::Error(v) => {
                10u8.hash(state);
                v.hash(state);
            }
            Value::Opcion(v) => {
                11u8.hash(state);
                v.hash(state);
            }
            Value::Tuple(items) => {
                12u8.hash(state);
                for item in items {
                    item.hash(state);
                }
            }
            Value::Map(map) => {
                13u8.hash(state);
                let mut hashes: Vec<u64> = map
                    .iter()
                    .map(|(k, v)| {
                        let mut s = std::collections::hash_map::DefaultHasher::new();
                        k.hash(&mut s);
                        v.hash(&mut s);
                        s.finish()
                    })
                    .collect();
                hashes.sort_unstable();
                for h in hashes {
                    h.hash(state);
                }
            }
        }
    }
}

impl Value {
    pub fn str(s: impl Into<String>) -> Value {
        Value::Str(Arc::from(s.into()))
    }

    pub fn arr(v: Vec<Value>) -> Value {
        Value::Array(Arc::new(v))
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, Value::Exito(_))
    }

    pub fn unwrap_ok(self) -> Option<Value> {
        match self {
            Value::Exito(v) => Some(*v),
            _ => None,
        }
    }

    pub fn unwrap_err(self) -> Option<Value> {
        match self {
            Value::Error(v) => Some(*v),
            _ => None,
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            Value::Int(n) => Some(*n as f64),
            Value::Float(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Int(n) => *n != 0,
            Value::Float(n) => *n != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::Array(v) => !v.is_empty(),
            Value::Func(_) => true,
            Value::Struct { .. } => true,
            Value::Enum { .. } => true,
            Value::Exito(_) => true,
            Value::Error(_) => true,
            Value::Opcion(Some(_)) => true,
            Value::Opcion(None) => false,
            Value::Map(m) => !m.is_empty(),
            Value::Tuple(_) => true,
            Value::Void => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Int(n) => write!(f, "{}", n),
            Value::Float(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::Str(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Array(v) => {
                let items: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
                write!(f, "[{}]", items.join(", "))
            }
            Value::Func(s) => write!(f, "<funcion {}>", s),
            Value::Struct { name: _, fields } => {
                let items: Vec<String> = fields
                    .iter()
                    .map(|(k, v)| format!("{}: {}", k, v))
                    .collect();
                write!(f, "{{ {} }}", items.join(", "))
            }
            Value::Enum {
                name,
                variant,
                fields,
            } => {
                if fields.is_empty() {
                    write!(f, "{}::{}", name, variant)
                } else {
                    let items: Vec<String> = fields.iter().map(|x| format!("{}", x)).collect();
                    write!(f, "{}::{}({})", name, variant, items.join(", "))
                }
            }
            Value::Exito(v) => write!(f, "exito({})", v),
            Value::Error(v) => write!(f, "error({})", v),
            Value::Opcion(Some(v)) => write!(f, "algun({})", v),
            Value::Opcion(None) => write!(f, "ninguno"),
            Value::Tuple(v) => {
                let items: Vec<String> = v.iter().map(|x| format!("{}", x)).collect();
                write!(f, "({})", items.join(", "))
            }
            Value::Map(map) => {
                let items: Vec<String> =
                    map.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{} }}", items.join(", "))
            }
            Value::Void => write!(f, "void"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display_int() {
        assert_eq!(format!("{}", Value::Int(42)), "42");
    }

    #[test]
    fn test_display_float() {
        assert_eq!(format!("{}", Value::Float(2.71)), "2.71");
    }

    #[test]
    fn test_display_float_integer() {
        assert_eq!(format!("{}", Value::Float(42.0)), "42");
    }

    #[test]
    fn test_display_str() {
        assert_eq!(format!("{}", Value::str("hola")), "hola");
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(format!("{}", Value::Bool(true)), "true");
        assert_eq!(format!("{}", Value::Bool(false)), "false");
    }

    #[test]
    fn test_display_void() {
        assert_eq!(format!("{}", Value::Void), "void");
    }

    #[test]
    fn test_truthy_bool() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
    }

    #[test]
    fn test_truthy_int() {
        assert!(Value::Int(1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
        assert!(Value::Int(-1).is_truthy());
    }

    #[test]
    fn test_truthy_float() {
        assert!(Value::Float(1.0).is_truthy());
        assert!(!Value::Float(0.0).is_truthy());
        assert!(Value::Float(-1.0).is_truthy());
    }

    #[test]
    fn test_truthy_str() {
        assert!(Value::str("hello").is_truthy());
        assert!(!Value::str("").is_truthy());
    }

    #[test]
    fn test_truthy_void() {
        assert!(!Value::Void.is_truthy());
    }

    #[test]
    fn test_as_num() {
        assert_eq!(Value::Int(5).as_num(), Some(5.0));
        assert_eq!(Value::Float(3.5).as_num(), Some(3.5));
        assert_eq!(Value::str("x").as_num(), None);
    }

    #[test]
    fn test_as_bool() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int(0).as_bool(), None);
    }
}
