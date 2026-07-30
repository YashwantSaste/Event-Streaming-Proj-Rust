use crate::error::application_error::ApplicationError;
use crate::logging::log_level::LogLevel;

pub trait Logger {
    fn log(&self, level: LogLevel, message: &str) -> Result<(), ApplicationError>;

    fn trace(&self, message: &str) -> Result<(), ApplicationError> {
        self.log(LogLevel::Trace, message)
    }

    fn debug(&self, message: &str) -> Result<(), ApplicationError> {
        self.log(LogLevel::Debug, message)
    }

    fn info(&self, message: &str) -> Result<(), ApplicationError> {
        self.log(LogLevel::Info, message)
    }

    fn warn(&self, message: &str) -> Result<(), ApplicationError> {
        self.log(LogLevel::Warn, message)
    }

    fn error(&self, message: &str) -> Result<(), ApplicationError> {
        self.log(LogLevel::Error, message)
    }
}
