use std::path::PathBuf;

use crate::error::application_error::ApplicationError;
use crate::filesystem::file_system::FileSystem;
use crate::logging::log_level::LogLevel;
use crate::logging::logger::Logger;

pub struct FileLogger<F>
where
    F: FileSystem,
{
    file_system: F,
    file_path: PathBuf,
    minimum_level: LogLevel,
}

impl<F> FileLogger<F>
where
    F: FileSystem,
{
    pub fn new(file_system: F, file_path: PathBuf, minimum_level: LogLevel) -> Self {
        Self {
            file_system,
            file_path,
            minimum_level,
        }
    }

    fn should_log(&self, level: LogLevel) -> bool {
        level >= self.minimum_level
    }
}

impl<F> Logger for FileLogger<F>
where
    F: FileSystem,
{
    fn log(&self, level: LogLevel, message: &str) -> Result<(), ApplicationError> {
        if !self.should_log(level) {
            return Ok(());
        }

        let line = format!("[{level}] {message}\n");
        self.file_system
            .append_file(&self.file_path, line.as_bytes())
    }
}
