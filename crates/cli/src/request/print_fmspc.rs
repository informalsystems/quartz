use reqwest::Url;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct PrintFmspcRequest {
    pub pccs_url: Option<Url>,
}

impl From<PrintFmspcRequest> for Request {
    fn from(request: PrintFmspcRequest) -> Self {
        Self::PrintFmspc(request)
    }
}
