use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Array(Vec<Value>),
    Func(String),
    Struct {
        name: String,
        fields: Vec<(String, Value)>,
    },
    Exito(Box<Value>),
    Error(Box<Value>),
    Opcion(Option<Box<Value>>),
    Void,
}

impl Value {
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
            Value::Exito(_) => true,
            Value::Error(_) => true,
            Value::Opcion(Some(_)) => true,
            Value::Opcion(None) => false,
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
                let items: Vec<String> = fields.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{ {} }}", items.join(", "))
            }
            Value::Exito(v) => write!(f, "exito({})", v),
            Value::Error(v) => write!(f, "error({})", v),
            Value::Opcion(Some(v)) => write!(f, "algun({})", v),
            Value::Opcion(None) => write!(f, "ninguno"),
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
        assert_eq!(format!("{}", Value::Float(3.14)), "3.14");
    }

    #[test]
    fn test_display_float_integer() {
        assert_eq!(format!("{}", Value::Float(42.0)), "42");
    }

    #[test]
    fn test_display_str() {
        assert_eq!(format!("{}", Value::Str("hola".to_string())), "hola");
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
        assert!(Value::Str("hello".to_string()).is_truthy());
        assert!(!Value::Str("".to_string()).is_truthy());
    }

    #[test]
    fn test_truthy_void() {
        assert!(!Value::Void.is_truthy());
    }

    #[test]
    fn test_as_num() {
        assert_eq!(Value::Int(5).as_num(), Some(5.0));
        assert_eq!(Value::Float(3.5).as_num(), Some(3.5));
        assert_eq!(Value::Str("x".to_string()).as_num(), None);
    }

    #[test]
    fn test_as_bool() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int(0).as_bool(), None);
    }
}
