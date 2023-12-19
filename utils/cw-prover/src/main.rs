#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

use std::error::Error;

use clap::{Parser, Subcommand};
use cosmrs::AccountId;
use tendermint_rpc::{client::HttpClient as TmRpcClient, Client, HttpClientUrl};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Main command
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Retrieve a merkle-proof for CosmWasm state
    CwQueryProofs {
        #[clap(long, default_value = "http://127.0.0.1:26657")]
        rpc_url: HttpClientUrl,

        /// Address of the CosmWasm contract
        #[clap(long)]
        contract_address: AccountId,

        /// Storage key of the state item for which proofs must be retrieved
        #[clap(long)]
        storage_key: String,
    },
}

const WASM_STORE_KEY: &str = "/store/wasm/key";
const CONTRACT_STORE_PREFIX: u8 = 0x03;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.command {
        Command::CwQueryProofs {
            rpc_url,
            contract_address,
            storage_key,
        } => {
            let path = WASM_STORE_KEY.to_owned();
            let data = {
                let mut data = vec![CONTRACT_STORE_PREFIX];
                data.append(&mut contract_address.to_bytes());
                data.append(&mut storage_key.into_bytes());
                data
            };

            let client = TmRpcClient::builder(rpc_url).build()?;
            let latest_height = client.status().await?.sync_info.latest_block_height;
            let result = client
                .abci_query(Some(path), data, Some(latest_height), true)
                .await?;

            println!(
                "{}",
                serde_json::to_string(&result).expect("infallible serializer")
            );
        }
    };

    Ok(())
}
