use crate::protocol::request_type::RequestType;

pub const MESSAGE_HEADER_LENGTH: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MessageHeader {
    payload_length: u32,
    request_type: RequestType,
    correlation_id: u32,
}

impl MessageHeader {
    pub fn new(payload_length: u32, request_type: RequestType, correlation_id: u32) -> Self {
        Self {
            payload_length,
            request_type,
            correlation_id,
        }
    }

    pub fn payload_length(self) -> u32 {
        self.payload_length
    }

    pub fn request_type(self) -> RequestType {
        self.request_type
    }

    pub fn correlation_id(self) -> u32 {
        self.correlation_id
    }
}
