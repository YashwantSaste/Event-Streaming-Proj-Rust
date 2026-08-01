use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::parsed_command::ParsedCommand;
use crate::command::consume_command::ConsumeCommand;
use crate::command::create_topic_command::CreateTopicCommand;
use crate::command::init_workspace::InitCommand;
use crate::command::list_topics_command::ListTopicsCommand;
use crate::command::produce_command::ProduceCommand;
use crate::command::start_broker_command::StartBrokerCommand;

pub struct CommandFactory;

impl CommandFactory {
    pub fn create(parsed_command: ParsedCommand) -> Result<Box<dyn BaseCommand>, CliError> {
        match parsed_command.name.as_str() {
            "init" => Ok(Box::new(InitCommand::create_executable_cmd(
                parsed_command,
            )?)),
            "start-broker" => Ok(Box::new(StartBrokerCommand::create_executable_cmd(
                parsed_command,
            )?)),
            "create-topic" => Ok(Box::new(CreateTopicCommand::create_executable_cmd(
                parsed_command,
            )?)),
            "list-topics" => Ok(Box::new(ListTopicsCommand::create_executable_cmd(
                parsed_command,
            )?)),
            "produce" => Ok(Box::new(ProduceCommand::create_executable_cmd(
                parsed_command,
            )?)),
            "consume" => Ok(Box::new(ConsumeCommand::create_executable_cmd(
                parsed_command,
            )?)),
            command => Err(CliError::UnsupportedCommand(command.to_string())),
        }
    }
}
