use anyhow::anyhow;
use cosmrs::AccountId;
use regex::Regex;
use serde::de::DeserializeOwned;
use subtle_encoding::bech32::decode as bech32_decode;
use std::{path::Path, process::Command, time::Duration};
use tendermint_rpc::{error::ErrorDetail, Client, HttpClient, endpoint::tx::Response as TmTxResponse};
use tendermint::Hash;

pub fn wasmaddr_to_id(address_str: &str) -> anyhow::Result<AccountId> {
    let (hr, _) = bech32_decode(address_str).map_err(|e| anyhow!(e))?;
    if hr != "wasm" {
        return Err(anyhow!(hr));
    }

    Ok(address_str.parse().unwrap())
}

// TODO: move wrapping result with "quartz:" struct into here
pub fn run_relay<R: DeserializeOwned>(
    base_path: &Path,
    msg: &str,
    arg: Option<&str>,
) -> Result<R, anyhow::Error> {
    let relayer_path = base_path.join("packages/relayer/scripts/relay.sh");

    let mut bash = Command::new("bash");
    let command = bash.arg(relayer_path).arg(msg);

    if let Some(arg) = arg {
        command.arg(arg);
    }

    let output = command.output()?;

    if !output.status.success() {
        return Err(anyhow!("{:?}", output));
    }

    let query_result: R = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("Error deserializing: {}", e))?;

    Ok(query_result)
}

// Note: time until tx commit is empiraclly 800ms on DO wasmd chain.
pub async fn block_tx_commit(client: &HttpClient, tx: Hash) -> Result<TmTxResponse, anyhow::Error> {
    let re = Regex::new(r"tx \([A-F0-9]{64}\) not found").unwrap();

    tokio::time::sleep(Duration::from_millis(400)).await;
    loop {
        match client.tx(tx, false).await {
            Ok(resp) => {
                return Ok(resp);
            }, 
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
                            println!("🔗 Waiting for tx commit... (+400ms)");
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