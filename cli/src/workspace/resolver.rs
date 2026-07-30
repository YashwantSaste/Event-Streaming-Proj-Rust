use std::path::PathBuf;

use crate::base::cli_error::CliError;
use crate::workspace::workspace::Workspace;

pub struct WorkspaceResolver;

impl WorkspaceResolver {
    pub fn resolve(workspace_name: &str, path: Option<&PathBuf>) -> Result<Workspace, CliError> {
        let root = match path {
            Some(path) => path.join(workspace_name),
            None => {
                let home = dirs::home_dir().ok_or(CliError::UnexpectedError(
                    "Unable to determine user home directory".into(),
                ))?;
                home.join(workspace_name)
            }
        };
        Ok(Workspace::new(root))
    }
}
