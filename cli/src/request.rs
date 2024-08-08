use std::{env::current_dir, path::PathBuf};

use crate::{
    cli::{Command, ContractCommand, EnclaveCommand},
    error::Error,
    request::{
        contract_deploy::ContractDeployRequest, enclave_build::EnclaveBuildRequest,
        handshake::HandshakeRequest, init::InitRequest,
    },
};

pub mod contract_deploy;
pub mod enclave_build;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
    Handshake(HandshakeRequest),
    ContractDeploy(ContractDeployRequest),
    EnclaveBuild(EnclaveBuildRequest),
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { name } => Ok(Request::Init(InitRequest { name })),
            Command::Handshake {
                contract,
                port,
                sender,
                chain_id,
                node_url,
                enclave_rpc_addr,
                app_dir,
            } => Ok(HandshakeRequest {
                contract,
                port,
                sender,
                chain_id,
                node_url,
                enclave_rpc_addr,
                app_dir: Self::path_checked(app_dir)?,
            }
            .into()),
            Command::Contract { contract_command } => contract_command.try_into(),
            Command::Enclave { enclave_command } => Ok(enclave_command.into()),
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

impl TryFrom<ContractCommand> for Request {
    type Error = Error;

    fn try_from(cmd: ContractCommand) -> Result<Request, Error> {
        match cmd {
            ContractCommand::Deploy {
                init_msg,
                node_url,
                chain_id,
                sender,
                label,
                wasm_bin_path,
            } => {
                if !wasm_bin_path.exists() {
                    return Err(Error::PathNotFile(wasm_bin_path.display().to_string()));
                }

                Ok(ContractDeployRequest {
                    init_msg: serde_json::from_str(&init_msg)
                        .map_err(|e| Error::GenericErr(e.to_string()))?,
                    node_url,
                    chain_id,
                    sender,
                    label,
                    wasm_bin_path,
                }
                .into())
            }
            ContractCommand::Build { path: _ } => todo!(),
        }
    }
}

impl From<EnclaveCommand> for Request {
    fn from(cmd: EnclaveCommand) -> Request {
        match cmd {
            EnclaveCommand::Build { manifest_path } => EnclaveBuildRequest { manifest_path }.into(),
            EnclaveCommand::Start { path: _ } => todo!(),
        }
    }
}
