use std::{env::current_dir, path::PathBuf};

use crate::{cli::Command, error::Error, request::init::InitRequest};

pub mod init;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { name } => Ok(Request::Init(InitRequest { name })),
        }
    }
}

impl Request {
    fn path_checked(path: Option<PathBuf>) -> Result<PathBuf, Error> {
        if let Some(path) = path {
            Ok(path)
        } else {
            Ok(current_dir().map_err(|e| Error::GenericErr(e.to_string()))?)
        }
    }
}
