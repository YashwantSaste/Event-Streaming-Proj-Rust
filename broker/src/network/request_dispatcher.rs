use common::protocol::request::Request;
use common::protocol::response::Response;

pub trait RequestDispatcher {
    fn dispatch(&mut self, request: Request) -> Response;
}
