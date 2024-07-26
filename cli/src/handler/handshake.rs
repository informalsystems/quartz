use tracing::trace;
use std::{env::current_dir, fs::File, io::Read, path::Path, str::FromStr};

use async_trait::async_trait;
use anyhow::anyhow;
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

use crate::handler::utils::{helpers::{run_relay, block_tx_commit}, types::WasmdTxResponse};

use crate::{
    cli::Verbosity, error::Error, handler::Handler, request::handshake::HandshakeRequest,
    response::handshake::HandshakeResponse,
};

#[async_trait]
impl Handler for HandshakeRequest {
    type Error = Error;
    type Response = HandshakeResponse;

    async fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        trace!("starting handshake...");

        handshake(self).await.map_err(|e| Error::GenericErr(e.to_string()))?;

        Ok(Self::Response::from(HandshakeResponse::default()))
    }
}

#[derive(Serialize)]
struct Message<'a> {
    message: &'a str,
}

async fn handshake(args: HandshakeRequest) -> Result<(), anyhow::Error> {
    let httpurl = Url::parse(&format!("http://{}", args.node_url))?;
    let wsurl = format!("ws://{}/websocket", args.node_url);

    let tmrpc_client = HttpClient::new(httpurl.as_str()).unwrap();
    let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    // TODO: dir logic issue #125
    // Read trusted hash and height from files
    let base_path = current_dir()?.join("../");
    let trusted_files_path = base_path.join("apps/mtcs/");
    let (trusted_height, trusted_hash) = read_hash_height(trusted_files_path.as_path()).await?;

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
    println!("\n\n SessionCreate tx output: {:?}", output);

    // Wait for tx to commit
    let tx_hash = Hash::from_str(&output.txhash).expect("Invalid hex string for transaction hash");
    block_tx_commit(&tmrpc_client, tx_hash).await?;

    // Wait 2 blocks
    two_block_waitoor(&wsurl).await?;

    // TODO: dir logic issue #125
    let proof_path = current_dir()?.join("../utils/tm-prover/light-client-proof.json");
    println!("Proof path: {:?}", proof_path.to_str());

    // Call tm prover with trusted hash and height
    let mut config = TmProverConfig::default();
    config.chain_id = "testing".parse()?;
    config.primary = httpurl.as_str().parse()?;
    config.witnesses = httpurl.as_str().parse()?;
    config.trusted_height = trusted_height;
    config.trusted_hash = trusted_hash;
    config.trace_file = Some(proof_path.clone());
    config.verbose = "1".parse()?;
    config.contract_address = args.contract.clone();
    config.storage_key = "quartz_session".to_owned();

    if let Err(report) = prove(config).await {
        return Err(anyhow!("Tendermint prover failed. Report: {}", report));
    }

    // Read proof file
    let proof = read_file(proof_path.as_path()).await?;
    let json_msg = serde_json::to_string(&Message { message: &proof })?;

    // Execute SessionSetPubKey on enclave
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

    println!("\n\n SessionSetPubKey tx output: {:?}", output);

    // Wait for tx to commit
    let tx_hash = Hash::from_str(&output.txhash).expect("Invalid hex string for transaction hash");
    block_tx_commit(&tmrpc_client, tx_hash).await?;

    if let MtcsExecuteMsg::Quartz(QuartzExecuteMsg::RawSessionSetPubKey(quartz)) = res {
        println!("\n\n\n{}", quartz.msg.pub_key); // TODO: return this instead later
    } else {
        return Err(anyhow!("Invalid relay response from SessionSetPubKey"));
    }

    Ok(())
}

async fn two_block_waitoor(wsurl: &str) -> Result<(), anyhow::Error> {
    let (client, driver) = WebSocketClient::new(wsurl).await.unwrap();

    let driver_handle = tokio::spawn(async move { driver.run().await });

    // Subscription functionality
    let mut subs = client.subscribe(EventType::NewBlock.into()).await.unwrap();

    // Wait 2 NewBlock events
    let mut ev_count = 5_i32;

    while let Some(res) = subs.next().await {
        let ev = res.unwrap();
        println!("Got event: {:?}", ev);
        ev_count -= 1;
        if ev_count < 0 {
            break;
        }
    }

    // Signal to the driver to terminate.
    client.close().unwrap();
    // Await the driver's termination to ensure proper connection closure.
    let _ = driver_handle.await.unwrap();

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
