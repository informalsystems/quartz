use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use figment::{providers::Serialized, Figment};
use quartz_common::enclave::types::Fmspc;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tracing::metadata::LevelFilter;

use crate::handler::utils::helpers::wasmaddr_to_id;

#[derive(clap::Args, Debug, Clone, Serialize)]
pub struct Verbosity {
    /// Increase verbosity, can be repeated up to 2 times
    #[arg(long, short, action = clap::ArgAction::Count)]
    pub verbose: u8,
}

impl Verbosity {
    pub fn to_level_filter(&self) -> LevelFilter {
        match self.verbose {
            0 => LevelFilter::INFO,
            1 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        }
    }
}

#[derive(Debug, Parser, Serialize)]
#[command(version, long_about = None)]
pub struct Cli {
    /// Increase log verbosity
    #[command(flatten)]
    pub verbose: Verbosity,

    /// Enable mock SGX mode for testing purposes.
    /// This flag disables the use of an Intel SGX processor and allows the system to run without remote attestations.
    #[arg(long)]
    #[serde(skip_serializing_if = "is_false")]
    pub mock_sgx: bool,

    /// Path to Quartz app directory.
    /// Defaults to current working dir.
    /// For quartz init, root serves as the parent directory of the directory in which the quartz app is generated
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_dir: Option<PathBuf>,

    /// Main command
    #[command(subcommand)]
    pub command: Command,
}

fn is_false(b: &bool) -> bool {
    !(*b)
}

#[derive(Debug, Subcommand, Serialize, Clone)]
pub enum Command {
    /// Create an empty Quartz app from a template
    Init(InitArgs),

    /// Perform handshake
    Handshake(HandshakeArgs),

    /// Subcommands for handling the Quartz app contract
    Contract {
        #[command(subcommand)]
        contract_command: ContractCommand,
    },

    /// Subcommands for handling the Quartz app enclave
    Enclave {
        #[command(subcommand)]
        enclave_command: EnclaveCommand,
    },

    /// Build, deploy, perform handshake, and run quartz app while listening for changes
    Dev(DevArgs),

    /// Print the FMSPC of the current platform (SGX only)
    PrintFmspc(PrintFmspcArgs),
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Subcommand, Serialize)]
pub enum ContractCommand {
    Build(ContractBuildArgs),
    Deploy(ContractDeployArgs),
}

#[derive(Debug, Clone, Subcommand, Serialize)]
pub enum EnclaveCommand {
    /// Build the Quartz app's enclave
    Build(EnclaveBuildArgs),
    /// Run the Quartz app's enclave
    Start(EnclaveStartArgs),
}

#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct InitArgs {
    /// The name of your Quartz app directory, defaults to quartz_app
    #[arg(default_value = "quartz_app")]
    pub name: PathBuf,
}

#[serde_as]
#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct HandshakeArgs {
    /// Path to create & init a Quartz app, defaults to current path if unspecified
    #[arg(short, long, value_parser = wasmaddr_to_id)]
    pub contract: AccountId,

    /// Fetch latest trusted hash and height from the chain instead of existing configuration
    #[arg(long, default_value_t = false)]
    pub unsafe_trust_latest: bool,

    /// Name or address of private key with which to sign
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_sender: Option<String>,

    /// The network chain ID
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<ChainId>,

    /// `<host>:<port>` to tendermint rpc interface for this chain
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub node_url: Option<Url>,

    /// websocket URL
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub ws_url: Option<Url>,

    /// gRPC URL
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub grpc_url: Option<Url>,

    /// RPC interface for the Quartz enclave
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enclave_rpc_addr: Option<String>,

    /// Port enclave is listening on
    #[arg(long)]
    #[serde(skip_serializing_if = "::std::option::Option::is_none")]
    pub enclave_rpc_port: Option<u16>,
}

#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct ContractBuildArgs {
    /// Path to Cargo manifest file for CosmWasm contract package
    #[arg(long, default_value = "./contracts/Cargo.toml")]
    pub contract_manifest: PathBuf,
}

#[serde_as]
#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct ContractDeployArgs {
    /// Json-formatted cosmwasm contract initialization message
    #[arg(long, default_value = "{}")]
    pub init_msg: String,

    /// `<host>:<port>` to tendermint rpc interface for this chain
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub node_url: Option<Url>,

    /// websocket URL
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub ws_url: Option<Url>,

    /// gRPC URL
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub grpc_url: Option<Url>,

    /// Name or address of private key with which to sign
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_sender: Option<String>,

    /// The network chain ID
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<ChainId>,

    /// A human-readable name for this contract in lists
    #[arg(long, default_value = "Quartz App Contract")]
    pub label: String,

    /// Path to Cargo manifest file for CosmWasm contract package
    #[arg(long, default_value = "./contracts/Cargo.toml")]
    pub contract_manifest: PathBuf,
}

#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct EnclaveBuildArgs {
    /// Whether to target release or dev
    #[arg(long)]
    #[serde(skip_serializing_if = "is_false")]
    pub release: bool,
}

#[serde_as]
#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct EnclaveStartArgs {
    /// The network chain ID
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<ChainId>,

    /// Fetch latest trusted hash and height from the chain instead of existing configuration
    #[arg(long, default_value_t = false)]
    pub unsafe_trust_latest: bool,

    /// FMSPC (Family-Model-Stepping-Platform-Custom SKU); required if `MOCK_SGX` is not set
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fmspc: Option<Fmspc>,

    /// PCCS URL; required if `MOCK_SGX` is not set
    #[arg(long)]
    #[serde_as(as = "Option<DisplayFromStr>")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pccs_url: Option<Url>,

    /// Address of the TcbInfo contract
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcbinfo_contract: Option<AccountId>,

    /// Address of the DCAP verifier contract
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dcap_verifier_contract: Option<AccountId>,

    /// Whether to target release or dev
    #[arg(long)]
    #[serde(skip_serializing_if = "is_false")]
    pub release: bool,
}

#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct DevArgs {
    /// Automatically deploy and instantiate new cosmwasm contract instance upon changes to source
    #[arg(long)]
    pub watch: bool,

    /// Fetch latest trusted hash and height from the chain instead of existing configuration
    #[arg(long, default_value_t = true)]
    pub unsafe_trust_latest: bool,

    #[command(flatten)]
    pub contract_deploy: ContractDeployArgs,

    #[command(flatten)]
    pub enclave_build: EnclaveBuildArgs,

    /// FMSPC (Family-Model-Stepping-Platform-Custom SKU); required if `MOCK_SGX` is not set
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fmspc: Option<Fmspc>,

    /// Address of the TcbInfo contract
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcbinfo_contract: Option<AccountId>,

    /// Address of the DCAP verifier contract
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dcap_verifier_contract: Option<AccountId>,
}

#[serde_as]
#[derive(Debug, Parser, Clone, Serialize, Deserialize)]
pub struct PrintFmspcArgs {
    /// PCCS URL
    #[arg(long)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde_as(as = "Option<DisplayFromStr>")]
    pub pccs_url: Option<Url>,
}

pub trait ToFigment {
    fn to_figment(&self) -> Figment;
}

impl ToFigment for Command {
    fn to_figment(&self) -> Figment {
        match self {
            Command::Init(args) => Figment::from(Serialized::defaults(args)),
            Command::Handshake(args) => Figment::from(Serialized::defaults(args)),
            Command::Contract { contract_command } => match contract_command {
                ContractCommand::Build(args) => Figment::from(Serialized::defaults(args)),
                ContractCommand::Deploy(args) => Figment::from(Serialized::defaults(args)),
            },
            Command::Enclave { enclave_command } => match enclave_command {
                EnclaveCommand::Build(args) => Figment::from(Serialized::defaults(args)),
                EnclaveCommand::Start(args) => Figment::from(Serialized::defaults(args)),
            },
            Command::Dev(args) => Figment::from(Serialized::defaults(args))
                .merge(Serialized::defaults(&args.contract_deploy))
                .merge(Serialized::defaults(&args.enclave_build)),
            Command::PrintFmspc(args) => Figment::from(Serialized::defaults(args)),
        }
    }
}
