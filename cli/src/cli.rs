use std::path::PathBuf;

use clap::{Parser, Subcommand};
use tracing::metadata::LevelFilter;

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
    /// Subcommands for handling the Quartz app enclave
    Enclave {
        #[command(subcommand)]
        enclave_command: EnclaveCommand,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum EnclaveCommand {
    /// Build the Quartz app's enclave
    Build {
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
