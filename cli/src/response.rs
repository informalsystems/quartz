use serde::Serialize;

use crate::response::{
    contract_build::ContractBuildResponse, contract_deploy::ContractDeployResponse,
    enclave_build::EnclaveBuildResponse, handshake::HandshakeResponse, init::InitResponse,
};

pub mod contract_build;
pub mod contract_deploy;
pub mod enclave_build;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
    ContractBuild(ContractBuildResponse),
    ContractDeploy(ContractDeployResponse),
    EnclaveBuild(EnclaveBuildResponse),
}
