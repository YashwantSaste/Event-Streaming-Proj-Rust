use std::fs;
use common::filesystem::file_manager::FileManager;
use common::constants::constants::*;
use crate::base::cli_error::CliError;
use crate::workspace::workspace::Workspace;


pub struct Directory;

impl Directory {

    pub fn create(workspace: &Workspace) -> Result<(),CliError>{
        let root = workspace.root();

        let directories = [
            root.to_path_buf(),
            root.join(CONFIG),
            root.join(DATA),
            root.join(DATA).join(BROKER_DATA),
            root.join(DATA).join(TOPICS),
            root.join(LOGS),
            root.join(PLUGINS),
            root.join(TEMP),
        ];

        let result = FileManager::create_directories(directories);
        result.map_err(|e| CliError::UnsupportedCommand(e.to_string()))
    }
}