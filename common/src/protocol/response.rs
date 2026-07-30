use crate::models::identifiers::{Offset, TopicName};
use crate::models::partition::TopicPartition;
use crate::models::record::Record;
use crate::protocol::request_type::RequestType;
use crate::protocol::response_status::ResponseStatus;

#[derive(Debug, Clone)]
pub struct Response {
    correlation_id: u32,
    request_type: RequestType,
    status: ResponseStatus,
    payload: ResponsePayload,
}

impl Response {
    pub fn new(
        correlation_id: u32,
        request_type: RequestType,
        status: ResponseStatus,
        payload: ResponsePayload,
    ) -> Self {
        Self {
            correlation_id,
            request_type,
            status,
            payload,
        }
    }

    pub fn ok(correlation_id: u32, request_type: RequestType, payload: ResponsePayload) -> Self {
        Self::new(correlation_id, request_type, ResponseStatus::Ok, payload)
    }

    pub fn error(correlation_id: u32, request_type: RequestType, message: String) -> Self {
        Self::new(
            correlation_id,
            request_type,
            ResponseStatus::Error,
            ResponsePayload::Error(ErrorResponse::new(message)),
        )
    }

    pub fn correlation_id(&self) -> u32 {
        self.correlation_id
    }

    pub fn request_type(&self) -> RequestType {
        self.request_type
    }

    pub fn status(&self) -> ResponseStatus {
        self.status
    }

    pub fn payload(&self) -> &ResponsePayload {
        &self.payload
    }
}

#[derive(Debug, Clone)]
pub enum ResponsePayload {
    Produce(ProduceResponse),
    Fetch(FetchResponse),
    CreateTopic(CreateTopicResponse),
    CommitOffset(CommitOffsetResponse),
    ListTopics(ListTopicsResponse),
    Error(ErrorResponse),
}

#[derive(Debug, Clone)]
pub struct ProduceResponse {
    partition: TopicPartition,
    offset: Offset,
}

impl ProduceResponse {
    pub fn new(partition: TopicPartition, offset: Offset) -> Self {
        Self { partition, offset }
    }

    pub fn partition(&self) -> &TopicPartition {
        &self.partition
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }
}

#[derive(Debug, Clone)]
pub struct FetchResponse {
    records: Vec<Record>,
}

impl FetchResponse {
    pub fn new(records: Vec<Record>) -> Self {
        Self { records }
    }

    pub fn records(&self) -> &[Record] {
        &self.records
    }
}

#[derive(Debug, Clone)]
pub struct CreateTopicResponse {
    topic: TopicName,
}

impl CreateTopicResponse {
    pub fn new(topic: TopicName) -> Self {
        Self { topic }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }
}

#[derive(Debug, Clone)]
pub struct CommitOffsetResponse {
    offset: Offset,
}

impl CommitOffsetResponse {
    pub fn new(offset: Offset) -> Self {
        Self { offset }
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }
}

#[derive(Debug, Clone)]
pub struct ListTopicsResponse {
    topics: Vec<TopicName>,
}

impl ListTopicsResponse {
    pub fn new(topics: Vec<TopicName>) -> Self {
        Self { topics }
    }

    pub fn topics(&self) -> &[TopicName] {
        &self.topics
    }
}

#[derive(Debug, Clone)]
pub struct ErrorResponse {
    message: String,
}

impl ErrorResponse {
    pub fn new(message: String) -> Self {
        Self { message }
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}
