use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct ContractBuildResponse;

impl From<ContractBuildResponse> for Response {
    fn from(response: ContractBuildResponse) -> Self {
        Self::ContractBuild(response)
    }
}