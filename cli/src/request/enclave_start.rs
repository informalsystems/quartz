use std::path::PathBuf;

use tokio::sync::oneshot;

use crate::request::Request;

#[derive(Debug)]
pub struct EnclaveStartRequest {
    pub app_dir: PathBuf,
    pub chain_id: String,
    pub ready_signal: Option<oneshot::Sender<()>>,
    pub node_url: String,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}

impl Clone for EnclaveStartRequest {
    fn clone(&self) -> Self {
        // Create a new oneshot channel
        let (new_sender, _new_receiver) = oneshot::channel();
        
        EnclaveStartRequest {
            app_dir: self.app_dir.clone(),
            chain_id: self.chain_id.clone(),
            ready_signal: Some(new_sender), // Assign the new Sender to the clone
            node_url: self.node_url.clone()
        }
    }
}
