use std::path::Path;

use crate::error::application_error::ApplicationError;

/// Abstraction for filesystem operations used by application components.
pub trait FileSystem {
    /// Creates a directory and any missing parent directories.
    fn create_directory(&self, path: &Path) -> Result<(), ApplicationError>;

    /// Creates all directories in the provided slice.
    fn create_directories(&self, paths: &[&Path]) -> Result<(), ApplicationError>;

    /// Writes bytes to a file, replacing any existing content.
    fn write_file(&self, path: &Path, contents: &[u8]) -> Result<(), ApplicationError>;

    /// Appends bytes to a file, creating the file if it does not exist.
    fn append_file(&self, path: &Path, contents: &[u8]) -> Result<(), ApplicationError>;

    /// Reads all bytes from a file.
    fn read_file(&self, path: &Path) -> Result<Vec<u8>, ApplicationError>;

    /// Deletes a file or directory tree.
    fn delete(&self, path: &Path) -> Result<(), ApplicationError>;

    /// Returns true when the path exists.
    fn exists(&self, path: &Path) -> bool;
}
