use common::error::consumer_error::ConsumerError;
use common::models::identifiers::TopicName;

#[derive(Debug, Default, Clone)]
pub struct SubscriptionManager {
    topics: Vec<TopicName>,
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn subscribe(&mut self, topic: &str) -> Result<(), ConsumerError> {
        let topic = TopicName::new(topic.to_string())
            .map_err(|error| ConsumerError::new(error.to_string()))?;
        if !self.topics.contains(&topic) {
            self.topics.push(topic);
        }
        Ok(())
    }

    pub fn topics(&self) -> &[TopicName] {
        &self.topics
    }
}
