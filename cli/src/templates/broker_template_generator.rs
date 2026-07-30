use common::filesystem::file_manager::FileManager;
use crate::base::cli_error::CliError;
use crate::templates::template_generator::TemplateGenerator;
use crate::workspace::workspace::Workspace;
use common::constants::constants::*;


pub struct BrokerTemplateGenerator;

impl TemplateGenerator for BrokerTemplateGenerator {

    fn generate(
        &self,
        workspace: &Workspace,
    ) -> Result<(), CliError> {

        let file = workspace
            .root()
            .join(CONFIG)
            .join(BROKER_CONFIG_FILE);

        FileManager::write_file(file, include_str!("../../"),
        )
    }

}