use std::{
    env::{self},
    path::PathBuf,
};

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

    /// Path to Quartz app directory
    /// Defaults to current working dir
    #[clap(long, default_value = ".")]
    pub app_dir: PathBuf,

    /// Main command
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create an empty Quartz app from a template
    Init {
        /// the name of your Quartz app directory, defaults to quartz_app
        #[clap(long, default_value = "quartz_app")]
        name: String,
    },
    /// Perform handshake
    Handshake {
        /// path to create & init a Quartz app, defaults to current path if unspecified
        #[arg(short, long, value_parser = wasmaddr_to_id)]
        contract: AccountId,
    },
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
}

#[derive(Debug, Clone, Subcommand)]
pub enum ContractCommand {
    Build {
        #[clap(long)]
        manifest_path: PathBuf,
    },
    Deploy {
        /// Json-formatted cosmwasm contract initialization message
        #[clap(long, default_value = "{}")]
        init_msg: String,
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
        /// Path to Cargo.toml file of the Quartz app's enclave package, defaults to './enclave/Cargo.toml' if unspecified
        #[arg(long, default_value = "./enclave/Cargo.toml")]
        manifest_path: PathBuf,
    },
    // Run the Quartz app's enclave
    Start { },
}

fn default_rpc_addr() -> String {
    env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
}

fn default_node_url() -> String {
    env::var("NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:26657".to_string())
}
