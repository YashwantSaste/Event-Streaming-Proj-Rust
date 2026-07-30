use std::io::ErrorKind;

use common::error::network_error::NetworkError;
use common::protocol::header::MESSAGE_HEADER_LENGTH;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;

pub struct FrameReader {
    max_frame_bytes: usize,
}

impl FrameReader {
    pub fn new(max_frame_bytes: usize) -> Result<Self, NetworkError> {
        if max_frame_bytes < MESSAGE_HEADER_LENGTH {
            return Err(NetworkError::new(format!(
                "max_frame_bytes must be at least {MESSAGE_HEADER_LENGTH}"
            )));
        }

        Ok(Self { max_frame_bytes })
    }

    pub async fn read_frame(
        &self,
        stream: &mut TcpStream,
    ) -> Result<Option<Vec<u8>>, NetworkError> {
        let mut length_prefix = [0_u8; 4];
        match stream.read_exact(&mut length_prefix).await {
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::UnexpectedEof => return Ok(None),
            Err(error) => {
                return Err(NetworkError::new(format!(
                    "Failed to read protocol frame length: {error}"
                )));
            }
        }

        let payload_length = u32::from_be_bytes(length_prefix) as usize;
        let frame_length = MESSAGE_HEADER_LENGTH
            .checked_add(payload_length)
            .ok_or_else(|| NetworkError::new("Protocol frame length overflow"))?;

        if frame_length > self.max_frame_bytes {
            return Err(NetworkError::new(format!(
                "Protocol frame exceeds max_frame_bytes: {frame_length} > {}",
                self.max_frame_bytes
            )));
        }

        let remaining_length = frame_length - length_prefix.len();
        let mut frame = Vec::with_capacity(frame_length);
        frame.extend_from_slice(&length_prefix);

        let mut remaining = vec![0_u8; remaining_length];
        stream.read_exact(&mut remaining).await.map_err(|error| {
            NetworkError::new(format!("Failed to read protocol frame: {error}"))
        })?;
        frame.extend_from_slice(&remaining);

        Ok(Some(frame))
    }
}
