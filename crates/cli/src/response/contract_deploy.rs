use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct ContractDeployResponse {
    pub code_id: u64,
    pub contract_addr: String,
}

impl From<ContractDeployResponse> for Response {
    fn from(response: ContractDeployResponse) -> Self {
        Self::ContractDeploy(response)
    }
}
