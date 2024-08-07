use std::{collections::BTreeMap, env, str::FromStr};

use anyhow::anyhow;
use base64::prelude::*;
use clap::Parser;
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use cosmwasm_std::{HexBinary, Uint64};
use cw_tee_mtcs::msg::{
    execute::SubmitSetoffsMsg, AttestedMsg, ExecuteMsg, GetLiquiditySourcesResponse,
    QueryMsg::GetLiquiditySources,
};
use cycles_sync::wasmd_client::{CliWasmdClient, QueryResult, WasmdClient};
use futures_util::stream::StreamExt;
use mtcs_enclave::{
    proto::{clearing_client::ClearingClient, RunClearingRequest},
    types::RunClearingMessage,
};
use quartz_common::contract::msg::execute::attested::{
    RawAttested, RawAttestedMsgSansHandler, RawDcapAttestation,
};
use reqwest::Url;
use scripts::utils::wasmaddr_to_id;
use serde_json::json;
use tendermint_rpc::{
    query::{EventType, Query},
    SubscriptionClient, WebSocketClient,
};
use tonic::Request;

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Contract to listen to
    #[arg(short, long, value_parser = wasmaddr_to_id)]
    contract: AccountId,
    /// Port enclave is listening on
    #[arg(short, long, default_value = "11090")]
    port: u16,

    #[arg(
        short,
        long,
        default_value = "wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"
    )]
    sender: String,

    #[clap(long, default_value = "143.244.186.205:26657")]
    node_url: String,

    #[clap(long, default_value_t = default_rpc_addr())]
    rpc_addr: String,
}

fn default_rpc_addr() -> String {
    env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    // Subscribe to "init_clearing" events
    let wsurl = format!("ws://{}/websocket", cli.node_url);
    let (client, driver) = WebSocketClient::new(wsurl.as_str()).await.unwrap();
    let driver_handle = tokio::spawn(async move { driver.run().await });

    let mut subs = client
        .subscribe(Query::from(EventType::Tx).and_contains("wasm.action", "init_clearing"))
        .await
        .unwrap();

    while subs.next().await.is_some() {
        // On init_clearing, run process
        if let Err(e) = handler(
            &cli.contract,
            cli.sender.clone(),
            format!("{}:{}", cli.rpc_addr, cli.port),
            &cli.node_url,
        )
        .await
        {
            println!("{}", e);
        }
    }

    // Close connection
    // Await the driver's termination to ensure proper connection closure.
    client.close().unwrap();
    let _ = driver_handle.await.unwrap();

    Ok(())
}

async fn handler(
    contract: &AccountId,
    sender: String,
    rpc_addr: String,
    node_url: &str,
) -> Result<(), anyhow::Error> {
    let chain_id = &ChainId::from_str("testing")?;
    let httpurl = Url::parse(&format!("http://{}", node_url))?;
    let wasmd_client = CliWasmdClient::new(httpurl);

    // Query obligations and liquidity sources from chain
    let clearing_contents = query_chain(&wasmd_client, contract).await?;

    // Send queried data to enclave over gRPC
    let request = Request::new(RunClearingRequest {
        message: json!(clearing_contents).to_string(),
    });

    let mut client = ClearingClient::connect(rpc_addr).await?;
    let clearing_response = client
        .run(request)
        .await
        .map_err(|e| anyhow!("Failed to communicate to relayer. {e}"))?
        .into_inner();

    // Extract json from the Protobuf message
    let attested_msg: RawAttested<SubmitSetoffsMsg, RawDcapAttestation> =
        serde_json::from_str(&clearing_response.message)
            .map_err(|e| anyhow!("Error serializing SubmitSetoffs: {}", e))?;

    let setoffs_msg = ExecuteMsg::SubmitSetoffs(AttestedMsg {
        msg: RawAttestedMsgSansHandler(attested_msg.msg),
        attestation: attested_msg.attestation,
    });

    // Send setoffs to mtcs contract on chain
    let output =
        wasmd_client.tx_execute(contract, chain_id, 2000000, &sender, json!(setoffs_msg))?;

    println!("output: {}", output);
    Ok(())
}

// TODO: replace raw queries with smart
async fn query_chain(
    wasmd_client: &CliWasmdClient,
    contract: &AccountId,
) -> Result<RunClearingMessage, anyhow::Error> {
    // Get epoch counter
    let resp: QueryResult<String> = wasmd_client
        .query_raw(contract, hex::encode("epoch_counter"))
        .map_err(|e| anyhow!("Problem querying epoch: {}", e))?;
    let mut epoch_counter: usize =
        String::from_utf8(BASE64_STANDARD.decode(resp.data)?)?.parse::<usize>()?;
    if epoch_counter > 1 {
        epoch_counter -= 1;
    }

    // TODO: replace with tracer log here
    // println!("epoch: {}", epoch_counter);

    // Get obligations
    let resp: QueryResult<String> = wasmd_client
        .query_raw(
            contract,
            hex::encode(format!("{}/obligations", epoch_counter)),
        )
        .map_err(|e| anyhow!("Problem querying obligatons: {}", e))?;

    let decoded_obligs = BASE64_STANDARD.decode(resp.data)?;
    let obligations_map: BTreeMap<HexBinary, HexBinary> =
        serde_json::from_slice(&decoded_obligs).unwrap_or_default();
    // println!("obligations \n {:?}", obligations_map);
    // TODO: replace with tracer log here

    // Get liquidity sources
    let resp: QueryResult<GetLiquiditySourcesResponse> = wasmd_client
        .query_smart(
            contract,
            json!(GetLiquiditySources {
                epoch: Some(Uint64::new(epoch_counter as u64))
            }),
        )
        .map_err(|e| anyhow!("Problem querying liquidity sources: {}", e))?;

    let liquidity_sources = resp.data.liquidity_sources;
    // TODO: replace with tracer log here
    // println!("liquidity_sources \n {:?}", liquidity_sources);

    Ok(RunClearingMessage {
        intents: obligations_map,
        liquidity_sources: liquidity_sources.into_iter().collect(),
    })
}
