use common::error::producer_error::ProducerError;
use common::models::identifiers::TopicName;
use common::protocol::request::Request;
use common::protocol::response::{CreateTopicResponse, ProduceResponse, Response, ResponsePayload};
use common::protocol::response_status::ResponseStatus;
use tokio::time::sleep;

use crate::batch_builder::BatchBuilder;
use crate::producer_configuration::ProducerConfiguration;
use crate::producer_connection::ProducerConnection;
use crate::retry_policy::RetryPolicy;

pub struct ProducerClient {
    configuration: ProducerConfiguration,
    retry_policy: RetryPolicy,
    batch_builder: BatchBuilder,
}

impl ProducerClient {
    pub fn new(configuration: ProducerConfiguration, retry_policy: RetryPolicy) -> Self {
        Self {
            configuration,
            retry_policy,
            batch_builder: BatchBuilder::new(),
        }
    }

    pub async fn send(
        &mut self,
        topic: &str,
        partition_id: u32,
        key: Option<Vec<u8>>,
        payload: Vec<u8>,
    ) -> Result<ProduceResponse, ProducerError> {
        let request =
            self.batch_builder
                .build_produce_request(topic, partition_id, key, payload)?;
        let response = self.send_with_retry(&request).await?;
        Self::extract_produce_response(response.payload())
    }

    pub async fn create_topic(
        &mut self,
        topic: &str,
        partition_count: u32,
        segment_max_bytes: u64,
    ) -> Result<CreateTopicResponse, ProducerError> {
        let request = self.batch_builder.build_create_topic_request(
            topic,
            partition_count,
            segment_max_bytes,
        )?;
        let response = self.send_with_retry(&request).await?;
        Self::extract_create_topic_response(response.payload())
    }

    pub async fn list_topics(&self) -> Result<Vec<TopicName>, ProducerError> {
        let mut builder = BatchBuilder::new();
        let request = builder.build_list_topics_request();
        let response = self.send_with_retry(&request).await?;
        match response.payload() {
            ResponsePayload::ListTopics(payload) => Ok(payload.topics().to_vec()),
            ResponsePayload::Error(error) => Err(ProducerError::new(error.message().to_string())),
            _ => Err(ProducerError::new(
                "Broker returned unexpected response type",
            )),
        }
    }

    async fn send_with_retry(&self, request: &Request) -> Result<Response, ProducerError> {
        let mut last_error: Option<ProducerError> = None;
        for attempt in 1..=self.retry_policy.max_attempts() {
            match ProducerConnection::connect(&self.configuration).await {
                Ok(mut connection) => match connection.send(request).await {
                    Ok(response) if response.status() == ResponseStatus::Ok => {
                        return Ok(response);
                    }
                    Ok(response) => {
                        last_error = Some(ProducerError::new(format!(
                            "Broker returned error response for correlation {}",
                            response.correlation_id()
                        )));
                    }
                    Err(error) => last_error = Some(error),
                },
                Err(error) => last_error = Some(error),
            }

            if attempt < self.retry_policy.max_attempts() {
                sleep(self.retry_policy.delay()).await;
            }
        }

        Err(last_error.unwrap_or_else(|| ProducerError::new("Producer request failed")))
    }

    fn extract_produce_response(
        payload: &ResponsePayload,
    ) -> Result<ProduceResponse, ProducerError> {
        match payload {
            ResponsePayload::Produce(response) => Ok(response.clone()),
            ResponsePayload::Error(error) => Err(ProducerError::new(error.message().to_string())),
            _ => Err(ProducerError::new(
                "Broker returned unexpected response type",
            )),
        }
    }

    fn extract_create_topic_response(
        payload: &ResponsePayload,
    ) -> Result<CreateTopicResponse, ProducerError> {
        match payload {
            ResponsePayload::CreateTopic(response) => Ok(response.clone()),
            ResponsePayload::Error(error) => Err(ProducerError::new(error.message().to_string())),
            _ => Err(ProducerError::new(
                "Broker returned unexpected response type",
            )),
        }
    }
}
