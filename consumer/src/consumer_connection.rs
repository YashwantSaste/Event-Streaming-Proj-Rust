use common::error::consumer_error::ConsumerError;
use common::protocol::binary_protocol_codec::BinaryProtocolCodec;
use common::protocol::decoder::Decoder;
use common::protocol::encoder::Encoder;
use common::protocol::header::MESSAGE_HEADER_LENGTH;
use common::protocol::request::Request;
use common::protocol::response::Response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::consumer_configuration::ConsumerConfiguration;

pub struct ConsumerConnection {
    stream: TcpStream,
    codec: BinaryProtocolCodec,
    max_frame_bytes: usize,
}

impl ConsumerConnection {
    pub async fn connect(configuration: &ConsumerConfiguration) -> Result<Self, ConsumerError> {
        let stream = TcpStream::connect(configuration.broker_address())
            .await
            .map_err(|error| ConsumerError::new(format!("Failed to connect to broker: {error}")))?;
        Ok(Self {
            stream,
            codec: BinaryProtocolCodec::new(),
            max_frame_bytes: configuration.max_frame_bytes(),
        })
    }

    pub async fn send(&mut self, request: &Request) -> Result<Response, ConsumerError> {
        let frame = self
            .codec
            .encode_request(request)
            .map_err(|error| ConsumerError::new(error.to_string()))?;
        self.stream
            .write_all(&frame)
            .await
            .map_err(|error| ConsumerError::new(format!("Failed to write request: {error}")))?;
        let response_frame = self.read_frame().await?;
        self.codec
            .decode_response(&response_frame)
            .map_err(|error| ConsumerError::new(error.to_string()))
    }

    async fn read_frame(&mut self) -> Result<Vec<u8>, ConsumerError> {
        let mut length_prefix = [0_u8; 4];
        self.stream
            .read_exact(&mut length_prefix)
            .await
            .map_err(|error| {
                ConsumerError::new(format!("Failed to read response length: {error}"))
            })?;
        let payload_length = u32::from_be_bytes(length_prefix) as usize;
        let frame_length = MESSAGE_HEADER_LENGTH
            .checked_add(payload_length)
            .ok_or_else(|| ConsumerError::new("Response frame length overflow"))?;
        if frame_length > self.max_frame_bytes {
            return Err(ConsumerError::new(format!(
                "Response frame exceeds max_frame_bytes: {frame_length}"
            )));
        }
        let mut frame = Vec::with_capacity(frame_length);
        frame.extend_from_slice(&length_prefix);
        let mut remaining = vec![0_u8; frame_length - length_prefix.len()];
        self.stream
            .read_exact(&mut remaining)
            .await
            .map_err(|error| ConsumerError::new(format!("Failed to read response: {error}")))?;
        frame.extend_from_slice(&remaining);
        Ok(frame)
    }
}
