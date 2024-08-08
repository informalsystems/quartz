use async_trait::async_trait;
use tracing::trace;

use crate::{
    error::Error,
    handler::Handler,
    request::init::InitRequest,
    response::{init::InitResponse, Response},
    Config,
};

#[async_trait]
impl Handler for InitRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, _config: Config) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        Ok(InitResponse.into())
    }
}
