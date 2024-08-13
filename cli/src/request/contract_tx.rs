use cosmrs::{tendermint::chain::Id as ChainId, AccountId};

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct ContractTxRequest {
    pub node_url: String,
    pub contract: AccountId,
    pub chain_id: ChainId,
    pub gas: u64,
    pub sender: String,
    pub msg: String,
    pub amount: Option<String>,
}

impl From<ContractTxRequest> for Request {
    fn from(request: ContractTxRequest) -> Self {
        Self::ContractTx(request)
    }
}
