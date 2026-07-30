use std::time::{Duration, SystemTime, UNIX_EPOCH};

use common::error::storage_error::StorageError;
use common::models::identifiers::{Offset, PartitionId, TopicName};
use common::models::record::{Record, RecordKey, RecordPayload};

const RECORD_MAGIC: u32 = 0x4553_5231;
const HEADER_LENGTH_BYTES: usize = 4;

pub struct RecordCodec;

impl RecordCodec {
    pub fn encode(record: &Record) -> Result<Vec<u8>, StorageError> {
        let mut body = Vec::new();
        Self::write_u32(&mut body, RECORD_MAGIC);
        Self::write_u64(&mut body, record.offset().value());
        Self::write_u128(&mut body, Self::timestamp_millis(record.timestamp())?);

        match record.key() {
            Some(key) => {
                body.push(1);
                Self::write_u32(
                    &mut body,
                    Self::usize_to_u32(key.bytes().len(), "record key")?,
                );
                body.extend_from_slice(key.bytes());
            }
            None => {
                body.push(0);
                Self::write_u32(&mut body, 0);
            }
        }

        Self::write_u32(
            &mut body,
            Self::usize_to_u32(record.payload().bytes().len(), "record payload")?,
        );
        body.extend_from_slice(record.payload().bytes());

        let mut frame = Vec::new();
        Self::write_u32(&mut frame, Self::usize_to_u32(body.len(), "record frame")?);
        frame.extend_from_slice(&body);
        Ok(frame)
    }

    pub fn decode_all(
        bytes: &[u8],
        topic: &TopicName,
        partition_id: PartitionId,
    ) -> Result<Vec<Record>, StorageError> {
        let mut position = 0;
        let mut records = Vec::new();

        while position < bytes.len() {
            let frame_length = Self::read_u32(bytes, &mut position)? as usize;
            let frame_end = position
                .checked_add(frame_length)
                .ok_or_else(|| StorageError::new("Record frame length overflow"))?;

            if frame_end > bytes.len() {
                return Err(StorageError::new("Record frame exceeds segment length"));
            }

            let record = Self::decode_frame(&bytes[position..frame_end], topic, partition_id)?;
            records.push(record);
            position = frame_end;
        }

        Ok(records)
    }

    fn decode_frame(
        frame: &[u8],
        topic: &TopicName,
        partition_id: PartitionId,
    ) -> Result<Record, StorageError> {
        let mut position = 0;
        let magic = Self::read_u32(frame, &mut position)?;
        if magic != RECORD_MAGIC {
            return Err(StorageError::new("Invalid record magic number"));
        }

        let offset = Offset::new(Self::read_u64(frame, &mut position)?);
        let timestamp = Self::system_time_from_millis(Self::read_u128(frame, &mut position)?)?;
        let has_key = Self::read_u8(frame, &mut position)?;
        let key_length = Self::read_u32(frame, &mut position)? as usize;
        let key = if has_key == 1 {
            Some(RecordKey::new(Self::read_bytes(
                frame,
                &mut position,
                key_length,
            )?))
        } else {
            None
        };

        let payload_length = Self::read_u32(frame, &mut position)? as usize;
        let payload = RecordPayload::new(Self::read_bytes(frame, &mut position, payload_length)?);

        Ok(Record::new(
            topic.clone(),
            partition_id,
            offset,
            key,
            payload,
            timestamp,
        ))
    }

    fn timestamp_millis(timestamp: SystemTime) -> Result<u128, StorageError> {
        timestamp
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_millis())
            .map_err(|error| {
                StorageError::new(format!("Record timestamp is before epoch: {error}"))
            })
    }

    fn system_time_from_millis(value: u128) -> Result<SystemTime, StorageError> {
        let millis = u64::try_from(value)
            .map_err(|error| StorageError::new(format!("Timestamp is too large: {error}")))?;
        Ok(UNIX_EPOCH + Duration::from_millis(millis))
    }

    fn usize_to_u32(value: usize, label: &str) -> Result<u32, StorageError> {
        u32::try_from(value)
            .map_err(|error| StorageError::new(format!("{label} is too large to encode: {error}")))
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

    fn read_u8(bytes: &[u8], position: &mut usize) -> Result<u8, StorageError> {
        if *position >= bytes.len() {
            return Err(StorageError::new("Unexpected end of record"));
        }

        let value = bytes[*position];
        *position += 1;
        Ok(value)
    }

    fn read_u32(bytes: &[u8], position: &mut usize) -> Result<u32, StorageError> {
        let value = Self::read_fixed::<4>(bytes, position)?;
        Ok(u32::from_be_bytes(value))
    }

    fn read_u64(bytes: &[u8], position: &mut usize) -> Result<u64, StorageError> {
        let value = Self::read_fixed::<8>(bytes, position)?;
        Ok(u64::from_be_bytes(value))
    }

    fn read_u128(bytes: &[u8], position: &mut usize) -> Result<u128, StorageError> {
        let value = Self::read_fixed::<16>(bytes, position)?;
        Ok(u128::from_be_bytes(value))
    }

    fn read_fixed<const N: usize>(
        bytes: &[u8],
        position: &mut usize,
    ) -> Result<[u8; N], StorageError> {
        let end = position
            .checked_add(N)
            .ok_or_else(|| StorageError::new("Record offset overflow"))?;

        if end > bytes.len() {
            return Err(StorageError::new("Unexpected end of record"));
        }

        let mut value = [0; N];
        value.copy_from_slice(&bytes[*position..end]);
        *position = end;
        Ok(value)
    }

    fn read_bytes(
        bytes: &[u8],
        position: &mut usize,
        length: usize,
    ) -> Result<Vec<u8>, StorageError> {
        let end = position
            .checked_add(length)
            .ok_or_else(|| StorageError::new("Record byte length overflow"))?;

        if end > bytes.len() {
            return Err(StorageError::new("Unexpected end of record"));
        }

        let value = bytes[*position..end].to_vec();
        *position = end;
        Ok(value)
    }

    pub fn frame_header_length() -> usize {
        HEADER_LENGTH_BYTES
    }
}
