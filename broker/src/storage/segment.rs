use std::path::{Path, PathBuf};

use common::error::storage_error::StorageError;

const SEGMENT_EXTENSION: &str = "log";
const SEGMENT_PREFIX: &str = "segment-";

#[derive(Debug, Clone)]
pub struct Segment {
    id: u64,
    path: PathBuf,
    size_bytes: u64,
}

impl Segment {
    pub fn new(id: u64, path: PathBuf, size_bytes: u64) -> Self {
        Self {
            id,
            path,
            size_bytes,
        }
    }

    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn size_bytes(&self) -> u64 {
        self.size_bytes
    }

    pub fn with_size(&self, size_bytes: u64) -> Self {
        Self {
            id: self.id,
            path: self.path.clone(),
            size_bytes,
        }
    }
}

pub struct SegmentNamer;

impl SegmentNamer {
    pub fn file_name(id: u64) -> String {
        format!("{SEGMENT_PREFIX}{id:06}.{SEGMENT_EXTENSION}")
    }

    pub fn path(directory: &Path, id: u64) -> PathBuf {
        directory.join(Self::file_name(id))
    }

    pub fn parse_id(path: &Path) -> Result<Option<u64>, StorageError> {
        let Some(file_name) = path.file_name().and_then(|name| name.to_str()) else {
            return Ok(None);
        };

        if !file_name.starts_with(SEGMENT_PREFIX) || !file_name.ends_with(SEGMENT_EXTENSION) {
            return Ok(None);
        }

        let id_text = file_name
            .trim_start_matches(SEGMENT_PREFIX)
            .trim_end_matches(&format!(".{SEGMENT_EXTENSION}"));

        id_text.parse::<u64>().map(Some).map_err(|error| {
            StorageError::new(format!("Invalid segment file '{file_name}': {error}"))
        })
    }
}
