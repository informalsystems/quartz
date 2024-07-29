use crate::{cli::Verbosity, error::Error, request::Request, response::Response};
use async_trait::async_trait;

pub mod utils;
// commands
pub mod handshake;
pub mod init;
pub mod listen;

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
            Request::Init(request) => request.handle(verbosity).await.map(Into::into),
            Request::Handshake(request) => request.handle(verbosity).await.map(Into::into),
            Request::Listen(request) => request.handle(verbosity).await.map(Into::into),
        }
    }
}
