use std::path::PathBuf;

use cosmrs::{tendermint::chain::Id as ChainId, AccountId};

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct HandshakeRequest {
    pub contract: AccountId,
    pub port: u16,
    pub sender: String,
    pub chain_id: ChainId,
    pub node_url: String,
    pub rpc_addr: String,
    pub path: PathBuf,
}

impl From<HandshakeRequest> for Request {
    fn from(request: HandshakeRequest) -> Self {
        Self::Handshake(request)
    }
}
