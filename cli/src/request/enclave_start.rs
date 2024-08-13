use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub app_dir: PathBuf,
    pub chain_id: String,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}
