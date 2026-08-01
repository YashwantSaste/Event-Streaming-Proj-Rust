use common::models::partition::PartitionConfiguration;
use common::models::partition::TopicPartition;
use common::protocol::request::{Request, RequestPayload};
use common::protocol::response::{
    CommitOffsetResponse, CreateTopicResponse, FetchResponse, ListTopicsResponse, ProduceResponse,
    Response, ResponsePayload,
};

use crate::consumer_group::consumer_group_manager::ConsumerGroupManager;
use crate::network::request_dispatcher::RequestDispatcher;
use crate::partition::partition_manager::PartitionManager;
use crate::storage::storage_engine::StorageEngine;
use crate::topic::topic_manager::TopicManager;

pub struct BrokerRuntime<F, S, G>
where
    F: common::filesystem::file_system::FileSystem,
    S: StorageEngine,
    G: common::filesystem::file_system::FileSystem,
{
    topic_manager: TopicManager<F>,
    partition_manager: PartitionManager<S>,
    consumer_group_manager: ConsumerGroupManager<G>,
}

impl<F, S, G> BrokerRuntime<F, S, G>
where
    F: common::filesystem::file_system::FileSystem,
    S: StorageEngine,
    G: common::filesystem::file_system::FileSystem,
{
    pub fn new(
        topic_manager: TopicManager<F>,
        partition_manager: PartitionManager<S>,
        consumer_group_manager: ConsumerGroupManager<G>,
    ) -> Self {
        Self {
            topic_manager,
            partition_manager,
            consumer_group_manager,
        }
    }

    pub fn recover(&mut self) -> Result<(), common::error::broker_error::BrokerError> {
        self.topic_manager.recover()?;
        self.partition_manager
            .recover(self.topic_manager.list_topics().as_slice())
    }
}

impl<F, S, G> RequestDispatcher for BrokerRuntime<F, S, G>
where
    F: common::filesystem::file_system::FileSystem + Send,
    S: StorageEngine + Send,
    G: common::filesystem::file_system::FileSystem + Send,
{
    fn dispatch(&mut self, request: Request) -> Response {
        let correlation_id = request.correlation_id();
        let request_type = request.request_type();
        match self.dispatch_inner(request) {
            Ok(payload) => Response::ok(correlation_id, request_type, payload),
            Err(error) => Response::error(correlation_id, request_type, error.to_string()),
        }
    }
}

impl<F, S, G> BrokerRuntime<F, S, G>
where
    F: common::filesystem::file_system::FileSystem,
    S: StorageEngine,
    G: common::filesystem::file_system::FileSystem,
{
    fn dispatch_inner(
        &mut self,
        request: Request,
    ) -> Result<ResponsePayload, common::error::broker_error::BrokerError> {
        match request.into_payload() {
            RequestPayload::Produce(payload) => {
                let metadata = self.partition_manager.append(
                    payload.topic(),
                    payload.partition_id(),
                    payload.key().cloned(),
                    payload.payload().clone(),
                )?;
                Ok(ResponsePayload::Produce(ProduceResponse::new(
                    metadata.partition().clone(),
                    metadata.offset(),
                )))
            }
            RequestPayload::Fetch(payload) => {
                let max_records = usize::try_from(payload.max_records()).map_err(|error| {
                    common::error::broker_error::BrokerError::new(format!(
                        "Invalid max_records value: {error}"
                    ))
                })?;
                let records = self.partition_manager.read(
                    payload.topic(),
                    payload.partition_id(),
                    payload.offset(),
                    max_records,
                )?;
                Ok(ResponsePayload::Fetch(FetchResponse::new(records)))
            }
            RequestPayload::CreateTopic(payload) => {
                let partition_config = PartitionConfiguration::new(payload.segment_max_bytes())
                    .map_err(|error| {
                        common::error::broker_error::BrokerError::new(error.to_string())
                    })?;
                let topic = self.topic_manager.create_topic(
                    payload.topic().clone(),
                    payload.partition_count(),
                    partition_config,
                )?;
                self.partition_manager.create_partitions_for_topic(&topic)?;
                Ok(ResponsePayload::CreateTopic(CreateTopicResponse::new(
                    topic.name().clone(),
                )))
            }
            RequestPayload::CommitOffset(payload) => {
                let partition =
                    TopicPartition::new(payload.topic().clone(), payload.partition_id());
                let offset = self.consumer_group_manager.commit_offset(
                    payload.group_id(),
                    partition,
                    payload.offset(),
                )?;
                Ok(ResponsePayload::CommitOffset(CommitOffsetResponse::new(
                    offset,
                )))
            }
            RequestPayload::ListTopics => Ok(ResponsePayload::ListTopics(ListTopicsResponse::new(
                self.topic_manager
                    .list_topics()
                    .into_iter()
                    .map(|topic| topic.name().clone())
                    .collect(),
            ))),
        }
    }
}
