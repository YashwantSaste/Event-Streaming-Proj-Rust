use crate::error::protocol_error::ProtocolError;
use crate::protocol::request::Request;
use crate::protocol::response::Response;

pub trait Decoder {
    fn decode_request(&self, bytes: &[u8]) -> Result<Request, ProtocolError>;

    fn decode_response(&self, bytes: &[u8]) -> Result<Response, ProtocolError>;
}
