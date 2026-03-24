use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    formatted: String,
    line: usize,
}

impl Error {
    pub(crate) fn new(formatted: String, line: usize) -> Self {
        Self { formatted, line }
    }

    pub fn line(&self) -> usize {
        self.line
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.formatted)
    }
}

impl std::error::Error for Error {}
