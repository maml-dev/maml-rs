use std::fmt;

use crate::value::Value;

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    line: Option<usize>,
    formatted: Option<String>,
}

impl Error {
    pub(crate) fn parse_error(formatted: String, line: usize) -> Self {
        Self {
            message: formatted.clone(),
            line: Some(line),
            formatted: Some(formatted),
        }
    }

    pub fn line(&self) -> Option<usize> {
        self.line
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(ref formatted) = self.formatted {
            write!(f, "{formatted}")
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for Error {}

impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self {
            message: msg.to_string(),
            line: None,
            formatted: None,
        }
    }
}

pub fn from_str<'de, T>(s: &str) -> Result<T, Error>
where
    T: serde::de::Deserialize<'de>,
{
    let value = crate::parse::parse(s)?;
    T::deserialize(value)
}

pub fn from_value<'de, T>(value: Value) -> Result<T, Error>
where
    T: serde::de::Deserialize<'de>,
{
    T::deserialize(value)
}
