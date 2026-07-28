use std::fmt;

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub exit_code: i32,
    pub message: String,
}

impl fmt::Display for CommandResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Command Result")?;
        writeln!(f, "--------------")?;
        writeln!(f, "Success   : {}", self.success)?;
        writeln!(f, "Exit Code : {}", self.exit_code)?;
        write!(f, "Message   : {}", self.message)
    }
}