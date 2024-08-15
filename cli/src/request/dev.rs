use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct DevRequest {
    pub watch: bool,
    pub with_contract: bool,
}

impl From<DevRequest> for Request {
    fn from(request: DevRequest) -> Self {
        Self::Dev(request)
    }
}
