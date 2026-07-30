use std::collections::HashMap;

use common::error::broker_error::BrokerError;
use common::models::identifiers::{Offset, PartitionId, TopicName};
use common::models::partition::{Partition, TopicPartition};
use common::models::record::{Record, RecordKey, RecordPayload};
use common::models::topic::Topic;

use crate::storage::storage_engine::{StorageEngine, StoredRecordMetadata};

pub struct PartitionManager<S>
where
    S: StorageEngine,
{
    storage_engine: S,
    partitions: HashMap<TopicPartition, Partition>,
}

impl<S> PartitionManager<S>
where
    S: StorageEngine,
{
    pub fn new(storage_engine: S) -> Self {
        Self {
            storage_engine,
            partitions: HashMap::new(),
        }
    }

    pub fn create_partitions_for_topic(
        &mut self,
        topic: &Topic,
    ) -> Result<Vec<Partition>, BrokerError> {
        let partitions = (0..topic.configuration().partition_count())
            .map(|partition_index| {
                Partition::new(
                    topic.name().clone(),
                    PartitionId::new(partition_index),
                    topic.configuration().partition_configuration().clone(),
                )
            })
            .collect::<Vec<_>>();

        for partition in &partitions {
            let topic_partition = TopicPartition::new(partition.topic().clone(), partition.id());
            self.storage_engine
                .flush(&topic_partition)
                .map_err(|error| BrokerError::new(error.to_string()))?;
            self.partitions.insert(topic_partition, partition.clone());
        }

        Ok(partitions)
    }

    pub fn delete_partitions_for_topic(&mut self, topic_name: &TopicName) {
        self.partitions
            .retain(|topic_partition, _| topic_partition.topic() != topic_name);
    }

    pub fn append(
        &mut self,
        topic_name: &TopicName,
        partition_id: PartitionId,
        key: Option<RecordKey>,
        payload: RecordPayload,
    ) -> Result<StoredRecordMetadata, BrokerError> {
        let topic_partition = TopicPartition::new(topic_name.clone(), partition_id);
        self.ensure_partition_exists(&topic_partition)?;

        self.storage_engine
            .append(&topic_partition, key, payload)
            .map_err(|error| BrokerError::new(error.to_string()))
    }

    pub fn read(
        &mut self,
        topic_name: &TopicName,
        partition_id: PartitionId,
        offset: Offset,
        max_records: usize,
    ) -> Result<Vec<Record>, BrokerError> {
        let topic_partition = TopicPartition::new(topic_name.clone(), partition_id);
        self.ensure_partition_exists(&topic_partition)?;

        self.storage_engine
            .read(&topic_partition, offset, max_records)
            .map_err(|error| BrokerError::new(error.to_string()))
    }

    pub fn get_partition(
        &self,
        topic_name: &TopicName,
        partition_id: PartitionId,
    ) -> Option<&Partition> {
        self.partitions
            .get(&TopicPartition::new(topic_name.clone(), partition_id))
    }

    pub fn list_partitions(&self, topic_name: &TopicName) -> Vec<Partition> {
        let mut partitions = self
            .partitions
            .iter()
            .filter(|(topic_partition, _)| topic_partition.topic() == topic_name)
            .map(|(_, partition)| partition.clone())
            .collect::<Vec<_>>();

        partitions.sort_by_key(Partition::id);
        partitions
    }

    pub fn recover(&mut self, topics: &[Topic]) -> Result<(), BrokerError> {
        self.storage_engine
            .recover()
            .map_err(|error| BrokerError::new(error.to_string()))?;
        self.partitions.clear();

        for topic in topics {
            self.create_partitions_for_topic(topic)?;
        }

        Ok(())
    }

    pub fn into_storage_engine(self) -> S {
        self.storage_engine
    }

    fn ensure_partition_exists(&self, topic_partition: &TopicPartition) -> Result<(), BrokerError> {
        if self.partitions.contains_key(topic_partition) {
            Ok(())
        } else {
            Err(BrokerError::new(format!(
                "Partition '{}:{}' does not exist",
                topic_partition.topic(),
                topic_partition.partition_id()
            )))
        }
    }
}
