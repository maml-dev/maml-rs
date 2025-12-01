use logos::Logos;

use crate::utils::parse_string;

/// Token definitions for MAML
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t]+")]
pub enum Token {
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
        let content = &s[3..s.len()-3]; // The content within the triple quotes (which take up three chars each)

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
