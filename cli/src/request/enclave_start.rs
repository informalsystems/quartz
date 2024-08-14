use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub use_latest_trusted: bool,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}
