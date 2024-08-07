use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct DevRequest {
    pub watch_contract: bool,
    pub app_dir: PathBuf,
}

impl From<DevRequest> for Request {
    fn from(request: DevRequest) -> Self {
        Self::Dev(request)
    }
}
