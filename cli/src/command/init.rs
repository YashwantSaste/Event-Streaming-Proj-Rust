use std::error::Error;
use std::path::PathBuf;
use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;

pub struct InitCommand {

    pub workspace_name : String,
    pub path: Option<PathBuf>
}

impl BaseCommand for InitCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        todo!()
        // Setting up the workspace.....
    }
}
