use std::collections::HashMap;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::prelude::*;
use logos::Logos;

use crate::{MamlValue, tokenizer::Token};

/// Raw parser entrypoint
pub fn parser<'src>() -> impl Parser<'src, &'src [Token], MamlValue, extra::Err<Rich<'src, Token>>>
{
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

/// Parse from string
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
