use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}
