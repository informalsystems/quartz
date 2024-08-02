use serde::Serialize;

use crate::response::{contract_build::ContractBuildResponse, enclave_build::EnclaveBuildResponse, init::InitResponse};

pub mod contract_build;
pub mod enclave_build;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    ContractBuild(ContractBuildResponse),
    EnclaveBuild(EnclaveBuildResponse),
}
