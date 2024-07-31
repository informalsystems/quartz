use std::path::PathBuf;

use cosmrs::tendermint::chain::Id as ChainId;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct ContractDeployRequest {
    pub node_url: String,
    pub chain_id: ChainId,
    pub sender: String,
    pub label: String,
    pub directory: PathBuf,
}

impl From<ContractDeployRequest> for Request {
    fn from(request: ContractDeployRequest) -> Self {
        Self::ContractDeploy(request)
    }
}
