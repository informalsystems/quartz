use crate::{cli::Verbosity, error::Error, request::Request, response::Response};

pub mod enclave_build;
pub mod init;

pub trait Handler {
    type Error;
    type Response;

    fn handle(self, verbosity: Verbosity, mock_sgx: bool) -> Result<Self::Response, Self::Error>;
}

impl Handler for Request {
    type Error = Error;
    type Response = Response;

    fn handle(self, verbosity: Verbosity, mock_sgx: bool) -> Result<Self::Response, Self::Error> {
        match self {
            Request::Init(request) => request.handle(verbosity, mock_sgx),
            Request::EnclaveBuild(request) => request.handle(verbosity, mock_sgx),
        }
        .map(Into::into)
    }
}
