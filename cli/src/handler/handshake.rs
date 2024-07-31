use std::{env::current_dir, fs::File, io::Read, path::Path, str::FromStr};

use anyhow::anyhow;
use async_trait::async_trait;
use cosmrs::tendermint::chain::Id as ChainId; // TODO see if this redundancy in dependencies can be decreased
use cw_tee_mtcs::msg::ExecuteMsg as MtcsExecuteMsg;
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use futures_util::stream::StreamExt;
use quartz_common::contract::prelude::QuartzExecuteMsg;
use reqwest::Url;
use serde::Serialize;
use serde_json::json;
use tendermint::{block::Height, Hash};
use tendermint_rpc::{query::EventType, HttpClient, SubscriptionClient, WebSocketClient};
use tm_prover::{config::Config as TmProverConfig, prover::prove};
use tracing::trace;

use super::utils::{
    helpers::{block_tx_commit, run_relay},
    types::WasmdTxResponse,
};
use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::handshake::HandshakeRequest,
    response::{handshake::HandshakeResponse, Response},
};

#[async_trait]
impl Handler for HandshakeRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        trace!("starting handshake...");

        handshake(self, verbosity)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        Ok(HandshakeResponse.into())
    }
}

#[derive(Serialize)]
struct Message<'a> {
    message: &'a str,
}

async fn handshake(args: HandshakeRequest, _verbosity: Verbosity) -> Result<(), anyhow::Error> {
    let httpurl = Url::parse(&format!("http://{}", args.node_url))?;
    let wsurl = format!("ws://{}/websocket", args.node_url);

    let tmrpc_client = HttpClient::new(httpurl.as_str())?;
    let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    // TODO: dir logic issue #125
    // Read trusted hash and height from files
    let base_path = current_dir()?.join("../");
    let trusted_files_path = base_path.join("apps/mtcs/");
    let (trusted_height, trusted_hash) = read_hash_height(trusted_files_path.as_path()).await?;

    println!("Running SessionCreate");
    let res: MtcsExecuteMsg = run_relay(base_path.as_path(), "SessionCreate", None)?;

    let output: WasmdTxResponse = serde_json::from_str(
        wasmd_client
            .tx_execute(
                &args.contract.clone(),
                &args.chain_id,
                2000000,
                args.sender.clone(),
                json!(res),
            )?
            .as_str(),
    )?;
    // println!("\n\n SessionCreate tx output: {:?}", output);

    // Wait for tx to commit
    block_tx_commit(&tmrpc_client, output.txhash).await?;
    println!("SessionCreate tx committed");

    // Wait 2 blocks
    println!("Waiting 2 blocks for light client proof");
    two_block_waitoor(&wsurl).await?;

    // TODO: dir logic issue #125
    let proof_path = current_dir()?.join("../utils/tm-prover/light-client-proof.json");
    println!("Proof path: {:?}", proof_path.to_str());

    // Call tm prover with trusted hash and height
    let config = TmProverConfig {
        primary: httpurl.as_str().parse()?,
        witnesses: httpurl.as_str().parse()?,
        trusted_height,
        trusted_hash,
        trace_file: Some(proof_path.clone()),
        verbose: "1".parse()?, // TODO: both tm-prover and cli define the same Verbosity struct. Need to define this once and import
        contract_address: args.contract.clone(),
        storage_key: "quartz_session".to_string(),
        chain_id: args.chain_id.to_string(),
        ..Default::default()
    };
    println!("config: {:?}", config);
    if let Err(report) = prove(config).await {
        return Err(anyhow!("Tendermint prover failed. Report: {}", report));
    }

    // Read proof file
    let proof = read_file(proof_path.as_path()).await?;
    let json_msg = serde_json::to_string(&Message { message: &proof })?;

    // Execute SessionSetPubKey on enclave
    println!("Running SessionSetPubKey");
    let res: MtcsExecuteMsg = run_relay(
        base_path.as_path(),
        "SessionSetPubKey",
        Some(json_msg.as_str()),
    )?;

    // Submit SessionSetPubKey to contract
    let output: WasmdTxResponse = serde_json::from_str(
        wasmd_client
            .tx_execute(
                &args.contract.clone(),
                &ChainId::from_str("testing")?,
                2000000,
                args.sender.clone(),
                json!(res),
            )?
            .as_str(),
    )?;

    // println!("\n\n SessionSetPubKey tx output: {:?}", output);

    // Wait for tx to commit
    block_tx_commit(&tmrpc_client, output.txhash).await?;
    println!("SessionSetPubKey tx committed");

    if let MtcsExecuteMsg::Quartz(QuartzExecuteMsg::RawSessionSetPubKey(quartz)) = res {
        println!("\n\n\n{}", quartz.msg.pub_key()); // TODO: return this instead later
    } else {
        return Err(anyhow!("Invalid relay response from SessionSetPubKey"));
    }

    Ok(())
}

async fn two_block_waitoor(wsurl: &str) -> Result<(), anyhow::Error> {
    let (client, driver) = WebSocketClient::new(wsurl).await?;

    let driver_handle = tokio::spawn(async move { driver.run().await });

    // Subscription functionality
    let mut subs = client.subscribe(EventType::NewBlock.into()).await?;

    // Wait 2 NewBlock events
    let mut ev_count = 2_i32;
    println!("Blocks left: {ev_count} ...");

    while let Some(res) = subs.next().await {
        let _ev = res?;
        ev_count -= 1;
        println!("Blocks left: {ev_count} ...");
        if ev_count == 0 {
            break;
        }
    }

    // Signal to the driver to terminate.
    client.close()?;
    // Await the driver's termination to ensure proper connection closure.
    let _ = driver_handle.await?;

    Ok(())
}

async fn read_hash_height(base_path: &Path) -> Result<(Height, Hash), anyhow::Error> {
    let height_path = base_path.join("trusted.height");
    let trusted_height: Height = read_file(height_path.as_path()).await?.parse()?;

    let hash_path = base_path.join("trusted.hash");
    let trusted_hash: Hash = read_file(hash_path.as_path()).await?.parse()?;

    Ok((trusted_height, trusted_hash))
}

async fn read_file(path: &Path) -> Result<String, anyhow::Error> {
    // Open the file
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(e) => {
            return Err(anyhow!(format!("Error opening file {:?}: {:?}", path, e)));
        }
    };

    // Read the file contents into a string
    let mut value = String::new();
    if let Err(e) = file.read_to_string(&mut value) {
        return Err(anyhow!(format!("Error reading file {:?}: {:?}", file, e)));
    }

    Ok(value.trim().to_owned())
}
