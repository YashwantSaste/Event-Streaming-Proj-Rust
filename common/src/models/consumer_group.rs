use std::collections::HashMap;

use crate::models::consumer::Consumer;
use crate::models::identifiers::{ConsumerGroupId, ConsumerId, Offset};
use crate::models::partition::TopicPartition;

#[derive(Debug, Clone)]
pub struct PartitionAssignment {
    consumer_id: ConsumerId,
    partitions: Vec<TopicPartition>,
}

impl PartitionAssignment {
    pub fn new(consumer_id: ConsumerId, partitions: Vec<TopicPartition>) -> Self {
        Self {
            consumer_id,
            partitions,
        }
    }

    pub fn consumer_id(&self) -> &ConsumerId {
        &self.consumer_id
    }

    pub fn partitions(&self) -> &[TopicPartition] {
        &self.partitions
    }
}

#[derive(Debug, Clone)]
pub struct CommittedOffset {
    partition: TopicPartition,
    offset: Offset,
}

impl CommittedOffset {
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

#[derive(Debug, Clone)]
pub struct ConsumerGroup {
    id: ConsumerGroupId,
    consumers: HashMap<ConsumerId, Consumer>,
    assignments: Vec<PartitionAssignment>,
    committed_offsets: HashMap<TopicPartition, Offset>,
}

impl ConsumerGroup {
    pub fn new(id: ConsumerGroupId) -> Self {
        Self {
            id,
            consumers: HashMap::new(),
            assignments: Vec::new(),
            committed_offsets: HashMap::new(),
        }
    }

    pub fn id(&self) -> &ConsumerGroupId {
        &self.id
    }

    pub fn consumers(&self) -> &HashMap<ConsumerId, Consumer> {
        &self.consumers
    }

    pub fn assignments(&self) -> &[PartitionAssignment] {
        &self.assignments
    }

    pub fn committed_offsets(&self) -> &HashMap<TopicPartition, Offset> {
        &self.committed_offsets
    }
}
