use std::{path::Path, time::Duration};

use anyhow::anyhow;
use cosmrs::{AccountId, ErrorReport};
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use regex::Regex;
use reqwest::Url;
use serde::de::DeserializeOwned;
use subtle_encoding::bech32::decode as bech32_decode;
use tendermint::{block::Height, Hash};
use tendermint_rpc::{
    endpoint::tx::Response as TmTxResponse, error::ErrorDetail, Client, HttpClient,
};
use tokio::{fs, process::Command};
use tracing::debug;

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

/// Returns the trusted hash and height
pub fn get_hash_height(
    use_latest: bool,
    config: &mut Config,
) -> Result<(Height, Hash), error::Error> {
    if use_latest || config.trusted_height == 0 || config.trusted_hash.is_empty() {
        debug!("querying latest trusted hash & height from node");
        let (trusted_height, trusted_hash) = latest_height_hash(&config.node_url)?;
        config.trusted_hash = trusted_hash.to_string();
        config.trusted_height = trusted_height.into();

        Ok((trusted_height, trusted_hash))
    } else {
        debug!("reusing config trusted hash & height");
        Ok((
            config.trusted_height.try_into()?,
            config
                .trusted_hash
                .parse()
                .map_err(|_| error::Error::GenericErr("invalid hash".to_string()))?,
        ))
    }
}

// Queries the chain for the latested height and hash
pub fn latest_height_hash(node_url: &String) -> Result<(Height, Hash), error::Error> {
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

pub async fn persist_config_hash_height(config: &Config) -> Result<(), error::Error> {
    let config_path = config.app_dir.join("quartz.toml");

    let toml_content = fs::read_to_string(&config_path).await?;
    let mut written_config: Config = toml::from_str(&toml_content)?;

    written_config.trusted_hash.clone_from(&config.trusted_hash);
    written_config.trusted_height = config.trusted_height;

    let toml_string = toml::to_string(config)?;
    fs::write(&config_path, toml_string).await?;

    Ok(())
}
