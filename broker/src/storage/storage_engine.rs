use common::error::storage_error::StorageError;
use common::models::identifiers::Offset;
use common::models::partition::TopicPartition;
use common::models::record::{Record, RecordKey, RecordPayload};

#[derive(Debug, Clone)]
pub struct StoredRecordMetadata {
    partition: TopicPartition,
    offset: Offset,
}

impl StoredRecordMetadata {
    pub fn new(partition: TopicPartition, offset: Offset) -> Self {
        Self { partition, offset }
    }

    pub fn partition(&self) -> &TopicPartition {
        &self.partition
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }
}

pub trait StorageEngine {
    fn append(
        &mut self,
        partition: &TopicPartition,
        key: Option<RecordKey>,
        payload: RecordPayload,
    ) -> Result<StoredRecordMetadata, StorageError>;

    fn read(
        &mut self,
        partition: &TopicPartition,
        offset: Offset,
        max_records: usize,
    ) -> Result<Vec<Record>, StorageError>;

    fn flush(&mut self, partition: &TopicPartition) -> Result<(), StorageError>;

    fn rotate(&mut self, partition: &TopicPartition) -> Result<(), StorageError>;

    fn recover(&mut self) -> Result<(), StorageError>;
}
