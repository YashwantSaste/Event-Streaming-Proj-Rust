use std::error::Error;
use std::path::PathBuf;
use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;
use crate::templates::workspace_template_generator::WorkspaceTemplateGenerator;
use crate::workspace::initializer::WorkspaceInitializer;

pub struct InitCommand {
    pub workspace_name : String,
    pub path: Option<PathBuf>
}

impl BaseCommand for InitCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let template_generator = WorkspaceTemplateGenerator::new(vec![]);
        let workspace_initializer = WorkspaceInitializer::new(self.workspace_name.clone(), self.path.clone(), template_generator);
        return workspace_initializer.initialize();
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