use async_trait::async_trait;
use tracing::trace;

use crate::{
    error::Error,
    handler::Handler,
    request::dev::DevRequest,
    response::{dev::DevResponse, Response},
    Config,
};

#[async_trait]
impl Handler for DevRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, _config: Config) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        // Build enclave

        // In separate process, start enclave

        // Build contract

        // Deploy contract

        // Run handshake

        // Check for existing listening process
        // Spawn if doesn't exist

        Ok(DevResponse.into())
    }
}
