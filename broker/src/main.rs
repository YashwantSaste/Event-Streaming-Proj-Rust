use std::path::PathBuf;

use broker::application::broker_application::BrokerApplication;
use broker::application::broker_configuration::BrokerConfiguration;
use common::configuration::configuration_reader::ConfigurationReader;
use common::configuration::toml_configuration_reader::TomlConfigurationReader;
use common::filesystem::local_file_system::LocalFileSystem;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = parse_config_path();
    let reader = TomlConfigurationReader::new(LocalFileSystem::new(), config_path);
    let configuration = reader.read()?;
    let broker_configuration = BrokerConfiguration::from_configuration(&configuration)?;
    let application = BrokerApplication::new(broker_configuration);
    application.run().await?;
    Ok(())
}

fn parse_config_path() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(argument) = args.next() {
        if argument == "--config" {
            if let Some(path) = args.next() {
                return PathBuf::from(path);
            }
        }
    }

    PathBuf::from("assets/broker.toml")
}
