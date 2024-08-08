use serde::Serialize;

use crate::response::{
    contract_deploy::ContractDeployResponse, enclave_build::EnclaveBuildResponse,
    enclave_start::EnclaveStartResponse, handshake::HandshakeResponse, init::InitResponse,
};

pub mod contract_deploy;
pub mod enclave_build;
pub mod enclave_start;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
    ContractDeploy(ContractDeployResponse),
    EnclaveBuild(EnclaveBuildResponse),
    EnclaveStart(EnclaveStartResponse),
}
