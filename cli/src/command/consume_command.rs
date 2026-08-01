use std::time::Duration;

use consumer::consumer_client::ConsumerClient;
use consumer::consumer_configuration::ConsumerConfiguration;
use tokio::time::sleep;

use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;
use crate::command::command_utils::{
    ClientConnectionOptions, client_error, parse_u32, parse_u64, required, runtime,
};

pub struct ConsumeCommand {
    topic: String,
    partition: u32,
    offset: u64,
    max_records: u32,
    commit: bool,
    group: String,
    poll_interval_ms: u64,
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
        let mut offset = self.offset;
        let max_records = self.max_records;
        let commit = self.commit;
        let poll_interval = Duration::from_millis(self.poll_interval_ms);

        runtime()?.block_on(async move {
            loop {
                let records = client
                    .poll(partition, offset, max_records)
                    .await
                    .map_err(client_error)?;

                for record in &records {
                    println!("{}", String::from_utf8_lossy(record.payload().bytes()));
                }

                if let Some(last_record) = records.last() {
                    offset = last_record.offset().next().value();
                    if commit {
                        client
                            .commit(&topic, partition, offset)
                            .await
                            .map_err(client_error)?;
                    }
                } else {
                    sleep(poll_interval).await;
                }
            }
        })
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
            poll_interval_ms: parse_u64(
                parsed_command.option("poll-interval-ms").unwrap_or("500"),
                "poll-interval-ms",
            )?,
            connection: ClientConnectionOptions::from_command(&parsed_command)?,
        })
    }
}
