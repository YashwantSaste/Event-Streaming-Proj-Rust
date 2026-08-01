use std::path::PathBuf;
use std::time::Duration;

use common::configuration::configuration_reader::ConfigurationReader;
use common::configuration::toml_configuration_reader::TomlConfigurationReader;
use common::filesystem::file_manager::FileManager;
use common::filesystem::local_file_system::LocalFileSystem;
use producer::producer_client::ProducerClient;
use producer::producer_configuration::ProducerConfiguration;
use producer::retry_policy::RetryPolicy;
use tokio::runtime::Runtime;

use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;

#[derive(Debug, Clone)]
pub struct ClientConnectionOptions {
    pub host: String,
    pub port: u16,
    pub max_frame_bytes: usize,
}

impl ClientConnectionOptions {
    pub fn from_command(parsed_command: &ParsedCommand) -> Result<Self, CliError> {
        let config_path = broker_config_path(parsed_command);
        let configuration = if FileManager::exists(&config_path) {
            let reader = TomlConfigurationReader::new(LocalFileSystem::new(), config_path.clone());
            Some(
                reader
                    .read()
                    .map_err(|error| CliError::UnexpectedError(error.to_string()))?,
            )
        } else {
            None
        };

        let default_host = configuration
            .as_ref()
            .and_then(|configuration| configuration.get("broker.host"))
            .unwrap_or("127.0.0.1");
        let default_port = configuration
            .as_ref()
            .and_then(|configuration| configuration.get("broker.port"))
            .unwrap_or("9092");
        let default_max_frame_bytes = configuration
            .as_ref()
            .and_then(|configuration| configuration.get("network.max_frame_bytes"))
            .unwrap_or("1048576");

        Ok(Self {
            host: parsed_command
                .option("host")
                .unwrap_or(default_host)
                .to_string(),
            port: parse_u16(
                parsed_command.option("port").unwrap_or(default_port),
                "port",
            )?,
            max_frame_bytes: parse_usize(
                parsed_command
                    .option("max-frame-bytes")
                    .unwrap_or(default_max_frame_bytes),
                "max-frame-bytes",
            )?,
        })
    }
}

pub fn workspace_root(parsed_command: &ParsedCommand) -> PathBuf {
    parsed_command
        .option("workspace")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if let Some(home) = dirs::home_dir() {
                let active_ws_file = home.join(".es_workspace");
                if let Ok(path_str) = std::fs::read_to_string(&active_ws_file) {
                    let path = PathBuf::from(path_str.trim());
                    if path.exists() {
                        return path;
                    }
                }
            }
            PathBuf::from(".")
        })
}

pub fn broker_config_path(parsed_command: &ParsedCommand) -> PathBuf {
    parsed_command
        .option("config")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            workspace_root(parsed_command)
                .join("config")
                .join("broker.toml")
        })
}

pub fn client_error(error: impl std::fmt::Display) -> CliError {
    let message = error.to_string();
    if message.contains("Failed to connect") || message.contains("actively refused") {
        CliError::UnexpectedError(format!(
            "{message}\nStart the broker first, for example: cargo run -p cli -- start-broker --workspace <workspace-path>"
        ))
    } else {
        CliError::UnexpectedError(message)
    }
}

pub fn producer_client(options: &ClientConnectionOptions) -> Result<ProducerClient, CliError> {
    let configuration =
        ProducerConfiguration::new(&options.host, options.port, options.max_frame_bytes)
            .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
    Ok(ProducerClient::new(
        configuration,
        RetryPolicy::new(3, Duration::from_millis(100)),
    ))
}

pub fn runtime() -> Result<Runtime, CliError> {
    Runtime::new().map_err(|error| CliError::UnexpectedError(error.to_string()))
}

pub fn success(message: impl Into<String>) -> CommandResult {
    CommandResult {
        success: true,
        exit_code: 0,
        message: message.into(),
    }
}

pub fn required(parsed_command: &ParsedCommand, key: &str) -> Result<String, CliError> {
    parsed_command
        .option(key)
        .map(ToString::to_string)
        .ok_or_else(|| CliError::InvalidArguments(format!("Missing required option --{key}")))
}

pub fn parse_u16(value: &str, key: &str) -> Result<u16, CliError> {
    value
        .parse::<u16>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}

pub fn parse_u32(value: &str, key: &str) -> Result<u32, CliError> {
    value
        .parse::<u32>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}

pub fn parse_u64(value: &str, key: &str) -> Result<u64, CliError> {
    value
        .parse::<u64>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}

pub fn parse_usize(value: &str, key: &str) -> Result<usize, CliError> {
    value
        .parse::<usize>()
        .map_err(|error| CliError::InvalidArguments(format!("Invalid --{key}: {error}")))
}
