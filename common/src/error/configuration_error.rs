use core::fmt;
use std::error::Error;
use std::fmt::Formatter;

#[derive(Clone, Debug)]
pub struct ConfigurationError {
    message: String
}

impl ConfigurationError {
    pub fn new(message: impl Into<String>) -> ConfigurationError {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for ConfigurationError {}
