use serde::Serialize;

use crate::response::{contract_build::ContractBuildResponse, init::InitResponse};

pub mod contract_build;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    ContractBuild(ContractBuildResponse),
}
