use color_eyre::eyre::eyre;

use crate::{
    cli::{Command, ContractCommand, EnclaveCommand},
    request::{
        contract_build::ContractBuildRequest, contract_deploy::ContractDeployRequest,
        dev::DevRequest, enclave_build::EnclaveBuildRequest, enclave_start::EnclaveStartRequest,
        handshake::HandshakeRequest, init::InitRequest,
    },
};

use color_eyre::{Result, Report};

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
    type Error = Report;

    fn try_from(cmd: Command) -> Result<Self, Self::Error> {
        match cmd {
            Command::Init(args) => Ok(InitRequest { name: args.name }.try_into()?),
            Command::Handshake(args) => Ok(HandshakeRequest {
                contract: args.contract,
                unsafe_trust_latest: args.unsafe_trust_latest,
            }
            .into()),
            Command::Contract { contract_command } => contract_command.try_into(),
            Command::Enclave { enclave_command } => enclave_command.try_into(),
            Command::Dev(args) => Ok(DevRequest {
                watch: args.watch,
                unsafe_trust_latest: args.unsafe_trust_latest,
                init_msg: serde_json::from_str(&args.contract_deploy.init_msg)?,
                label: args.contract_deploy.label,
                contract_manifest: args.contract_deploy.contract_manifest,
                release: args.enclave_build.release,
                fmspc: args.fmspc,
                tcbinfo_contract: args.tcbinfo_contract,
                dcap_verifier_contract: args.dcap_verifier_contract,
            }
            .into()),
        }
    }
}

impl TryFrom<ContractCommand> for Request {
    type Error = Report;

    fn try_from(cmd: ContractCommand) -> Result<Request> {
        match cmd {
            ContractCommand::Deploy(args) => {
                if !args.contract_manifest.exists() {
                    return Err(eyre!("The contract manifest file does not exist: {}", args.contract_manifest.display()));
                }

                Ok(ContractDeployRequest {
                    init_msg: serde_json::from_str(&args.init_msg)?,
                    label: args.label,
                    contract_manifest: args.contract_manifest,
                }
                .into())
            }
            ContractCommand::Build(args) => {
                if !args.contract_manifest.exists() {
                    return Err(eyre!("The contract manifest file does not exist: {}", args.contract_manifest.display()));
                }

                Ok(ContractBuildRequest {
                    contract_manifest: args.contract_manifest,
                }
                .into())
            }
        }
    }
}

impl TryFrom<EnclaveCommand> for Request {
type Error = Report;

    fn try_from(cmd: EnclaveCommand) -> Result<Request> {
        match cmd {
            EnclaveCommand::Build(_) => Ok(EnclaveBuildRequest {}.into()),
            EnclaveCommand::Start(args) => Ok(EnclaveStartRequest {
                unsafe_trust_latest: args.unsafe_trust_latest,
                fmspc: args.fmspc,
                tcbinfo_contract: args.tcbinfo_contract,
                dcap_verifier_contract: args.dcap_verifier_contract,
            }
            .into()),
        }
    }
}
