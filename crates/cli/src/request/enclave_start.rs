use std::path::PathBuf;

use cosmrs::AccountId;
use quartz_common::enclave::types::Fmspc;
use tendermint::{block::Height, Hash};
use tracing::debug;

use crate::{
    config::Config, error::Error, handler::utils::helpers::query_latest_height_hash,
    request::Request,
};

#[derive(Clone, Debug)]
pub struct EnclaveStartRequest {
    pub unsafe_trust_latest: bool,
    pub sk_file: PathBuf,
    pub fmspc: Option<Fmspc>,
    pub tcbinfo_contract: Option<AccountId>,
    pub dcap_verifier_contract: Option<AccountId>,
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
            let (trusted_height, trusted_hash) = query_latest_height_hash(config.node_url.clone())?;

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
