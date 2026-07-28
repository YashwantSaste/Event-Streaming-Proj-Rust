use std::path::PathBuf;
use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::parsed_command::ParsedCommand;
use crate::command::init_workspace::InitCommand;

pub struct CommandFactory;

impl CommandFactory {

    pub fn create(parsed_command: ParsedCommand) -> Result<Box<dyn BaseCommand>, CliError>{
        match parsed_command.name.as_str() {
            "init" => Ok(Box::new(InitCommand::create_executable_cmd(parsed_command)?)),
            command => Err(CliError::UnsupportedCommand(
                command.to_string(),
            )),
        }
    }

}