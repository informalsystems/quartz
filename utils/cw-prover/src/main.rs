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
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use cosmrs::AccountId;
use ibc_relayer_types::{
    core::ics23_commitment::commitment::CommitmentRoot, core::ics23_commitment::specs::ProofSpecs,
};
use tendermint::block::Height;
use tendermint::AppHash;
use tendermint_rpc::endpoint::abci_query::AbciQuery;
use tendermint_rpc::endpoint::status::Response;
use tendermint_rpc::{client::HttpClient as TmRpcClient, Client, HttpClientUrl};

mod merkle;

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

        /// Storage namespace of the state item for which proofs must be retrieved
        /// (only makes sense when dealing with maps)
        #[clap(long)]
        storage_namespace: Option<String>,

        /// Output file to store merkle proof
        #[clap(long)]
        proof_file: Option<PathBuf>,
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
            storage_namespace,
            proof_file,
        } => {
            let client = TmRpcClient::builder(rpc_url).build()?;
            let status = client.status().await?;
            let (proof_height, latest_app_hash) = latest_proof_height_hash(status);

            let path = WASM_STORE_KEY.to_owned();
            let data = query_data(&contract_address, storage_key, storage_namespace);
            let result = client
                .abci_query(Some(path), data, Some(proof_height), true)
                .await?;

            let value = verify_proof(latest_app_hash, result.clone())?;
            println!("{}", String::from_utf8(value)?);

            if let Some(proof_file) = proof_file {
                write_proof_to_file(proof_file, result)?;
            }
        }
    };

    Ok(())
}

fn latest_proof_height_hash(status: Response) -> (Height, AppHash) {
    let proof_height = {
        let latest_height = status.sync_info.latest_block_height;
        (latest_height.value() - 1)
            .try_into()
            .expect("infallible conversion")
    };
    let latest_app_hash = status.sync_info.latest_app_hash;

    (proof_height, latest_app_hash)
}

fn query_data(
    contract_address: &AccountId,
    storage_key: String,
    storage_namespace: Option<String>,
) -> Vec<u8> {
    let mut data = vec![CONTRACT_STORE_PREFIX];
    data.append(&mut contract_address.to_bytes());
    if let Some(namespace) = storage_namespace {
        data.extend_from_slice(&encode_length(namespace.as_bytes()));
        data.append(&mut namespace.into_bytes());
    }
    data.append(&mut storage_key.into_bytes());
    data
}

// Copied from cw-storage-plus
fn encode_length(namespace: &[u8]) -> [u8; 2] {
    assert!(
        namespace.len() <= 0xFFFF,
        "only supports namespaces up to length 0xFFFF"
    );

    let length_bytes = (namespace.len() as u32).to_be_bytes();
    [length_bytes[2], length_bytes[3]]
}

fn verify_proof(latest_app_hash: AppHash, result: AbciQuery) -> Result<Vec<u8>, Box<dyn Error>> {
    let proof = merkle::convert_tm_to_ics_merkle_proof(&result.proof.expect("queried with proof"))?;
    let root = CommitmentRoot::from_bytes(latest_app_hash.as_bytes());
    let prefixed_key = vec!["wasm".to_string().into_bytes(), result.key];

    proof.verify_membership(
        &ProofSpecs::cosmos(),
        root.into(),
        prefixed_key,
        result.value.clone(),
        0,
    )?;

    Ok(result.value)
}

fn write_proof_to_file(proof_file: PathBuf, output: AbciQuery) -> Result<(), Box<dyn Error>> {
    let file = File::create(proof_file)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &output)?;
    writer.flush()?;
    Ok(())
}
