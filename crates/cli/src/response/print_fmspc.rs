use quartz_common::enclave::types::Fmspc;
use serde::Serialize;

use crate::response::Response;

#[derive(Clone, Debug, Serialize)]
pub struct PrintFmspcResponse {
    pub fmspc: Fmspc,
}

impl From<PrintFmspcResponse> for Response {
    fn from(response: PrintFmspcResponse) -> Self {
        Self::PrintFmspc(response)
    }
}
