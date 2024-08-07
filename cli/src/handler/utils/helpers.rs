use std::{path::Path, time::Duration};
use tokio::process::Command;

use anyhow::anyhow;
use cosmrs::{AccountId, ErrorReport};
use regex::Regex;
use serde::de::DeserializeOwned;
use subtle_encoding::bech32::decode as bech32_decode;
use tendermint::Hash;
use tendermint_rpc::{
    endpoint::tx::Response as TmTxResponse, error::ErrorDetail, Client, HttpClient,
};
use tracing::debug;

use super::types::RelayMessage;

pub fn wasmaddr_to_id(address_str: &str) -> Result<AccountId, anyhow::Error> {
    let (hr, _) = bech32_decode(address_str).map_err(|e| anyhow!(e))?;
    if hr != "wasm" {
        return Err(anyhow!(hr));
    }

    address_str.parse().map_err(|e: ErrorReport| anyhow!(e))
}

// TODO: move wrapping result with "quartz:" struct into here
pub async fn run_relay<R: DeserializeOwned>(
    base_path: &Path,
    mock_sgx: bool,
    msg: RelayMessage,
) -> Result<R, anyhow::Error> {
    let relayer_path = base_path.join("relayer/scripts/relay.sh");

    let mut bash = Command::new("bash");
    let command = bash
        .arg(relayer_path)
        .arg(msg.to_string())
        .env("MOCK_SGX", mock_sgx.to_string());

    if let RelayMessage::SessionSetPubKey(proof) = msg {
        command.arg(proof);
    }

    let output = command.output().await?;

    if !output.status.success() {
        return Err(anyhow!("{:?}", output));
    }

    let query_result: R = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("Error deserializing: {}", e))?;

    Ok(query_result)
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
