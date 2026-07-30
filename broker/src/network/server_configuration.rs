use std::net::SocketAddr;

use common::error::network_error::NetworkError;

#[derive(Debug, Clone)]
pub struct BrokerServerConfiguration {
    bind_address: SocketAddr,
    max_frame_bytes: usize,
}

impl BrokerServerConfiguration {
    pub fn new(bind_address: SocketAddr, max_frame_bytes: usize) -> Result<Self, NetworkError> {
        if max_frame_bytes == 0 {
            return Err(NetworkError::new(
                "max_frame_bytes must be greater than zero",
            ));
        }

        Ok(Self {
            bind_address,
            max_frame_bytes,
        })
    }

    pub fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }

    pub fn max_frame_bytes(&self) -> usize {
        self.max_frame_bytes
    }
}
