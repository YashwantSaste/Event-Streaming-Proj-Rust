use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;

pub trait BaseCommand {
    fn execute(&self) -> Result<CommandResult, CliError>;

    fn create_executable_cmd(parsed_command: ParsedCommand) -> Result<Self, CliError>
    where
        Self: Sized;
}
