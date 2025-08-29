use std::{collections::BTreeMap, path::PathBuf};

use cosmwasm_std::{Addr, Uint128};
use quartz_common::enclave::{
    backup_restore::{Export, Import},
    DefaultSharedEnclave,
};
use serde::{Deserialize, Serialize};

pub type AppEnclave = DefaultSharedEnclave<AppCtx>;

#[derive(Clone, Debug, PartialEq, Eq, Default, Deserialize, Serialize)]
pub struct State {
    pub state: BTreeMap<Addr, Uint128>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct Balance {
    pub balance: Uint128,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct AppCtx {
    pub backup_path: PathBuf,
}

#[async_trait::async_trait]
impl Import for AppCtx {
    type Error = serde_json::Error;

    async fn import(data: Vec<u8>) -> Result<Self, Self::Error> {
        serde_json::from_slice(&data)
    }
}

#[async_trait::async_trait]
impl Export for AppCtx {
    type Error = serde_json::Error;

    async fn export(&self) -> Result<Vec<u8>, Self::Error> {
        serde_json::to_vec(self)
    }
}
