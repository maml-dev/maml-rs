use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use logos::Logos;
use std::collections::HashMap;

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

/// Tokens for MAML
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
enum Token {
    #[token("null")]
    Null,

    #[token("true")]
    True,

    #[token("false")]
    False,

    // Float MUST come before Int to ensure proper decimal matching, so priority three
    #[regex(r"-?(?:0|[1-9][0-9]*)\.[0-9]+(?:[eE][+-]?[0-9]+)?", |lex| lex.slice().parse::<f64>().ok(), priority = 3)]
    #[regex(r"-?(?:0|[1-9][0-9]*)[eE][+-]?[0-9]+", |lex| lex.slice().parse::<f64>().ok(), priority = 3)]
    Float(f64),

    #[regex(r"-?(?:0|[1-9][0-9]*)", |lex| lex.slice().parse::<i64>().ok(), priority = 2)]
    Int(i64),

    #[regex(r#""(?:[^"\\]|\\["\\/bfnrt]|\\u\{[0-9a-fA-F]{1,6}\})*""#, |lex| {
        let s = lex.slice();
        parse_string(&s[1..s.len()-1])
    })]
    String(String),

    // Surrounded by triple quotes
    #[regex(r#""""([^"]|"[^"]|""[^"])*""""#, |lex| {
        let s = lex.slice();
        let content = &s[3..s.len()-3];

        // Make sure triple quotes are checked
        if content.contains(r#"""""#) {
            return None;
        }

        Some(content.strip_prefix('\n')
            .or_else(|| content.strip_prefix("\r\n"))
            .unwrap_or(content)
            .to_string())
    })]
    RawString(String),

    // An object key
    #[regex(r"[a-zA-Z_-][a-zA-Z0-9_-]*", |lex| lex.slice().to_string(), priority = 1)]
    #[regex(r"[0-9]+", |lex| lex.slice().to_string(), priority = 1)]
    Key(String),

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("\n")]
    Newline,

    // Anything that comes after a #
    #[regex(r"#[^\n]*", logos::skip)]
    Comment,
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Null => write!(f, "null"),
            Token::True => write!(f, "true"),
            Token::False => write!(f, "false"),
            Token::Float(n) => write!(f, "{}", n),
            Token::Int(n) => write!(f, "{}", n),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::RawString(s) => write!(f, "\"\"\"{}\"\"\"", s),
            Token::Key(s) => write!(f, "{}", s),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::Colon => write!(f, ":"),
            Token::Comma => write!(f, ","),
            Token::Newline => write!(f, "\\n"),
            Token::Comment => write!(f, "#comment"),
        }
    }
}

// Helper function to parse escape sequences in strings
fn parse_string(s: &str) -> Option<String> {
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

/// Parser definition
fn parser<'src>() -> impl Parser<'src, &'src [Token], MamlValue, extra::Err<Rich<'src, Token>>> {
    recursive(|value| {
        // Separator: comma or newline
        let separator = choice((just(Token::Comma).ignored(), just(Token::Newline).ignored()));

        // The number types
        let number = choice((
            select! { Token::Float(f) => MamlValue::Float(f) },
            select! { Token::Int(i) => MamlValue::Int(i) },
        ))
        .labelled("number");

        // Strings, raw or typical
        let string_val = choice((
            select! { Token::RawString(s) => MamlValue::String(s) },
            select! { Token::String(s) => MamlValue::String(s) },
        ));

        // Handling object keys
        let key = choice((
            select! { Token::String(s) => s },
            select! { Token::Key(s) => s },
        ));

        let array = value
            .clone()
            .separated_by(separator.clone().repeated().at_least(1))
            .allow_trailing()
            .collect()
            .padded_by(just(Token::Newline).repeated())
            .delimited_by(just(Token::LBracket), just(Token::RBracket))
            .map(MamlValue::Array)
            .labelled("array")
            .recover_with(via_parser(nested_delimiters(
                Token::LBracket,
                Token::RBracket,
                [
                    (Token::LBracket, Token::RBracket),
                    (Token::LBrace, Token::RBrace),
                ],
                |_| MamlValue::Array(vec![]),
            )));

        // Object parsing
        let member = key.then_ignore(just(Token::Colon)).then(value.clone());

        let object = member
            .separated_by(separator.repeated().at_least(1))
            .allow_trailing()
            .collect()
            .padded_by(just(Token::Newline).repeated())
            .delimited_by(just(Token::LBrace), just(Token::RBrace))
            .map(MamlValue::Object)
            .labelled("object")
            .recover_with(via_parser(nested_delimiters(
                Token::LBrace,
                Token::RBrace,
                [
                    (Token::LBracket, Token::RBracket),
                    (Token::LBrace, Token::RBrace),
                ],
                |_| MamlValue::Object(HashMap::new()),
            )));

        // Entry point/top-level choice
        choice((
            just(Token::Null).to(MamlValue::Null),
            just(Token::True).to(MamlValue::Bool(true)),
            just(Token::False).to(MamlValue::Bool(false)),
            number,
            string_val,
            array,
            object,
        ))
    })
}

/// Parse from string (like `serde_json::from_str`)
pub fn from_str(input: &str) -> Result<MamlValue, String> {
    // Tokenize
    let lexer = Token::lexer(input);
    let mut tokens = vec![];

    for (token_result, span) in lexer.spanned() {
        match token_result {
            Ok(token) => tokens.push(token),
            Err(_) => {
                return Err(format!("Lexer error at {:?}", span));
            }
        }
    }

    // Parse
    let (val, errs) = parser()
        .padded_by(just(Token::Newline).repeated())
        .parse(&tokens)
        .into_output_errors();

    if !errs.is_empty() {
        let mut buffer = Vec::new();
        for e in errs {
            Report::build(ReportKind::Error, ("<input>", e.span().into_range()))
                .with_message(format!("{:?}", e))
                .with_label(
                    Label::new(("<input>", e.span().into_range()))
                        .with_message(format!("{:?}", e.reason()))
                        .with_color(Color::Red),
                )
                .finish()
                .write(("<input>", Source::from(input)), &mut buffer)
                .unwrap();
        }
        return Err(String::from_utf8_lossy(&buffer).to_string());
    }

    val.ok_or_else(|| "Unexpected parsing failure".to_string())
}

/// Parse with detailed error reporting to stderr
pub fn parse_with_report(filename: &str, input: &str) -> Option<MamlValue> {
    // Tokenize
    let lexer = Token::lexer(input);
    let mut tokens = vec![];

    for (token_result, span) in lexer.spanned() {
        match token_result {
            Ok(token) => tokens.push(token),
            Err(_) => {
                eprintln!("Lexer error at {:?}", span);
                return None;
            }
        }
    }

    // Parse
    let (val, errs) = parser()
        .padded_by(just(Token::Newline).repeated())
        .parse(&tokens)
        .into_output_errors();

    if errs.is_empty() {
        return val;
    }

    for e in errs {
        let span = e.span().into_range();
        Report::build(ReportKind::Error, (filename, span.clone()))
            .with_message(format!("{}", e))
            .with_label(
                Label::new((filename, span.clone()))
                    .with_message(e.reason().to_string())
                    .with_color(Color::Red),
            )
            .finish()
            .eprint((filename, Source::from(input)))
            .unwrap();
    }

    None
}

#[cfg(test)]
mod tests {
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
