use crate::{
    cli::{Command, ContractCommand},
    error::Error,
    request::{contract_build::ContractBuildRequest, enclave_build::EnclaveBuildRequest, init::InitRequest},
};

pub mod contract_build;
pub mod enclave_build;
pub mod init;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
    ContractBuild(ContractBuildRequest),
    EnclaveBuild(EnclaveBuildRequest),
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { path } => InitRequest::try_from(path).map(Into::into),
            Command::Contract { contract_command } => match contract_command {
                ContractCommand::Build { manifest_path } => {
                    Ok(ContractBuildRequest { manifest_path }.into())
                },
                _ => todo!(),
            },
            Command::Enclave { enclave_command } => match enclave_command {
                EnclaveCommand::Build { manifest_path } => {
                    Ok(EnclaveBuildRequest { manifest_path }.into())
                }
                _ => todo!(),
            },
            _ => todo!(),
        }
    }
}
