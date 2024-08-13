use std::{env::current_dir, path::PathBuf};

use tokio::sync::{watch};

use crate::{
    cli::{Command, ContractCommand, EnclaveCommand},
    error::Error,
    request::{
        contract_build::ContractBuildRequest, contract_deploy::ContractDeployRequest,
        dev::DevRequest, enclave_build::EnclaveBuildRequest, enclave_start::EnclaveStartRequest,
        handshake::HandshakeRequest, init::InitRequest,
    },
};

pub mod contract_build;
pub mod contract_deploy;
pub mod dev;
pub mod enclave_build;
pub mod enclave_start;
pub mod handshake;
pub mod init;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
    Handshake(HandshakeRequest),
    ContractBuild(ContractBuildRequest),
    ContractDeploy(ContractDeployRequest),
    EnclaveBuild(EnclaveBuildRequest),
    EnclaveStart(EnclaveStartRequest),
    Dev(DevRequest),
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { name } => Ok(InitRequest { name }.try_into()?),
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
            Command::Enclave { enclave_command } => enclave_command.try_into(),
            Command::Dev {
                watch,
                with_contract,
                app_dir,
                node_url,
            } => Ok(DevRequest {
                watch,
                with_contract,
                app_dir: Self::path_checked(app_dir)?,
                node_url,
            }
            .into()),
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
            ContractCommand::Build { manifest_path } => {
                if !manifest_path.exists() {
                    return Err(Error::PathNotFile(manifest_path.display().to_string()));
                }

                Ok(ContractBuildRequest { manifest_path }.into())
            }
        }
    }
}

impl TryFrom<EnclaveCommand> for Request {
    type Error = Error;

    fn try_from(cmd: EnclaveCommand) -> Result<Request, Error> {
        match cmd {
            EnclaveCommand::Build {
                release,
                manifest_path,
            } => Ok(EnclaveBuildRequest {
                release,
                manifest_path,
            }
            .into()),
            EnclaveCommand::Start {
                app_dir,
                chain_id,
                node_url,
            } => Ok(EnclaveStartRequest {
                app_dir: Self::path_checked(app_dir)?,
                chain_id,
                node_url,
                shutdown_rx: watch::channel(()).1,
            }
            .into()),
        }
    }
}
