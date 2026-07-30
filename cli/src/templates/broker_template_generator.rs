use common::constants::constants::*;
use common::filesystem::file_manager::FileManager;

use crate::base::cli_error::CliError;
use crate::templates::template_generator::TemplateGenerator;
use crate::workspace::workspace::Workspace;

/// Generates the default broker configuration file for a workspace.
pub struct BrokerTemplateGenerator;

impl TemplateGenerator for BrokerTemplateGenerator {
    fn generate(&self, workspace: &Workspace) -> Result<(), CliError> {
        let file = workspace.root().join(CONFIG).join(BROKER_CONFIG_FILE);

        FileManager::write_file(file, include_str!("../../../assets/broker.toml"))
            .map_err(|error| CliError::UnexpectedError(error.to_string()))
    }
}
