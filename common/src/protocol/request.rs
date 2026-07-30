use crate::models::identifiers::{ConsumerGroupId, Offset, PartitionId, TopicName};
use crate::models::record::{RecordKey, RecordPayload};
use crate::protocol::request_type::RequestType;

#[derive(Debug, Clone)]
pub struct Request {
    correlation_id: u32,
    payload: RequestPayload,
}

impl Request {
    pub fn new(correlation_id: u32, payload: RequestPayload) -> Self {
        Self {
            correlation_id,
            payload,
        }
    }

    pub fn correlation_id(&self) -> u32 {
        self.correlation_id
    }

    pub fn request_type(&self) -> RequestType {
        self.payload.request_type()
    }

    pub fn payload(&self) -> &RequestPayload {
        &self.payload
    }

    pub fn into_payload(self) -> RequestPayload {
        self.payload
    }
}

#[derive(Debug, Clone)]
pub enum RequestPayload {
    Produce(ProduceRequest),
    Fetch(FetchRequest),
    CreateTopic(CreateTopicRequest),
    CommitOffset(CommitOffsetRequest),
    ListTopics,
}

impl RequestPayload {
    pub fn request_type(&self) -> RequestType {
        match self {
            RequestPayload::Produce(_) => RequestType::Produce,
            RequestPayload::Fetch(_) => RequestType::Fetch,
            RequestPayload::CreateTopic(_) => RequestType::CreateTopic,
            RequestPayload::CommitOffset(_) => RequestType::CommitOffset,
            RequestPayload::ListTopics => RequestType::ListTopics,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProduceRequest {
    topic: TopicName,
    partition_id: PartitionId,
    key: Option<RecordKey>,
    payload: RecordPayload,
}

impl ProduceRequest {
    pub fn new(
        topic: TopicName,
        partition_id: PartitionId,
        key: Option<RecordKey>,
        payload: RecordPayload,
    ) -> Self {
        Self {
            topic,
            partition_id,
            key,
            payload,
        }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn key(&self) -> Option<&RecordKey> {
        self.key.as_ref()
    }

    pub fn payload(&self) -> &RecordPayload {
        &self.payload
    }
}

#[derive(Debug, Clone)]
pub struct FetchRequest {
    topic: TopicName,
    partition_id: PartitionId,
    offset: Offset,
    max_records: u32,
}

impl FetchRequest {
    pub fn new(
        topic: TopicName,
        partition_id: PartitionId,
        offset: Offset,
        max_records: u32,
    ) -> Self {
        Self {
            topic,
            partition_id,
            offset,
            max_records,
        }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }

    pub fn max_records(&self) -> u32 {
        self.max_records
    }
}

#[derive(Debug, Clone)]
pub struct CreateTopicRequest {
    topic: TopicName,
    partition_count: u32,
    segment_max_bytes: u64,
}

impl CreateTopicRequest {
    pub fn new(topic: TopicName, partition_count: u32, segment_max_bytes: u64) -> Self {
        Self {
            topic,
            partition_count,
            segment_max_bytes,
        }
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn partition_count(&self) -> u32 {
        self.partition_count
    }

    pub fn segment_max_bytes(&self) -> u64 {
        self.segment_max_bytes
    }
}

#[derive(Debug, Clone)]
pub struct CommitOffsetRequest {
    group_id: ConsumerGroupId,
    topic: TopicName,
    partition_id: PartitionId,
    offset: Offset,
}

impl CommitOffsetRequest {
    pub fn new(
        group_id: ConsumerGroupId,
        topic: TopicName,
        partition_id: PartitionId,
        offset: Offset,
    ) -> Self {
        Self {
            group_id,
            topic,
            partition_id,
            offset,
        }
    }

    pub fn group_id(&self) -> &ConsumerGroupId {
        &self.group_id
    }

    pub fn topic(&self) -> &TopicName {
        &self.topic
    }

    pub fn partition_id(&self) -> PartitionId {
        self.partition_id
    }

    pub fn offset(&self) -> Offset {
        self.offset
    }
}
