use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub manifest_path: PathBuf,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}
