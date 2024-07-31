use serde::Serialize;

use crate::response::{init::InitResponse, contract_build::ContractBuildResponse};

pub mod init;
pub mod contract_build;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    ContractBuild(ContractBuildResponse)
}
