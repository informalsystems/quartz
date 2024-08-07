use std::{env, path::PathBuf};

use clap::{Parser, Subcommand};
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use tracing::metadata::LevelFilter;

use crate::handler::utils::helpers::wasmaddr_to_id;

#[derive(clap::Args, Debug, Clone)]
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

#[derive(Debug, Parser)]
#[command(version, long_about = None)]
pub struct Cli {
    /// Increase log verbosity
    #[clap(flatten)]
    pub verbose: Verbosity,

    /// Enable mock SGX mode for testing purposes.
    /// This flag disables the use of an Intel SGX processor and allows the system to run without remote attestations.
    #[clap(long, env)]
    pub mock_sgx: bool,

    /// Main command
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create an empty Quartz app from a template
    Init {
        /// path to create & init a Quartz app, defaults to current path if unspecified
        #[clap(long)]
        path: Option<PathBuf>,
    },
    Handshake {
        /// path to create & init a quartz app, defaults to current path if unspecified
        #[arg(short, long, value_parser = wasmaddr_to_id)]
        contract: AccountId,
        /// Port enclave is listening on
        #[arg(short, long, default_value = "11090")]
        port: u16,
        /// Name or address of private key with which to sign
        #[arg(short, long, default_value = "admin")]
        sender: String,
        /// The network chain ID
        #[arg(long, default_value = "testing")]
        chain_id: ChainId,
        /// <host>:<port> to tendermint rpc interface for this chain
        #[clap(long, default_value_t = default_node_url())]
        node_url: String,
        /// RPC interface for the quartz enclave
        #[clap(long, default_value_t = default_rpc_addr())]
        enclave_rpc_addr: String,
        /// Path to quartz app directory
        /// Defaults to current working dir
        #[clap(long)]
        app_dir: Option<PathBuf>,
    },
    /// Build, deploy, perform handshake, and run quartz app while listening for changes
    Dev {
        /// Automatically deploy and instantiate new cosmwasm contract instance upon changes to contract source
        #[clap(long)]
        watch_contract: bool,
        /// Path to quartz app directory
        /// Defaults to current working dir
        #[clap(long)]
        app_dir: Option<PathBuf>,
    },
    /// Create an empty Quartz app from a template
    Contract {
        #[command(subcommand)]
        contract_command: ContractCommand,
    },
    /// Subcommands for handling the Quartz app enclave
    Enclave {
        #[command(subcommand)]
        enclave_command: EnclaveCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ContractCommand {
    Build {
        #[clap(long)]
        path: Option<PathBuf>,
    },
    Deploy {
        /// Json-formatted cosmwasm contract initialization message
        #[clap(long, default_value = "{}")]
        init_msg: String,
        /// <host>:<port> to tendermint rpc interface for this chain
        #[clap(long, default_value_t = default_node_url())]
        node_url: String,
        /// Name or address of private key with which to sign
        #[arg(short, long, default_value = "admin")]
        sender: String,
        /// The network chain ID
        #[arg(long, default_value = "testing")]
        chain_id: ChainId,
        /// A human-readable name for this contract in lists
        #[arg(long, default_value = "Quartz App Contract")]
        label: String,
        /// Path to contract wasm binary for deployment
        #[clap(long)]
        wasm_bin_path: PathBuf,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum EnclaveCommand {
    /// Build the Quartz app's enclave
    Build {
        /// Whether to target release or dev
        #[arg(long)]
        release: bool,
        /// path to Cargo.toml file of the Quartz app's enclave package, defaults to './enclave/Cargo.toml' if unspecified
        #[arg(long, default_value = "./enclave/Cargo.toml")]
        manifest_path: PathBuf,
    },
    // Run the Quartz app's enclave
    Start {
        #[clap(long)]
        path: Option<PathBuf>,
    },
}

fn default_rpc_addr() -> String {
    env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
}

fn default_node_url() -> String {
    env::var("NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:26657".to_string())
}
