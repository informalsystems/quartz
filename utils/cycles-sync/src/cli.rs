use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cosmrs::{tendermint::chain::Id, AccountId};
use displaydoc::Display;
use subtle_encoding::{bech32::decode as bech32_decode, Error as Bech32DecodeError};
use thiserror::Error;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Increase output logging verbosity to debug level.
    #[arg(short, long)]
    pub verbose: bool,

    /// The host to which to bind the API server.
    #[arg(short = 'H', long, default_value = "0.0.0.0")]
    pub host: String,

    /// The port to which to bind the API server.
    #[arg(short, long, default_value = "8000")]
    pub port: u16,

    /// Path to output CSV file
    #[arg(short, long)]
    pub keys_file: PathBuf,

    /// Path to obligation-user map
    #[arg(short, long)]
    pub obligation_user_map_file: PathBuf,

    /// Chain-id of MTCS chain
    #[arg(long, default_value = "testing")]
    pub chain_id: Id,

    /// Smart contract address
    #[arg(short, long, value_parser = wasm_address)]
    pub contract: AccountId,

    /// tx sender address
    #[arg(short, long)]
    pub user: String,

    /// Main command
    #[command(subcommand)]
    pub command: CliCommand,
}

#[derive(Clone, Debug, Subcommand)]
pub enum CliCommand {
    /// Sync obligations
    SyncObligations {
        /// epoch pk
        #[arg(short, long)]
        epoch_pk: String,
    },
    /// Sync set-offs
    SyncSetOffs,
}

#[derive(Display, Error, Debug)]
pub enum AddressError {
    /// Address is not bech32 encoded
    NotBech32Encoded(#[source] Bech32DecodeError),
    /// Human readable part mismatch (expected `wasm`, found {0})
    HumanReadableMismatch(String),
}

fn wasm_address(address_str: &str) -> Result<AccountId, AddressError> {
    let (hr, _) = bech32_decode(address_str).map_err(AddressError::NotBech32Encoded)?;
    if hr != "wasm" {
        return Err(AddressError::HumanReadableMismatch(hr));
    }

    Ok(address_str.parse().unwrap())
}
