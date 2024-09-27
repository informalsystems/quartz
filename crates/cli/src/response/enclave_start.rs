use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct EnclaveStartResponse;

impl From<EnclaveStartResponse> for Response {
    fn from(response: EnclaveStartResponse) -> Self {
        Self::EnclaveStart(response)
    }
}
