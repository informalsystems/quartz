use quartz_common::proto::{
    core_client::CoreClient, InstantiateRequest, SessionCreateRequest, SessionSetPubKeyRequest,
};

use crate::error::Error;

#[derive(Debug)]
pub enum RelayMessage {
    Instantiate,
    SessionCreate,
    SessionSetPubKey(String),
}

impl RelayMessage {
    pub async fn run_relay(
        &self,
        enclave_rpc: String,
        _mock_sgx: bool,
    ) -> Result<serde_json::Value, Error> {
        // Query the gRPC quartz enclave service
        let mut qc_client = CoreClient::connect(enclave_rpc)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        let attested_msg = match self {
            RelayMessage::Instantiate => qc_client
                .instantiate(tonic::Request::new(InstantiateRequest {}))
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
                .get_ref()
                .message
                .clone(),
            RelayMessage::SessionCreate => qc_client
                .session_create(tonic::Request::new(SessionCreateRequest {}))
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
                .get_ref()
                .message
                .clone(),
            RelayMessage::SessionSetPubKey(proof) => qc_client
                .session_set_pub_key(SessionSetPubKeyRequest {
                    message: proof.to_string(),
                })
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
                .get_ref()
                .message
                .clone(),
        };

        serde_json::from_str(&attested_msg).map_err(Into::into)
    }
}
