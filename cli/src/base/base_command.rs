use std::error::Error;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;

pub trait BaseCommand {
    fn execute(&self) -> Result<CommandResult, CliError>;
}