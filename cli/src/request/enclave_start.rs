use quartz_common::enclave::types::Fmspc;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub use_latest_trusted: bool,
    pub fmspc: Option<Fmspc>,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}
