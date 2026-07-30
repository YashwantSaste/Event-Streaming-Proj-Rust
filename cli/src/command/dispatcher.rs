use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;

pub struct CommandDispatcher;

impl CommandDispatcher {
    pub fn dispatch(command: Box<dyn BaseCommand>) -> Result<CommandResult, CliError> {
        command.execute()
    }
}
