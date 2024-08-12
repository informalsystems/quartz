use serde::Serialize;

use crate::response::{
    contract_build::ContractBuildResponse, contract_deploy::ContractDeployResponse,
    contract_tx::ContractTxResponse, enclave_build::EnclaveBuildResponse,
    enclave_start::EnclaveStartResponse, handshake::HandshakeResponse, init::InitResponse,
};

pub mod contract_build;
pub mod contract_deploy;
pub mod contract_tx;
pub mod enclave_build;
pub mod enclave_start;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
    ContractBuild(ContractBuildResponse),
    ContractDeploy(ContractDeployResponse),
    ContractTx(ContractTxResponse),
    EnclaveBuild(EnclaveBuildResponse),
    EnclaveStart(EnclaveStartResponse),
}
