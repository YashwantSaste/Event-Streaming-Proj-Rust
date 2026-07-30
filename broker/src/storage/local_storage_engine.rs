use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use common::error::storage_error::StorageError;
use common::filesystem::file_system::FileSystem;
use common::models::identifiers::{Offset, PartitionId, TopicName};
use common::models::partition::TopicPartition;
use common::models::record::{Record, RecordKey, RecordPayload};

use crate::storage::record_codec::RecordCodec;
use crate::storage::segment::{Segment, SegmentNamer};
use crate::storage::storage_engine::{StorageEngine, StoredRecordMetadata};

const PARTITION_PREFIX: &str = "partition-";
const SEGMENTS_DIRECTORY: &str = "segments";

#[derive(Debug, Clone)]
pub struct LocalStorageConfiguration {
    root_directory: PathBuf,
    segment_max_bytes: u64,
}

impl LocalStorageConfiguration {
    pub fn new(root_directory: PathBuf, segment_max_bytes: u64) -> Result<Self, StorageError> {
        if segment_max_bytes == 0 {
            return Err(StorageError::new(
                "storage segment_max_bytes must be greater than zero",
            ));
        }

        Ok(Self {
            root_directory,
            segment_max_bytes,
        })
    }

    pub fn root_directory(&self) -> &Path {
        &self.root_directory
    }

    pub fn segment_max_bytes(&self) -> u64 {
        self.segment_max_bytes
    }
}

#[derive(Debug, Clone)]
struct PartitionState {
    active_segment: Segment,
    next_offset: Offset,
}

pub struct LocalStorageEngine<F>
where
    F: FileSystem,
{
    file_system: F,
    configuration: LocalStorageConfiguration,
    partitions: HashMap<TopicPartition, PartitionState>,
}

impl<F> LocalStorageEngine<F>
where
    F: FileSystem,
{
    pub fn new(file_system: F, configuration: LocalStorageConfiguration) -> Self {
        Self {
            file_system,
            configuration,
            partitions: HashMap::new(),
        }
    }

    fn ensure_partition_state(
        &mut self,
        partition: &TopicPartition,
    ) -> Result<&mut PartitionState, StorageError> {
        if !self.partitions.contains_key(partition) {
            let state = self.recover_partition(partition)?;
            self.partitions.insert(partition.clone(), state);
        }

        self.partitions
            .get_mut(partition)
            .ok_or_else(|| StorageError::new("Partition state was not initialized"))
    }

    fn recover_partition(
        &self,
        partition: &TopicPartition,
    ) -> Result<PartitionState, StorageError> {
        let segments_directory = self.segments_directory(partition);
        self.create_directory(&segments_directory)?;

        let segments = self.load_segments(&segments_directory)?;
        let active_segment = segments
            .last()
            .cloned()
            .unwrap_or_else(|| Segment::new(1, SegmentNamer::path(&segments_directory, 1), 0));

        let next_offset = self.next_offset_from_segments(partition, &segments)?;

        Ok(PartitionState {
            active_segment,
            next_offset,
        })
    }

    fn load_segments(&self, segments_directory: &Path) -> Result<Vec<Segment>, StorageError> {
        if !self.file_system.exists(segments_directory) {
            return Ok(Vec::new());
        }

        let mut segments = self
            .file_system
            .read_directory(segments_directory)
            .map_err(|error| StorageError::new(error.to_string()))?
            .into_iter()
            .filter_map(|path| self.segment_from_path(path).transpose())
            .collect::<Result<Vec<_>, _>>()?;

        segments.sort_by_key(Segment::id);
        Ok(segments)
    }

    fn segment_from_path(&self, path: PathBuf) -> Result<Option<Segment>, StorageError> {
        let Some(segment_id) = SegmentNamer::parse_id(&path)? else {
            return Ok(None);
        };

        let size_bytes = self
            .file_system
            .file_size(&path)
            .map_err(|error| StorageError::new(error.to_string()))?;

        Ok(Some(Segment::new(segment_id, path, size_bytes)))
    }

    fn next_offset_from_segments(
        &self,
        partition: &TopicPartition,
        segments: &[Segment],
    ) -> Result<Offset, StorageError> {
        let mut next_offset = Offset::zero();

        for segment in segments {
            let records = self.read_segment_records(partition, segment)?;
            if let Some(last_record) = records.last() {
                next_offset = last_record.offset().next();
            }
        }

        Ok(next_offset)
    }

    fn read_partition_records(
        &self,
        partition: &TopicPartition,
    ) -> Result<Vec<Record>, StorageError> {
        let segments = self.load_segments(&self.segments_directory(partition))?;
        segments
            .iter()
            .map(|segment| self.read_segment_records(partition, segment))
            .try_fold(Vec::new(), |mut records, segment_records| {
                records.extend(segment_records?);
                Ok(records)
            })
    }

    fn read_segment_records(
        &self,
        partition: &TopicPartition,
        segment: &Segment,
    ) -> Result<Vec<Record>, StorageError> {
        if segment.size_bytes() == 0 || !self.file_system.exists(segment.path()) {
            return Ok(Vec::new());
        }

        let bytes = self
            .file_system
            .read_file(segment.path())
            .map_err(|error| StorageError::new(error.to_string()))?;

        RecordCodec::decode_all(
            bytes.as_slice(),
            partition.topic(),
            partition.partition_id(),
        )
    }

    fn rotate_partition(&mut self, partition: &TopicPartition) -> Result<(), StorageError> {
        let segments_directory = self.segments_directory(partition);
        self.create_directory(&segments_directory)?;

        let state = self.ensure_partition_state(partition)?;
        let next_segment_id = state.active_segment.id().saturating_add(1);
        state.active_segment = Segment::new(
            next_segment_id,
            SegmentNamer::path(&segments_directory, next_segment_id),
            0,
        );

        Ok(())
    }

    fn create_directory(&self, path: &Path) -> Result<(), StorageError> {
        self.file_system
            .create_directory(path)
            .map_err(|error| StorageError::new(error.to_string()))
    }

    fn append_file(&self, path: &Path, contents: &[u8]) -> Result<(), StorageError> {
        self.file_system
            .append_file(path, contents)
            .map_err(|error| StorageError::new(error.to_string()))
    }

    fn partition_directory(&self, partition: &TopicPartition) -> PathBuf {
        self.configuration
            .root_directory()
            .join(partition.topic().as_str())
            .join(Self::partition_directory_name(partition.partition_id()))
    }

    fn segments_directory(&self, partition: &TopicPartition) -> PathBuf {
        self.partition_directory(partition).join(SEGMENTS_DIRECTORY)
    }

    fn partition_directory_name(partition_id: PartitionId) -> String {
        format!("{PARTITION_PREFIX}{:06}", partition_id.value())
    }

    fn parse_partition_id(path: &Path) -> Result<Option<PartitionId>, StorageError> {
        let Some(directory_name) = path.file_name().and_then(|name| name.to_str()) else {
            return Ok(None);
        };

        let Some(id_text) = directory_name.strip_prefix(PARTITION_PREFIX) else {
            return Ok(None);
        };

        id_text
            .parse::<u32>()
            .map(|id| Some(PartitionId::new(id)))
            .map_err(|error| {
                StorageError::new(format!(
                    "Invalid partition directory '{directory_name}': {error}"
                ))
            })
    }

    fn discover_partitions(&self) -> Result<Vec<TopicPartition>, StorageError> {
        if !self.file_system.exists(self.configuration.root_directory()) {
            return Ok(Vec::new());
        }

        let topic_directories = self
            .file_system
            .read_directory(self.configuration.root_directory())
            .map_err(|error| StorageError::new(error.to_string()))?;

        topic_directories
            .into_iter()
            .filter(|path| self.file_system.is_directory(path))
            .map(|topic_directory| self.discover_topic_partitions(topic_directory))
            .try_fold(Vec::new(), |mut partitions, topic_partitions| {
                partitions.extend(topic_partitions?);
                Ok(partitions)
            })
    }

    fn discover_topic_partitions(
        &self,
        topic_directory: PathBuf,
    ) -> Result<Vec<TopicPartition>, StorageError> {
        let topic_name = topic_directory
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| StorageError::new("Invalid topic directory name"))
            .and_then(|name| {
                TopicName::new(name.to_string())
                    .map_err(|error| StorageError::new(error.to_string()))
            })?;

        let partition_directories = self
            .file_system
            .read_directory(&topic_directory)
            .map_err(|error| StorageError::new(error.to_string()))?;

        partition_directories
            .into_iter()
            .filter(|path| self.file_system.is_directory(path))
            .filter_map(|path| Self::parse_partition_id(&path).transpose())
            .map(|partition_id| {
                partition_id
                    .map(|partition_id| TopicPartition::new(topic_name.clone(), partition_id))
            })
            .collect()
    }
}

impl<F> StorageEngine for LocalStorageEngine<F>
where
    F: FileSystem,
{
    fn append(
        &mut self,
        partition: &TopicPartition,
        key: Option<RecordKey>,
        payload: RecordPayload,
    ) -> Result<StoredRecordMetadata, StorageError> {
        let next_offset = self.ensure_partition_state(partition)?.next_offset;
        let record = Record::new(
            partition.topic().clone(),
            partition.partition_id(),
            next_offset,
            key,
            payload,
            SystemTime::now(),
        );
        let encoded = RecordCodec::encode(&record)?;
        let encoded_size = u64::try_from(encoded.len())
            .map_err(|error| StorageError::new(format!("Encoded record is too large: {error}")))?;

        let should_rotate = {
            let state = self.ensure_partition_state(partition)?;
            state.active_segment.size_bytes() > 0
                && state
                    .active_segment
                    .size_bytes()
                    .saturating_add(encoded_size)
                    > self.configuration.segment_max_bytes()
        };

        if should_rotate {
            self.rotate_partition(partition)?;
        }

        let active_segment_path = {
            let state = self.ensure_partition_state(partition)?;
            state.active_segment.path().to_path_buf()
        };

        self.append_file(&active_segment_path, &encoded)?;

        let state = self.ensure_partition_state(partition)?;
        state.active_segment = state.active_segment.with_size(
            state
                .active_segment
                .size_bytes()
                .saturating_add(encoded_size),
        );
        state.next_offset = state.next_offset.next();

        Ok(StoredRecordMetadata::new(
            partition.clone(),
            record.offset(),
        ))
    }

    fn read(
        &mut self,
        partition: &TopicPartition,
        offset: Offset,
        max_records: usize,
    ) -> Result<Vec<Record>, StorageError> {
        self.ensure_partition_state(partition)?;

        if max_records == 0 {
            return Ok(Vec::new());
        }

        Ok(self
            .read_partition_records(partition)?
            .into_iter()
            .filter(|record| record.offset() >= offset)
            .take(max_records)
            .collect())
    }

    fn flush(&mut self, partition: &TopicPartition) -> Result<(), StorageError> {
        self.ensure_partition_state(partition).map(|_| ())
    }

    fn rotate(&mut self, partition: &TopicPartition) -> Result<(), StorageError> {
        self.rotate_partition(partition)
    }

    fn recover(&mut self) -> Result<(), StorageError> {
        self.create_directory(self.configuration.root_directory())?;

        for partition in self.discover_partitions()? {
            let state = self.recover_partition(&partition)?;
            self.partitions.insert(partition, state);
        }

        Ok(())
    }
}
