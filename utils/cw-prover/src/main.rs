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
use ibc_proto::{
    ibc::core::commitment::v1::MerkleProof as RawMerkleProof, ibc::core::commitment::v1::MerkleRoot,
};
use ibc_relayer_types::{
    core::ics23_commitment::commitment::CommitmentRoot,
    core::ics23_commitment::error::Error as ProofError, core::ics23_commitment::specs::ProofSpecs,
};
use ics23::{
    calculate_existence_root, commitment_proof::Proof, verify_membership, CommitmentProof,
    ProofSpec,
};
use tendermint::block::Height;
use tendermint::merkle::proof::ProofOps as TendermintProof;
use tendermint::AppHash;
use tendermint_rpc::endpoint::abci_query::AbciQuery;
use tendermint_rpc::endpoint::status::Response;
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

        /// Storage namespace of the state item for which proofs must be retrieved
        /// (only makes sense when dealing with maps)
        #[clap(long)]
        storage_namespace: Option<String>,
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
        } => {
            let client = TmRpcClient::builder(rpc_url).build()?;
            let status = client.status().await?;
            let (proof_height, latest_app_hash) = latest_proof_height_hash(status);

            let path = WASM_STORE_KEY.to_owned();
            let data = query_data(&contract_address, storage_key, storage_namespace);
            let result = client
                .abci_query(Some(path), data, Some(proof_height), true)
                .await?;

            let value = verify_proof(latest_app_hash, result)?;
            println!("{}", String::from_utf8(value)?);
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
    let proof = convert_tm_to_ics_merkle_proof(&result.proof.expect("queried with proof"))?;
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

// Copied from hermes and patched to allow non-string keys
#[derive(Clone, Debug, PartialEq)]
struct MerkleProof {
    proofs: Vec<CommitmentProof>,
}

/// Convert to ics23::CommitmentProof
impl From<RawMerkleProof> for MerkleProof {
    fn from(proof: RawMerkleProof) -> Self {
        Self {
            proofs: proof.proofs,
        }
    }
}

impl From<MerkleProof> for RawMerkleProof {
    fn from(proof: MerkleProof) -> Self {
        Self {
            proofs: proof.proofs,
        }
    }
}

impl MerkleProof {
    pub fn verify_membership(
        &self,
        specs: &ProofSpecs,
        root: MerkleRoot,
        keys: Vec<Vec<u8>>,
        value: Vec<u8>,
        start_index: usize,
    ) -> Result<(), ProofError> {
        // validate arguments
        if self.proofs.is_empty() {
            return Err(ProofError::empty_merkle_proof());
        }
        if root.hash.is_empty() {
            return Err(ProofError::empty_merkle_root());
        }
        let num = self.proofs.len();
        let ics23_specs = Vec::<ProofSpec>::from(specs.clone());
        if ics23_specs.len() != num {
            return Err(ProofError::number_of_specs_mismatch());
        }
        if keys.len() != num {
            return Err(ProofError::number_of_keys_mismatch());
        }
        if value.is_empty() {
            return Err(ProofError::empty_verified_value());
        }

        let mut subroot = value.clone();
        let mut value = value;

        // keys are represented from root-to-leaf
        for ((proof, spec), key) in self
            .proofs
            .iter()
            .zip(ics23_specs.iter())
            .zip(keys.iter().rev())
            .skip(start_index)
        {
            match &proof.proof {
                Some(Proof::Exist(existence_proof)) => {
                    subroot =
                        calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof)
                            .map_err(|_| ProofError::invalid_merkle_proof())?;

                    if !verify_membership::<ics23::HostFunctionsManager>(
                        proof, spec, &subroot, key, &value,
                    ) {
                        return Err(ProofError::verification_failure());
                    }
                    value = subroot.clone();
                }
                _ => return Err(ProofError::invalid_merkle_proof()),
            }
        }

        if root.hash != subroot {
            return Err(ProofError::verification_failure());
        }

        Ok(())
    }
}

fn convert_tm_to_ics_merkle_proof(tm_proof: &TendermintProof) -> Result<MerkleProof, ProofError> {
    let mut proofs = Vec::new();

    for op in &tm_proof.ops {
        let mut parsed = CommitmentProof { proof: None };

        prost::Message::merge(&mut parsed, op.data.as_slice())
            .map_err(ProofError::commitment_proof_decoding_failed)?;

        proofs.push(parsed);
    }

    Ok(MerkleProof::from(RawMerkleProof { proofs }))
}
