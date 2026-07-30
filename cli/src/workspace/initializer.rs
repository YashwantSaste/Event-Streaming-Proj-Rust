use std::path::PathBuf;

use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::templates::workspace_template_generator::WorkspaceTemplateGenerator;
use crate::workspace::directory::Directory;
use super::resolver::WorkspaceResolver;
use super::validator::WorkspaceValidator;

pub struct WorkspaceInitializer {

    workspace_name: String,

    path: Option<PathBuf>,

    template_generator: WorkspaceTemplateGenerator,


}

impl WorkspaceInitializer {

    pub fn new(workspace_name: String,path: Option<PathBuf>,template_generator: WorkspaceTemplateGenerator) -> Self {
        Self {workspace_name, path,template_generator}
    }

    pub fn initialize(&self,) -> Result<CommandResult, CliError> {
        let workspace = WorkspaceResolver::resolve(&self.workspace_name,self.path.as_ref())?;
        WorkspaceValidator::validate(&workspace)?;
        Directory::create(&workspace)?;
        Ok(
            CommandResult{
                success:true,
                exit_code: 0,
                message: format!(
                "Workspace initialized at {}", workspace.root().display()
                ),
            })

    }

}