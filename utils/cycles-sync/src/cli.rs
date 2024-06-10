use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cosmrs::{tendermint::chain::Id, AccountId};
use displaydoc::Display;
use reqwest::Url;
use subtle_encoding::{bech32::decode as bech32_decode, Error as Bech32DecodeError};
use thiserror::Error;

#[derive(Clone, Debug, Parser)]
#[command(author, version, about)]
pub struct Cli {
    /// Increase output logging verbosity to debug level.
    #[arg(short, long)]
    pub verbose: bool,

    /// The host to which to bind the API server.
    #[arg(short = 'N', long, default_value = "http://127.0.0.1:26657")]
    pub node: Url,

    /// Obligato API server.
    #[arg(long, default_value = "https://bisenzone.obligato.network")]
    pub obligato_url: Url,

    /// Obligato key.
    #[arg(
        long,
        default_value = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZSIsInJlZiI6ImNhZXNzdGpjdG16bXVqaW55cGJlIiwicm9sZSI6InNlcnZpY2Vfcm9sZSIsImlhdCI6MTcwMzAxOTE0OSwiZXhwIjoyMDE4NTk1MTQ5fQ.EV6v5J3dz8WHAdTK4_IEisKzF-n1Gqyn4wCce_Zrqf4"
    )]
    pub obligato_key: String,

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
