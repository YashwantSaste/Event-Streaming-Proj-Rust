use std::time::SystemTime;

use crate::models::identifiers::{Offset, PartitionId, TopicName};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordKey(Vec<u8>);

impl RecordKey {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self(value.into())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordPayload(Vec<u8>);

impl RecordPayload {
    pub fn new(value: impl Into<Vec<u8>>) -> Self {
        Self(value.into())
    }

    pub fn bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    topic: TopicName,
    partition_id: PartitionId,
    offset: Offset,
    key: Option<RecordKey>,
    payload: RecordPayload,
    timestamp: SystemTime,
}

impl Record {
    pub fn new(
        topic: TopicName,
        partition_id: PartitionId,
        offset: Offset,
        key: Option<RecordKey>,
        payload: RecordPayload,
        timestamp: SystemTime,
    ) -> Self {
        Self {
            topic,
            partition_id,
            offset,
            key,
            payload,
            timestamp,
        }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn key(&self) -> Option<&RecordKey> {
        self.key.as_ref()
    }

    pub fn payload(&self) -> &RecordPayload {
        &self.payload
    }

    pub fn timestamp(&self) -> SystemTime {
        self.timestamp
    }
}

#[derive(Debug, Clone, Default)]
pub struct RecordBatch {
    records: Vec<Record>,
}

impl RecordBatch {
    pub fn new(records: Vec<Record>) -> Self {
        Self { records }
    }

    pub fn empty() -> Self {
        Self::default()
    }

    pub fn push(&mut self, record: Record) {
        self.records.push(record);
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn records(&self) -> &[Record] {
        &self.records
    }

    pub fn into_records(self) -> Vec<Record> {
        self.records
    }
}
