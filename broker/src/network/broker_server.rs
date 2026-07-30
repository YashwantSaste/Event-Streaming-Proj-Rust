use std::sync::Arc;

use common::error::network_error::NetworkError;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

use crate::network::client_session::ClientSession;
use crate::network::request_dispatcher::RequestDispatcher;
use crate::network::server_configuration::BrokerServerConfiguration;

pub struct BrokerServer<D>
where
    D: RequestDispatcher + Send + 'static,
{
    configuration: BrokerServerConfiguration,
    dispatcher: Arc<Mutex<D>>,
}

impl<D> BrokerServer<D>
where
    D: RequestDispatcher + Send + 'static,
{
    pub fn new(configuration: BrokerServerConfiguration, dispatcher: D) -> Self {
        Self {
            configuration,
            dispatcher: Arc::new(Mutex::new(dispatcher)),
        }
    }

    pub async fn run(&self) -> Result<(), NetworkError> {
        let listener = TcpListener::bind(self.configuration.bind_address())
            .await
            .map_err(|error| NetworkError::new(format!("Failed to bind broker server: {error}")))?;

        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|error| NetworkError::new(format!("Failed to accept client: {error}")))?;

            let dispatcher = Arc::clone(&self.dispatcher);
            let max_frame_bytes = self.configuration.max_frame_bytes();

            tokio::spawn(async move {
                match ClientSession::new(stream, max_frame_bytes, dispatcher) {
                    Ok(mut session) => {
                        if let Err(error) = session.run().await {
                            eprintln!("Client session ended with error: {error}");
                        }
                    }
                    Err(error) => eprintln!("Failed to create client session: {error}"),
                }
            });
        }
    }
}
