use common::filesystem::file_manager::FileManager;

use crate::base::cli_error::CliError;
use crate::workspace::workspace::Workspace;

pub struct WorkspaceValidator;

impl WorkspaceValidator {
    pub fn validate(workspace: &Workspace) -> Result<(), CliError> {
        if FileManager::exists(workspace.root()) {
            return Err(CliError::InvalidArguments(format!(
                "Workspace '{}' already exists.",
                workspace.root().display()
            )));
        }
        Ok(())
    }
}
