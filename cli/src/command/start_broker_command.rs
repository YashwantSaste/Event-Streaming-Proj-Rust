use std::path::PathBuf;

use broker::application::broker_application::BrokerApplication;
use broker::application::broker_configuration::BrokerConfiguration;
use common::configuration::configuration_reader::ConfigurationReader;
use common::configuration::toml_configuration_reader::TomlConfigurationReader;
use common::filesystem::local_file_system::LocalFileSystem;

use crate::base::base_command::BaseCommand;
use crate::base::cli_error::CliError;
use crate::base::command_result::CommandResult;
use crate::base::parsed_command::ParsedCommand;
use crate::command::command_utils::{broker_config_path, runtime, success, workspace_root};

pub struct StartBrokerCommand {
    config_path: PathBuf,
    workspace_root: PathBuf,
}

impl BaseCommand for StartBrokerCommand {
    fn execute(&self) -> Result<CommandResult, CliError> {
        let config_path = self.config_path.clone();
        let workspace_root = self.workspace_root.clone();
        runtime()?.block_on(async move {
            let reader = TomlConfigurationReader::new(LocalFileSystem::new(), config_path.clone());
            let configuration = reader
                .read()
                .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
            let broker_configuration = BrokerConfiguration::from_configuration_with_base_dir(
                &configuration,
                &workspace_root,
            )
            .map_err(|error| CliError::UnexpectedError(error.to_string()))?;
            BrokerApplication::new(broker_configuration)
                .run()
                .await
                .map_err(|error| CliError::UnexpectedError(error.to_string()))
        })?;

        Ok(success("Broker stopped"))
    }

    fn create_executable_cmd(parsed_command: ParsedCommand) -> Result<Self, CliError> {
        Ok(Self {
            config_path: broker_config_path(&parsed_command),
            workspace_root: workspace_root(&parsed_command),
        })
    }
}
