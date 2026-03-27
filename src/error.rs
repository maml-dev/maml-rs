use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    line: Option<usize>,
}

impl Error {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            line: None,
        }
    }

    pub(crate) fn parse_error(formatted: String, line: usize) -> Self {
        Self {
            message: formatted,
            line: Some(line),
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
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for Error {}

#[cfg(feature = "serde")]
impl serde::de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self {
            message: msg.to_string(),
            line: None,
        }
    }
}

#[cfg(feature = "serde")]
impl serde::ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Self {
        Self {
            message: msg.to_string(),
            line: None,
        }
    }
}
