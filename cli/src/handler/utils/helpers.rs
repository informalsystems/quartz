use std::time::Duration;

use anyhow::anyhow;
use cosmrs::{AccountId, ErrorReport};
use quartz_common::{
    contract::msg::{
        execute::{session_create::RawSessionCreate, session_set_pub_key::RawSessionSetPubKey},
        instantiate::RawCoreInstantiate,
    },
    proto::{
        core_client::CoreClient, InstantiateRequest, SessionCreateRequest, SessionSetPubKeyRequest,
    },
};
use regex::Regex;
use reqwest::Url;
use serde::de::DeserializeOwned;
use serde_json::json;
use subtle_encoding::bech32::decode as bech32_decode;
use tendermint::{block::Height, Hash};
use tendermint_rpc::{
    endpoint::tx::Response as TmTxResponse, error::ErrorDetail, Client, HttpClient,
};
use tokio::fs;
use tracing::debug;
use wasmd_client::{CliWasmdClient, WasmdClient};

use super::types::RelayMessage;
use crate::{config::Config, error};

pub fn wasmaddr_to_id(address_str: &str) -> Result<AccountId, anyhow::Error> {
    let (hr, _) = bech32_decode(address_str).map_err(|e| anyhow!(e))?;
    if hr != "wasm" {
        return Err(anyhow!(hr));
    }

    address_str.parse().map_err(|e: ErrorReport| anyhow!(e))
}

// TODO: move wrapping result with "quartz:" struct into here
pub async fn run_relay_rust<R: DeserializeOwned>(
    enclave_rpc: String,
    mock_sgx: bool,
    relay_msg: RelayMessage,
) -> Result<R, anyhow::Error> {
    // Query the gRPC quartz enclave service
    let mut qc_client = CoreClient::connect(enclave_rpc).await?;

    let attested_msg = match &relay_msg {
        RelayMessage::Instantiate => &qc_client
            .instantiate(tonic::Request::new(InstantiateRequest {}))
            .await?
            .get_ref()
            .message
            .clone(),
        RelayMessage::SessionCreate => &qc_client
            .session_create(tonic::Request::new(SessionCreateRequest {}))
            .await?
            .get_ref()
            .message
            .clone(),
        RelayMessage::SessionSetPubKey(proof) => &qc_client
            .session_set_pub_key(SessionSetPubKeyRequest {
                message: proof.to_string(),
            })
            .await?
            .get_ref()
            .message
            .clone(),
    };
    let mut attested_msg_json: serde_json::Value = serde_json::from_str(attested_msg)?;
    let quote = attested_msg_json["quote"].take();

    if mock_sgx {
        match relay_msg {
            RelayMessage::Instantiate => {
                // Construct CoreInstantiate
                let msg: RawCoreInstantiate = serde_json::from_value(attested_msg_json)?;

                let query_result: R = serde_json::from_value(json!({
                    "msg": RawCoreInstantiate::from(msg),
                    "attestation": quote
                }))?;

                return Ok(query_result);
            }
            RelayMessage::SessionCreate => {
                // Convert RelayMessage to a snake_case string
                let request_key = relay_msg.to_string();

                let msg: RawSessionCreate = serde_json::from_value(attested_msg_json)?;
                // Build the nested JSON structures
                let inner_json = json!({
                    "msg": msg,
                    "attestation": quote
                });

                let quartz_json = json!({
                    request_key: inner_json
                });

                let jsonify = json!({
                    "quartz": quartz_json
                });

                let query_result: R = serde_json::from_value(jsonify)?;

                return Ok(query_result);
            }
            RelayMessage::SessionSetPubKey(_) => {
                // Convert RelayMessage to a snake_case string
                let request_key = relay_msg.to_string();

                let msg: RawSessionSetPubKey = serde_json::from_value(attested_msg_json)?;
                // Build the nested JSON structures
                let inner_json = json!({
                    "msg": msg,
                    "attestation": quote
                });

                let quartz_json = json!({
                    request_key: inner_json
                });

                let jsonify = json!({
                    "quartz": quartz_json
                });

                let query_result: R = serde_json::from_value(jsonify)?;

                return Ok(query_result);
            }
        }
    }

    todo!()
    // if mock_sgx

    // match against relaymessage variant and construct

    // else

    // docker

    // same request thing

    // if let RelayMessage::SessionSetPubKey(proof) = msg {
    //     command.arg(proof);
    // }
}

// Note: time until tx commit is empiraclly 800ms on DO wasmd chain.
pub async fn block_tx_commit(client: &HttpClient, tx: Hash) -> Result<TmTxResponse, anyhow::Error> {
    let re = Regex::new(r"tx \([A-F0-9]{64}\) not found")?;

    tokio::time::sleep(Duration::from_millis(400)).await;
    loop {
        match client.tx(tx, false).await {
            Ok(resp) => {
                return Ok(resp);
            }
            Err(e) => {
                // If error, make sure it is only because of a not yet committed tx
                match e.0 {
                    ErrorDetail::Response(subdetail) => {
                        if !re.is_match(subdetail.source.data().unwrap_or_default()) {
                            return Err(anyhow!(
                                "Error querying for tx: {}",
                                ErrorDetail::Response(subdetail)
                            ));
                        } else {
                            debug!("🔗 Waiting for tx commit... (+400ms)");
                            tokio::time::sleep(Duration::from_millis(400)).await;
                            continue;
                        }
                    }
                    _ => {
                        return Err(anyhow!("Error querying for tx: {}", e.0));
                    }
                }
            }
        }
    }
}

// Queries the chain for the latested height and hash
pub fn query_latest_height_hash(node_url: &String) -> Result<(Height, Hash), error::Error> {
    let httpurl = Url::parse(&format!("http://{}", node_url))
        .map_err(|e| error::Error::GenericErr(e.to_string()))?;
    let wasmd_client = CliWasmdClient::new(httpurl);

    let (trusted_height, trusted_hash) = wasmd_client
        .trusted_height_hash()
        .map_err(|e| error::Error::GenericErr(e.to_string()))?;

    Ok((
        trusted_height.try_into()?,
        trusted_hash.parse().expect("invalid hash from wasmd"),
    ))
}

pub async fn write_cache_hash_height(
    trusted_height: Height,
    trusted_hash: Hash,
    config: &Config,
) -> Result<(), error::Error> {
    let height_path = config.cache_dir()?.join("trusted.height");
    fs::write(height_path.as_path(), trusted_height.to_string()).await?;

    let hash_path = config.cache_dir()?.join("trusted.hash");
    fs::write(hash_path.as_path(), trusted_hash.to_string()).await?;

    Ok(())
}

pub async fn read_cached_hash_height(config: &Config) -> Result<(Height, Hash), error::Error> {
    let height_path = config.cache_dir()?.join("trusted.height");
    let hash_path = config.cache_dir()?.join("trusted.hash");

    if !height_path.exists() {
        return Err(error::Error::PathNotFile(height_path.display().to_string()));
    }
    if !hash_path.exists() {
        return Err(error::Error::PathNotFile(hash_path.display().to_string()));
    }

    let trusted_height: Height = fs::read_to_string(height_path.as_path()).await?.parse()?;
    let trusted_hash: Hash = fs::read_to_string(hash_path.as_path()).await?.parse()?;

    Ok((trusted_height, trusted_hash))
}
