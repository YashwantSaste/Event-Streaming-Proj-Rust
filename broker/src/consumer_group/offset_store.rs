use std::collections::HashMap;
use std::path::PathBuf;

use common::error::broker_error::BrokerError;
use common::filesystem::file_system::FileSystem;
use common::models::identifiers::{ConsumerGroupId, Offset, PartitionId, TopicName};
use common::models::partition::TopicPartition;

const OFFSETS_FILE: &str = "offsets.toml";

pub struct OffsetStore<F>
where
    F: FileSystem,
{
    file_system: F,
    root_directory: PathBuf,
}

impl<F> OffsetStore<F>
where
    F: FileSystem,
{
    pub fn new(file_system: F, root_directory: PathBuf) -> Self {
        Self {
            file_system,
            root_directory,
        }
    }

    pub fn save(
        &self,
        group_id: &ConsumerGroupId,
        offsets: &HashMap<TopicPartition, Offset>,
    ) -> Result<(), BrokerError> {
        let group_directory = self.group_directory(group_id);
        self.file_system
            .create_directory(&group_directory)
            .map_err(|error| BrokerError::new(error.to_string()))?;

        let mut lines = offsets
            .iter()
            .map(|(partition, offset)| {
                format!(
                    "{}.{} = {}\n",
                    partition.topic().as_str(),
                    partition.partition_id().value(),
                    offset.value()
                )
            })
            .collect::<Vec<_>>();
        lines.sort();

        self.file_system
            .write_file(
                &group_directory.join(OFFSETS_FILE),
                lines.concat().as_bytes(),
            )
            .map_err(|error| BrokerError::new(error.to_string()))
    }

    pub fn load(
        &self,
        group_id: &ConsumerGroupId,
    ) -> Result<HashMap<TopicPartition, Offset>, BrokerError> {
        let path = self.group_directory(group_id).join(OFFSETS_FILE);
        if !self.file_system.exists(&path) {
            return Ok(HashMap::new());
        }

        let bytes = self
            .file_system
            .read_file(&path)
            .map_err(|error| BrokerError::new(error.to_string()))?;
        let content = String::from_utf8(bytes)
            .map_err(|error| BrokerError::new(format!("Offset file is not UTF-8: {error}")))?;

        content
            .lines()
            .filter_map(Self::parse_line)
            .map(Self::parse_offset_entry)
            .collect()
    }

    fn group_directory(&self, group_id: &ConsumerGroupId) -> PathBuf {
        self.root_directory.join(group_id.as_str())
    }

    fn parse_line(line: &str) -> Option<(String, String)> {
        let line = line.split('#').next()?.trim();
        if line.is_empty() {
            return None;
        }
        let (key, value) = line.split_once('=')?;
        Some((key.trim().to_string(), value.trim().to_string()))
    }

    fn parse_offset_entry(
        entry: (String, String),
    ) -> Result<(TopicPartition, Offset), BrokerError> {
        let (key, value) = entry;
        let (topic, partition) = key
            .rsplit_once('.')
            .ok_or_else(|| BrokerError::new(format!("Invalid offset key '{key}'")))?;
        let topic = TopicName::new(topic.to_string())
            .map_err(|error| BrokerError::new(error.to_string()))?;
        let partition = partition.parse::<u32>().map_err(|error| {
            BrokerError::new(format!("Invalid partition in offset key '{key}': {error}"))
        })?;
        let offset = value.parse::<u64>().map_err(|error| {
            BrokerError::new(format!("Invalid offset value '{value}': {error}"))
        })?;

        Ok((
            TopicPartition::new(topic, PartitionId::new(partition)),
            Offset::new(offset),
        ))
    }
}
