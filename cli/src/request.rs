use crate::{cli::Command, error::Error, request::{init::InitRequest, handshake::HandshakeRequest}};

pub mod init;
pub mod handshake;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest), 
    Handshake(HandshakeRequest),   
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { path } => InitRequest::try_from(path).map(Into::into),
            Command::Handshake { contract, port, sender, chain_id, node_url, rpc_addr } => Ok(Request::Handshake(HandshakeRequest {contract, port, sender, chain_id, node_url, rpc_addr}))
        }
    }
}
