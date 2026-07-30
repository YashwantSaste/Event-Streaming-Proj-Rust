use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};

use common::configuration::configuration::Configuration;
use common::error::broker_error::BrokerError;

#[derive(Debug, Clone)]
pub struct BrokerConfiguration {
    bind_address: SocketAddr,
    storage_root_directory: PathBuf,
    topic_metadata_directory: PathBuf,
    segment_max_bytes: u64,
    max_frame_bytes: usize,
}

impl BrokerConfiguration {
    pub fn from_configuration(configuration: &Configuration) -> Result<Self, BrokerError> {
        let host = configuration.get_or("broker.host", "127.0.0.1");
        let port = Self::parse_u16(configuration.get_or("broker.port", "9092"), "broker.port")?;
        let bind_address = SocketAddr::new(Self::parse_ip_address(host)?, port);

        let storage_root_directory =
            PathBuf::from(configuration.get_or("storage.data_directory", "data/broker"));
        let topic_metadata_directory =
            PathBuf::from(configuration.get_or("storage.topics_directory", "data/topics"));
        let segment_max_bytes = Self::parse_u64(
            configuration.get_or("storage.segment_max_bytes", "1048576"),
            "storage.segment_max_bytes",
        )?;
        let max_frame_bytes = Self::parse_usize(
            configuration.get_or("network.max_frame_bytes", "1048576"),
            "network.max_frame_bytes",
        )?;

        Ok(Self {
            bind_address,
            storage_root_directory,
            topic_metadata_directory,
            segment_max_bytes,
            max_frame_bytes,
        })
    }

    pub fn bind_address(&self) -> SocketAddr {
        self.bind_address
    }

    pub fn storage_root_directory(&self) -> &Path {
        &self.storage_root_directory
    }

    pub fn topic_metadata_directory(&self) -> &Path {
        &self.topic_metadata_directory
    }

    pub fn segment_max_bytes(&self) -> u64 {
        self.segment_max_bytes
    }

    pub fn max_frame_bytes(&self) -> usize {
        self.max_frame_bytes
    }

    fn parse_ip_address(value: &str) -> Result<IpAddr, BrokerError> {
        value
            .parse::<IpAddr>()
            .map_err(|error| BrokerError::new(format!("Invalid broker.host '{value}': {error}")))
    }

    fn parse_u16(value: &str, key: &str) -> Result<u16, BrokerError> {
        value
            .parse::<u16>()
            .map_err(|error| BrokerError::new(format!("Invalid {key} '{value}': {error}")))
    }

    fn parse_u64(value: &str, key: &str) -> Result<u64, BrokerError> {
        value
            .parse::<u64>()
            .map_err(|error| BrokerError::new(format!("Invalid {key} '{value}': {error}")))
    }

    fn parse_usize(value: &str, key: &str) -> Result<usize, BrokerError> {
        value
            .parse::<usize>()
            .map_err(|error| BrokerError::new(format!("Invalid {key} '{value}': {error}")))
    }
}
