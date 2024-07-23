use tracing::trace;

use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::init::InitRequest,
    response::init::InitResponse,
};

impl Handler for InitRequest {
    type Error = Error;
    type Response = InitResponse;

    fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");
        todo!()
    }
}
