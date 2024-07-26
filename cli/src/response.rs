use serde::Serialize;

use crate::response::{init::InitResponse, handshake::HandshakeResponse};

pub mod init;
pub mod handshake;

#[derive(Clone, Debug, Serialize)]
pub enum Response {
    Init(InitResponse),
    Handshake(HandshakeResponse),
}
