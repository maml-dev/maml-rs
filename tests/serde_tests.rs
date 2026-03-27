use std::collections::HashMap;

use maml::{Value, from_str, from_value, parse, stringify, to_string};
use serde::{Deserialize, Serialize};

// =========================================================================
// Deserialization tests
// =========================================================================

#[test]
fn de_bool() {
    assert!(from_str::<bool>("true").unwrap());
    assert!(!from_str::<bool>("false").unwrap());
}

#[test]
fn de_integers() {
    assert_eq!(from_str::<i8>("42").unwrap(), 42i8);
    assert_eq!(from_str::<i16>("-100").unwrap(), -100i16);
    assert_eq!(from_str::<i32>("0").unwrap(), 0i32);
    assert_eq!(from_str::<i64>("9223372036854775807").unwrap(), i64::MAX);
    assert_eq!(from_str::<u8>("255").unwrap(), 255u8);
    assert_eq!(from_str::<u16>("1000").unwrap(), 1000u16);
    assert_eq!(from_str::<u32>("70000").unwrap(), 70000u32);
    assert_eq!(from_str::<u64>("42").unwrap(), 42u64);
}

#[test]
fn de_floats() {
    assert_eq!(from_str::<f32>("3.15").unwrap(), 3.15f32);
    assert_eq!(from_str::<f64>("2.719").unwrap(), 2.719f64);
}

#[test]
fn de_string() {
    assert_eq!(from_str::<String>("\"hello\"").unwrap(), "hello");
}

#[test]
fn de_char() {
    assert_eq!(from_str::<char>("\"a\"").unwrap(), 'a');
}

#[test]
fn de_option_none() {
    assert_eq!(from_str::<Option<i64>>("null").unwrap(), None);
}

#[test]
fn de_option_some() {
    assert_eq!(from_str::<Option<i64>>("42").unwrap(), Some(42));
}

#[test]
fn de_unit() {
    assert_eq!(from_str::<()>("null").unwrap(), ());
}

#[test]
fn de_vec() {
    assert_eq!(from_str::<Vec<i64>>("[1, 2, 3]").unwrap(), vec![1, 2, 3]);
}

#[test]
fn de_empty_vec() {
    assert_eq!(from_str::<Vec<i64>>("[]").unwrap(), Vec::<i64>::new());
}

#[test]
fn de_tuple() {
    let result: (i64, bool, String) = from_str("[42, true, \"hi\"]").unwrap();
    assert_eq!(result, (42, true, "hi".to_string()));
}

#[test]
fn de_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        port: u16,
        enabled: bool,
    }

    let input = r#"{name: "myapp", port: 8080, enabled: true}"#;
    let config: Config = from_str(input).unwrap();
    assert_eq!(
        config,
        Config {
            name: "myapp".to_string(),
            port: 8080,
            enabled: true,
        }
    );
}

#[test]
fn de_nested_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Inner {
        x: i64,
    }
    #[derive(Deserialize, Debug, PartialEq)]
    struct Outer {
        inner: Inner,
        label: String,
    }

    let input = r#"{inner: {x: 10}, label: "test"}"#;
    let outer: Outer = from_str(input).unwrap();
    assert_eq!(
        outer,
        Outer {
            inner: Inner { x: 10 },
            label: "test".to_string(),
        }
    );
}

#[test]
fn de_struct_with_optional_field() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        debug: Option<bool>,
    }

    let with_null: Config = from_str(r#"{name: "app", debug: null}"#).unwrap();
    assert_eq!(with_null.debug, None);

    let with_value: Config = from_str(r#"{name: "app", debug: true}"#).unwrap();
    assert_eq!(with_value.debug, Some(true));
}

#[test]
fn de_hashmap() {
    let result: HashMap<String, i64> = from_str("{a: 1, b: 2}").unwrap();
    assert_eq!(result.get("a"), Some(&1));
    assert_eq!(result.get("b"), Some(&2));
}

#[test]
fn de_enum_unit_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Color {
        Red,
        Green,
        Blue,
    }

    assert_eq!(from_str::<Color>("\"Red\"").unwrap(), Color::Red);
    assert_eq!(from_str::<Color>("\"Green\"").unwrap(), Color::Green);
}

#[test]
fn de_enum_newtype_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Data {
        Count(i64),
        Label(String),
    }

    let result: Data = from_str("{Count: 42}").unwrap();
    assert_eq!(result, Data::Count(42));

    let result: Data = from_str(r#"{Label: "hi"}"#).unwrap();
    assert_eq!(result, Data::Label("hi".to_string()));
}

#[test]
fn de_enum_tuple_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Shape {
        Point(f64, f64),
    }

    let result: Shape = from_str("{Point: [1.0, 2.0]}").unwrap();
    assert_eq!(result, Shape::Point(1.0, 2.0));
}

#[test]
fn de_enum_struct_variant() {
    #[derive(Deserialize, Debug, PartialEq)]
    enum Action {
        Move { x: i64, y: i64 },
    }

    let result: Action = from_str("{Move: {x: 10, y: 20}}").unwrap();
    assert_eq!(result, Action::Move { x: 10, y: 20 });
}

#[test]
fn de_enum_error_not_one_key() {
    #[derive(Deserialize, Debug)]
    enum E {
        A,
        B,
    }
    let err = from_str::<E>("{A: 1, B: 2}").unwrap_err();
    assert!(err.to_string().contains("exactly one key"));
}

#[test]
fn de_enum_error_wrong_type() {
    #[derive(Deserialize, Debug)]
    enum E {
        A,
    }
    let err = from_str::<E>("42").unwrap_err();
    assert!(err.to_string().contains("string or object"));
}

#[test]
fn de_newtype_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Wrapper(i64);

    let result: Wrapper = from_str("42").unwrap();
    assert_eq!(result, Wrapper(42));
}

#[test]
fn de_struct_error_not_object() {
    #[derive(Deserialize, Debug)]
    struct S {
        _x: i64,
    }
    let err = from_str::<S>("42").unwrap_err();
    assert!(err.to_string().contains("expected an object"));
}

#[test]
fn de_ignored_any() {
    // Deserialize a struct that ignores unknown fields
    #[derive(Deserialize, Debug, PartialEq)]
    struct Partial {
        name: String,
    }

    let result: Partial = from_str(r#"{name: "test", extra: 42}"#).unwrap();
    assert_eq!(result.name, "test");
}

#[test]
fn de_value_roundtrip_with_parse() {
    // from_str::<Value>() should match parse() for all fixture cases
    let cases = load_test_cases("parse.test.txt");
    for (name, input, _) in &cases {
        let parsed = parse(input).unwrap_or_else(|e| {
            panic!("Failed to parse '{name}':\n{e}");
        });
        let deserialized: Value = from_str(input).unwrap_or_else(|e| {
            panic!("Failed to from_str '{name}':\n{e}");
        });

        // Special float comparison for -0.0
        match (&parsed, &deserialized) {
            (Value::Float(a), Value::Float(b)) => {
                assert_eq!(a.to_bits(), b.to_bits(), "Test '{name}' float mismatch");
            }
            _ => {
                assert_eq!(parsed, deserialized, "Test '{name}' mismatch");
            }
        }
    }
}

// =========================================================================
// Serialization tests
// =========================================================================

#[test]
fn ser_bool() {
    assert_eq!(to_string(&true).unwrap(), "true");
    assert_eq!(to_string(&false).unwrap(), "false");
}

#[test]
fn ser_integers() {
    assert_eq!(to_string(&42i8).unwrap(), "42");
    assert_eq!(to_string(&-100i16).unwrap(), "-100");
    assert_eq!(to_string(&0i32).unwrap(), "0");
    assert_eq!(
        to_string(&9223372036854775807i64).unwrap(),
        "9223372036854775807"
    );
    assert_eq!(to_string(&255u8).unwrap(), "255");
    assert_eq!(to_string(&1000u16).unwrap(), "1000");
    assert_eq!(to_string(&70000u32).unwrap(), "70000");
    assert_eq!(to_string(&42u64).unwrap(), "42");
}

#[test]
fn ser_floats() {
    assert_eq!(to_string(&3.15f64).unwrap(), "3.15");
    assert_eq!(to_string(&1.0f32).unwrap(), "1.0");
    assert_eq!(to_string(&(-0.0f64)).unwrap(), "-0");
    assert_eq!(to_string(&1e10f64).unwrap(), "10000000000.0");
}

#[test]
fn ser_non_finite_float_error() {
    assert!(to_string(&f64::NAN).is_err());
    assert!(to_string(&f64::INFINITY).is_err());
    assert!(to_string(&f64::NEG_INFINITY).is_err());
    assert!(to_string(&f32::NAN).is_err());
}

#[test]
fn ser_u64_in_range() {
    assert_eq!(
        to_string(&(i64::MAX as u64)).unwrap(),
        "9223372036854775807"
    );
}

#[test]
fn ser_u64_out_of_range() {
    assert!(to_string(&u64::MAX).is_err());
    assert!(to_string(&(i64::MAX as u64 + 1)).is_err());
}

#[test]
fn ser_string() {
    assert_eq!(to_string(&"hello").unwrap(), "\"hello\"");
    assert_eq!(to_string(&"line\nnew").unwrap(), "\"line\\nnew\"");
}

#[test]
fn ser_char() {
    assert_eq!(to_string(&'a').unwrap(), "\"a\"");
}

#[test]
fn ser_none() {
    assert_eq!(to_string(&None::<i64>).unwrap(), "null");
}

#[test]
fn ser_some() {
    assert_eq!(to_string(&Some(42)).unwrap(), "42");
}

#[test]
fn ser_unit() {
    assert_eq!(to_string(&()).unwrap(), "null");
}

#[test]
fn ser_unit_struct() {
    #[derive(Serialize)]
    struct Unit;
    assert_eq!(to_string(&Unit).unwrap(), "null");
}

#[test]
fn ser_vec() {
    assert_eq!(to_string(&vec![1, 2, 3]).unwrap(), "[\n  1\n  2\n  3\n]");
}

#[test]
fn ser_empty_vec() {
    assert_eq!(to_string(&Vec::<i64>::new()).unwrap(), "[]");
}

#[test]
fn ser_tuple() {
    assert_eq!(
        to_string(&(42, true, "hi")).unwrap(),
        "[\n  42\n  true\n  \"hi\"\n]"
    );
}

#[test]
fn ser_struct() {
    #[derive(Serialize)]
    struct Config {
        name: String,
        port: u16,
        enabled: bool,
    }

    let config = Config {
        name: "myapp".to_string(),
        port: 8080,
        enabled: true,
    };
    let result = to_string(&config).unwrap();
    assert_eq!(
        result,
        "{\n  name: \"myapp\"\n  port: 8080\n  enabled: true\n}"
    );
}

#[test]
fn ser_nested_struct() {
    #[derive(Serialize)]
    struct Inner {
        x: i64,
    }
    #[derive(Serialize)]
    struct Outer {
        inner: Inner,
        label: String,
    }

    let outer = Outer {
        inner: Inner { x: 10 },
        label: "test".to_string(),
    };
    let result = to_string(&outer).unwrap();
    assert_eq!(
        result,
        "{\n  inner: {\n    x: 10\n  }\n  label: \"test\"\n}"
    );
}

#[test]
fn ser_empty_struct() {
    #[derive(Serialize)]
    struct Empty {}

    assert_eq!(to_string(&Empty {}).unwrap(), "{}");
}

#[test]
fn ser_hashmap() {
    let mut map = HashMap::new();
    map.insert("key", 42);
    let result = to_string(&map).unwrap();
    assert_eq!(result, "{\n  key: 42\n}");
}

#[test]
fn ser_empty_map() {
    let map: HashMap<String, i64> = HashMap::new();
    assert_eq!(to_string(&map).unwrap(), "{}");
}

#[test]
fn ser_enum_unit_variant() {
    #[derive(Serialize)]
    enum Color {
        Red,
    }
    assert_eq!(to_string(&Color::Red).unwrap(), "\"Red\"");
}

#[test]
fn ser_enum_newtype_variant() {
    #[derive(Serialize)]
    enum Data {
        Count(i64),
    }
    let result = to_string(&Data::Count(42)).unwrap();
    assert_eq!(result, "{\n  Count: 42\n}");
}

#[test]
fn ser_enum_tuple_variant() {
    #[derive(Serialize)]
    enum Shape {
        Point(f64, f64),
    }
    let result = to_string(&Shape::Point(1.0, 2.0)).unwrap();
    assert_eq!(result, "{\n  Point: [\n    1.0\n    2.0\n  ]\n}");
}

#[test]
fn ser_enum_struct_variant() {
    #[derive(Serialize)]
    enum Action {
        Move { x: i64, y: i64 },
    }
    let result = to_string(&Action::Move { x: 10, y: 20 }).unwrap();
    assert_eq!(result, "{\n  Move: {\n    x: 10\n    y: 20\n  }\n}");
}

#[test]
fn ser_vec_u8() {
    assert_eq!(to_string(&vec![1u8, 2, 3]).unwrap(), "[\n  1\n  2\n  3\n]");
}

#[test]
fn ser_newtype_struct() {
    #[derive(Serialize)]
    struct Wrapper(i64);
    assert_eq!(to_string(&Wrapper(42)).unwrap(), "42");
}

#[test]
fn ser_tuple_struct() {
    #[derive(Serialize)]
    struct Pair(i64, String);
    let result = to_string(&Pair(1, "two".to_string())).unwrap();
    assert_eq!(result, "[\n  1\n  \"two\"\n]");
}

// =========================================================================
// Value Serialize/Deserialize tests
// =========================================================================

#[test]
fn ser_value_null() {
    assert_eq!(to_string(&Value::Null).unwrap(), "null");
}

#[test]
fn ser_value_bool() {
    assert_eq!(to_string(&Value::Bool(true)).unwrap(), "true");
}

#[test]
fn ser_value_int() {
    assert_eq!(to_string(&Value::Int(42)).unwrap(), "42");
}

#[test]
fn ser_value_float() {
    assert_eq!(to_string(&Value::Float(3.15)).unwrap(), "3.15");
}

#[test]
fn ser_value_string() {
    assert_eq!(
        to_string(&Value::String("hello".into())).unwrap(),
        "\"hello\""
    );
}

#[test]
fn ser_value_array() {
    let v = Value::Array(vec![Value::Int(1), Value::Int(2)]);
    assert_eq!(to_string(&v).unwrap(), "[\n  1\n  2\n]");
}

#[test]
fn ser_value_empty_array() {
    assert_eq!(to_string(&Value::Array(vec![])).unwrap(), "[]");
}

#[test]
fn ser_value_object() {
    let v = Value::Object(vec![("a".into(), Value::Int(1))]);
    assert_eq!(to_string(&v).unwrap(), "{\n  a: 1\n}");
}

#[test]
fn ser_value_empty_object() {
    assert_eq!(to_string(&Value::Object(vec![])).unwrap(), "{}");
}

#[test]
fn ser_value_matches_stringify() {
    let cases = vec![
        Value::Null,
        Value::Bool(true),
        Value::Bool(false),
        Value::Int(42),
        Value::Int(-1),
        Value::Float(3.15),
        Value::Float(-0.0),
        Value::String("hello world".into()),
        Value::Array(vec![Value::Int(1), Value::String("two".into())]),
        Value::Object(vec![
            ("name".into(), Value::String("test".into())),
            ("count".into(), Value::Int(5)),
        ]),
    ];

    for value in &cases {
        let ser_result = to_string(value).unwrap();
        let stringify_result = stringify(value).unwrap();
        assert_eq!(ser_result, stringify_result, "Mismatch for {:?}", value);
    }
}

// =========================================================================
// from_value tests
// =========================================================================

#[test]
fn from_value_struct() {
    #[derive(Deserialize, Debug, PartialEq)]
    struct Point {
        x: i64,
        y: i64,
    }

    let value = Value::Object(vec![
        ("x".into(), Value::Int(10)),
        ("y".into(), Value::Int(20)),
    ]);
    let point: Point = from_value(value).unwrap();
    assert_eq!(point, Point { x: 10, y: 20 });
}

#[test]
fn from_value_primitive() {
    let n: i64 = from_value(Value::Int(42)).unwrap();
    assert_eq!(n, 42);
}

// =========================================================================
// Roundtrip tests (serialize then deserialize)
// =========================================================================

#[test]
fn roundtrip_struct() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Config {
        name: String,
        port: u16,
        enabled: bool,
    }

    let config = Config {
        name: "myapp".to_string(),
        port: 8080,
        enabled: true,
    };
    let maml_str = to_string(&config).unwrap();
    let back: Config = from_str(&maml_str).unwrap();
    assert_eq!(config, back);
}

#[test]
fn roundtrip_nested() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Inner {
        val: i64,
    }
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    struct Outer {
        inner: Inner,
        items: Vec<i64>,
    }

    let data = Outer {
        inner: Inner { val: 99 },
        items: vec![1, 2, 3],
    };
    let maml_str = to_string(&data).unwrap();
    let back: Outer = from_str(&maml_str).unwrap();
    assert_eq!(data, back);
}

#[test]
fn roundtrip_enum() {
    #[derive(Serialize, Deserialize, Debug, PartialEq)]
    enum E {
        Unit,
        Newtype(i64),
        Tuple(i64, String),
        Struct { a: bool, b: String },
    }

    for val in [
        E::Unit,
        E::Newtype(42),
        E::Tuple(1, "two".into()),
        E::Struct {
            a: true,
            b: "test".into(),
        },
    ] {
        let s = to_string(&val).unwrap();
        let back: E = from_str(&s).unwrap();
        assert_eq!(val, back);
    }
}

#[test]
fn roundtrip_option() {
    let some_val: Option<i64> = Some(42);
    let s = to_string(&some_val).unwrap();
    assert_eq!(from_str::<Option<i64>>(&s).unwrap(), Some(42));

    let none_val: Option<i64> = None;
    let s = to_string(&none_val).unwrap();
    assert_eq!(from_str::<Option<i64>>(&s).unwrap(), None);
}

// =========================================================================
// Error tests
// =========================================================================

#[test]
fn de_error_display() {
    let err = from_str::<i64>("\"not a number\"").unwrap_err();
    let msg = err.to_string();
    assert!(!msg.is_empty());
}

#[test]
fn de_error_no_line() {
    // Serde-level errors (type mismatch) have no line number
    let err = from_str::<bool>("42").unwrap_err();
    assert!(err.line().is_none());
}

#[test]
fn de_error_has_line() {
    // Parse errors have a line number
    let err = from_str::<Value>("!!!").unwrap_err();
    assert!(err.line().is_some());
}

#[test]
fn de_error_message() {
    let err = from_str::<i64>("\"not a number\"").unwrap_err();
    assert!(!err.message().is_empty());
}

#[test]
fn error_display() {
    let err = maml::Error::new("test error");
    assert_eq!(err.to_string(), "test error");
}

#[test]
fn error_message() {
    let err = maml::Error::new("test error");
    assert_eq!(err.message(), "test error");
}

#[test]
fn ser_variant_unit_variant_error() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    enum E {
        A,
    }
    // Unit variant with unexpected value
    let err = from_str::<E>("{A: 42}").unwrap_err();
    assert!(err.to_string().contains("unit variant"));
}

#[test]
fn de_variant_missing_newtype() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    enum E {
        A(i64),
    }
    // String form for newtype variant
    let err = from_str::<E>("\"A\"").unwrap_err();
    assert!(err.to_string().contains("newtype variant"));
}

#[test]
fn de_variant_tuple_not_array() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    enum E {
        A(i64, i64),
    }
    // Non-array value for tuple variant
    let err = from_str::<E>("{A: 42}").unwrap_err();
    assert!(err.to_string().contains("array for tuple variant"));
}

#[test]
fn de_variant_struct_not_object() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    enum E {
        A { x: i64 },
    }
    // Non-object for struct variant
    let err = from_str::<E>("{A: 42}").unwrap_err();
    assert!(err.to_string().contains("object for struct variant"));
}

// =========================================================================
// Coverage: MapKeySerializer — reachable key types
// =========================================================================

/// Helper: wraps a key-value pair as a serializable single-entry map.
struct MapWith<K: Serialize, V: Serialize> {
    key: K,
    value: V,
}

impl<K: Serialize, V: Serialize> Serialize for MapWith<K, V> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&self.key, &self.value)?;
        map.end()
    }
}

#[test]
fn ser_map_with_non_string_keys_integer() {
    let mut map = HashMap::new();
    map.insert(1i64, 10i64);
    let result = to_string(&map).unwrap();
    assert!(result.contains("1: 10"));
}

#[test]
fn ser_map_key_bool() {
    let result = to_string(&MapWith {
        key: true,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("true: 1"));
}

#[test]
fn ser_map_key_i8() {
    let result = to_string(&MapWith { key: 1i8, value: 1 }).unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_i16() {
    let result = to_string(&MapWith {
        key: 1i16,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_i32() {
    let result = to_string(&MapWith {
        key: 1i32,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_i64() {
    let result = to_string(&MapWith {
        key: 1i64,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_u8() {
    let result = to_string(&MapWith { key: 1u8, value: 1 }).unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_u16() {
    let result = to_string(&MapWith {
        key: 1u16,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_u32() {
    let result = to_string(&MapWith {
        key: 1u32,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_u64() {
    let result = to_string(&MapWith {
        key: 1u64,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("1: 1"));
}

#[test]
fn ser_map_key_char() {
    let result = to_string(&MapWith { key: 'a', value: 1 }).unwrap();
    assert!(result.contains("a: 1"));
}

#[test]
fn ser_map_key_unit_variant() {
    #[derive(Serialize)]
    enum K {
        A,
    }
    let result = to_string(&MapWith {
        key: K::A,
        value: 1,
    })
    .unwrap();
    assert!(result.contains("A: 1"));
}

// =========================================================================
// Coverage: MapKeySerializer — unsupported key type errors
// =========================================================================

#[test]
fn ser_map_key_f32_error() {
    assert!(
        to_string(&MapWith {
            key: 1.0f32,
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_f64_error() {
    assert!(
        to_string(&MapWith {
            key: 1.0f64,
            value: 1
        })
        .is_err()
    );
}

struct BytesKey;
impl Serialize for BytesKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(b"hello")
    }
}

#[test]
fn ser_map_key_bytes_error() {
    assert!(
        to_string(&MapWith {
            key: BytesKey,
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_none_error() {
    assert!(
        to_string(&MapWith {
            key: None::<i64>,
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_some_error() {
    assert!(
        to_string(&MapWith {
            key: Some(42),
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_unit_error() {
    assert!(to_string(&MapWith { key: (), value: 1 }).is_err());
}

#[test]
fn ser_map_key_unit_struct_error() {
    #[derive(Serialize)]
    struct U;
    assert!(to_string(&MapWith { key: U, value: 1 }).is_err());
}

#[test]
fn ser_map_key_newtype_struct_error() {
    #[derive(Serialize)]
    struct N(i64);
    assert!(
        to_string(&MapWith {
            key: N(1),
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_newtype_variant_error() {
    #[derive(Serialize)]
    enum E {
        A(i64),
    }
    assert!(
        to_string(&MapWith {
            key: E::A(1),
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_seq_error() {
    assert!(
        to_string(&MapWith {
            key: vec![1, 2],
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_tuple_error() {
    assert!(
        to_string(&MapWith {
            key: (1, 2),
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_tuple_struct_error() {
    #[derive(Serialize)]
    struct T(i64, i64);
    assert!(
        to_string(&MapWith {
            key: T(1, 2),
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_tuple_variant_error() {
    #[derive(Serialize)]
    enum E {
        A(i64, i64),
    }
    assert!(
        to_string(&MapWith {
            key: E::A(1, 2),
            value: 1
        })
        .is_err()
    );
}

struct MapKey;
impl Serialize for MapKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeMap;
        let map = serializer.serialize_map(Some(0))?;
        map.end()
    }
}

#[test]
fn ser_map_key_map_error() {
    assert!(
        to_string(&MapWith {
            key: MapKey,
            value: 1
        })
        .is_err()
    );
}

struct StructKey;
impl Serialize for StructKey {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let s = serializer.serialize_struct("S", 0)?;
        s.end()
    }
}

#[test]
fn ser_map_key_struct_error() {
    assert!(
        to_string(&MapWith {
            key: StructKey,
            value: 1
        })
        .is_err()
    );
}

#[test]
fn ser_map_key_struct_variant_error() {
    #[derive(Serialize)]
    enum E {
        A { x: i64 },
    }
    assert!(
        to_string(&MapWith {
            key: E::A { x: 1 },
            value: 1
        })
        .is_err()
    );
}

// =========================================================================
// Coverage: Serializer::serialize_bytes
// =========================================================================

struct BytesWrapper<'a>(&'a [u8]);
impl Serialize for BytesWrapper<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_bytes(self.0)
    }
}

#[test]
fn ser_bytes_main_serializer() {
    let result = to_string(&BytesWrapper(&[1, 2, 3])).unwrap();
    assert_eq!(result, "[\n  1\n  2\n  3\n]");
}

// =========================================================================
// Coverage: Empty tuple variant / empty struct variant
// =========================================================================

struct EmptyTupleVariant;
impl Serialize for EmptyTupleVariant {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeTupleVariant;
        let tv = serializer.serialize_tuple_variant("E", 0, "A", 0)?;
        tv.end()
    }
}

#[test]
fn ser_empty_tuple_variant() {
    let result = to_string(&EmptyTupleVariant).unwrap();
    assert_eq!(result, "{\n  A: []\n}");
}

struct EmptyStructVariant;
impl Serialize for EmptyStructVariant {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStructVariant;
        let sv = serializer.serialize_struct_variant("E", 0, "A", 0)?;
        sv.end()
    }
}

#[test]
fn ser_empty_struct_variant() {
    let result = to_string(&EmptyStructVariant).unwrap();
    assert_eq!(result, "{\n  A: {}\n}");
}

// =========================================================================
// Coverage: ValueVisitor methods via serde::de::value deserializers
// =========================================================================

#[test]
fn de_value_from_unit_deserializer() {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::UnitDeserializer<maml::Error> = ().into_deserializer();
    let val = Value::deserialize(de).unwrap();
    assert_eq!(val, Value::Null);
}

#[test]
fn de_value_from_u64_deserializer() {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::U64Deserializer<maml::Error> = 42u64.into_deserializer();
    let val = Value::deserialize(de).unwrap();
    assert_eq!(val, Value::Int(42));
}

#[test]
fn de_value_from_u64_out_of_range() {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::U64Deserializer<maml::Error> = u64::MAX.into_deserializer();
    let err = Value::deserialize(de).unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[test]
fn de_value_from_str_deserializer() {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::StrDeserializer<'_, maml::Error> = "hello".into_deserializer();
    let val = Value::deserialize(de).unwrap();
    assert_eq!(val, Value::String("hello".into()));
}

#[test]
fn de_value_from_string_deserializer() {
    use serde::de::IntoDeserializer;
    let de: serde::de::value::StringDeserializer<maml::Error> =
        "world".to_string().into_deserializer();
    let val = Value::deserialize(de).unwrap();
    assert_eq!(val, Value::String("world".into()));
}

#[test]
fn de_value_expecting_triggered() {
    // Use a bytes deserializer to trigger the error path which calls expecting()
    let de = serde::de::value::BorrowedBytesDeserializer::<'_, maml::Error>::new(b"hello");
    let err = Value::deserialize(de).unwrap_err();
    assert!(err.to_string().contains("MAML value"));
}

// Custom mini-deserializer for visit_none
struct NoneDeserializer;
impl<'de> serde::Deserializer<'de> for NoneDeserializer {
    type Error = maml::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_none()
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[test]
fn de_value_from_none_deserializer() {
    let val = Value::deserialize(NoneDeserializer).unwrap();
    assert_eq!(val, Value::Null);
}

// Custom mini-deserializer for visit_some
struct SomeDeserializer;
impl<'de> serde::Deserializer<'de> for SomeDeserializer {
    type Error = maml::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_some(serde::de::value::I64Deserializer::<maml::Error>::new(42))
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

#[test]
fn de_value_from_some_deserializer() {
    let val = Value::deserialize(SomeDeserializer).unwrap();
    assert_eq!(val, Value::Int(42));
}

// =========================================================================
// Coverage: VariantDeserializer None arms for tuple/struct variants
// =========================================================================

#[test]
fn de_string_for_tuple_variant_error() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    enum E {
        A(i64, i64),
    }
    let err = from_str::<E>("\"A\"").unwrap_err();
    assert!(err.to_string().contains("tuple variant"));
}

#[test]
fn de_string_for_struct_variant_error() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    enum E {
        A { x: i64 },
    }
    let err = from_str::<E>("\"A\"").unwrap_err();
    assert!(err.to_string().contains("struct variant"));
}

// =========================================================================
// Coverage: explicit null → Value deserialization
// =========================================================================

#[test]
fn de_value_null_explicit() {
    let v: Value = from_str("null").unwrap();
    assert_eq!(v, Value::Null);
}

// =========================================================================
// Coverage: visit_seq / visit_map error paths via custom failing access
// =========================================================================

struct FailingSeqDeserializer;
impl<'de> serde::Deserializer<'de> for FailingSeqDeserializer {
    type Error = maml::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_seq(FailingSeqAccess)
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct FailingSeqAccess;
impl<'de> serde::de::SeqAccess<'de> for FailingSeqAccess {
    type Error = maml::Error;
    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        _seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        Err(serde::de::Error::custom("seq error"))
    }
}

#[test]
fn de_value_visit_seq_error() {
    let err = Value::deserialize(FailingSeqDeserializer).unwrap_err();
    assert!(err.to_string().contains("seq error"));
}

struct FailingMapDeserializer;
impl<'de> serde::Deserializer<'de> for FailingMapDeserializer {
    type Error = maml::Error;
    fn deserialize_any<V: serde::de::Visitor<'de>>(
        self,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_map(FailingMapAccess)
    }
    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 u8 u16 u32 u64 f32 f64 char str string
        bytes byte_buf option unit unit_struct newtype_struct seq tuple
        tuple_struct map struct enum identifier ignored_any
    }
}

struct FailingMapAccess;
impl<'de> serde::de::MapAccess<'de> for FailingMapAccess {
    type Error = maml::Error;
    fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(
        &mut self,
        _seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        Err(serde::de::Error::custom("map error"))
    }
    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        _seed: V,
    ) -> Result<V::Value, Self::Error> {
        Err(serde::de::Error::custom("map error"))
    }
}

#[test]
fn de_value_visit_map_error() {
    let err = Value::deserialize(FailingMapDeserializer).unwrap_err();
    assert!(err.to_string().contains("map error"));
}

// =========================================================================
// Coverage: EnumDeserializer unknown variant
// =========================================================================

#[test]
fn de_enum_unknown_variant() {
    #[derive(Deserialize, Debug)]
    #[allow(dead_code)]
    enum E {
        A,
    }
    let err = from_str::<E>("{Unknown: 42}").unwrap_err();
    assert!(!err.to_string().is_empty());
}

// =========================================================================
// Helpers
// =========================================================================

fn load_test_cases(filename: &str) -> Vec<(String, String, String)> {
    let path = format!("tests/fixtures/{filename}");
    let content = std::fs::read_to_string(&path).expect("failed to read test file");
    let mut cases = Vec::new();

    for section in content.split("===") {
        let section = section.trim();
        if section.is_empty() {
            continue;
        }
        let lines: Vec<&str> = section.split('\n').collect();
        let name = lines[0].trim().to_string();
        let body = lines[1..].join("\n");
        let sep = body.find("---").expect("no --- separator");
        let input = body[..sep].to_string();
        let expected = body[sep + 3..].trim().to_string();
        cases.push((name, input, expected));
    }
    cases
}
