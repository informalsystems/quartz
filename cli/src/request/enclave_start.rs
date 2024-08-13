use std::path::PathBuf;

use tokio::sync::watch;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub app_dir: PathBuf,
    pub chain_id: String,
    pub node_url: String,
    pub shutdown_rx: watch::Receiver<()>,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}
