use crate::{cli::{Command, ContractCommand}, error::Error, request::{contract_build::ContractBuildRequest, init::InitRequest}};

pub mod init;
pub mod contract_build;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
    ContractBuild(ContractBuildRequest)
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { path } => InitRequest::try_from(path).map(Into::into),
            Command::Contract { contract_command } => {
                match contract_command {
                    ContractCommand::Build { manifest_path } => Ok(Request::ContractBuild(ContractBuildRequest {manifest_path})),
                    _ => todo!()
                }
            }
        }
    }
}