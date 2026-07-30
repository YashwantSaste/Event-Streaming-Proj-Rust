use std::net::{IpAddr, SocketAddr};

use common::error::producer_error::ProducerError;

#[derive(Debug, Clone)]
pub struct ProducerConfiguration {
    broker_address: SocketAddr,
    max_frame_bytes: usize,
}

impl ProducerConfiguration {
    pub fn new(host: &str, port: u16, max_frame_bytes: usize) -> Result<Self, ProducerError> {
        if max_frame_bytes == 0 {
            return Err(ProducerError::new(
                "max_frame_bytes must be greater than zero",
            ));
        }

        let ip = host
            .parse::<IpAddr>()
            .map_err(|error| ProducerError::new(format!("Invalid broker host: {error}")))?;

        Ok(Self {
            broker_address: SocketAddr::new(ip, port),
            max_frame_bytes,
        })
    }

    pub fn broker_address(&self) -> SocketAddr {
        self.broker_address
    }

    pub fn max_frame_bytes(&self) -> usize {
        self.max_frame_bytes
    }
}
