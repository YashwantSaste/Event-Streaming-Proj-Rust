use crate::error::protocol_error::ProtocolError;
use crate::protocol::request::Request;
use crate::protocol::response::Response;

pub trait Encoder {
    fn encode_request(&self, request: &Request) -> Result<Vec<u8>, ProtocolError>;

    fn encode_response(&self, response: &Response) -> Result<Vec<u8>, ProtocolError>;
}
