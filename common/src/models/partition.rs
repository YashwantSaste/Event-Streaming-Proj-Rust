use crate::error::application_error::ApplicationError;
use crate::models::identifiers::{Offset, PartitionId, TopicName};

#[derive(Debug, Clone)]
pub struct PartitionConfiguration {
    segment_max_bytes: u64,
}

impl PartitionConfiguration {
    pub fn new(segment_max_bytes: u64) -> Result<Self, ApplicationError> {
        if segment_max_bytes == 0 {
            return Err(ApplicationError::new(
                "partition segment_max_bytes must be greater than zero",
            ));
        }

        Ok(Self { segment_max_bytes })
    }

    pub fn segment_max_bytes(&self) -> u64 {
        self.segment_max_bytes
    }
}

#[derive(Debug, Clone)]
pub struct Partition {
    topic: TopicName,
    id: PartitionId,
    next_offset: Offset,
    configuration: PartitionConfiguration,
}

impl Partition {
    pub fn new(topic: TopicName, id: PartitionId, configuration: PartitionConfiguration) -> Self {
        Self {
            topic,
            id,
            next_offset: Offset::zero(),
            configuration,
        }
    }

    pub fn with_next_offset(
        topic: TopicName,
        id: PartitionId,
        next_offset: Offset,
        configuration: PartitionConfiguration,
    ) -> Self {
        Self {
            topic,
            id,
            next_offset,
            configuration,
        }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn id(&self) -> PartitionId {
        self.id
    }

    pub fn next_offset(&self) -> Offset {
        self.next_offset
    }

    pub fn configuration(&self) -> &PartitionConfiguration {
        &self.configuration
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TopicPartition {
    topic: TopicName,
    partition_id: PartitionId,
}

impl TopicPartition {
    pub fn new(topic: TopicName, partition_id: PartitionId) -> Self {
        Self {
            topic,
            partition_id,
        }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }
}
