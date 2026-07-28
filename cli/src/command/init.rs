use std::error::Error;
use std::path::PathBuf;
use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;

pub struct InitCommand {

    pub workspace_name : String,
    pub path: Option<PathBuf>
}

impl BaseCommand for InitCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        todo!()
    }

    fn create_executable_cmd(parsed_command: ParsedCommand, ) -> Result<Self, CliError> where Self: Sized,
    {
        let workspace_name = parsed_command
            .argument(0)
            .ok_or(CliError::InvalidArguments(
                "Workspace name must be provided".to_string(),
            ))?
            .to_string();

        let path = parsed_command.option("path").map(PathBuf::from);

        Ok(InitCommand {
            workspace_name,
            path,
        })
    }
}