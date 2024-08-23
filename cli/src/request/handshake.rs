use cosmrs::AccountId;
use tendermint::{block::Height, Hash};
use tracing::debug;

use crate::{
    config::Config, error::Error, handler::utils::helpers::read_cached_hash_height,
    request::Request,
};

#[derive(Clone, Debug)]
pub struct HandshakeRequest {
    pub contract: AccountId,
    pub use_latest_trusted: bool,
}

impl From<HandshakeRequest> for Request {
    fn from(request: HandshakeRequest) -> Self {
        Self::Handshake(request)
    }
}

impl HandshakeRequest {
    /// Returns the trusted hash and height
    pub async fn get_hash_height(&self, config: &Config) -> Result<(Height, Hash), Error> {
        if self.use_latest_trusted || config.trusted_height == 0 || config.trusted_hash.is_empty() {
            debug!("querying latest trusted hash & height from node");

            let res = read_cached_hash_height(config).await;
            if let Err(Error::PathNotFile(e)) = res {
                return Err(Error::GenericErr(format!(
                    "File not found error from reading cache: {}. Have you started the enclave?",
                    e
                )));
            }

            Ok(res?)
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
