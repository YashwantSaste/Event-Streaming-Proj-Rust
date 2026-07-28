use std::fs;
use std::path::Path;

use crate::error::application_error::ApplicationError;

pub struct FileManager;

impl FileManager {
    pub fn create_directory(path: impl AsRef<Path>) -> Result<(), ApplicationError> {
        fs::create_dir_all(path.as_ref())
            .map_err(|error| ApplicationError::new(error.to_string()))
    }

    pub fn create_directories<I, P>(paths: I) -> Result<(), ApplicationError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        for path in paths {
            Self::create_directory(path)?;
        }

        Ok(())
    }
}