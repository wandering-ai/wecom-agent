use std::error::Error as StdError;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Error {
    code: i64,
    text: String,
}

impl Error {
    pub fn new(code: i64, text: String) -> Self {
        Self { code, text }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error code: {}, {}", self.code, self.text)
    }
}

impl StdError for Error {}
