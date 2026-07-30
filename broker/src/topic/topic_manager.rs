use std::collections::HashMap;

use common::error::broker_error::BrokerError;
use common::filesystem::file_system::FileSystem;
use common::models::identifiers::TopicName;
use common::models::partition::PartitionConfiguration;
use common::models::topic::{Topic, TopicConfiguration};

use crate::topic::topic_metadata_store::TopicMetadataStore;

pub struct TopicManager<F>
where
    F: FileSystem,
{
    metadata_store: TopicMetadataStore<F>,
    topics: HashMap<TopicName, Topic>,
}

impl<F> TopicManager<F>
where
    F: FileSystem,
{
    pub fn new(metadata_store: TopicMetadataStore<F>) -> Self {
        Self {
            metadata_store,
            topics: HashMap::new(),
        }
    }

    pub fn create_topic(
        &mut self,
        name: TopicName,
        partition_count: u32,
        partition_configuration: PartitionConfiguration,
    ) -> Result<Topic, BrokerError> {
        if self.topics.contains_key(&name) || self.metadata_store.exists(&name) {
            return Err(BrokerError::new(format!("Topic '{name}' already exists")));
        }

        let configuration = TopicConfiguration::new(partition_count, partition_configuration)
            .map_err(|error| BrokerError::new(error.to_string()))?;
        let topic = Topic::new(name.clone(), configuration);

        self.metadata_store.save(&topic)?;
        self.topics.insert(name, topic.clone());

        Ok(topic)
    }

    pub fn delete_topic(&mut self, name: &TopicName) -> Result<(), BrokerError> {
        if !self.topics.contains_key(name) && !self.metadata_store.exists(name) {
            return Err(BrokerError::new(format!("Topic '{name}' does not exist")));
        }

        self.metadata_store.delete(name)?;
        self.topics.remove(name);
        Ok(())
    }

    pub fn get_topic(&self, name: &TopicName) -> Option<&Topic> {
        self.topics.get(name)
    }

    pub fn list_topics(&self) -> Vec<Topic> {
        let mut topics = self.topics.values().cloned().collect::<Vec<_>>();
        topics.sort_by(|left, right| left.name().cmp(right.name()));
        topics
    }

    pub fn recover(&mut self) -> Result<(), BrokerError> {
        self.topics = self
            .metadata_store
            .load_all()?
            .into_iter()
            .map(|topic| (topic.name().clone(), topic))
            .collect();

        Ok(())
    }

    pub fn contains_topic(&self, name: &TopicName) -> bool {
        self.topics.contains_key(name)
    }
}
