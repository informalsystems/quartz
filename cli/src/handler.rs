use async_trait::async_trait;

use crate::{cli::Verbosity, error::Error, request::Request, response::Response};

pub mod utils;
// commands
pub mod contract_deploy;
pub mod handshake;
pub mod init;

#[async_trait]
pub trait Handler {
    type Error;
    type Response;

    async fn handle(self, verbosity: Verbosity) -> Result<Self::Response, Self::Error>;
}

#[async_trait]
impl Handler for Request {
    type Error = Error;
    type Response = Response;

    async fn handle(self, verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        match self {
            Request::Init(request) => request.handle(verbosity).await,
            Request::Handshake(request) => request.handle(verbosity).await,
            Request::ContractDeploy(request) => request.handle(verbosity).await,
        }
        .map(Into::into)
    }
}
