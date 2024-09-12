use tendermint::{block::Height, Hash};
use tokio::sync::watch;
use tracing::debug;

use crate::{
    config::Config, error::Error, handler::utils::helpers::query_latest_height_hash,
    request::Request,
};

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub shutdown_rx: Option<watch::Receiver<()>>,
    pub unsafe_trust_latest: bool,
}

impl From<EnclaveStartRequest> for Request {
    fn from(request: EnclaveStartRequest) -> Self {
        Self::EnclaveStart(request)
    }
}

impl EnclaveStartRequest {
    /// Returns the trusted hash and height
    pub fn get_hash_height(&self, config: &Config) -> Result<(Height, Hash), Error> {
        if self.unsafe_trust_latest || config.trusted_height == 0 || config.trusted_hash.is_empty()
        {
            debug!("querying latest trusted hash & height from node");
            let (trusted_height, trusted_hash) = query_latest_height_hash(&config.node_url)?;

            Ok((trusted_height, trusted_hash))
        } else {
            debug!("reusing config trusted hash & height");
            Ok((
                config.trusted_height.try_into()?,
                config
                    .trusted_hash
                    .parse()
                    .map_err(|_| Error::GenericErr("invalid hash".to_string()))?,
            ))
        }
    }
}
