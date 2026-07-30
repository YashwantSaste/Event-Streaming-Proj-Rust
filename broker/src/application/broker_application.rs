use common::error::broker_error::BrokerError;
use common::filesystem::local_file_system::LocalFileSystem;

use crate::application::broker_configuration::BrokerConfiguration;
use crate::application::broker_runtime::BrokerRuntime;
use crate::network::broker_server::BrokerServer;
use crate::network::server_configuration::BrokerServerConfiguration;
use crate::partition::partition_manager::PartitionManager;
use crate::storage::local_storage_engine::{LocalStorageConfiguration, LocalStorageEngine};
use crate::topic::topic_manager::TopicManager;
use crate::topic::topic_metadata_store::TopicMetadataStore;

pub struct BrokerApplication {
    configuration: BrokerConfiguration,
}

impl BrokerApplication {
    pub fn new(configuration: BrokerConfiguration) -> Self {
        Self { configuration }
    }

    pub async fn run(&self) -> Result<(), BrokerError> {
        let topic_store = TopicMetadataStore::new(
            LocalFileSystem::new(),
            self.configuration.topic_metadata_directory().to_path_buf(),
        );
        let topic_manager = TopicManager::new(topic_store);

        let storage_configuration = LocalStorageConfiguration::new(
            self.configuration.storage_root_directory().to_path_buf(),
            self.configuration.segment_max_bytes(),
        )
        .map_err(|error| BrokerError::new(error.to_string()))?;
        let storage_engine = LocalStorageEngine::new(LocalFileSystem::new(), storage_configuration);
        let partition_manager = PartitionManager::new(storage_engine);

        let mut runtime = BrokerRuntime::new(topic_manager, partition_manager);
        runtime.recover()?;

        let server_configuration = BrokerServerConfiguration::new(
            self.configuration.bind_address(),
            self.configuration.max_frame_bytes(),
        )
        .map_err(|error| BrokerError::new(error.to_string()))?;
        let server = BrokerServer::new(server_configuration, runtime);
        server
            .run()
            .await
            .map_err(|error| BrokerError::new(error.to_string()))
    }
}
