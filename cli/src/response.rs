use serde::Serialize;

use crate::response::{
    deploy::DeployResponse, handshake::HandshakeResponse, init::InitResponse,
    listen::ListenResponse,
};

pub mod deploy;
pub mod handshake;
pub mod init;
pub mod listen;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
    Listen(ListenResponse),
    Deploy(DeployResponse),
}
