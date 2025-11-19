use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
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

/// Parser definition
fn parser<'a>() -> impl Parser<'a, &'a str, MamlValue, extra::Err<Rich<'a, char>>> {
    recursive(|value| {
        // Comments
        let comment = just('#')
            .then(any().and_is(text::newline().not()).repeated())
            .ignored();

        // Whitespace (not including newlines)
        let ws = one_of(" \t").ignored().repeated();

        // Separator: comma, newline, or comment+newline
        // All branches must return () so we use .ignored()
        let separator = choice((
            just(',').ignored(),
            text::newline().ignored(),
            comment.then(text::newline().or_not()).ignored(),
        ))
        .padded_by(ws);

        // --- Numbers ---
        let digits = text::digits(10);
        let integer_part = choice((
            just('0').to_slice(),
            one_of("123456789").then(digits.or_not()).to_slice(),
        ));

        let fraction = just('.').then(digits);
        let exponent = one_of("eE").then(one_of("+-").or_not()).then(digits);

        let number = just('-')
            .or_not()
            .then(integer_part)
            .then(fraction.or_not())
            .then(exponent.or_not())
            .to_slice()
            .try_map(|s: &str, span| {
                if s.contains(['.', 'e', 'E']) {
                    s.parse::<f64>()
                        .map(MamlValue::Float)
                        .map_err(|_| Rich::custom(span, "Invalid float"))
                } else {
                    s.parse::<i64>()
                        .map(MamlValue::Int)
                        .map_err(|_| Rich::custom(span, "Integer overflow (must fit in 64-bit)"))
                }
            })
            .labelled("number");

        // --- Strings ---
        let escape = just('\\').ignore_then(choice((
            just('\\'),
            just('/'),
            just('"'),
            just('b').to('\x08'),
            just('f').to('\x0C'),
            just('n').to('\n'),
            just('r').to('\r'),
            just('t').to('\t'),
            just('u').ignore_then(
                text::digits(16)
                    .at_least(1)
                    .at_most(6)
                    .to_slice()
                    .delimited_by(just('{'), just('}'))
                    .try_map(|digits: &str, span| {
                        let code = u32::from_str_radix(digits, 16).unwrap();
                        char::from_u32(code).ok_or_else(|| {
                            Rich::custom(span, format!("Invalid unicode codepoint: U+{:X}", code))
                        })
                    }),
            ),
        )));

        let string_content = none_of("\\\"").or(escape).repeated().collect::<String>();
        let simple_string = string_content
            .delimited_by(just('"'), just('"'))
            .labelled("string");

        // Raw string: """..."""
        let raw_string = just("\"\"\"")
            .ignore_then(
                any()
                    .and_is(just("\"\"\"").not())
                    .repeated()
                    .collect::<String>(),
            )
            .then_ignore(just("\"\"\""))
            .map(|s: String| {
                // Strip leading newline if present
                s.strip_prefix('\n')
                    .or_else(|| s.strip_prefix("\r\n"))
                    .unwrap_or(&s)
                    .to_string()
            })
            .labelled("raw string");

        let string_val = choice((raw_string, simple_string)).map(MamlValue::String);

        // --- Keys ---
        let identifier = one_of("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_-")
            .repeated()
            .at_least(1)
            .collect::<String>();

        let key = choice((simple_string.clone(), identifier)).padded_by(ws);

        // --- Array ---
        let array = value
            .clone()
            .separated_by(separator.clone().repeated().at_least(1))
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(just('['), just(']'))
            .map(MamlValue::Array)
            .labelled("array")
            .recover_with(via_parser(nested_delimiters(
                '[',
                ']',
                [('[', ']'), ('{', '}')],
                |_| MamlValue::Array(vec![]),
            )));

        // --- Object ---
        let member = key.then_ignore(just(':').padded_by(ws)).then(value.clone());

        let object = member
            .separated_by(separator.repeated().at_least(1))
            .allow_trailing()
            .collect()
            .padded()
            .delimited_by(just('{'), just('}'))
            .map(MamlValue::Object)
            .labelled("object")
            .recover_with(via_parser(nested_delimiters(
                '{',
                '}',
                [('[', ']'), ('{', '}')],
                |_| MamlValue::Object(HashMap::new()),
            )));

        // --- Top-level choice ---
        choice((
            just("null").to(MamlValue::Null),
            just("true").to(MamlValue::Bool(true)),
            just("false").to(MamlValue::Bool(false)),
            number,
            string_val,
            array,
            object,
        ))
        .padded()
    })
}

/// Parse from string (like `serde_json::from_str`)
pub fn from_str(input: &str) -> Result<MamlValue, String> {
    let (val, errs) = parser().parse(input).into_output_errors();

    if !errs.is_empty() {
        let mut buffer = Vec::new();
        for e in errs {
            Report::build(ReportKind::Error, ("<input>", e.span().into_range()))
                .with_message(e.to_string())
                .with_label(
                    Label::new(("<input>", e.span().into_range()))
                        .with_message(e.reason().to_string())
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
    let (val, errs) = parser().parse(input).into_output_errors();

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
            .with_note(format!(
                "Error at line {} column {}",
                input[..span.start].lines().count(),
                input[..span.start]
                    .lines()
                    .last()
                    .map(|l| l.len())
                    .unwrap_or(0)
                    + 1
            ))
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
