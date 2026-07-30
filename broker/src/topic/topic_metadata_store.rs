use std::path::{Path, PathBuf};

use common::error::broker_error::BrokerError;
use common::filesystem::file_system::FileSystem;
use common::models::identifiers::TopicName;
use common::models::partition::PartitionConfiguration;
use common::models::topic::{Topic, TopicConfiguration};

const TOPIC_METADATA_FILE: &str = "metadata.toml";

pub struct TopicMetadataStore<F>
where
    F: FileSystem,
{
    file_system: F,
    root_directory: PathBuf,
}

impl<F> TopicMetadataStore<F>
where
    F: FileSystem,
{
    pub fn new(file_system: F, root_directory: PathBuf) -> Self {
        Self {
            file_system,
            root_directory,
        }
    }

    pub fn save(&self, topic: &Topic) -> Result<(), BrokerError> {
        let topic_directory = self.topic_directory(topic.name());
        self.create_directory(&topic_directory)?;

        self.file_system
            .write_file(
                &self.metadata_path(topic.name()),
                Self::encode_topic(topic).as_bytes(),
            )
            .map_err(|error| BrokerError::new(error.to_string()))
    }

    pub fn delete(&self, topic_name: &TopicName) -> Result<(), BrokerError> {
        let topic_directory = self.topic_directory(topic_name);
        if !self.file_system.exists(&topic_directory) {
            return Ok(());
        }

        self.file_system
            .delete(&topic_directory)
            .map_err(|error| BrokerError::new(error.to_string()))
    }

    pub fn load(&self, topic_name: &TopicName) -> Result<Option<Topic>, BrokerError> {
        let metadata_path = self.metadata_path(topic_name);
        if !self.file_system.exists(&metadata_path) {
            return Ok(None);
        }

        let bytes = self
            .file_system
            .read_file(&metadata_path)
            .map_err(|error| BrokerError::new(error.to_string()))?;
        let content = String::from_utf8(bytes)
            .map_err(|error| BrokerError::new(format!("Topic metadata is not UTF-8: {error}")))?;

        Self::decode_topic(&content).map(Some)
    }

    pub fn load_all(&self) -> Result<Vec<Topic>, BrokerError> {
        if !self.file_system.exists(&self.root_directory) {
            return Ok(Vec::new());
        }

        self.file_system
            .read_directory(&self.root_directory)
            .map_err(|error| BrokerError::new(error.to_string()))?
            .into_iter()
            .filter(|path| self.file_system.is_directory(path))
            .map(|path| self.load_topic_from_directory(&path))
            .filter_map(Result::transpose)
            .collect()
    }

    pub fn exists(&self, topic_name: &TopicName) -> bool {
        self.file_system.exists(&self.metadata_path(topic_name))
    }

    fn load_topic_from_directory(&self, path: &Path) -> Result<Option<Topic>, BrokerError> {
        let Some(directory_name) = path.file_name().and_then(|name| name.to_str()) else {
            return Ok(None);
        };

        let topic_name = TopicName::new(directory_name.to_string())
            .map_err(|error| BrokerError::new(error.to_string()))?;
        self.load(&topic_name)
    }

    fn create_directory(&self, path: &Path) -> Result<(), BrokerError> {
        self.file_system
            .create_directory(path)
            .map_err(|error| BrokerError::new(error.to_string()))
    }

    fn topic_directory(&self, topic_name: &TopicName) -> PathBuf {
        self.root_directory.join(topic_name.as_str())
    }

    fn metadata_path(&self, topic_name: &TopicName) -> PathBuf {
        self.topic_directory(topic_name).join(TOPIC_METADATA_FILE)
    }

    fn encode_topic(topic: &Topic) -> String {
        format!(
            "name = \"{}\"\npartition_count = {}\nsegment_max_bytes = {}\n",
            topic.name().as_str(),
            topic.configuration().partition_count(),
            topic
                .configuration()
                .partition_configuration()
                .segment_max_bytes()
        )
    }

    fn decode_topic(content: &str) -> Result<Topic, BrokerError> {
        let name = Self::read_required_value(content, "name")?;
        let partition_count = Self::read_required_value(content, "partition_count")?
            .parse::<u32>()
            .map_err(|error| BrokerError::new(format!("Invalid topic partition_count: {error}")))?;
        let segment_max_bytes = Self::read_required_value(content, "segment_max_bytes")?
            .parse::<u64>()
            .map_err(|error| BrokerError::new(format!("Invalid segment_max_bytes: {error}")))?;

        let topic_name =
            TopicName::new(name).map_err(|error| BrokerError::new(error.to_string()))?;
        let partition_configuration = PartitionConfiguration::new(segment_max_bytes)
            .map_err(|error| BrokerError::new(error.to_string()))?;
        let topic_configuration = TopicConfiguration::new(partition_count, partition_configuration)
            .map_err(|error| BrokerError::new(error.to_string()))?;

        Ok(Topic::new(topic_name, topic_configuration))
    }

    fn read_required_value(content: &str, key: &str) -> Result<String, BrokerError> {
        content
            .lines()
            .filter_map(Self::parse_line)
            .find(|(candidate_key, _)| candidate_key == key)
            .map(|(_, value)| value)
            .ok_or_else(|| BrokerError::new(format!("Topic metadata missing '{key}'")))
    }

    fn parse_line(line: &str) -> Option<(String, String)> {
        let line = line.split('#').next()?.trim();
        if line.is_empty() {
            return None;
        }

        let (key, value) = line.split_once('=')?;
        Some((key.trim().to_string(), Self::normalize_value(value.trim())))
    }

    fn normalize_value(value: &str) -> String {
        if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
            value[1..value.len() - 1].to_string()
        } else {
            value.to_string()
        }
    }
}
