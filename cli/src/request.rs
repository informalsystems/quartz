use std::{env::current_dir, path::PathBuf};

use crate::{
    cli::{Command, ContractCommand, EnclaveCommand},
    error::Error,
    request::{
        contract_build::ContractBuildRequest, contract_deploy::ContractDeployRequest,
        enclave_build::EnclaveBuildRequest, enclave_start::EnclaveStartRequest,
        handshake::HandshakeRequest, init::InitRequest,
    },
};

pub mod contract_build;
pub mod contract_deploy;
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
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { name } => Ok(InitRequest { name }.try_into()?),
            Command::Handshake { contract} => Ok(HandshakeRequest { contract }.into()),
            Command::Contract { contract_command } => contract_command.try_into(),
            Command::Enclave { enclave_command } => enclave_command.try_into(),
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
                label,
                wasm_bin_path,
            } => {
                if !wasm_bin_path.exists() {
                    return Err(Error::PathNotFile(wasm_bin_path.display().to_string()));
                }

                Ok(ContractDeployRequest {
                    init_msg: serde_json::from_str(&init_msg)
                        .map_err(|e| Error::GenericErr(e.to_string()))?,
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
            EnclaveCommand::Build { manifest_path } => {
                Ok(EnclaveBuildRequest { manifest_path }.into())
            }
            EnclaveCommand::Start { } => Ok(EnclaveStartRequest {}.into()),
        }
    }
}
