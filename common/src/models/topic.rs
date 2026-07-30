use crate::error::application_error::ApplicationError;
use crate::models::identifiers::TopicName;
use crate::models::partition::PartitionConfiguration;

#[derive(Debug, Clone)]
pub struct TopicConfiguration {
    partition_count: u32,
    partition_configuration: PartitionConfiguration,
}

impl TopicConfiguration {
    pub fn new(
        partition_count: u32,
        partition_configuration: PartitionConfiguration,
    ) -> Result<Self, ApplicationError> {
        if partition_count == 0 {
            return Err(ApplicationError::new(
                "topic partition_count must be greater than zero",
            ));
        }

        Ok(Self {
            partition_count,
            partition_configuration,
        })
    }

    pub fn partition_count(&self) -> u32 {
        self.partition_count
    }

    pub fn partition_configuration(&self) -> &PartitionConfiguration {
        &self.partition_configuration
    }
}

#[derive(Debug, Clone)]
pub struct Topic {
    name: TopicName,
    configuration: TopicConfiguration,
}

impl Topic {
    pub fn new(name: TopicName, configuration: TopicConfiguration) -> Self {
        Self {
            name,
            configuration,
        }
    }

    pub fn name(&self) -> &TopicName {
        &self.name
    }

    pub fn configuration(&self) -> &TopicConfiguration {
        &self.configuration
    }
}
