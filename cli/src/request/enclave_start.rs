use std::path::PathBuf;

use tokio::sync::watch;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub shutdown_rx: Option<watch::Receiver<()>>,
    pub use_latest_trusted: bool,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}
