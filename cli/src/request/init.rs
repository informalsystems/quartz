use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct InitRequest {
    pub name: String,
    pub directory: PathBuf,
}

impl From<InitRequest> for Request {
    fn from(request: InitRequest) -> Self {
        Self::Init(request)
    }
}
