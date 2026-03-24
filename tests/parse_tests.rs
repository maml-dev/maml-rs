use maml::{Value, parse, stringify};

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
        serde_json::Value::Array(arr) => Value::Array(arr.iter().map(json_to_value).collect()),
        serde_json::Value::Object(obj) => Value::Object(
            obj.iter()
                .map(|(k, v)| (k.clone(), json_to_value(v)))
                .collect(),
        ),
    }
}

#[test]
fn parse_test_cases() {
    let cases = load_test_cases("parse.test.txt");
    assert!(!cases.is_empty(), "no test cases loaded");

    for (name, input, expected_json) in &cases {
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
    assert_eq!(result, Value::String("line1\r\nline2\nline3\r\n".into()));
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
                assert_eq!(
                    a.to_bits(),
                    b.to_bits(),
                    "Roundtrip float mismatch for '{name}'"
                );
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
    assert_eq!(stringify(&Value::Object(vec![])), "{}");
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
    let val = Value::Object(vec![("foo bar".into(), Value::String("value".into()))]);
    assert_eq!(stringify(&val), "{\n  \"foo bar\": \"value\"\n}");
}

#[test]
fn stringify_escapes() {
    let val = Value::String("line1\nline2\ttab\\back\"quote".into());
    assert_eq!(stringify(&val), "\"line1\\nline2\\ttab\\\\back\\\"quote\"");
}

#[test]
fn unicode_scalar_value_boundaries() {
    assert_eq!(parse("\"\\u{0}\"").unwrap(), Value::String("\u{0000}".into()));
    assert_eq!(parse("\"\\u{D7FF}\"").unwrap(), Value::String("\u{D7FF}".into()));
    assert_eq!(parse("\"\\u{E000}\"").unwrap(), Value::String("\u{E000}".into()));
    assert_eq!(parse("\"\\u{FFFF}\"").unwrap(), Value::String("\u{FFFF}".into()));
    assert_eq!(parse("\"\\u{10000}\"").unwrap(), Value::String("\u{10000}".into()));
    assert_eq!(parse("\"\\u{10FFFF}\"").unwrap(), Value::String("\u{10FFFF}".into()));
}

#[test]
fn surrogate_codepoints_rejected() {
    assert!(parse("\"\\u{D800}\"").is_err());
    assert!(parse("\"\\u{DBFF}\"").is_err());
    assert!(parse("\"\\u{DC00}\"").is_err());
    assert!(parse("\"\\u{DFFF}\"").is_err());
}

#[test]
fn integer_overflow() {
    // i64::MAX + 1 should fail
    assert!(parse("9223372036854775808").is_err());
}

#[test]
fn value_accessors() {
    let obj = parse("{a: 1, b: [true, null], c: 3.14, d: \"hello\"}").unwrap();

    // Index access
    assert_eq!(obj["a"], Value::Int(1));
    assert_eq!(obj["b"][0], Value::Bool(true));
    assert_eq!(obj["b"][1], Value::Null);

    // get() on missing key
    assert_eq!(obj.get("missing"), None);
    // get() on non-object
    assert_eq!(obj["a"].get("x"), None);

    // is_null
    assert!(obj["b"][1].is_null());
    assert!(!obj["a"].is_null());

    // as_bool
    assert_eq!(obj["b"][0].as_bool(), Some(true));
    assert_eq!(obj["a"].as_bool(), None);

    // as_i64
    assert_eq!(obj["a"].as_i64(), Some(1));
    assert_eq!(obj["d"].as_i64(), None);

    // as_f64
    assert_eq!(obj["c"].as_f64(), Some(3.14));
    assert_eq!(obj["a"].as_f64(), None);

    // as_str
    assert_eq!(obj["d"].as_str(), Some("hello"));
    assert_eq!(obj["a"].as_str(), None);

    // as_array
    assert!(obj["b"].as_array().is_some());
    assert_eq!(obj["a"].as_array(), None);

    // as_object
    assert!(obj.as_object().is_some());
    assert_eq!(obj["a"].as_object(), None);
}

#[test]
fn value_from_impls() {
    let _: Value = true.into();
    let _: Value = 42i64.into();
    let _: Value = 3.14f64.into();
    let _: Value = "hello".into();
    let _: Value = String::from("world").into();
    let _: Value = vec![Value::Null].into();
}

#[test]
#[should_panic(expected = "not an array")]
fn index_usize_on_non_array() {
    let val = parse("42").unwrap();
    let _ = &val[0];
}

#[test]
#[should_panic(expected = "key not found")]
fn index_str_on_missing_key() {
    let val = parse("{a: 1}").unwrap();
    let _ = &val["missing"];
}

#[test]
fn error_line_number() {
    let err = parse("{\n  a: 1\n  a: 2\n}").unwrap_err();
    assert!(err.line() > 0);
    let msg = err.to_string();
    assert!(msg.contains("Duplicate key"));
}

#[test]
fn unicode_escape_non_hex_first_char() {
    // \u{g} — first char after { is not hex and not }
    let err = parse("\"\\u{g}\"").unwrap_err();
    assert!(err.to_string().contains("Invalid escape sequence"));
}

#[test]
fn backslash_at_eof_no_trailing_newline() {
    // Backslash at end of input with no trailing newline
    let err = parse("\"abc\\").unwrap_err();
    assert!(err.to_string().contains("Unexpected end of input"));
}

#[test]
fn all_control_chars_below_u0020_rejected_except_tab() {
    for code in 0u8..0x20 {
        if code == 0x09 { continue; } // tab is allowed
        let input = format!("\"{ch}\"", ch = char::from(code));
        assert!(
            parse(&input).is_err(),
            "Expected error for control character 0x{code:02X}"
        );
    }
}

#[test]
fn string_rejects_del_u007f() {
    let input = "\"hello\x7Fworld\"";
    assert!(parse(input).is_err());
}

#[test]
fn string_allows_tab() {
    let result = parse("\"hello\tworld\"").unwrap();
    assert_eq!(result, Value::String("hello\tworld".into()));
}

#[test]
fn stringify_unicode_boundary_chars_pass_through() {
    let d7ff = Value::String("\u{D7FF}".into());
    assert_eq!(stringify(&d7ff), format!("\"\u{D7FF}\""));
    let e000 = Value::String("\u{E000}".into());
    assert_eq!(stringify(&e000), format!("\"\u{E000}\""));
    let sup = Value::String("\u{10000}".into());
    assert_eq!(stringify(&sup), format!("\"\u{10000}\""));
    let max = Value::String("\u{10FFFF}".into());
    assert_eq!(stringify(&max), format!("\"\u{10FFFF}\""));
}

#[test]
fn stringify_control_chars_0x01_to_0x1f_except_tab_escaped() {
    for code in 1u8..0x20 {
        if code == 0x09 { continue; } // tab uses \t
        if code == 0x0A { continue; } // newline uses \n
        if code == 0x0D { continue; } // CR uses \r
        let val = Value::String(String::from(char::from(code)));
        let result = stringify(&val);
        let expected = format!("\"\\u{{{:X}}}\"", code);
        assert_eq!(result, expected, "Mismatch for control character 0x{code:02X}");
    }
}
