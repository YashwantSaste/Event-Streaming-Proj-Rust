use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;
use crate::command::command_utils::{
    ClientConnectionOptions, client_error, parse_u32, parse_u64, producer_client, required,
    runtime, success,
};

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
                .map_err(client_error)
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
