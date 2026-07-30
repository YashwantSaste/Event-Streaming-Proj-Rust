use std::net::{IpAddr, SocketAddr};

use common::error::consumer_error::ConsumerError;
use common::models::identifiers::ConsumerGroupId;

#[derive(Debug, Clone)]
pub struct ConsumerConfiguration {
    broker_address: SocketAddr,
    group_id: ConsumerGroupId,
    max_frame_bytes: usize,
}

impl ConsumerConfiguration {
    pub fn new(
        host: &str,
        port: u16,
        group_id: &str,
        max_frame_bytes: usize,
    ) -> Result<Self, ConsumerError> {
        if max_frame_bytes == 0 {
            return Err(ConsumerError::new(
                "max_frame_bytes must be greater than zero",
            ));
        }
        let ip = host
            .parse::<IpAddr>()
            .map_err(|error| ConsumerError::new(format!("Invalid broker host: {error}")))?;
        let group_id = ConsumerGroupId::new(group_id.to_string())
            .map_err(|error| ConsumerError::new(error.to_string()))?;

        Ok(Self {
            broker_address: SocketAddr::new(ip, port),
            group_id,
            max_frame_bytes,
        })
    }

    pub fn broker_address(&self) -> SocketAddr {
        self.broker_address
    }

    pub fn group_id(&self) -> &ConsumerGroupId {
        &self.group_id
    }

    pub fn max_frame_bytes(&self) -> usize {
        self.max_frame_bytes
    }
}
