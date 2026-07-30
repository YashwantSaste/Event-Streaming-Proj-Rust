use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::error::application_error::ApplicationError;
use crate::filesystem::file_system::FileSystem;

/// Local filesystem adapter backed by the operating system filesystem.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFileSystem;

impl LocalFileSystem {
    /// Creates a new local filesystem adapter.
    pub fn new() -> Self {
        Self
    }

    fn map_error(operation: &str, path: &Path, error: std::io::Error) -> ApplicationError {
        ApplicationError::new(format!(
            "Failed to {operation} '{}': {error}",
            path.display()
        ))
    }
}

impl FileSystem for LocalFileSystem {
    fn create_directory(&self, path: &Path) -> Result<(), ApplicationError> {
        fs::create_dir_all(path).map_err(|error| Self::map_error("create directory", path, error))
    }

    fn create_directories(&self, paths: &[&Path]) -> Result<(), ApplicationError> {
        paths
            .iter()
            .try_for_each(|path| self.create_directory(path))
    }

    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), ApplicationError> {
        fs::write(path, contents).map_err(|error| Self::map_error("write file", path, error))
    }

    fn append_file(&self, path: &Path, contents: &[u8]) -> Result<(), ApplicationError> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|error| Self::map_error("open file for append", path, error))?;

        file.write_all(contents)
            .map_err(|error| Self::map_error("append file", path, error))
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, ApplicationError> {
        fs::read(path).map_err(|error| Self::map_error("read file", path, error))
    }

    fn delete(&self, path: &Path) -> Result<(), ApplicationError> {
        if path.is_dir() {
            fs::remove_dir_all(path)
                .map_err(|error| Self::map_error("delete directory", path, error))
        } else {
            fs::remove_file(path).map_err(|error| Self::map_error("delete file", path, error))
        }
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    fn test_root(name: &str) -> Result<PathBuf, ApplicationError> {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| ApplicationError::new(error.to_string()))?
            .as_nanos();

        Ok(std::env::temp_dir().join(format!("event_streaming_{name}_{suffix}")))
    }

    #[test]
    fn creates_directory_tree() -> Result<(), ApplicationError> {
        let file_system = LocalFileSystem::new();
        let root = test_root("creates_directory_tree")?;
        let nested = root.join("a").join("b");

        file_system.create_directory(&nested)?;

        assert!(file_system.exists(&nested));

        file_system.delete(&root)
    }

    #[test]
    fn writes_appends_and_reads_file() -> Result<(), ApplicationError> {
        let file_system = LocalFileSystem::new();
        let root = test_root("writes_appends_and_reads_file")?;
        let file = root.join("records.log");

        file_system.create_directory(&root)?;
        file_system.write_file(&file, b"first")?;
        file_system.append_file(&file, b"-second")?;

        assert_eq!(file_system.read_file(&file)?, b"first-second");

        file_system.delete(&root)
    }
}
