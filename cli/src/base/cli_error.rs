use std::fmt;

#[derive(Debug, Clone)]
pub struct CliError {
    pub exit_code: i32,
    pub message: String,
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Exit Code : {}\nMessage   : {}",
            self.exit_code,
            self.message
        )
    }
}

impl std::error::Error for CliError {}