use common::error::consumer_error::ConsumerError;
use common::models::identifiers::{Offset, PartitionId};
use common::models::record::Record;

use crate::consumer_configuration::ConsumerConfiguration;
use crate::consumer_connection::ConsumerConnection;
use crate::offset_committer::OffsetCommitter;
use crate::poller::Poller;
use crate::subscription_manager::SubscriptionManager;

pub struct ConsumerClient {
    configuration: ConsumerConfiguration,
    subscriptions: SubscriptionManager,
    poller: Poller,
    offset_committer: OffsetCommitter,
}

impl ConsumerClient {
    pub fn new(configuration: ConsumerConfiguration) -> Self {
        Self {
            offset_committer: OffsetCommitter::new(configuration.group_id().clone()),
            configuration,
            subscriptions: SubscriptionManager::new(),
            poller: Poller::new(),
        }
    }

    pub fn subscribe(&mut self, topic: &str) -> Result<(), ConsumerError> {
        self.subscriptions.subscribe(topic)
    }

    pub async fn poll(
        &mut self,
        partition_id: u32,
        offset: u64,
        max_records: u32,
    ) -> Result<Vec<Record>, ConsumerError> {
        let topic = self
            .subscriptions
            .topics()
            .first()
            .ok_or_else(|| ConsumerError::new("Consumer has no subscriptions"))?
            .clone();
        let mut connection = ConsumerConnection::connect(&self.configuration).await?;
        self.poller
            .poll(
                &mut connection,
                &topic,
                PartitionId::new(partition_id),
                Offset::new(offset),
                max_records,
            )
            .await
    }

    pub async fn commit(
        &mut self,
        topic: &str,
        partition_id: u32,
        offset: u64,
    ) -> Result<Offset, ConsumerError> {
        let topic = common::models::identifiers::TopicName::new(topic.to_string())
            .map_err(|error| ConsumerError::new(error.to_string()))?;
        let mut connection = ConsumerConnection::connect(&self.configuration).await?;
        self.offset_committer
            .commit(
                &mut connection,
                &topic,
                PartitionId::new(partition_id),
                Offset::new(offset),
            )
            .await
    }
}
