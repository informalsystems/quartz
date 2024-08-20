use std::path::PathBuf;

use async_trait::async_trait;
use color_eyre::owo_colors::OwoColorize;
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use quartz_common::contract::{
    msg::execute::attested::{RawEpidAttestation, RawMockAttestation},
    prelude::QuartzInstantiateMsg,
};
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tendermint_rpc::HttpClient;
use tracing::{debug, info};

use super::utils::{
    helpers::{block_tx_commit, run_relay},
    types::{Log, WasmdTxResponse},
};
use crate::{
    config::Config,
    error::Error,
    handler::{utils::types::RelayMessage, Handler},
    request::contract_deploy::ContractDeployRequest,
    response::{contract_deploy::ContractDeployResponse, Response},
};

#[async_trait]
impl Handler for ContractDeployRequest {
    type Error = Error;
    type Response = Response;

    async fn handle<C: AsRef<Config> + Send>(
        self,
        config: C,
    ) -> Result<Self::Response, Self::Error> {
        let config = config.as_ref();
        info!("{}", "\nPeforming Contract Deploy".blue().bold());

        let (code_id, contract_addr) = if config.mock_sgx {
            deploy::<RawMockAttestation>(self, config)
                .await
                .map_err(|e| Error::GenericErr(e.to_string()))?
        } else {
            deploy::<RawEpidAttestation>(self, config)
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
    config: &Config,
) -> Result<(u64, String), anyhow::Error> {
    // TODO: Replace with call to Rust package
    let relay_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..");

    let httpurl = Url::parse(&format!("http://{}", config.node_url))?;
    let tmrpc_client = HttpClient::new(httpurl.as_str())?;
    let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    info!("🚀 Deploying {} Contract", args.label);
    let contract_path = args.wasm_bin_path;

    let code_id = if config.contract_has_changed(&contract_path).await? {
        let deploy_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.deploy(
            &config.chain_id,
            &config.tx_sender,
            contract_path.display().to_string(),
        )?)?;
        let res = block_tx_commit(&tmrpc_client, deploy_output.txhash).await?;

        let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
        let code_id: u64 = log[0].events[1].attributes[1].value.parse()?;
        config.save_codeid_to_cache(&contract_path, code_id).await?;

        code_id
    } else {
        config.get_cached_codeid(&contract_path).await?
    };

    info!("🚀 Communicating with Relay to Instantiate...");
    let raw_init_msg = run_relay::<QuartzInstantiateMsg<DA>>(
        relay_path.as_path(),
        config.mock_sgx,
        RelayMessage::Instantiate,
    )
    .await?;

    info!("🚀 Instantiating {}", args.label);
    let mut init_msg = args.init_msg;
    init_msg["quartz"] = json!(raw_init_msg);

    let init_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.init(
        &config.chain_id,
        &config.tx_sender,
        code_id,
        json!(init_msg),
        &format!("{} Contract #{}", args.label, code_id),
    )?)?;
    let res = block_tx_commit(&tmrpc_client, init_output.txhash).await?;

    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
    let contract_addr: &String = &log[0].events[1].attributes[0].value;

    info!("🚀 Successfully deployed and instantiated contract!");
    info!("🆔 Code ID: {}", code_id);
    info!("📌 Contract Address: {}", contract_addr);

    debug!("{contract_addr}");

    Ok((code_id, contract_addr.to_owned()))
}

//RES=$($CMD tx wasm instantiate "$CODE_ID" "$INSTANTIATE_MSG" --from "$USER_ADDR" --label $LABEL $TXFLAG -y --no-admin --output json)
