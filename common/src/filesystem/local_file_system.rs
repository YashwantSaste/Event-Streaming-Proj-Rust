use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

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

    fn read_directory(&self, path: &Path) -> Result<Vec<PathBuf>, ApplicationError> {
        let entries =
            fs::read_dir(path).map_err(|error| Self::map_error("read directory", path, error))?;

        entries
            .map(|entry| {
                entry
                    .map(|entry| entry.path())
                    .map_err(|error| Self::map_error("read directory entry", path, error))
            })
            .collect()
    }

    fn file_size(&self, path: &Path) -> Result<u64, ApplicationError> {
        fs::metadata(path)
            .map(|metadata| metadata.len())
            .map_err(|error| Self::map_error("read file metadata", path, error))
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

    fn is_directory(&self, path: &Path) -> bool {
        path.is_dir()
    }
}
