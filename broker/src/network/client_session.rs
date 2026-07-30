use std::sync::Arc;

use common::error::network_error::NetworkError;
use common::protocol::binary_protocol_codec::BinaryProtocolCodec;
use common::protocol::decoder::Decoder;
use common::protocol::encoder::Encoder;
use common::protocol::request_type::RequestType;
use common::protocol::response::Response;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::Mutex;

use crate::network::frame_reader::FrameReader;
use crate::network::request_dispatcher::RequestDispatcher;

pub struct ClientSession<D>
where
    D: RequestDispatcher + Send,
{
    stream: TcpStream,
    frame_reader: FrameReader,
    codec: BinaryProtocolCodec,
    dispatcher: Arc<Mutex<D>>,
}

impl<D> ClientSession<D>
where
    D: RequestDispatcher + Send,
{
    pub fn new(
        stream: TcpStream,
        max_frame_bytes: usize,
        dispatcher: Arc<Mutex<D>>,
    ) -> Result<Self, NetworkError> {
        Ok(Self {
            stream,
            frame_reader: FrameReader::new(max_frame_bytes)?,
            codec: BinaryProtocolCodec::new(),
            dispatcher,
        })
    }

    pub async fn run(&mut self) -> Result<(), NetworkError> {
        while let Some(frame) = self.frame_reader.read_frame(&mut self.stream).await? {
            let response = match self.codec.decode_request(&frame) {
                Ok(request) => self.dispatcher.lock().await.dispatch(request),
                Err(error) => Response::error(0, RequestType::ListTopics, error.to_string()),
            };

            let encoded = self
                .codec
                .encode_response(&response)
                .map_err(|error| NetworkError::new(error.to_string()))?;

            self.stream
                .write_all(&encoded)
                .await
                .map_err(|error| NetworkError::new(format!("Failed to write response: {error}")))?;
        }

        Ok(())
    }
}
