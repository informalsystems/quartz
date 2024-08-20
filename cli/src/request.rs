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
            Command::Init(args) => Ok(InitRequest { name: args.name }.try_into()?),
            Command::Handshake(args) => Ok(HandshakeRequest {
                contract: args.contract,
                use_latest_trusted: args.use_latest_trusted,
            }
            .into()),
            Command::Contract { contract_command } => contract_command.try_into(),
            Command::Enclave { enclave_command } => enclave_command.try_into(),
            Command::Dev(args) => Ok(DevRequest {
                watch: args.watch,
                use_latest_trusted: args.use_latest_trusted,
                init_msg: serde_json::from_str(&args.contract_deploy.init_msg)
                    .map_err(|e| Error::GenericErr(e.to_string()))?,
                label: args.contract_deploy.label,
                wasm_bin_path: args.contract_deploy.wasm_bin_path,
                release: args.enclave_build.release,
            }
            .into()),
        }
    }
}

impl TryFrom<ContractCommand> for Request {
    type Error = Error;

    fn try_from(cmd: ContractCommand) -> Result<Request, Error> {
        match cmd {
            ContractCommand::Deploy(args) => {
                if !args.wasm_bin_path.exists() {
                    return Err(Error::PathNotFile(args.wasm_bin_path.display().to_string()));
                }

                Ok(ContractDeployRequest {
                    init_msg: serde_json::from_str(&args.init_msg)
                        .map_err(|e| Error::GenericErr(e.to_string()))?,
                    label: args.label,
                    wasm_bin_path: args.wasm_bin_path,
                }
                .into())
            }
            ContractCommand::Build(args) => {
                if !args.manifest_path.exists() {
                    return Err(Error::PathNotFile(args.manifest_path.display().to_string()));
                }

                Ok(ContractBuildRequest {
                    manifest_path: args.manifest_path,
                }
                .into())
            }
        }
    }
}

impl TryFrom<EnclaveCommand> for Request {
    type Error = Error;

    fn try_from(cmd: EnclaveCommand) -> Result<Request, Error> {
        match cmd {
            EnclaveCommand::Build(args) => Ok(EnclaveBuildRequest {
                release: args.release,
                manifest_path: args.manifest_path,
            }
            .into()),
            EnclaveCommand::Start(args) => Ok(EnclaveStartRequest {
                shutdown_rx: None,
                use_latest_trusted: args.use_latest_trusted,
            }
            .into()),
        }
    }
}
