use std::{env::current_dir, path::PathBuf};

use crate::{
    cli::{Command, ContractCommand},
    error::Error,
    request::{
        contract_deploy::ContractDeployRequest, handshake::HandshakeRequest, init::InitRequest,
    },
};

pub mod contract_deploy;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
    Handshake(HandshakeRequest),
    ContractDeploy(ContractDeployRequest),
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
                path,
            } => Ok(Request::Handshake(HandshakeRequest {
                contract,
                port,
                sender,
                chain_id,
                node_url,
                rpc_addr,
                path: Self::path_checked(path)?,
            })),
            Command::Contract { contract_command } => match contract_command {
                ContractCommand::Deploy {
                    init_msg,
                    node_url,
                    chain_id,
                    sender,
                    label,
                    path,
                } => Ok(Request::ContractDeploy(ContractDeployRequest {
                    init_msg: ContractDeployRequest::checked_init(init_msg)?,
                    node_url,
                    chain_id,
                    sender,
                    label,
                    directory: Self::path_checked(path)?,
                })),
                _ => todo!(),
            },
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
