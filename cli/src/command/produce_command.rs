use std::io::{self, BufRead};

use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;
use crate::command::command_utils::{
    ClientConnectionOptions, client_error, parse_u32, producer_client, required, runtime, success,
};

pub struct ProduceCommand {
    topic: String,
    partition: u32,
    key: Option<Vec<u8>>,
    message: Option<Vec<u8>>,
    connection: ClientConnectionOptions,
}

impl BaseCommand for ProduceCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let rt = runtime()?;
        let mut client = producer_client(&self.connection)?;
        let topic = self.topic.clone();
        let partition = self.partition;
        let key = self.key.clone();

        if let Some(message) = &self.message {
            send_line(&rt, &mut client, &topic, partition, key, message.clone())?;
            return Ok(success(""));
        }

        for line in io::stdin().lock().lines() {
            let line = line.map_err(|error| CliError::UnexpectedError(error.to_string()))?;
            send_line(
                &rt,
                &mut client,
                &topic,
                partition,
                key.clone(),
                line.into_bytes(),
            )?;
        }

        Ok(success(""))
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
            message: parsed_command
                .option("message")
                .map(|value| value.as_bytes().to_vec()),
            connection: ClientConnectionOptions::from_command(&parsed_command)?,
        })
    }
}

fn send_line(
    runtime: &tokio::runtime::Runtime,
    client: &mut producer::producer_client::ProducerClient,
    topic: &str,
    partition: u32,
    key: Option<Vec<u8>>,
    message: Vec<u8>,
) -> Result<(), CliError> {
    runtime.block_on(async {
        client
            .send(topic, partition, key, message)
            .await
            .map_err(client_error)
            .map(|_| ())
    })
}
