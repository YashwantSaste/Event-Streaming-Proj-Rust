use crate::models::identifiers::{ConsumerGroupId, ConsumerId, TopicName};

#[derive(Debug, Clone)]
pub struct Consumer {
    id: ConsumerId,
    group_id: ConsumerGroupId,
    subscriptions: Vec<TopicName>,
}

impl Consumer {
    pub fn new(id: ConsumerId, group_id: ConsumerGroupId, subscriptions: Vec<TopicName>) -> Self {
        Self {
            id,
            group_id,
            subscriptions,
        }
    }

    pub fn id(&self) -> &ConsumerId {
        &self.id
    }

    pub fn group_id(&self) -> &ConsumerGroupId {
        &self.group_id
    }

    pub fn subscriptions(&self) -> &[TopicName] {
        &self.subscriptions
    }
}
