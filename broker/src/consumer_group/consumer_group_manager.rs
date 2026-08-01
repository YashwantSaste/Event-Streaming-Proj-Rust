use std::collections::HashMap;

use common::error::broker_error::BrokerError;
use common::filesystem::file_system::FileSystem;
use common::models::consumer::Consumer;
use common::models::consumer_group::PartitionAssignment;
use common::models::identifiers::{ConsumerGroupId, ConsumerId, Offset, TopicName};
use common::models::partition::TopicPartition;

use crate::consumer_group::offset_store::OffsetStore;

#[derive(Debug, Clone)]
struct ConsumerGroupState {
    consumers: HashMap<ConsumerId, Consumer>,
    assignments: Vec<PartitionAssignment>,
    committed_offsets: HashMap<TopicPartition, Offset>,
}

impl ConsumerGroupState {
    fn new(committed_offsets: HashMap<TopicPartition, Offset>) -> Self {
        Self {
            consumers: HashMap::new(),
            assignments: Vec::new(),
            committed_offsets,
        }
    }
}

pub struct ConsumerGroupManager<F>
where
    F: FileSystem,
{
    offset_store: OffsetStore<F>,
    groups: HashMap<ConsumerGroupId, ConsumerGroupState>,
}

impl<F> ConsumerGroupManager<F>
where
    F: FileSystem,
{
    pub fn new(offset_store: OffsetStore<F>) -> Self {
        Self {
            offset_store,
            groups: HashMap::new(),
        }
    }

    pub fn register_consumer(
        &mut self,
        group_id: ConsumerGroupId,
        consumer_id: ConsumerId,
        subscriptions: Vec<TopicName>,
    ) -> Result<(), BrokerError> {
        let group = self.ensure_group(&group_id)?;
        group.consumers.insert(
            consumer_id.clone(),
            Consumer::new(consumer_id, group_id, subscriptions),
        );
        Ok(())
    }

    pub fn remove_consumer(
        &mut self,
        group_id: &ConsumerGroupId,
        consumer_id: &ConsumerId,
    ) -> Result<(), BrokerError> {
        let group = self.ensure_group(group_id)?;
        group.consumers.remove(consumer_id);
        group
            .assignments
            .retain(|assignment| assignment.consumer_id() != consumer_id);
        Ok(())
    }

    pub fn assign_partitions(
        &mut self,
        group_id: &ConsumerGroupId,
        partitions: Vec<TopicPartition>,
    ) -> Result<Vec<PartitionAssignment>, BrokerError> {
        let group = self.ensure_group(group_id)?;
        let consumers = group.consumers.keys().cloned().collect::<Vec<_>>();
        if consumers.is_empty() {
            group.assignments.clear();
            return Ok(Vec::new());
        }

        let mut assigned: HashMap<ConsumerId, Vec<TopicPartition>> = consumers
            .iter()
            .cloned()
            .map(|consumer_id| (consumer_id, Vec::new()))
            .collect();

        for (index, partition) in partitions.into_iter().enumerate() {
            let consumer = &consumers[index % consumers.len()];
            if let Some(partitions) = assigned.get_mut(consumer) {
                partitions.push(partition);
            }
        }

        let mut assignments = assigned
            .into_iter()
            .map(|(consumer_id, partitions)| PartitionAssignment::new(consumer_id, partitions))
            .collect::<Vec<_>>();
        assignments.sort_by(|left, right| left.consumer_id().cmp(right.consumer_id()));
        group.assignments = assignments.clone();
        Ok(assignments)
    }

    pub fn commit_offset(
        &mut self,
        group_id: &ConsumerGroupId,
        partition: TopicPartition,
        offset: Offset,
    ) -> Result<Offset, BrokerError> {
        let offsets = {
            let group = self.ensure_group(group_id)?;
            group.committed_offsets.insert(partition, offset);
            group.committed_offsets.clone()
        };
        self.offset_store.save(group_id, &offsets)?;
        Ok(offset)
    }

    pub fn committed_offset(
        &mut self,
        group_id: &ConsumerGroupId,
        partition: &TopicPartition,
    ) -> Result<Option<Offset>, BrokerError> {
        Ok(self
            .ensure_group(group_id)?
            .committed_offsets
            .get(partition)
            .copied())
    }

    fn ensure_group(
        &mut self,
        group_id: &ConsumerGroupId,
    ) -> Result<&mut ConsumerGroupState, BrokerError> {
        if !self.groups.contains_key(group_id) {
            let offsets = self.offset_store.load(group_id)?;
            self.groups
                .insert(group_id.clone(), ConsumerGroupState::new(offsets));
        }

        self.groups
            .get_mut(group_id)
            .ok_or_else(|| BrokerError::new("Consumer group state was not initialized"))
    }
}
