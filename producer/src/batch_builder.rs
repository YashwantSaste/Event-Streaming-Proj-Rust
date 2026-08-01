use common::error::producer_error::ProducerError;
use common::models::identifiers::{PartitionId, TopicName};
use common::models::record::{RecordKey, RecordPayload};
use common::protocol::request::{CreateTopicRequest, ProduceRequest, Request, RequestPayload};

#[derive(Debug, Default)]
pub struct BatchBuilder {
    next_correlation_id: u32,
}

impl BatchBuilder {
    pub fn new() -> Self {
        Self {
            next_correlation_id: 1,
        }
    }

    pub fn build_produce_request(
        &mut self,
        topic: &str,
        partition_id: u32,
        key: Option<Vec<u8>>,
        payload: Vec<u8>,
    ) -> Result<Request, ProducerError> {
        let topic = TopicName::new(topic.to_string())
            .map_err(|error| ProducerError::new(error.to_string()))?;
        let request = Request::new(
            self.next_correlation_id,
            RequestPayload::Produce(ProduceRequest::new(
                topic,
                PartitionId::new(partition_id),
                key.map(RecordKey::new),
                RecordPayload::new(payload),
            )),
        );
        self.next_correlation_id = self.next_correlation_id.saturating_add(1);
        Ok(request)
    }

    pub fn build_create_topic_request(
        &mut self,
        topic: &str,
        partition_count: u32,
        segment_max_bytes: u64,
    ) -> Result<Request, ProducerError> {
        let topic = TopicName::new(topic.to_string())
            .map_err(|error| ProducerError::new(error.to_string()))?;
        let request = Request::new(
            self.next_correlation_id,
            RequestPayload::CreateTopic(CreateTopicRequest::new(
                topic,
                partition_count,
                segment_max_bytes,
            )),
        );
        self.next_correlation_id = self.next_correlation_id.saturating_add(1);
        Ok(request)
    }

    pub fn build_list_topics_request(&mut self) -> Request {
        let request = Request::new(self.next_correlation_id, RequestPayload::ListTopics);
        self.next_correlation_id = self.next_correlation_id.saturating_add(1);
        request
    }
}
