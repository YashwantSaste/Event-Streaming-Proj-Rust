use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::error::protocol_error::ProtocolError;
use crate::models::identifiers::{ConsumerGroupId, Offset, PartitionId, TopicName};
use crate::models::partition::TopicPartition;
use crate::models::record::{Record, RecordKey, RecordPayload};
use crate::protocol::decoder::Decoder;
use crate::protocol::encoder::Encoder;
use crate::protocol::header::{MESSAGE_HEADER_LENGTH, MessageHeader};
use crate::protocol::request::{
    CommitOffsetRequest, CreateTopicRequest, FetchRequest, ProduceRequest, Request, RequestPayload,
};
use crate::protocol::request_type::RequestType;
use crate::protocol::response::{
    CommitOffsetResponse, CreateTopicResponse, ErrorResponse, FetchResponse, ListTopicsResponse,
    ProduceResponse, Response, ResponsePayload,
};
use crate::protocol::response_status::ResponseStatus;

pub struct BinaryProtocolCodec;

impl BinaryProtocolCodec {
    pub fn new() -> Self {
        Self
    }

    fn encode_header(header: MessageHeader, buffer: &mut Vec<u8>) {
        Self::write_u32(buffer, header.payload_length());
        Self::write_u16(buffer, header.request_type().code());
        Self::write_u32(buffer, header.correlation_id());
    }

    fn decode_header(bytes: &[u8]) -> Result<(MessageHeader, usize), ProtocolError> {
        if bytes.len() < MESSAGE_HEADER_LENGTH {
            return Err(ProtocolError::new("Protocol frame is shorter than header"));
        }

        let mut position = 0;
        let payload_length = Self::read_u32(bytes, &mut position)?;
        let request_type = RequestType::from_code(Self::read_u16(bytes, &mut position)?)?;
        let correlation_id = Self::read_u32(bytes, &mut position)?;
        let expected_length = MESSAGE_HEADER_LENGTH
            .checked_add(payload_length as usize)
            .ok_or_else(|| ProtocolError::new("Protocol frame length overflow"))?;

        if bytes.len() != expected_length {
            return Err(ProtocolError::new(format!(
                "Protocol frame length mismatch: expected {expected_length}, got {}",
                bytes.len()
            )));
        }

        Ok((
            MessageHeader::new(payload_length, request_type, correlation_id),
            position,
        ))
    }

    fn encode_request_payload(payload: &RequestPayload) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Vec::new();

        match payload {
            RequestPayload::Produce(request) => {
                Self::write_string(&mut buffer, request.topic().as_str())?;
                Self::write_u32(&mut buffer, request.partition_id().value());
                Self::write_optional_bytes(&mut buffer, request.key().map(RecordKey::bytes))?;
                Self::write_bytes(&mut buffer, request.payload().bytes())?;
            }
            RequestPayload::Fetch(request) => {
                Self::write_string(&mut buffer, request.topic().as_str())?;
                Self::write_u32(&mut buffer, request.partition_id().value());
                Self::write_u64(&mut buffer, request.offset().value());
                Self::write_u32(&mut buffer, request.max_records());
            }
            RequestPayload::CreateTopic(request) => {
                Self::write_string(&mut buffer, request.topic().as_str())?;
                Self::write_u32(&mut buffer, request.partition_count());
                Self::write_u64(&mut buffer, request.segment_max_bytes());
            }
            RequestPayload::CommitOffset(request) => {
                Self::write_string(&mut buffer, request.group_id().as_str())?;
                Self::write_string(&mut buffer, request.topic().as_str())?;
                Self::write_u32(&mut buffer, request.partition_id().value());
                Self::write_u64(&mut buffer, request.offset().value());
            }
            RequestPayload::ListTopics => {}
        }

        Ok(buffer)
    }

    fn decode_request_payload(
        request_type: RequestType,
        bytes: &[u8],
        position: &mut usize,
    ) -> Result<RequestPayload, ProtocolError> {
        match request_type {
            RequestType::Produce => {
                let topic = TopicName::new(Self::read_string(bytes, position)?)
                    .map_err(|error| ProtocolError::new(error.to_string()))?;
                let partition_id = PartitionId::new(Self::read_u32(bytes, position)?);
                let key = Self::read_optional_bytes(bytes, position)?.map(RecordKey::new);
                let payload = RecordPayload::new(Self::read_bytes(bytes, position)?);
                Ok(RequestPayload::Produce(ProduceRequest::new(
                    topic,
                    partition_id,
                    key,
                    payload,
                )))
            }
            RequestType::Fetch => {
                let topic = TopicName::new(Self::read_string(bytes, position)?)
                    .map_err(|error| ProtocolError::new(error.to_string()))?;
                let partition_id = PartitionId::new(Self::read_u32(bytes, position)?);
                let offset = Offset::new(Self::read_u64(bytes, position)?);
                let max_records = Self::read_u32(bytes, position)?;
                Ok(RequestPayload::Fetch(FetchRequest::new(
                    topic,
                    partition_id,
                    offset,
                    max_records,
                )))
            }
            RequestType::CreateTopic => {
                let topic = TopicName::new(Self::read_string(bytes, position)?)
                    .map_err(|error| ProtocolError::new(error.to_string()))?;
                let partition_count = Self::read_u32(bytes, position)?;
                let segment_max_bytes = Self::read_u64(bytes, position)?;
                Ok(RequestPayload::CreateTopic(CreateTopicRequest::new(
                    topic,
                    partition_count,
                    segment_max_bytes,
                )))
            }
            RequestType::CommitOffset => {
                let group_id = ConsumerGroupId::new(Self::read_string(bytes, position)?)
                    .map_err(|error| ProtocolError::new(error.to_string()))?;
                let topic = TopicName::new(Self::read_string(bytes, position)?)
                    .map_err(|error| ProtocolError::new(error.to_string()))?;
                let partition_id = PartitionId::new(Self::read_u32(bytes, position)?);
                let offset = Offset::new(Self::read_u64(bytes, position)?);
                Ok(RequestPayload::CommitOffset(CommitOffsetRequest::new(
                    group_id,
                    topic,
                    partition_id,
                    offset,
                )))
            }
            RequestType::ListTopics => Ok(RequestPayload::ListTopics),
        }
    }

    fn encode_response_payload(response: &Response) -> Result<Vec<u8>, ProtocolError> {
        let mut buffer = Vec::new();
        Self::write_u16(&mut buffer, response.status().code());

        match response.payload() {
            ResponsePayload::Produce(payload) => {
                Self::write_topic_partition(&mut buffer, payload.partition())?;
                Self::write_u64(&mut buffer, payload.offset().value());
            }
            ResponsePayload::Fetch(payload) => {
                Self::write_u32(
                    &mut buffer,
                    Self::usize_to_u32(payload.records().len(), "record count")?,
                );
                for record in payload.records() {
                    Self::write_record(&mut buffer, record)?;
                }
            }
            ResponsePayload::CreateTopic(payload) => {
                Self::write_string(&mut buffer, payload.topic().as_str())?;
            }
            ResponsePayload::CommitOffset(payload) => {
                Self::write_u64(&mut buffer, payload.offset().value());
            }
            ResponsePayload::ListTopics(payload) => {
                Self::write_u32(
                    &mut buffer,
                    Self::usize_to_u32(payload.topics().len(), "topic count")?,
                );
                for topic in payload.topics() {
                    Self::write_string(&mut buffer, topic.as_str())?;
                }
            }
            ResponsePayload::Error(payload) => {
                Self::write_string(&mut buffer, payload.message())?;
            }
        }

        Ok(buffer)
    }

    fn decode_response_payload(
        request_type: RequestType,
        status: ResponseStatus,
        bytes: &[u8],
        position: &mut usize,
    ) -> Result<ResponsePayload, ProtocolError> {
        if status == ResponseStatus::Error {
            return Ok(ResponsePayload::Error(ErrorResponse::new(
                Self::read_string(bytes, position)?,
            )));
        }

        match request_type {
            RequestType::Produce => {
                let partition = Self::read_topic_partition(bytes, position)?;
                let offset = Offset::new(Self::read_u64(bytes, position)?);
                Ok(ResponsePayload::Produce(ProduceResponse::new(
                    partition, offset,
                )))
            }
            RequestType::Fetch => {
                let count = Self::read_u32(bytes, position)? as usize;
                let mut records = Vec::with_capacity(count);
                for _ in 0..count {
                    records.push(Self::read_record(bytes, position)?);
                }
                Ok(ResponsePayload::Fetch(FetchResponse::new(records)))
            }
            RequestType::CreateTopic => {
                let topic = TopicName::new(Self::read_string(bytes, position)?)
                    .map_err(|error| ProtocolError::new(error.to_string()))?;
                Ok(ResponsePayload::CreateTopic(CreateTopicResponse::new(
                    topic,
                )))
            }
            RequestType::CommitOffset => {
                let offset = Offset::new(Self::read_u64(bytes, position)?);
                Ok(ResponsePayload::CommitOffset(CommitOffsetResponse::new(
                    offset,
                )))
            }
            RequestType::ListTopics => {
                let count = Self::read_u32(bytes, position)? as usize;
                let mut topics = Vec::with_capacity(count);
                for _ in 0..count {
                    topics.push(
                        TopicName::new(Self::read_string(bytes, position)?)
                            .map_err(|error| ProtocolError::new(error.to_string()))?,
                    );
                }
                Ok(ResponsePayload::ListTopics(ListTopicsResponse::new(topics)))
            }
        }
    }

    fn write_record(buffer: &mut Vec<u8>, record: &Record) -> Result<(), ProtocolError> {
        Self::write_string(buffer, record.topic().as_str())?;
        Self::write_u32(buffer, record.partition_id().value());
        Self::write_u64(buffer, record.offset().value());
        Self::write_u128(buffer, Self::timestamp_millis(record.timestamp())?);
        Self::write_optional_bytes(buffer, record.key().map(RecordKey::bytes))?;
        Self::write_bytes(buffer, record.payload().bytes())?;
        Ok(())
    }

    fn read_record(bytes: &[u8], position: &mut usize) -> Result<Record, ProtocolError> {
        let topic = TopicName::new(Self::read_string(bytes, position)?)
            .map_err(|error| ProtocolError::new(error.to_string()))?;
        let partition_id = PartitionId::new(Self::read_u32(bytes, position)?);
        let offset = Offset::new(Self::read_u64(bytes, position)?);
        let timestamp = Self::system_time_from_millis(Self::read_u128(bytes, position)?)?;
        let key = Self::read_optional_bytes(bytes, position)?.map(RecordKey::new);
        let payload = RecordPayload::new(Self::read_bytes(bytes, position)?);
        Ok(Record::new(
            topic,
            partition_id,
            offset,
            key,
            payload,
            timestamp,
        ))
    }

    fn write_topic_partition(
        buffer: &mut Vec<u8>,
        partition: &TopicPartition,
    ) -> Result<(), ProtocolError> {
        Self::write_string(buffer, partition.topic().as_str())?;
        Self::write_u32(buffer, partition.partition_id().value());
        Ok(())
    }

    fn read_topic_partition(
        bytes: &[u8],
        position: &mut usize,
    ) -> Result<TopicPartition, ProtocolError> {
        let topic = TopicName::new(Self::read_string(bytes, position)?)
            .map_err(|error| ProtocolError::new(error.to_string()))?;
        let partition_id = PartitionId::new(Self::read_u32(bytes, position)?);
        Ok(TopicPartition::new(topic, partition_id))
    }

    fn write_optional_bytes(
        buffer: &mut Vec<u8>,
        bytes: Option<&[u8]>,
    ) -> Result<(), ProtocolError> {
        match bytes {
            Some(bytes) => {
                buffer.push(1);
                Self::write_bytes(buffer, bytes)?;
            }
            None => {
                buffer.push(0);
                Self::write_u32(buffer, 0);
            }
        }
        Ok(())
    }

    fn read_optional_bytes(
        bytes: &[u8],
        position: &mut usize,
    ) -> Result<Option<Vec<u8>>, ProtocolError> {
        let present = Self::read_u8(bytes, position)?;
        let value = Self::read_bytes(bytes, position)?;
        if present == 1 {
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn write_string(buffer: &mut Vec<u8>, value: &str) -> Result<(), ProtocolError> {
        Self::write_bytes(buffer, value.as_bytes())
    }

    fn read_string(bytes: &[u8], position: &mut usize) -> Result<String, ProtocolError> {
        String::from_utf8(Self::read_bytes(bytes, position)?)
            .map_err(|error| ProtocolError::new(format!("Protocol string is not UTF-8: {error}")))
    }

    fn write_bytes(buffer: &mut Vec<u8>, bytes: &[u8]) -> Result<(), ProtocolError> {
        Self::write_u32(buffer, Self::usize_to_u32(bytes.len(), "byte field")?);
        buffer.extend_from_slice(bytes);
        Ok(())
    }

    fn read_bytes(bytes: &[u8], position: &mut usize) -> Result<Vec<u8>, ProtocolError> {
        let length = Self::read_u32(bytes, position)? as usize;
        let end = position
            .checked_add(length)
            .ok_or_else(|| ProtocolError::new("Protocol byte length overflow"))?;

        if end > bytes.len() {
            return Err(ProtocolError::new("Unexpected end of protocol frame"));
        }

        let value = bytes[*position..end].to_vec();
        *position = end;
        Ok(value)
    }

    fn timestamp_millis(timestamp: SystemTime) -> Result<u128, ProtocolError> {
        timestamp
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .map_err(|error| {
                ProtocolError::new(format!("Record timestamp is before epoch: {error}"))
            })
    }

    fn system_time_from_millis(value: u128) -> Result<SystemTime, ProtocolError> {
        let millis = u64::try_from(value)
            .map_err(|error| ProtocolError::new(format!("Timestamp is too large: {error}")))?;
        Ok(UNIX_EPOCH + Duration::from_millis(millis))
    }

    fn usize_to_u32(value: usize, label: &str) -> Result<u32, ProtocolError> {
        u32::try_from(value)
            .map_err(|error| ProtocolError::new(format!("{label} is too large: {error}")))
    }

    fn write_u16(buffer: &mut Vec<u8>, value: u16) {
        buffer.extend_from_slice(&value.to_be_bytes());
    }

    fn write_u32(buffer: &mut Vec<u8>, value: u32) {
        buffer.extend_from_slice(&value.to_be_bytes());
    }

    fn write_u64(buffer: &mut Vec<u8>, value: u64) {
        buffer.extend_from_slice(&value.to_be_bytes());
    }

    fn write_u128(buffer: &mut Vec<u8>, value: u128) {
        buffer.extend_from_slice(&value.to_be_bytes());
    }

    fn read_u8(bytes: &[u8], position: &mut usize) -> Result<u8, ProtocolError> {
        if *position >= bytes.len() {
            return Err(ProtocolError::new("Unexpected end of protocol frame"));
        }
        let value = bytes[*position];
        *position += 1;
        Ok(value)
    }

    fn read_u16(bytes: &[u8], position: &mut usize) -> Result<u16, ProtocolError> {
        Ok(u16::from_be_bytes(Self::read_fixed::<2>(bytes, position)?))
    }

    fn read_u32(bytes: &[u8], position: &mut usize) -> Result<u32, ProtocolError> {
        Ok(u32::from_be_bytes(Self::read_fixed::<4>(bytes, position)?))
    }

    fn read_u64(bytes: &[u8], position: &mut usize) -> Result<u64, ProtocolError> {
        Ok(u64::from_be_bytes(Self::read_fixed::<8>(bytes, position)?))
    }

    fn read_u128(bytes: &[u8], position: &mut usize) -> Result<u128, ProtocolError> {
        Ok(u128::from_be_bytes(Self::read_fixed::<16>(
            bytes, position,
        )?))
    }

    fn read_fixed<const N: usize>(
        bytes: &[u8],
        position: &mut usize,
    ) -> Result<[u8; N], ProtocolError> {
        let end = position
            .checked_add(N)
            .ok_or_else(|| ProtocolError::new("Protocol frame offset overflow"))?;

        if end > bytes.len() {
            return Err(ProtocolError::new("Unexpected end of protocol frame"));
        }

        let mut value = [0; N];
        value.copy_from_slice(&bytes[*position..end]);
        *position = end;
        Ok(value)
    }
}

impl Default for BinaryProtocolCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder for BinaryProtocolCodec {
    fn encode_request(&self, request: &Request) -> Result<Vec<u8>, ProtocolError> {
        let payload = Self::encode_request_payload(request.payload())?;
        let payload_length = Self::usize_to_u32(payload.len(), "request payload")?;
        let header = MessageHeader::new(
            payload_length,
            request.request_type(),
            request.correlation_id(),
        );
        let mut buffer = Vec::with_capacity(MESSAGE_HEADER_LENGTH + payload.len());
        Self::encode_header(header, &mut buffer);
        buffer.extend_from_slice(&payload);
        Ok(buffer)
    }

    fn encode_response(&self, response: &Response) -> Result<Vec<u8>, ProtocolError> {
        let payload = Self::encode_response_payload(response)?;
        let payload_length = Self::usize_to_u32(payload.len(), "response payload")?;
        let header = MessageHeader::new(
            payload_length,
            response.request_type(),
            response.correlation_id(),
        );
        let mut buffer = Vec::with_capacity(MESSAGE_HEADER_LENGTH + payload.len());
        Self::encode_header(header, &mut buffer);
        buffer.extend_from_slice(&payload);
        Ok(buffer)
    }
}

impl Decoder for BinaryProtocolCodec {
    fn decode_request(&self, bytes: &[u8]) -> Result<Request, ProtocolError> {
        let (header, mut position) = Self::decode_header(bytes)?;
        let payload = Self::decode_request_payload(header.request_type(), bytes, &mut position)?;
        Ok(Request::new(header.correlation_id(), payload))
    }

    fn decode_response(&self, bytes: &[u8]) -> Result<Response, ProtocolError> {
        let (header, mut position) = Self::decode_header(bytes)?;
        let status = ResponseStatus::from_code(Self::read_u16(bytes, &mut position)?)?;
        let payload =
            Self::decode_response_payload(header.request_type(), status, bytes, &mut position)?;
        Ok(Response::new(
            header.correlation_id(),
            header.request_type(),
            status,
            payload,
        ))
    }
}
