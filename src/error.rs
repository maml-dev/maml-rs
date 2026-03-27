use std::fmt;

/// Error type for MAML parsing and serialization.
///
/// Parse errors include a line number and a formatted snippet showing where
/// the error occurred. Serialization errors contain a description of the problem.
///
/// # Examples
///
/// ```
/// let err = maml::parse("{ invalid }").unwrap_err();
/// assert!(err.line().is_some());
/// assert!(err.message().contains("Unexpected"));
/// ```
#[derive(Debug, Clone)]
pub struct Error {
    message: String,
    line: Option<usize>,
}

impl Error {
    /// Creates a new error with the given message.
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

    /// Returns the line number where the error occurred, if available.
    ///
    /// Only set for parse errors. Serialization and deserialization errors
    /// return `None`.
    pub fn line(&self) -> Option<usize> {
        self.line
    }

    /// Returns the error message. For parse errors, this includes the
    /// line number and a snippet pointing to the problem.
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
