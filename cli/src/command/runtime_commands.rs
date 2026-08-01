use std::path::PathBuf;
use std::time::Duration;

use broker::application::broker_application::BrokerApplication;
use broker::application::broker_configuration::BrokerConfiguration;
use common::configuration::configuration_reader::ConfigurationReader;
use common::configuration::toml_configuration_reader::TomlConfigurationReader;
use common::filesystem::local_file_system::LocalFileSystem;
use consumer::consumer_client::ConsumerClient;
use consumer::consumer_configuration::ConsumerConfiguration;
use producer::producer_client::ProducerClient;
use producer::producer_configuration::ProducerConfiguration;
use producer::retry_policy::RetryPolicy;
use tokio::runtime::Runtime;

use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;

pub struct StartBrokerCommand {
    config_path: PathBuf,
}

impl BaseCommand for StartBrokerCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let config_path = self.config_path.clone();
        runtime()?.block_on(async move {
            let reader = TomlConfigurationReader::new(LocalFileSystem::new(), config_path);
            let configuration = reader
                .read()
                .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
            let broker_configuration = BrokerConfiguration::from_configuration(&configuration)
                .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
            BrokerApplication::new(broker_configuration)
                .run()
                .await
                .map_err(|error| CliError::UnexpectedError(error.to_string()))
        })?;

        Ok(success("Broker stopped"))
    }

    fn create_executable_cmd(parsed_command: ParsedCommand) -> Result<Self, CliError> {
        Ok(Self {
            config_path: PathBuf::from(
                parsed_command
                    .option("config")
                    .unwrap_or("assets/broker.toml"),
            ),
        })
    }
}

pub struct CreateTopicCommand {
    topic: String,
    partitions: u32,
    segment_max_bytes: u64,
    connection: ClientConnectionOptions,
}

impl BaseCommand for CreateTopicCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let mut client = producer_client(&self.connection)?;
        let topic = self.topic.clone();
        let partitions = self.partitions;
        let segment_max_bytes = self.segment_max_bytes;

        let response = runtime()?.block_on(async move {
            client
                .create_topic(&topic, partitions, segment_max_bytes)
                .await
                .map_err(|error| CliError::UnexpectedError(error.to_string()))
        })?;

        Ok(success(format!("Created topic {}", response.topic())))
    }

    fn create_executable_cmd(parsed_command: ParsedCommand) -> Result<Self, CliError> {
        Ok(Self {
            topic: required(&parsed_command, "topic")?,
            partitions: parse_u32(
                parsed_command.option("partitions").unwrap_or("1"),
                "partitions",
            )?,
            segment_max_bytes: parse_u64(
                parsed_command
                    .option("segment-max-bytes")
                    .unwrap_or("1048576"),
                "segment-max-bytes",
            )?,
            connection: ClientConnectionOptions::from_command(&parsed_command)?,
        })
    }
}

pub struct ListTopicsCommand {
    connection: ClientConnectionOptions,
}

impl BaseCommand for ListTopicsCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let client = producer_client(&self.connection)?;
        let topics = runtime()?.block_on(async move {
            client
                .list_topics()
                .await
                .map_err(|error| CliError::UnexpectedError(error.to_string()))
        })?;
        let message = if topics.is_empty() {
            "No topics found".to_string()
        } else {
            topics
                .into_iter()
                .map(|topic| topic.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(success(message))
    }

    fn create_executable_cmd(parsed_command: ParsedCommand) -> Result<Self, CliError> {
        Ok(Self {
            connection: ClientConnectionOptions::from_command(&parsed_command)?,
        })
    }
}

pub struct ProduceCommand {
    topic: String,
    partition: u32,
    key: Option<Vec<u8>>,
    message: Vec<u8>,
    connection: ClientConnectionOptions,
}

impl BaseCommand for ProduceCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let mut client = producer_client(&self.connection)?;
        let topic = self.topic.clone();
        let partition = self.partition;
        let key = self.key.clone();
        let message = self.message.clone();

        let response = runtime()?.block_on(async move {
            client
                .send(&topic, partition, key, message)
                .await
                .map_err(|error| CliError::UnexpectedError(error.to_string()))
        })?;

        Ok(success(format!(
            "Produced to {}:{} at offset {}",
            response.partition().topic(),
            response.partition().partition_id(),
            response.offset()
        )))
    }

    fn create_executable_cmd(parsed_command: ParsedCommand) -> Result<Self, CliError> {
        Ok(Self {
            topic: required(&parsed_command, "topic")?,
            partition: parse_u32(
                parsed_command.option("partition").unwrap_or("0"),
                "partition",
            )?,
            key: parsed_command
                .option("key")
                .map(|value| value.as_bytes().to_vec()),
            message: required(&parsed_command, "message")?.into_bytes(),
            connection: ClientConnectionOptions::from_command(&parsed_command)?,
        })
    }
}

pub struct ConsumeCommand {
    topic: String,
    partition: u32,
    offset: u64,
    max_records: u32,
    commit: bool,
    group: String,
    connection: ClientConnectionOptions,
}

impl BaseCommand for ConsumeCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let configuration = ConsumerConfiguration::new(
            &self.connection.host,
            self.connection.port,
            &self.group,
            self.connection.max_frame_bytes,
        )
        .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
        let mut client = ConsumerClient::new(configuration);
        client
            .subscribe(&self.topic)
            .map_err(|error| CliError::UnexpectedError(error.to_string()))?;

        let topic = self.topic.clone();
        let partition = self.partition;
        let offset = self.offset;
        let max_records = self.max_records;
        let commit = self.commit;

        let lines = runtime()?.block_on(async move {
            let records = client
                .poll(partition, offset, max_records)
                .await
                .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
            let next_offset = records.last().map(|record| record.offset().next());
            let mut lines = records
                .iter()
                .map(|record| {
                    format!(
                        "{}:{}:{} {}",
                        record.topic(),
                        record.partition_id(),
                        record.offset(),
                        String::from_utf8_lossy(record.payload().bytes())
                    )
                })
                .collect::<Vec<_>>();

            if commit {
                if let Some(next_offset) = next_offset {
                    let committed = client
                        .commit(&topic, partition, next_offset.value())
                        .await
                        .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
                    lines.push(format!("Committed offset {committed}"));
                }
            }

            Ok::<Vec<String>, CliError>(lines)
        })?;

        Ok(success(if lines.is_empty() {
            "No records found".to_string()
        } else {
            lines.join("\n")
        }))
    }

    fn create_executable_cmd(parsed_command: ParsedCommand) -> Result<Self, CliError> {
        Ok(Self {
            topic: required(&parsed_command, "topic")?,
            partition: parse_u32(
                parsed_command.option("partition").unwrap_or("0"),
                "partition",
            )?,
            offset: parse_u64(parsed_command.option("offset").unwrap_or("0"), "offset")?,
            max_records: parse_u32(
                parsed_command.option("max-records").unwrap_or("10"),
                "max-records",
            )?,
            commit: parsed_command.option("commit") == Some("true"),
            group: parsed_command
                .option("group")
                .unwrap_or("default")
                .to_string(),
            connection: ClientConnectionOptions::from_command(&parsed_command)?,
        })
    }
}

#[derive(Debug, Clone)]
struct ClientConnectionOptions {
    host: String,
    port: u16,
    max_frame_bytes: usize,
}

impl ClientConnectionOptions {
    fn from_command(parsed_command: &ParsedCommand) -> Result<Self, CliError> {
        Ok(Self {
            host: parsed_command
                .option("host")
                .unwrap_or("127.0.0.1")
                .to_string(),
            port: parse_u16(parsed_command.option("port").unwrap_or("9092"), "port")?,
            max_frame_bytes: parse_usize(
                parsed_command
                    .option("max-frame-bytes")
                    .unwrap_or("1048576"),
                "max-frame-bytes",
            )?,
        })
    }
}

fn producer_client(options: &ClientConnectionOptions) -> Result<ProducerClient, CliError> {
    let configuration =
        ProducerConfiguration::new(&options.host, options.port, options.max_frame_bytes)
            .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
    Ok(ProducerClient::new(
        configuration,
        RetryPolicy::new(3, Duration::from_millis(100)),
    ))
}

fn runtime() -> Result<Runtime, CliError> {
    Runtime::new().map_err(|error| CliError::UnexpectedError(error.to_string()))
}

fn success(message: impl Into<String>) -> CommandResult {
    CommandResult {
        success: true,
        exit_code: 0,
        message: message.into(),
    }
}

fn required(parsed_command: &ParsedCommand, key: &str) -> Result<String, CliError> {
    parsed_command
        .option(key)
        .map(ToString::to_string)
        .ok_or_else(|| CliError::InvalidArguments(format!("Missing required option --{key}")))
}

fn parse_u16(value: &str, key: &str) -> Result<u16, CliError> {
    value
        .parse::<u16>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}

fn parse_u32(value: &str, key: &str) -> Result<u32, CliError> {
    value
        .parse::<u32>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}

fn parse_u64(value: &str, key: &str) -> Result<u64, CliError> {
    value
        .parse::<u64>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}

fn parse_usize(value: &str, key: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}
