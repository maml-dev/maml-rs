use crate::value::Value;

pub fn stringify(value: &Value) -> String {
    do_stringify(value, 0)
}

fn do_stringify(value: &Value, level: usize) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(n) => {
            if *n == 0.0 && n.is_sign_negative() {
                "-0".to_string()
            } else {
                let s = n.to_string();
                // Ensure floats always have a decimal point or exponent
                // so they re-parse as floats, not integers
                if s.contains('.') || s.contains('e') || s.contains('E') {
                    s
                } else {
                    format!("{s}.0")
                }
            }
        }
        Value::String(s) => quote_string(s),
        Value::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let child_indent = get_indent(level + 1);
            let parent_indent = get_indent(level);
            let mut out = "[\n".to_string();
            for (i, item) in arr.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                out.push_str(&child_indent);
                out.push_str(&do_stringify(item, level + 1));
            }
            out.push('\n');
            out.push_str(&parent_indent);
            out.push(']');
            out
        }
        Value::Object(pairs) => {
            if pairs.is_empty() {
                return "{}".to_string();
            }
            let child_indent = get_indent(level + 1);
            let parent_indent = get_indent(level);
            let mut out = "{\n".to_string();
            for (i, (key, val)) in pairs.iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                out.push_str(&child_indent);
                out.push_str(&stringify_key(key));
                out.push_str(": ");
                out.push_str(&do_stringify(val, level + 1));
            }
            out.push('\n');
            out.push_str(&parent_indent);
            out.push('}');
            out
        }
    }
}

fn is_identifier_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}

fn stringify_key(key: &str) -> String {
    if is_identifier_key(key) {
        key.to_string()
    } else {
        quote_string(key)
    }
}

fn quote_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || c == '\x7F' => {
                out.push_str(&format!("\\u{{{:X}}}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn get_indent(level: usize) -> String {
    " ".repeat(2 * level)
}
