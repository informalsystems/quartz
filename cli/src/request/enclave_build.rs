use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct  EnclaveBuildRequest {
    // TODO(hu55a1n1): remove `allow(unused)` here once init handler is implemented
    #[allow(unused)]
    pub directory: PathBuf,
}


impl From<EnclaveBuildRequest> for Request {
    fn from(request: EnclaveBuildRequest) -> Self {
        Self::EnclaveBuild(request)
    }
}