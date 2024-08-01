use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct  EnclaveBuildRequest {
    pub manifest_path: PathBuf,
}


impl From<EnclaveBuildRequest> for Request {
    fn from(request: EnclaveBuildRequest) -> Self {
        Self::EnclaveBuild(request)
    }
}