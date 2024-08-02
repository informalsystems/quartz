use crate::{error::Error, request::Request, response::Response, Config};

pub mod contract_build;
pub mod init;

pub trait Handler {
    type Error;
    type Response;

    fn handle(self, config: Config) -> Result<Self::Response, Self::Error>;
}

impl Handler for Request {
    type Error = Error;
    type Response = Response;

    fn handle(self, config: Config) -> Result<Self::Response, Self::Error> {
        match self {
            Request::Init(request) => request.handle(config),
            Request::ContractBuild(request) => request.handle(config),
        }
        .map(Into::into)
    }
}
