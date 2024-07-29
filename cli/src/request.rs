use std::{env::current_dir, path::PathBuf};

use crate::{
    cli::Command,
    error::Error,
    request::{handshake::HandshakeRequest, init::InitRequest, listen::ListenRequest},
};

pub mod handshake;
pub mod init;
pub mod listen;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
    Handshake(HandshakeRequest),
    Listen(ListenRequest),
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { path } => InitRequest::try_from(path).map(Into::into),
            Command::Handshake {
                contract,
                port,
                sender,
                chain_id,
                node_url,
                rpc_addr,
                path
            } => Ok(Request::Handshake(HandshakeRequest {
                contract,
                port,
                sender,
                chain_id,
                node_url,
                rpc_addr,
                path: Self::path_checked(path)?
            })),
            Command::Listen {
                contract,
                port,
                sender,
                chain_id,
                node_url,
                rpc_addr,
                path
            } => Ok(Request::Listen(ListenRequest {
                contract,
                port,
                sender,
                chain_id,
                node_url,
                rpc_addr,
                path: Self::path_checked(path)?
            })),
        }
    }
}

impl Request {
    fn path_checked(path: Option<PathBuf>) -> Result<PathBuf, Error> {
        if let Some(path) = path {
            if !path.is_dir() {
                return Err(Error::PathNotDir(format!("{}", path.display())));
            }

            Ok(path)
        } else {
            Ok(current_dir().map_err(|e| Error::GenericErr(e.to_string()))?)
        }
    }
}