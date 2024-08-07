use std::env::current_dir;

use async_trait::async_trait;
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use quartz_common::contract::{
    msg::execute::attested::{RawEpidAttestation, RawMockAttestation},
    prelude::QuartzInstantiateMsg,
};
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tendermint_rpc::HttpClient;
use tracing::{debug, info, trace};

use super::utils::{
    helpers::{block_tx_commit, run_relay},
    types::{Log, WasmdTxResponse},
};
use crate::{
    error::Error,
    handler::{utils::types::RelayMessage, Handler},
    request::contract_deploy::ContractDeployRequest,
    response::{contract_deploy::ContractDeployResponse, Response},
    Config,
};

#[async_trait]
impl Handler for ContractDeployRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, config: Config) -> Result<Self::Response, Self::Error> {
        trace!("initializing directory structure...");

        let (code_id, contract_addr) = if config.mock_sgx {
            deploy::<RawMockAttestation>(self, config.mock_sgx)
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
        } else {
            deploy::<RawEpidAttestation>(self, config.mock_sgx)
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
        };

        Ok(ContractDeployResponse {
            code_id,
            contract_addr,
        }
        .into())
    }
}

async fn deploy<DA: Serialize + DeserializeOwned>(
    args: ContractDeployRequest,
    mock_sgx: bool,
) -> Result<(u64, String), anyhow::Error> {
    // TODO: Replace with call to Rust package
    let relay_path = current_dir()?.join("../");

    let httpurl = Url::parse(&format!("http://{}", args.node_url))?;
    let tmrpc_client = HttpClient::new(httpurl.as_str())?;
    let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    info!("\n🚀 Deploying {} Contract\n", args.label);
    let contract_path = args.wasm_bin_path;
    // .join("contracts/cw-tee-mtcs/target/wasm32-unknown-unknown/release/cw_tee_mtcs.wasm");

    // TODO: uncertain about the path -> string conversion
    let deploy_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.deploy(
        &args.chain_id,
        &args.sender,
        contract_path.display().to_string(),
    )?)?;
    let res = block_tx_commit(&tmrpc_client, deploy_output.txhash).await?;

    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
    let code_id: usize = log[0].events[1].attributes[1].value.parse()?;

    info!("\n🚀 Communicating with Relay to Instantiate...\n");
    let raw_init_msg = run_relay::<QuartzInstantiateMsg<DA>>(
        relay_path.as_path(),
        mock_sgx,
        RelayMessage::Instantiate,
    ).await?;

    info!("\n🚀 Instantiating {} Contract\n", args.label);
    let mut init_msg = args.init_msg;
    init_msg["quartz"] = json!(raw_init_msg);

    let init_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.init(
        &args.chain_id,
        &args.sender,
        code_id,
        json!(init_msg),
        &format!("{} Contract #{}", args.label, code_id),
    )?)?;
    let res = block_tx_commit(&tmrpc_client, init_output.txhash).await?;

    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
    let contract_addr: &String = &log[0].events[1].attributes[0].value;

    info!("\n🚀 Successfully deployed and instantiated contract!");
    info!("\n🆔 Code ID: {}", code_id);
    info!("\n📌 Contract Address: {}", contract_addr);

    debug!("{contract_addr}");

    Ok((code_id as u64, contract_addr.to_owned()))
}

//RES=$($CMD tx wasm instantiate "$CODE_ID" "$INSTANTIATE_MSG" --from "$USER_ADDR" --label $LABEL $TXFLAG -y --no-admin --output json)
