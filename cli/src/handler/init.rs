use async_trait::async_trait;
use tracing::trace;

use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::init::InitRequest,
    response::{init::InitResponse, Response},
};

#[async_trait]
impl Handler for InitRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        Ok(InitResponse.into())
    }
}
