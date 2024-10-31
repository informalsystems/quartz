use std::path::PathBuf;

use cosmrs::AccountId;
use quartz_common::enclave::types::Fmspc;

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct DevRequest {
    pub watch: bool,
    pub unsafe_trust_latest: bool,
    pub init_msg: serde_json::Value,
    pub label: String,
    pub contract_manifest: PathBuf,
    pub release: bool,
}

impl From<DevRequest> for Request {
    fn from(request: DevRequest) -> Self {
        Self::Dev(request)
    }
}
