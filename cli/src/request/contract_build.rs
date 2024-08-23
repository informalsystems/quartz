use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct ContractBuildRequest {
    pub contract_manifest: PathBuf,
}

impl From<ContractBuildRequest> for Request {
    fn from(request: ContractBuildRequest) -> Self {
        Self::ContractBuild(request)
    }
}
