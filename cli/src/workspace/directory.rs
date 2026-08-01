use common::constants::constants::*;
use common::filesystem::file_manager::FileManager;

use crate::base::cli_error::CliError;
use crate::workspace::workspace::Workspace;

/// Creates the directory layout for a workspace.
pub struct Directory;

impl Directory {
    /// Creates all directories required by a broker workspace.
    pub fn create(workspace: &Workspace) -> Result<(), CliError> {
        let root = workspace.root();

        let directories = [
            root.to_path_buf(),
            root.join(CONFIG),
            root.join(DATA),
            root.join(DATA).join(BROKER_DATA),
            root.join(DATA).join(TOPICS),
            root.join(DATA).join(CONSUMER_GROUPS),
            root.join(LOGS),
            root.join(PLUGINS),
            root.join(TEMP),
        ];

        FileManager::create_directories(directories)
            .map_err(|error| CliError::UnexpectedError(error.to_string()))
    }
}
