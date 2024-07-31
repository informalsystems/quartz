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

    /// Main command
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Create an empty Quartz app from a template
    Init {
        /// path to create & init a quartz app, defaults to current path if unspecified
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

        #[arg(
            short,
            long,
            default_value = "admin"
        )]
        sender: String,

        #[arg(long, default_value = "testing")]
        chain_id: ChainId,

        #[clap(long, default_value_t = default_node_url())]
        node_url: String,

        #[clap(long, default_value_t = default_rpc_addr())]
        rpc_addr: String,

        #[clap(long)]
        path: Option<PathBuf>,
    },
    /// Create an empty Quartz app from a template
    Contract {
        #[command(subcommand)]
        contract_command: ContractCommand
    }    
}

#[derive(Debug, Clone, Subcommand)]
pub enum ContractCommand {
    Build {
        #[clap(long)]
        path: Option<PathBuf>,
    },
    Deploy {
        #[clap(long, default_value_t = default_node_url())]
        node_url: String,

        #[arg(
            short,
            long,
            default_value = "admin"
        )]
        sender: String,

        #[arg(long, default_value = "testing")]
        chain_id: ChainId,

        #[arg(long, default_value = "MTCS")]
        label: String,

        #[clap(long)]
        path: Option<PathBuf>,
    },
}

fn default_rpc_addr() -> String {
    env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
}

fn default_node_url() -> String {
    env::var("NODE_URL").unwrap_or_else(|_| "143.244.186.205:26657".to_string())
}