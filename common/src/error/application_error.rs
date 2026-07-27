use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct ApplicationError {
    message: String,
}

impl ApplicationError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApplicationError {}