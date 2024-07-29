use std::path::PathBuf;

use cosmrs::{tendermint::chain::Id as ChainId, AccountId};

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct ListenRequest {
    // TODO(hu55a1n1): remove `allow(unused)` here once init handler is implemented
    #[allow(unused)]
    pub contract: AccountId,
    pub port: u16,
    pub sender: String,
    pub chain_id: ChainId,
    pub node_url: String,
    pub rpc_addr: String,
    pub path: PathBuf,
}

impl From<ListenRequest> for Request {
    fn from(request: ListenRequest) -> Self {
        Self::Listen(request)
    }
}