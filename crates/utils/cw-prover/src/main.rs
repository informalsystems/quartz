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

use std::{
    error::Error,
    fmt::Debug,
    fs::File,
    io::{BufWriter, Write},
    path::PathBuf,
};

use clap::{Parser, Subcommand};
use cosmrs::AccountId;
use quartz_cw_proof::proof::{
    cw::{CwProof, RawCwProof},
    key::CwAbciKey,
    Proof,
};
use tendermint::{block::Height, AppHash};
use tendermint_rpc::{
    client::HttpClient as TmRpcClient, endpoint::status::Response, Client, HttpClientUrl,
};

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
            let data = CwAbciKey::new(contract_address, storage_key, storage_namespace);
            let result = client
                .abci_query(Some(path), data, Some(proof_height), true)
                .await?;

            let proof: CwProof = result.clone().try_into().map_err(into_string)?;
            proof
                .verify(latest_app_hash.clone().into())
                .map_err(into_string)?;

            println!("{}", String::from_utf8(result.value.clone())?);

            if let Some(proof_file) = proof_file {
                write_proof_to_file(proof_file, proof.into())?;
            }
        }
    };

    Ok(())
}

fn into_string<E: ToString>(e: E) -> String {
    e.to_string()
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

fn write_proof_to_file(proof_file: PathBuf, proof: RawCwProof) -> Result<(), Box<dyn Error>> {
    let file = File::create(proof_file)?;
    let mut writer = BufWriter::new(file);
    serde_json::to_writer(&mut writer, &proof)?;
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use quartz_cw_proof::proof::{
        cw::{CwProof, RawCwProof},
        Proof,
    };
    use tendermint_rpc::endpoint::abci_query::AbciQuery;

    #[test]
    fn test_query_item() {
        let abci_query_response = r#"{
          "code": 0,
          "log": "",
          "info": "",
          "index": "0",
          "key": "A63kpfWAOkOYNcY2OVqNZI3uV7L8kNmNwX+ohxWbaWOLc2d4c3RhdGU=",
          "value": "eyJjb21wdXRlX21yZW5jbGF2ZSI6ImRjNDNmOGM0MmQ4ZTVmNTJjOGJiZDY4ZjQyNjI0MjE1M2YwYmUxMDYzMGZmOGNjYTI1NTEyOWEzY2EwM2QyNzMiLCJrZXlfbWFuYWdlcl9tcmVuY2xhdmUiOiIxY2YyZTUyOTExNDEwZmJmM2YxOTkwNTZhOThkNTg3OTVhNTU5YTJlODAwOTMzZjdmY2QxM2QwNDg0NjIyNzFjIiwidGNiX2luZm8iOiIzMTIzODc2In0=",
          "proof": {
            "ops": [
              {
                "field_type": "ics23:iavl",
                "key": "A63kpfWAOkOYNcY2OVqNZI3uV7L8kNmNwX+ohxWbaWOLc2d4c3RhdGU=",
                "data": "CrgDCikDreSl9YA6Q5g1xjY5Wo1kje5XsvyQ2Y3Bf6iHFZtpY4tzZ3hzdGF0ZRLIAXsiY29tcHV0ZV9tcmVuY2xhdmUiOiJkYzQzZjhjNDJkOGU1ZjUyYzhiYmQ2OGY0MjYyNDIxNTNmMGJlMTA2MzBmZjhjY2EyNTUxMjlhM2NhMDNkMjczIiwia2V5X21hbmFnZXJfbXJlbmNsYXZlIjoiMWNmMmU1MjkxMTQxMGZiZjNmMTk5MDU2YTk4ZDU4Nzk1YTU1OWEyZTgwMDkzM2Y3ZmNkMTNkMDQ4NDYyMjcxYyIsInRjYl9pbmZvIjoiMzEyMzg3NiJ9GgwIARgBIAEqBAACmAEiKggBEiYCBJgBIFclzyzP2y2LTcBhP0IxBhvnlMJiEFCsDEMUQ9dM5dvYICIsCAESBQQGmAEgGiEgfUSWe0VMFTsxkzDuMQNE05aSzdRTTvkWzZXkfplWUbEiKggBEiYGDJBnIEkK+nmGmXpOfREXvfonLrK4mEZx1XF4DgJp86QIVF1EICIsCAESBQgakGcgGiEgBl/NSR16eG1vDenJA6GEEJ9xcQv9Bwxv8wyhAL5JLwE="
              },
              {
                "field_type": "ics23:simple",
                "key": "d2FzbQ==",
                "data": "CqgBCgR3YXNtEiDYWxn2B9M/eGP18Gwl3zgWZkT7Yn/iFlcS0THfmfcfDBoJCAEYASABKgEAIiUIARIhAWLU8PgnJ/EMp4BYvtTN9MX/rS70dNQ3ZAzrJLssrLjRIiUIARIhAcFEiiCwgvh2CwGJrnfnBCvuNl9u4BgngCVVKihSxYahIiUIARIhAVPQq6npMIxTVF19htERZGPpp0TZZaNLGho3+Y1oBFLg"
              }
            ]
          },
          "height": "7355",
          "codespace": ""
        }
        "#;

        let abci_query: AbciQuery = serde_json::from_str(abci_query_response)
            .expect("deserialization failure for hardcoded response");
        let proof: RawCwProof = abci_query
            .try_into()
            .expect("hardcoded response does not include proof");
        let root = "25a8b485e0ff095f7b60a1aab837d65756c9a4cdc216bae7ba9c59b3fb28fbec";

        CwProof::from(proof)
            .verify(hex::decode(root).expect("invalid hex"))
            .expect("");
    }

    #[test]
    fn test_query_map() {
        let abci_query_response = r#"{
          "code": 0,
          "log": "",
          "info": "",
          "index": "0",
          "key": "A63kpfWAOkOYNcY2OVqNZI3uV7L8kNmNwX+ohxWbaWOLAAhyZXF1ZXN0czQyNWQ4N2Y4NjIwZTFkZWRlZWU3MDU5MGNjNTViMTY0YjhmMDE0ODBlZTU5ZTBiMWRhMzU0MzZhMmY3YzI3Nzc=",
          "value": "eyJqb2luX2NvbXB1dGVfbm9kZSI6WyIwM0U2N0VGMDkyMTM2MzMwNzRGQjRGQkYzMzg2NDNGNEYwQzU3NEVENjBFRjExRDAzNDIyRUVCMDZGQTM4QzhGM0YiLCJ3YXNtMTBuNGRzbGp5eWZwMmsyaHk2ZTh2dWM5cnkzMnB4MmVnd3Q1ZTBtIl19",
          "proof": {
            "ops": [
              {
                "field_type": "ics23:iavl",
                "key": "A63kpfWAOkOYNcY2OVqNZI3uV7L8kNmNwX+ohxWbaWOLAAhyZXF1ZXN0czQyNWQ4N2Y4NjIwZTFkZWRlZWU3MDU5MGNjNTViMTY0YjhmMDE0ODBlZTU5ZTBiMWRhMzU0MzZhMmY3YzI3Nzc=",
                "data": "CrwDCmsDreSl9YA6Q5g1xjY5Wo1kje5XsvyQ2Y3Bf6iHFZtpY4sACHJlcXVlc3RzNDI1ZDg3Zjg2MjBlMWRlZGVlZTcwNTkwY2M1NWIxNjRiOGYwMTQ4MGVlNTllMGIxZGEzNTQzNmEyZjdjMjc3NxKKAXsiam9pbl9jb21wdXRlX25vZGUiOlsiMDNFNjdFRjA5MjEzNjMzMDc0RkI0RkJGMzM4NjQzRjRGMEM1NzRFRDYwRUYxMUQwMzQyMkVFQjA2RkEzOEM4RjNGIiwid2FzbTEwbjRkc2xqeXlmcDJrMmh5NmU4dnVjOXJ5MzJweDJlZ3d0NWUwbSJdfRoMCAEYASABKgQAApBnIioIARImAgSQZyDcejBg60yYaDEvKvExWQf9XKfIaNU/Amt6hqCn7y+CSiAiKggBEiYEBpBnICanihuey/DZHbttCL13YV1SMnCD6D6J2zssxb7sqwrlICIsCAESBQYMkGcgGiEg+89wQcyopgtcMvQ2ceLVOsi6b3IcMYCR2UZrrqAV1xsiLAgBEgUIGpBnIBohIAZfzUkdenhtbw3pyQOhhBCfcXEL/QcMb/MMoQC+SS8B"
              },
              {
                "field_type": "ics23:simple",
                "key": "d2FzbQ==",
                "data": "CqgBCgR3YXNtEiDYWxn2B9M/eGP18Gwl3zgWZkT7Yn/iFlcS0THfmfcfDBoJCAEYASABKgEAIiUIARIhAWLU8PgnJ/EMp4BYvtTN9MX/rS70dNQ3ZAzrJLssrLjRIiUIARIhARcbvq+IA7uFZQ37EHO4TUVW33UPw2gl4PnFAPf/w+LDIiUIARIhAZIp1f1XIqpz3QSNX3F9i7IGdc8DHeSpBJ/Qhg3httiR"
              }
            ]
          },
          "height": "7589",
          "codespace": ""
        }
        "#;

        let abci_query: AbciQuery = serde_json::from_str(abci_query_response)
            .expect("deserialization failure for hardcoded response");
        let proof: RawCwProof = abci_query
            .try_into()
            .expect("hardcoded response does not include proof");
        let root = "632612de75657f50bbb769157bf0ef8dd417409b367b0204bbda4529ab2b2d4f";

        CwProof::from(proof)
            .verify(hex::decode(root).expect("invalid hex"))
            .expect("");
    }
}
