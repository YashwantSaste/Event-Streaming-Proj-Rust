use std::path::{Path, PathBuf};

use crate::error::application_error::ApplicationError;
use crate::filesystem::file_system::FileSystem;
use crate::filesystem::local_file_system::LocalFileSystem;

/// Compatibility facade for simple filesystem operations.
pub struct FileManager;

impl FileManager {
    /// Creates a directory and any missing parent directories.
    pub fn create_directory(path: impl AsRef<Path>) -> Result<(), ApplicationError> {
        LocalFileSystem::new().create_directory(path.as_ref())
    }

    /// Creates every directory in the provided iterator.
    pub fn create_directories<I, P>(paths: I) -> Result<(), ApplicationError>
    where
        I: IntoIterator<Item = P>,
        P: AsRef<Path>,
    {
        paths.into_iter().try_for_each(Self::create_directory)
    }

    /// Writes bytes to a file, replacing any existing content.
    pub fn write_file(
        path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), ApplicationError> {
        LocalFileSystem::new().write_file(path.as_ref(), contents.as_ref())
    }

    /// Appends bytes to a file, creating the file if it does not exist.
    pub fn append_file(
        path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), ApplicationError> {
        LocalFileSystem::new().append_file(path.as_ref(), contents.as_ref())
    }

    /// Reads all bytes from a file.
    pub fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>, ApplicationError> {
        LocalFileSystem::new().read_file(path.as_ref())
    }

    /// Returns immediate child paths for a directory.
    pub fn read_directory(path: impl AsRef<Path>) -> Result<Vec<PathBuf>, ApplicationError> {
        LocalFileSystem::new().read_directory(path.as_ref())
    }

    /// Returns the current size of a file in bytes.
    pub fn file_size(path: impl AsRef<Path>) -> Result<u64, ApplicationError> {
        LocalFileSystem::new().file_size(path.as_ref())
    }

    /// Deletes a file or directory tree.
    pub fn delete(path: impl AsRef<Path>) -> Result<(), ApplicationError> {
        LocalFileSystem::new().delete(path.as_ref())
    }

    /// Returns true when the path exists.
    pub fn exists(path: impl AsRef<Path>) -> bool {
        LocalFileSystem::new().exists(path.as_ref())
    }

    /// Returns true when the path exists and is a directory.
    pub fn is_directory(path: impl AsRef<Path>) -> bool {
        LocalFileSystem::new().is_directory(path.as_ref())
    }
}
