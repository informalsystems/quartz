use async_trait::async_trait;
use tracing::trace;

use std::{
    collections::BTreeMap, path::Path, process::Command
};

use anyhow::anyhow;
use base64::prelude::*;
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use cosmwasm_std::{Binary, HexBinary, Uint64};
use cw_tee_mtcs::msg::{
    execute::SubmitSetoffsMsg, AttestedMsg, ExecuteMsg, GetLiquiditySourcesResponse,
    QueryMsg::GetLiquiditySources,
};
use cycles_sync::wasmd_client::{CliWasmdClient, QueryResult, WasmdClient};
use futures_util::stream::StreamExt;
use mtcs_enclave::{proto::{clearing_client::ClearingClient, RunClearingRequest}, types::RunClearingMessage};
use quartz_common::contract::msg::execute::attested::{
    EpidAttestation, RawAttested, RawAttestedMsgSansHandler,
};
use quartz_tee_ra::{intel_sgx::epid::types::ReportBody, IASReport};
use reqwest::Url;
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

use crate::{
    cli::Verbosity,
    error::Error,
    handler::Handler,
    request::listen::ListenRequest,
    response::listen::ListenResponse,
};

#[async_trait]
impl Handler for ListenRequest {
    type Error = Error;
    type Response = ListenResponse;

    async fn handle(self, _verbosity: Verbosity) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        listen(self)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        Ok(Self::Response::from(ListenResponse::default()))
    }
}

async fn listen(args: ListenRequest) -> Result<(), anyhow::Error> {
    // Subscribe to "init_clearing" events
    let wsurl = format!("ws://{}/websocket", args.node_url);
    let (client, driver) = WebSocketClient::new(wsurl.as_str()).await.unwrap();
    let driver_handle = tokio::spawn(async move { driver.run().await });

    let mut subs = client
        .subscribe(Query::from(EventType::Tx).and_contains("wasm.action", "init_clearing"))
        .await
        .unwrap();

    while subs.next().await.is_some() {
        // On init_clearing, run process
        if let Err(e) = handler(
            &args.contract,
            args.sender.clone(),
            args.chain_id.clone(),
            format!("{}:{}", args.rpc_addr, args.port),
            &args.node_url,
            args.path.as_path(),
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
    chain_id: ChainId,
    rpc_addr: String,
    node_url: &str,
    path: &Path,
) -> Result<(), anyhow::Error> {
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
    let attestation = gramine_ias_request(quote.attestation, path).await?;
    let msg = RawAttestedMsgSansHandler(quote.msg);

    let setoffs_msg =
        ExecuteMsg::SubmitSetoffs::<EpidAttestation>(AttestedMsg { msg, attestation });

    // Send setoffs to mtcs contract on chain
    let output =
        wasmd_client.tx_execute(contract, &chain_id, 2000000, sender, json!(setoffs_msg))?;

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

// TODO: Utilize relay rust package
// Request the IAS report for EPID attestations
async fn gramine_ias_request(
    attested_msg: Vec<u8>,
    path: &Path,
) -> Result<EpidAttestation, anyhow::Error> {
    let ias_api_key = String::from("669244b3e6364b5888289a11d2a1726d");
    let ra_client_spid = String::from("51CAF5A48B450D624AEFE3286D314894");
    let quote_file = path.join("/tmp/test.quote");
    let report_file = path.join("/tmp/datareport");
    let report_sig_file = path.join("/tmp/datareportsig");

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
        .args(["-q", &quote_file.display().to_string()])
        .args(["-r", &report_file.display().to_string()])
        .args(["-s", &report_sig_file.display().to_string()]);

    let output = command.output()?;
    if !output.status.success() {
        return Err(anyhow!("Couldn't run gramine. {:?}", output));
    }

    let report: ReportBody = serde_json::from_str(&fs::read_to_string(report_file).await?)?;
    let report_sig_str = fs::read_to_string(report_sig_file).await?.replace('\r', "");
    let report_sig: Binary = Binary::from_base64(report_sig_str.trim())?;

    Ok(EpidAttestation::new(IASReport { report, report_sig }))
}
