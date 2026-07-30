use common::error::consumer_error::ConsumerError;
use common::models::identifiers::{ConsumerGroupId, Offset, PartitionId, TopicName};
use common::protocol::request::{CommitOffsetRequest, Request, RequestPayload};
use common::protocol::response::ResponsePayload;
use common::protocol::response_status::ResponseStatus;

use crate::consumer_connection::ConsumerConnection;

#[derive(Debug)]
pub struct OffsetCommitter {
    group_id: ConsumerGroupId,
    next_correlation_id: u32,
}

impl OffsetCommitter {
    pub fn new(group_id: ConsumerGroupId) -> Self {
        Self {
            group_id,
            next_correlation_id: 1,
        }
    }

    pub async fn commit(
        &mut self,
        connection: &mut ConsumerConnection,
        topic: &TopicName,
        partition_id: PartitionId,
        offset: Offset,
    ) -> Result<Offset, ConsumerError> {
        let request = Request::new(
            self.next_correlation_id,
            RequestPayload::CommitOffset(CommitOffsetRequest::new(
                self.group_id.clone(),
                topic.clone(),
                partition_id,
                offset,
            )),
        );
        self.next_correlation_id = self.next_correlation_id.saturating_add(1);
        let response = connection.send(&request).await?;

        if response.status() != ResponseStatus::Ok {
            return Err(match response.payload() {
                ResponsePayload::Error(error) => ConsumerError::new(error.message().to_string()),
                _ => ConsumerError::new("Broker returned an error response"),
            });
        }

        match response.payload() {
            ResponsePayload::CommitOffset(payload) => Ok(payload.offset()),
            _ => Err(ConsumerError::new(
                "Broker returned unexpected response type",
            )),
        }
    }
}
