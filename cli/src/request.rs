use crate::{cli::{Command, EnclaveCommand}, error::Error, request::init::InitRequest, request::enclave_build::EnclaveBuildRequest};

pub mod init;
pub mod enclave_build;

#[derive(Clone, Debug)]
pub enum Request {
    Init(InitRequest),
    EnclaveBuild(EnclaveBuildRequest)
}

impl TryFrom<Command> for Request {
    type Error = Error;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init { path } => InitRequest::try_from(path).map(Into::into),
            Command::Enclave { enclave_command } => {
                match enclave_command {
                    EnclaveCommand::Build { manifest_path } => Ok(Request::EnclaveBuild(EnclaveBuildRequest {manifest_path})),
                    _ => todo!()
                }
            }
        }
    }
}
