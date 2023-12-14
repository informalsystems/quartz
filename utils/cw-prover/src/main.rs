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
use cosmos_sdk_proto::cosmwasm::wasm::v1::QueryRawContractStateRequest as RawQueryRawContractStateRequest;
use cosmos_sdk_proto::traits::Message;
use cosmrs::AccountId;
use tendermint_rpc::client::HttpClient as TmRpcClient;
use tendermint_rpc::{Client, HttpClientUrl};

struct QueryRawContractStateRequest {
    pub contract_address: AccountId,
    pub storage_key: String,
}

impl From<QueryRawContractStateRequest> for RawQueryRawContractStateRequest {
    fn from(request: QueryRawContractStateRequest) -> Self {
        Self {
            address: request.contract_address.to_string(),
            query_data: request.storage_key.into_bytes(),
        }
    }
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::parse();

    match args.command {
        Command::CwQueryProofs {
            rpc_url,
            contract_address,
            storage_key,
        } => {
            let path = "/store/wasm/key".to_owned();
            let request = QueryRawContractStateRequest {
                contract_address,
                storage_key,
            };
            let raw_request = RawQueryRawContractStateRequest::from(request);
            let data = raw_request.encode_to_vec();

            let client = TmRpcClient::builder(rpc_url).build()?;
            let latest_height = client.status().await?.sync_info.latest_block_height;
            println!("{:?}", latest_height);
            let result = client.abci_query(Some(path), data, Some(latest_height), true).await?;
            println!(
                "{}",
                serde_json::to_string(&result).expect("infallible serializer")
            );
        }
    };

    Ok(())
}
