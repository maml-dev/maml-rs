/// Parse escape sequences in strings
pub(crate) fn parse_string(s: &str) -> Option<String> {
    let mut result = String::new();
    let mut chars = s.chars();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next()? {
                '\\' => result.push('\\'),
                '/' => result.push('/'),
                '"' => result.push('"'),
                'b' => result.push('\x08'),
                'f' => result.push('\x0C'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                'u' => {
                    // Expect {XXXXXX}
                    if chars.next()? != '{' {
                        return None;
                    }
                    let mut hex = String::new();
                    loop {
                        match chars.next()? {
                            '}' => break,
                            c if c.is_ascii_hexdigit() && hex.len() < 6 => hex.push(c),
                            _ => return None,
                        }
                    }
                    let code = u32::from_str_radix(&hex, 16).ok()?;
                    result.push(char::from_u32(code)?);
                }
                _ => return None,
            }
        } else {
            result.push(ch);
        }
    }

    Some(result)
}
