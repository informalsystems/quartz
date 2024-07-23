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
            Command::Init { path } => InitRequest::try_from(path),
        }
        .map(Into::into)
    }
}
