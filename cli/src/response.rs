use serde::Serialize;

use crate::response::{
    deploy::DeployResponse, handshake::HandshakeResponse, init::InitResponse,
};

pub mod deploy;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
    Deploy(DeployResponse),
}
