use std::collections::HashMap;

pub mod parser;
pub mod tokenizer;
mod utils;

/// MAML AST
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(untagged))]
pub enum MamlValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Array(Vec<MamlValue>),
    Object(HashMap<String, MamlValue>),
}

#[cfg(test)]
mod tests {
    use crate::parser::from_str;

    use super::*;

    #[test]
    fn test_basic_values() {
        assert!(matches!(from_str("null").unwrap(), MamlValue::Null));
        assert!(matches!(from_str("true").unwrap(), MamlValue::Bool(true)));
        assert!(matches!(from_str("42").unwrap(), MamlValue::Int(42)));
        assert!(matches!(from_str("3.14").unwrap(), MamlValue::Float(_)));
    }

    #[test]
    fn test_string() {
        let val = from_str(r#""hello world""#).unwrap();
        assert!(matches!(val, MamlValue::String(s) if s == "hello world"));
    }

    #[test]
    fn test_array() {
        let val = from_str("[1, 2, 3]").unwrap();
        if let MamlValue::Array(arr) = val {
            assert_eq!(arr.len(), 3);
        } else {
            panic!("Expected array");
        }
    }

    #[test]
    fn test_object() {
        let val = from_str(r#"{ name: "test", value: 42 }"#).unwrap();
        if let MamlValue::Object(obj) = val {
            assert_eq!(obj.len(), 2);
            assert!(obj.contains_key("name"));
        } else {
            panic!("Expected object");
        }
    }

    #[test]
    fn test_raw_string() {
        let val = from_str(
            r#""""
hello
world
""""#,
        )
        .unwrap();
        if let MamlValue::String(s) = val {
            assert_eq!(s, "hello\nworld\n");
        } else {
            panic!("Expected string");
        }
    }
}
