use std::{collections::HashMap, path::PathBuf};

use color_eyre::{eyre::Context, Result};
use serde::{Deserialize, Serialize};

use crate::request::Request;

#[derive(Clone, Debug)]
pub struct ContractDeployRequest {
    pub init_msg: serde_json::Value,
    pub label: String,
    pub admin: Option<String>,
    pub contract_manifest: PathBuf,
}

impl From<ContractDeployRequest> for Request {
    fn from(request: ContractDeployRequest) -> Self {
        Self::ContractDeploy(request)
    }
}

impl ContractDeployRequest {
    pub fn checked_init(init_msg: String) -> Result<GenericQuartzInit> {
        let parsed: GenericQuartzInit = serde_json::from_str(&init_msg)
            .wrap_err("Init message doesn't contain mandatory quartz field")?;

        Ok(parsed)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GenericQuartzInit {
    pub quartz: serde_json::Value,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
