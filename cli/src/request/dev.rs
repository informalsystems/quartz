use std::path::PathBuf;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct DevRequest {
    pub watch: bool,
    pub use_latest_trusted: bool,
    pub init_msg: serde_json::Value,
    pub label: String,
    pub wasm_bin_path: PathBuf,
    pub release: bool,
    pub manifest_path: PathBuf,
}

impl From<DevRequest> for Request {
    fn from(request: DevRequest) -> Self {
        Self::Dev(request)
    }
}
