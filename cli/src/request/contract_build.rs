use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct  ContractBuildRequest {
    pub manifest_path: PathBuf,
}


impl From<ContractBuildRequest> for Request {
    fn from(request: ContractBuildRequest) -> Self {
        Self::ContractBuild(request)
    }
}