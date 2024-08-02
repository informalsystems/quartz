
use crate::request::Request;

#[derive(Clone, Debug)]
pub struct InitRequest {
    pub name: String,
}

impl From<InitRequest> for Request {
    fn from(request: InitRequest) -> Self {
        Self::Init(request)
    }
}
