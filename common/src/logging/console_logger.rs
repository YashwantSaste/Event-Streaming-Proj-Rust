use crate::error::application_error::ApplicationError;
use crate::logging::log_level::LogLevel;
use crate::logging::logger::Logger;

#[derive(Debug, Clone, Copy)]
pub struct ConsoleLogger {
    minimum_level: LogLevel,
}

impl ConsoleLogger {
    pub fn new(minimum_level: LogLevel) -> Self {
        Self { minimum_level }
    }

    fn should_log(&self, level: LogLevel) -> bool {
        level >= self.minimum_level
    }
}

impl Logger for ConsoleLogger {
    fn log(&self, level: LogLevel, message: &str) -> Result<(), ApplicationError> {
        if self.should_log(level) {
            println!("[{level}] {message}");
        }

        Ok(())
    }
}
