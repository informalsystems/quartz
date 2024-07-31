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

    #[clap(long)]
    pub mock_sgx: bool,

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
    /// Create an empty Quartz app from a template
    Enclave {
        #[command(subcommand)]
        enclave_command: EnclaveCommand
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum EnclaveCommand {
    Build {
        #[clap(long)]
        path: Option<PathBuf>,
    },
    Start {
        #[clap(long)]
        path: Option<PathBuf>,
    }
}