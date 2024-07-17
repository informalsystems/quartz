use std::path::PathBuf;

use clap::Parser;
use cosmrs::{tendermint::chain::Id, AccountId};
use displaydoc::Display;
use subtle_encoding::{bech32::decode as bech32_decode, Error as Bech32DecodeError};
use thiserror::Error;
use tonic::transport::Endpoint;

#[derive(Display, Error, Debug)]
pub enum AddressError {
    /// Address is not bech32 encoded
    NotBech32Encoded(#[source] Bech32DecodeError),
    /// Human readable part mismatch (expected `wasm`, found {0})
    HumanReadableMismatch(String),
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// RPC server address
    #[clap(long, default_value = "http://localhost:11090")]
    pub enclave_addr: Endpoint,

    /// Blockchain node gRPC URL
    #[arg(short, long, default_value = "tcp://127.0.0.1:9090")]
    pub node_addr: Endpoint,

    /// Chain-id of MTCS chain
    #[arg(long, default_value = "testing")]
    pub chain_id: Id,

    /// Smart contract address
    #[arg(short, long, value_parser = wasm_address)]
    pub contract: AccountId,

    /// Path to TSP secret key file
    #[arg(short, long)]
    pub secret: PathBuf,

    /// Gas limit for the set-offs submission transaction
    #[arg(long, default_value = "900000000")]
    pub gas_limit: u64,
}

fn wasm_address(address_str: &str) -> Result<AccountId, AddressError> {
    let (hr, _) = bech32_decode(address_str).map_err(AddressError::NotBech32Encoded)?;
    if hr != "wasm" {
        return Err(AddressError::HumanReadableMismatch(hr));
    }

    Ok(address_str.parse().unwrap())
}