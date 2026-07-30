use common::error::producer_error::ProducerError;
use common::protocol::response::{ProduceResponse, ResponsePayload};
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
        let mut last_error: Option<ProducerError> = None;

        for attempt in 1..=self.retry_policy.max_attempts() {
            match ProducerConnection::connect(&self.configuration).await {
                Ok(mut connection) => match connection.send(&request).await {
                    Ok(response) if response.status() == ResponseStatus::Ok => {
                        return Self::extract_produce_response(response.payload());
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

        Err(last_error.unwrap_or_else(|| ProducerError::new("Produce request failed")))
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
}
