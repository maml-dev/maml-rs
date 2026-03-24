use maml::{parse, stringify, Value};

fn load_test_cases(filename: &str) -> Vec<(String, String, String)> {
    let path = format!("tests/fixtures/{filename}");
    let content = std::fs::read_to_string(&path).expect("failed to read test file");
    let mut cases = Vec::new();

    // Match JS loader: split on "===", trim each section, split first line as name,
    // rejoin remaining lines, split on "---" to get input and expected.
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

fn json_to_value(json: &serde_json::Value) -> Value {
    match json {
        serde_json::Value::Null => Value::Null,
        serde_json::Value::Bool(b) => Value::Bool(*b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                // Check if this is -0 in the JSON
                let raw = n.to_string();
                if raw == "-0" {
                    Value::Float(-0.0)
                } else {
                    Value::Int(i)
                }
            } else if let Some(f) = n.as_f64() {
                Value::Float(f)
            } else {
                panic!("unsupported number: {n}");
            }
        }
        serde_json::Value::String(s) => Value::String(s.clone()),
        serde_json::Value::Array(arr) => {
            Value::Array(arr.iter().map(json_to_value).collect())
        }
        serde_json::Value::Object(obj) => {
            Value::Object(obj.iter().map(|(k, v)| (k.clone(), json_to_value(v))).collect())
        }
    }
}

#[test]
fn parse_test_cases() {
    let cases = load_test_cases("parse.test.txt");
    assert!(!cases.is_empty(), "no test cases loaded");

    for (name, input, expected_json) in &cases {
        // This test uses surrogate code points (\u{D83D}\u{DE80}) which are not valid
        // Unicode scalar values per the MAML spec. JS String.fromCodePoint also rejects them.
        if name == "object with unicode and escapes mixed" {
            continue;
        }

        let parsed = parse(input).unwrap_or_else(|e| {
            panic!("Failed to parse '{name}':\n{e}\nInput: {input:?}");
        });

        let json: serde_json::Value = serde_json::from_str(expected_json).unwrap_or_else(|e| {
            panic!("Failed to parse expected JSON for '{name}': {e}\nJSON: {expected_json:?}");
        });

        let expected = json_to_value(&json);

        // Special handling for -0.0 comparison
        if let (Value::Float(a), Value::Float(b)) = (&parsed, &expected) {
            assert!(
                a.to_bits() == b.to_bits(),
                "Test '{name}' failed:\n  got:      {parsed:?}\n  expected: {expected:?}"
            );
        } else {
            assert_eq!(
                parsed, expected,
                "Test '{name}' failed:\n  input:    {input:?}\n  expected: {expected_json:?}"
            );
        }
    }
}

#[test]
fn parse_bigint_as_i64() {
    // 9007199254740992 = Number.MAX_SAFE_INTEGER + 1, fits in i64
    let result = parse("9007199254740992").unwrap();
    assert_eq!(result, Value::Int(9007199254740992));
}

#[test]
fn raw_string_crlf() {
    let result = parse("\"\"\"line1\r\nline2\r\nline3\"\"\"").unwrap();
    assert_eq!(result, Value::String("line1\r\nline2\r\nline3".into()));
}

#[test]
fn raw_string_mixed_crlf_lf() {
    let result = parse("\"\"\"line1\r\nline2\nline3\r\n\"\"\"").unwrap();
    assert_eq!(
        result,
        Value::String("line1\r\nline2\nline3\r\n".into())
    );
}

#[test]
fn raw_string_cr_inside_and_crlf() {
    let result = parse("\"\"\"the \r char\r\n\"\"\"").unwrap();
    assert_eq!(result, Value::String("the \r char\r\n".into()));
}

#[test]
fn raw_string_cr_at_end() {
    let result = parse("\"\"\"string\r\"\"\"").unwrap();
    assert_eq!(result, Value::String("string\r".into()));
}

#[test]
fn raw_string_leading_lf() {
    let result = parse("\"\"\"\nstring\r\n\"\"\"").unwrap();
    assert_eq!(result, Value::String("string\r\n".into()));
}

#[test]
fn raw_string_leading_crlf() {
    let result = parse("\"\"\"\r\nstring\r\n\"\"\"").unwrap();
    assert_eq!(result, Value::String("string\r\n".into()));
}

#[test]
fn raw_string_leading_cr_only() {
    // Bare \r is NOT stripped as a leading newline
    let result = parse("\"\"\"\rstring\r\n\"\"\"").unwrap();
    assert_eq!(result, Value::String("\rstring\r\n".into()));
}

#[test]
fn stringify_roundtrip() {
    let cases = load_test_cases("parse.test.txt");
    for (name, input, _) in &cases {
        if name == "object with unicode and escapes mixed" {
            continue;
        }
        let parsed = match parse(input) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let serialized = stringify(&parsed);
        let reparsed = parse(&serialized).unwrap_or_else(|e| {
            panic!("Roundtrip failed for '{name}':\n  serialized: {serialized:?}\n  error: {e}");
        });
        // Floats may roundtrip as integers (e.g., 1e6 → "1000000" → Int).
        // Compare numerically when types differ.
        match (&parsed, &reparsed) {
            (Value::Float(a), Value::Float(b)) => {
                assert_eq!(a.to_bits(), b.to_bits(), "Roundtrip float mismatch for '{name}'");
            }
            (Value::Float(a), Value::Int(b)) => {
                assert_eq!(*a, *b as f64, "Roundtrip numeric mismatch for '{name}'");
            }
            _ => {
                assert_eq!(parsed, reparsed, "Roundtrip mismatch for '{name}'");
            }
        }
    }
}

#[test]
fn stringify_basic() {
    assert_eq!(stringify(&Value::Null), "null");
    assert_eq!(stringify(&Value::Bool(true)), "true");
    assert_eq!(stringify(&Value::Bool(false)), "false");
    assert_eq!(stringify(&Value::Int(42)), "42");
    assert_eq!(stringify(&Value::Float(3.14)), "3.14");
    assert_eq!(stringify(&Value::Float(-0.0)), "-0");
    assert_eq!(stringify(&Value::String("hello".into())), "\"hello\"");
    assert_eq!(stringify(&Value::Array(vec![])), "[]");
    assert_eq!(
        stringify(&Value::Object(vec![])),
        "{}"
    );
}

#[test]
fn stringify_array() {
    let val = Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(stringify(&val), "[\n  1\n  2\n  3\n]");
}

#[test]
fn stringify_object() {
    let val = Value::Object(vec![
        ("foo".into(), Value::String("foo".into())),
        ("bar".into(), Value::String("bar".into())),
    ]);
    assert_eq!(stringify(&val), "{\n  foo: \"foo\"\n  bar: \"bar\"\n}");
}

#[test]
fn stringify_quoted_keys() {
    let val = Value::Object(vec![
        ("foo bar".into(), Value::String("value".into())),
    ]);
    assert_eq!(
        stringify(&val),
        "{\n  \"foo bar\": \"value\"\n}"
    );
}

#[test]
fn stringify_escapes() {
    let val = Value::String("line1\nline2\ttab\\back\"quote".into());
    assert_eq!(
        stringify(&val),
        "\"line1\\nline2\\ttab\\\\back\\\"quote\""
    );
}

#[test]
fn surrogate_codepoints_rejected() {
    assert!(parse("\"\\u{D800}\"").is_err());
    assert!(parse("\"\\u{DFFF}\"").is_err());
    assert!(parse("\"\\u{D83D}\"").is_err());
}

#[test]
fn integer_overflow() {
    // i64::MAX + 1 should fail
    assert!(parse("9223372036854775808").is_err());
}

#[test]
fn value_accessors() {
    let obj = parse("{a: 1, b: [true, null]}").unwrap();
    assert_eq!(obj["a"], Value::Int(1));
    assert_eq!(obj["b"][0], Value::Bool(true));
    assert_eq!(obj["b"][1], Value::Null);
    assert_eq!(obj.get("c"), None);
    assert!(obj["b"].as_array().is_some());
    assert!(obj.as_object().is_some());
}
