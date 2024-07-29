use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct DeployResponse;

impl From<DeployResponse> for Response {
    fn from(response: DeployResponse) -> Self {
        Self::Deploy(response)
    }
}
