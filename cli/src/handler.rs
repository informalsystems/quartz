use crate::{cli::Verbosity, error::Error, request::Request, response::Response};

pub mod init;
pub mod enclave_build;

pub trait Handler {
    type Error;
    type Response;

    fn handle(self, verbosity: Verbosity) -> Result<Self::Response, Self::Error>;
}

impl Handler for Request {
    type Error = Error;
    type Response = Response;

    fn handle(self, verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        match self {
            Request::Init(request) => request.handle(verbosity),
            Request::EnclaveBuild(request) => request.handle(verbosity),
        }
        .map(Into::into)
    }
}
