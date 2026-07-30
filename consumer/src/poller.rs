use common::error::consumer_error::ConsumerError;
use common::models::identifiers::{Offset, PartitionId, TopicName};
use common::models::record::Record;
use common::protocol::request::{FetchRequest, Request, RequestPayload};
use common::protocol::response::ResponsePayload;
use common::protocol::response_status::ResponseStatus;

use crate::consumer_connection::ConsumerConnection;

#[derive(Debug, Default)]
pub struct Poller {
    next_correlation_id: u32,
}

impl Poller {
    pub fn new() -> Self {
        Self {
            next_correlation_id: 1,
        }
    }

    pub async fn poll(
        &mut self,
        connection: &mut ConsumerConnection,
        topic: &TopicName,
        partition_id: PartitionId,
        offset: Offset,
        max_records: u32,
    ) -> Result<Vec<Record>, ConsumerError> {
        let request = Request::new(
            self.next_correlation_id,
            RequestPayload::Fetch(FetchRequest::new(
                topic.clone(),
                partition_id,
                offset,
                max_records,
            )),
        );
        self.next_correlation_id = self.next_correlation_id.saturating_add(1);

        let response = connection.send(&request).await?;
        if response.status() != ResponseStatus::Ok {
            return Err(Self::error_response(response.payload()));
        }

        match response.payload() {
            ResponsePayload::Fetch(fetch) => Ok(fetch.records().to_vec()),
            _ => Err(ConsumerError::new(
                "Broker returned unexpected response type",
            )),
        }
    }

    fn error_response(payload: &ResponsePayload) -> ConsumerError {
        match payload {
            ResponsePayload::Error(error) => ConsumerError::new(error.message().to_string()),
            _ => ConsumerError::new("Broker returned an error response"),
        }
    }
}
