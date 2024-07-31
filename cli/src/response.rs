use serde::Serialize;

use crate::response::{
    contract_deploy::ContractDeployResponse, handshake::HandshakeResponse, init::InitResponse,
};

pub mod contract_deploy;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
    ContractDeploy(ContractDeployResponse),
}
