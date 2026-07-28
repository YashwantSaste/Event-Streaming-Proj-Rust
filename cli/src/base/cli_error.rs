use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum CliError {
    InvalidArguments(String),
    UnsupportedCommand(String),
    UnexpectedError(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CliError::InvalidArguments(msg) => write!(f, "{msg}"),
            CliError::UnsupportedCommand(cmd) => {
                write!(f, "Unsupported command: {cmd}")
            }
            CliError::UnexpectedError(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CliError {}