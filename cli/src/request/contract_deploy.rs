use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::{error::Error, request::Request};

#[derive(Clone, Debug)]
pub struct ContractDeployRequest {
    pub init_msg: serde_json::Value,
    pub label: String,
    pub wasm_bin_path: PathBuf,
}

impl From<ContractDeployRequest> for Request {
    fn from(request: ContractDeployRequest) -> Self {
        Self::ContractDeploy(request)
    }
}

impl ContractDeployRequest {
    pub fn checked_init(init_msg: String) -> Result<GenericQuartzInit, Error> {
        let parsed: GenericQuartzInit = serde_json::from_str(&init_msg).map_err(|_| {
            Error::GenericErr("Init message doesn't contain mandatory quartz field.".to_string())
        })?;

        Ok(parsed)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericQuartzInit {
    pub quartz: serde_json::Value,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
