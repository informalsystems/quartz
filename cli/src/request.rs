use std::{env::current_dir, path::PathBuf};

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
                    ContractCommand::Build { path } => Ok(Request::ContractBuild(ContractBuildRequest {directory: Self::path_checked(path)?})),
                    _ => todo!()
                }
            }
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