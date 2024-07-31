use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct  ContractBuildRequest {
    // TODO(hu55a1n1): remove `allow(unused)` here once init handler is implemented
    #[allow(unused)]
    pub directory: PathBuf,
}


impl From<ContractBuildRequest> for Request {
    fn from(request: ContractBuildRequest) -> Self {
        Self::ContractBuild(request)
    }
}