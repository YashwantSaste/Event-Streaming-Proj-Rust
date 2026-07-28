use crate::base::cli_error::CliError;
use crate::workspace::workspace::Workspace;

pub struct WorkspaceValidator;

impl WorkspaceValidator {

    pub fn validate(workspace: &Workspace) -> Result<(), CliError>
    {
        if workspace.root().exists() {
            return Err(CliError::InvalidArguments(
                    format!(
                        "Workspace '{}' already exists.",
                        workspace.root().display()
                    ),
                ),
            );
        }
        Ok(())
    }

}