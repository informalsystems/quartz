use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct InitResponse {
    pub result_dir: String
}

impl From<InitResponse> for Response {
    fn from(response: InitResponse) -> Self {
        Self::Init(response)
    }
}
