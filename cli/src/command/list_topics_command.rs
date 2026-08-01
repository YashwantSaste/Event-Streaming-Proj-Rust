use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;
use crate::command::command_utils::{
    ClientConnectionOptions, client_error, producer_client, runtime, success,
};

pub struct ListTopicsCommand {
    connection: ClientConnectionOptions,
}

impl BaseCommand for ListTopicsCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let client = producer_client(&self.connection)?;
        let topics =
            runtime()?.block_on(async move { client.list_topics().await.map_err(client_error) })?;
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
