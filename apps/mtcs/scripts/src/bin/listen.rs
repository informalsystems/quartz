use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    process::Command,
    str::FromStr,
};

use anyhow::anyhow;
use base64::prelude::*;
use clap::Parser;
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use cosmwasm_std::{Binary, HexBinary, Uint64};
use cw_tee_mtcs::{
    msg::{
        execute::SubmitSetoffsMsg, AttestedMsg, ExecuteMsg, GetLiquiditySourcesResponse,
        QueryMsg::GetLiquiditySources,
    },
    state::LiquiditySource,
};
use cycles_sync::wasmd_client::{CliWasmdClient, QueryResult, WasmdClient};
use futures_util::stream::StreamExt;
use mtcs_enclave::proto::{clearing_client::ClearingClient, RunClearingRequest};
use mtcs_overdraft::msg::{QueryMembersResp, QueryMsg::DumpMembers};
use quartz_common::contract::msg::execute::attested::{
    EpidAttestation, RawAttested, RawAttestedMsgSansHandler,
};
use quartz_tee_ra::{intel_sgx::epid::types::ReportBody, IASReport};
use reqwest::Url;
use scripts::utils::wasmaddr_to_id;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tendermint_rpc::{
    query::{EventType, Query},
    SubscriptionClient, WebSocketClient,
};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tonic::Request;

// TODO: import this from enclave or somewhere shared
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunClearingMessage {
    intents: BTreeMap<HexBinary, HexBinary>,
    liquidity_sources: BTreeSet<LiquiditySource>,
}

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

    #[arg(short, long, default_value = "dangush")]
    user: String, // The filesys user for gramine filepaths. TODO: improve this
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

    while let Some(_) = subs.next().await {
        // On init_clearing, run process
        if let Err(e) = handler(
            &cli.contract,
            cli.sender.clone(),
            format!("{}:{}", cli.rpc_addr, cli.port),
            &cli.node_url,
            &cli.user,
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
    user: &str,
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
    let quote: RawAttested<SubmitSetoffsMsg, Vec<u8>> =
        serde_json::from_str(&clearing_response.message)
            .map_err(|e| anyhow!("Error serializing SubmitSetoffs: {}", e))?;

    // Get IAS report and build attested message
    let attestation = gramine_ias_request(quote.attestation, &user).await?;
    let msg = RawAttestedMsgSansHandler(quote.msg);

    let setoffs_msg = ExecuteMsg::SubmitSetoffs::<EpidAttestation>(AttestedMsg {
        msg,
        attestation: attestation.into(),
    });

    // Send setoffs to mtcs contract on chain
    let output =
        wasmd_client.tx_execute(&contract, chain_id, 2000000, sender, json!(setoffs_msg))?;

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

// Request the IAS report for EPID attestations
async fn gramine_ias_request(
    attested_msg: Vec<u8>,
    user: &str,
) -> Result<EpidAttestation, anyhow::Error> {
    let ias_api_key = String::from("669244b3e6364b5888289a11d2a1726d");
    let ra_client_spid = String::from("51CAF5A48B450D624AEFE3286D314894");
    let quote_file = format!("/tmp/{}_test.quote", user);
    let report_file = format!("/tmp/{}_datareport", user);
    let report_sig_file = format!("/tmp/{}_datareportsig", user);

    // Write the binary data to a file
    let mut file = File::create(&quote_file).await?;
    file.write_all(&attested_msg)
        .await
        .map_err(|e| anyhow!("Couldn't write to file. {e}"))?;

    let mut gramine = Command::new("gramine-sgx-ias-request");
    let command = gramine
        .arg("report")
        .args(["-g", &ra_client_spid])
        .args(["-k", &ias_api_key])
        .args(["-q", &quote_file])
        .args(["-r", &report_file])
        .args(["-s", &report_sig_file]);

    let output = command.output()?;
    if !output.status.success() {
        return Err(anyhow!("Couldn't run gramine. {:?}", output));
    }

    let report: ReportBody = serde_json::from_str(&fs::read_to_string(report_file).await?)?;
    let report_sig_str = fs::read_to_string(report_sig_file).await?.replace('\r', "");
    let report_sig: Binary = Binary::from_base64(report_sig_str.trim())?;

    Ok(EpidAttestation::new(IASReport { report, report_sig }))
}
