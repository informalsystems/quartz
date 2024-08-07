use serde::Serialize;

use crate::response::{
    contract_deploy::ContractDeployResponse, dev::DevResponse, enclave_build::EnclaveBuildResponse,
    handshake::HandshakeResponse, init::InitResponse,
};

pub mod contract_deploy;
pub mod dev;
pub mod enclave_build;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
    ContractDeploy(ContractDeployResponse),
    EnclaveBuild(EnclaveBuildResponse),
    Dev(DevResponse),
}
