use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct ContractDeployResponse;

impl From<ContractDeployResponse> for Response {
    fn from(response: ContractDeployResponse) -> Self {
        Self::ContractDeploy(response)
    }
}
