use std::collections::HashSet;

use crate::de::Error;
use crate::value::Value;

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    ch: u8,
    line_number: usize,
    done: bool,
}

pub fn parse(source: &str) -> Result<Value, Error> {
    let mut p = Parser::new(source);
    let value = p.parse_value()?;
    p.skip_whitespace();
    if !p.done {
        return Err(p.error_snippet(None));
    }
    match value {
        Some(v) => Ok(v),
        None => Err(p.error_snippet(None)),
    }
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        let bytes = source.as_bytes();
        let mut p = Parser {
            source,
            bytes,
            pos: 0,
            ch: 0,
            line_number: 1,
            done: false,
        };
        p.next();
        p
    }

    fn next(&mut self) {
        if self.pos < self.bytes.len() {
            self.ch = self.bytes[self.pos];
            self.pos += 1;
            if self.ch == b'\n' {
                self.line_number += 1;
            }
        } else {
            self.ch = 0;
            self.done = true;
        }
    }

    fn lookahead(&self, n: usize) -> &[u8] {
        let end = (self.pos + n).min(self.bytes.len());
        &self.bytes[self.pos..end]
    }

    fn parse_value(&mut self) -> Result<Option<Value>, Error> {
        self.skip_whitespace();
        if let Some(v) = self.parse_raw_string()? {
            return Ok(Some(v));
        }
        if let Some(s) = self.parse_string()? {
            return Ok(Some(Value::String(s)));
        }
        if let Some(v) = self.parse_number()? {
            return Ok(Some(v));
        }
        if let Some(v) = self.parse_object()? {
            return Ok(Some(v));
        }
        if let Some(v) = self.parse_array()? {
            return Ok(Some(v));
        }
        if let Some(v) = self.parse_keyword(b"true", Value::Bool(true))? {
            return Ok(Some(v));
        }
        if let Some(v) = self.parse_keyword(b"false", Value::Bool(false))? {
            return Ok(Some(v));
        }
        if let Some(v) = self.parse_keyword(b"null", Value::Null)? {
            return Ok(Some(v));
        }
        Ok(None)
    }

    fn parse_string(&mut self) -> Result<Option<String>, Error> {
        if self.ch != b'"' {
            return Ok(None);
        }
        self.parse_string_body().map(Some)
    }

    fn parse_string_body(&mut self) -> Result<String, Error> {
        let mut s = String::new();
        let mut escaped = false;
        loop {
            self.next();
            if escaped {
                if self.ch == b'u' {
                    self.next();
                    if self.ch != b'{' {
                        return Err(self.error_snippet(Some(format!(
                            "Invalid escape sequence {} (expected \"{{\")",
                            format_char(self.ch)
                        ))));
                    }
                    let mut hex = String::new();
                    loop {
                        self.next();
                        if self.ch == b'}' {
                            break;
                        }
                        if !is_hex_digit(self.ch) {
                            if hex.is_empty() {
                                return Err(
                                    self.error_snippet(Some("Invalid escape sequence".into()))
                                );
                            }
                            return Err(self.error_snippet(Some(format!(
                                "Invalid escape sequence {}",
                                format_char(self.ch)
                            ))));
                        }
                        hex.push(self.ch as char);
                        if hex.len() > 6 {
                            return Err(self.error_snippet(Some(
                                "Invalid escape sequence (too many hex digits)".into(),
                            )));
                        }
                    }
                    if hex.is_empty() {
                        return Err(self.error_snippet(Some("Invalid escape sequence".into())));
                    }
                    let code_point = u32::from_str_radix(&hex, 16).unwrap();
                    if code_point > 0x10FFFF || (0xD800..=0xDFFF).contains(&code_point) {
                        return Err(self
                            .error_snippet(Some("Invalid escape sequence (out of range)".into())));
                    }
                    // Code point is a valid Unicode scalar value; safe to unwrap.
                    s.push(char::from_u32(code_point).unwrap());
                } else {
                    match escape_char(self.ch) {
                        Some(c) => s.push(c),
                        None => {
                            return Err(self.error_snippet(Some(format!(
                                "Invalid escape sequence {}",
                                format_char(self.ch)
                            ))));
                        }
                    }
                }
                escaped = false;
            } else if self.ch == b'\\' {
                escaped = true;
            } else if self.ch == b'"' {
                break;
            } else if self.ch == b'\n'
                || self.done
                || (self.ch < 0x20 && self.ch != b'\t')
                || self.ch == 0x7F
            {
                return Err(self.error_snippet(None));
            } else {
                // Multi-byte UTF-8: copy full character
                if self.ch < 0x80 {
                    s.push(self.ch as char);
                } else {
                    // Multi-byte UTF-8: source is valid &str, so chars().next() is guaranteed
                    let start = self.pos - 1;
                    let c = self.source[start..].chars().next().unwrap();
                    s.push(c);
                    for _ in 1..c.len_utf8() {
                        self.next();
                    }
                }
            }
        }
        self.next();
        Ok(s)
    }

    fn parse_raw_string(&mut self) -> Result<Option<Value>, Error> {
        if self.ch != b'"' || self.lookahead(2) != b"\"\"" {
            return Ok(None);
        }
        self.next(); // skip second "
        self.next(); // skip third "
        self.next(); // move to content

        let mut has_leading_newline = false;
        // Strip leading CRLF or LF (not bare CR)
        if self.ch == b'\r' && self.lookahead(1) == b"\n" {
            self.next(); // skip \r, now at \n
        }
        if self.ch == b'\n' {
            has_leading_newline = true;
            self.next();
        }

        let mut s = String::new();
        while !self.done {
            if self.ch == b'"' && self.lookahead(2) == b"\"\"" {
                self.next();
                self.next();
                self.next();
                if s.is_empty() && !has_leading_newline {
                    return Err(self.error_snippet(Some("Raw strings cannot be empty".into())));
                }
                return Ok(Some(Value::String(s)));
            }
            // Copy byte — for multi-byte UTF-8, copy full character
            if self.ch < 0x80 {
                s.push(self.ch as char);
            } else {
                // source is valid &str, so chars().next() is guaranteed
                let start = self.pos - 1;
                let c = self.source[start..].chars().next().unwrap();
                s.push(c);
                for _ in 1..c.len_utf8() {
                    self.next();
                }
            }
            self.next();
        }
        Err(self.error_snippet(None))
    }

    fn parse_number(&mut self) -> Result<Option<Value>, Error> {
        if !is_digit(self.ch) && self.ch != b'-' {
            return Ok(None);
        }
        let mut num_str = String::new();
        let mut is_float = false;

        if self.ch == b'-' {
            num_str.push('-');
            self.next();
            if !is_digit(self.ch) {
                return Err(self.error_snippet(None));
            }
        }

        if self.ch == b'0' {
            num_str.push('0');
            self.next();
        } else {
            while is_digit(self.ch) {
                num_str.push(self.ch as char);
                self.next();
            }
        }

        if self.ch == b'.' {
            is_float = true;
            num_str.push('.');
            self.next();
            if !is_digit(self.ch) {
                return Err(self.error_snippet(None));
            }
            while is_digit(self.ch) {
                num_str.push(self.ch as char);
                self.next();
            }
        }

        if self.ch == b'e' || self.ch == b'E' {
            is_float = true;
            num_str.push(self.ch as char);
            self.next();
            if self.ch == b'+' || self.ch == b'-' {
                num_str.push(self.ch as char);
                self.next();
            }
            if !is_digit(self.ch) {
                return Err(self.error_snippet(None));
            }
            while is_digit(self.ch) {
                num_str.push(self.ch as char);
                self.next();
            }
        }

        if is_float {
            // Format is pre-validated, parse cannot fail
            let n: f64 = num_str.parse().unwrap();
            Ok(Some(Value::Float(n)))
        } else if num_str == "-0" {
            Ok(Some(Value::Float(-0.0)))
        } else {
            let n: i64 = num_str.parse().map_err(|_| {
                self.error_snippet(Some(format!("Integer out of range: {num_str}")))
            })?;
            Ok(Some(Value::Int(n)))
        }
    }

    fn parse_object(&mut self) -> Result<Option<Value>, Error> {
        if self.ch != b'{' {
            return Ok(None);
        }
        self.next();
        self.skip_whitespace();

        let mut pairs: Vec<(String, Value)> = Vec::new();
        let mut seen_keys: HashSet<String> = HashSet::new();

        if self.ch == b'}' {
            self.next();
            return Ok(Some(Value::Object(pairs)));
        }

        loop {
            let key_pos = self.pos;
            let key = if self.ch == b'"' {
                self.parse_string_body()?
            } else {
                self.parse_key()?
            };

            if !seen_keys.insert(key.clone()) {
                self.pos = key_pos;
                return Err(self.error_snippet(Some(format!("Duplicate key {:?}", key))));
            }

            self.skip_whitespace();
            if self.ch != b':' {
                return Err(self.error_snippet(None));
            }
            self.next();

            let value = self.parse_value()?;
            match value {
                Some(v) => pairs.push((key, v)),
                None => return Err(self.error_snippet(None)),
            }

            let newline_after_value = self.skip_whitespace();
            if self.ch == b'}' {
                self.next();
                return Ok(Some(Value::Object(pairs)));
            } else if self.ch == b',' {
                self.next();
                self.skip_whitespace();
                if self.ch == b'}' {
                    self.next();
                    return Ok(Some(Value::Object(pairs)));
                }
            } else if newline_after_value {
                continue;
            } else {
                return Err(self.error_snippet(Some(
                    "Expected comma or newline between key-value pairs".into(),
                )));
            }
        }
    }

    fn parse_key(&mut self) -> Result<String, Error> {
        let mut ident = String::new();
        while is_key_char(self.ch) {
            ident.push(self.ch as char);
            self.next();
        }
        if ident.is_empty() {
            return Err(self.error_snippet(None));
        }
        Ok(ident)
    }

    fn parse_array(&mut self) -> Result<Option<Value>, Error> {
        if self.ch != b'[' {
            return Ok(None);
        }
        self.next();
        self.skip_whitespace();

        let mut arr: Vec<Value> = Vec::new();

        if self.ch == b']' {
            self.next();
            return Ok(Some(Value::Array(arr)));
        }

        loop {
            let value = self.parse_value()?;
            match value {
                Some(v) => arr.push(v),
                None => return Err(self.error_snippet(None)),
            }

            let newline_after_value = self.skip_whitespace();
            if self.ch == b']' {
                self.next();
                return Ok(Some(Value::Array(arr)));
            } else if self.ch == b',' {
                self.next();
                self.skip_whitespace();
                if self.ch == b']' {
                    self.next();
                    return Ok(Some(Value::Array(arr)));
                }
            } else if newline_after_value {
                continue;
            } else {
                return Err(
                    self.error_snippet(Some("Expected comma or newline between values".into()))
                );
            }
        }
    }

    fn parse_keyword(&mut self, name: &[u8], value: Value) -> Result<Option<Value>, Error> {
        if self.ch != name[0] {
            return Ok(None);
        }
        for &expected in &name[1..] {
            self.next();
            if self.ch != expected {
                return Err(self.error_snippet(None));
            }
        }
        self.next();
        if is_whitespace(self.ch)
            || self.ch == b','
            || self.ch == b'}'
            || self.ch == b']'
            || self.done
        {
            Ok(Some(value))
        } else {
            Err(self.error_snippet(None))
        }
    }

    fn skip_whitespace(&mut self) -> bool {
        let mut has_newline = false;
        while is_whitespace(self.ch) {
            has_newline |= self.ch == b'\n';
            self.next();
        }
        let has_newline_after_comment = self.skip_comment();
        has_newline || has_newline_after_comment
    }

    fn skip_comment(&mut self) -> bool {
        if self.ch == b'#' {
            while !self.done && self.ch != b'\n' {
                self.next();
            }
            return self.skip_whitespace();
        }
        false
    }

    fn error_snippet(&self, message: Option<String>) -> Error {
        let message = if self.done {
            "Unexpected end of input".to_string()
        } else {
            message.unwrap_or_else(|| {
                // !self.done guarantees pos >= 1 and source[pos-1..] is non-empty valid UTF-8
                let byte_pos = self.pos - 1;
                let c = self.source[byte_pos..].chars().next().unwrap();
                let ch_repr = format!("{:?}", c.to_string());
                format!("Unexpected character {ch_repr}")
            })
        };

        let pos = self.pos;
        // Ensure start and end are on char boundaries
        let start = floor_char_boundary(self.source, pos.saturating_sub(40));
        let safe_pos = floor_char_boundary(self.source, pos);
        let before = &self.source[start..safe_pos];
        let lines: Vec<&str> = before.split('\n').collect();

        let mut last_line = lines.last().copied().unwrap_or("");
        let end = ceil_char_boundary(self.source, pos + 40);
        let safe_pos_after = ceil_char_boundary(self.source, pos);
        let after = &self.source[safe_pos_after..end];
        let mut postfix = after.split('\n').next().unwrap_or("");

        let mut line_number = self.line_number;

        if last_line.is_empty() && lines.len() >= 2 {
            // error at "\n"
            last_line = lines[lines.len() - 2];
            postfix = "";
            line_number -= 1;
        }

        let snippet = format!("    {last_line}{postfix}");
        // Use char count for dots to match JS behavior
        let dot_count = last_line.chars().count().saturating_sub(1);
        let dots = ".".repeat(dot_count);
        let pointer = format!("    {dots}^");
        let formatted = format!("{message} on line {line_number}.\n\n{snippet}\n{pointer}");

        Error::parse_error(formatted, line_number)
    }
}

fn is_whitespace(ch: u8) -> bool {
    ch == b' ' || ch == b'\n' || ch == b'\t' || ch == b'\r'
}

fn is_digit(ch: u8) -> bool {
    ch.is_ascii_digit()
}

fn is_hex_digit(ch: u8) -> bool {
    ch.is_ascii_digit() || (b'A'..=b'F').contains(&ch)
}

fn is_key_char(ch: u8) -> bool {
    ch.is_ascii_alphanumeric() || ch == b'_' || ch == b'-'
}

fn escape_char(ch: u8) -> Option<char> {
    match ch {
        b'"' => Some('"'),
        b'\\' => Some('\\'),
        b'n' => Some('\n'),
        b'r' => Some('\r'),
        b't' => Some('\t'),
        _ => None,
    }
}

fn format_char(ch: u8) -> String {
    if ch == 0 {
        return "\"\"".to_string();
    }
    let c = ch as char;
    format!("{:?}", c.to_string())
}

fn floor_char_boundary(s: &str, pos: usize) -> usize {
    let pos = pos.min(s.len());
    let mut p = pos;
    while p > 0 && !s.is_char_boundary(p) {
        p -= 1;
    }
    p
}

fn ceil_char_boundary(s: &str, pos: usize) -> usize {
    let pos = pos.min(s.len());
    let mut p = pos;
    while p < s.len() && !s.is_char_boundary(p) {
        p += 1;
    }
    p
}
