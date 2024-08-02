use tracing::trace;

use crate::{
    error::Error, handler::Handler, request::init::InitRequest, response::Response, Config,
};

impl Handler for InitRequest {
    type Error = Error;
    type Response = Response;

    fn handle(self, _config: Config) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");
        todo!()
    }
}
