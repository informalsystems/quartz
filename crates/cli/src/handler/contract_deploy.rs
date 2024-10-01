use std::path::Path;

use async_trait::async_trait;
use cargo_metadata::MetadataCommand;
use color_eyre::owo_colors::OwoColorize;
use cw_client::{CliWasmdClient, CwClient};
use reqwest::Url;
use serde_json::json;
use tendermint_rpc::HttpClient;
use tracing::{debug, info};

use super::utils::{
    helpers::block_tx_commit,
    types::{Log, WasmdTxResponse},
};
use crate::{
    config::Config,
    error::Error,
    handler::{utils::relay::RelayMessage, Handler},
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

        // Get contract package name in snake_case
        let package_name = MetadataCommand::new()
            .manifest_path(&self.contract_manifest)
            .exec()
            .map_err(|e| Error::GenericErr(e.to_string()))?
            .root_package()
            .ok_or("No root package found in the metadata")
            .map_err(|e| Error::GenericErr(e.to_string()))?
            .name
            .clone()
            .replace('-', "_");

        let wasm_bin_path = config
            .app_dir
            .join("target/wasm32-unknown-unknown/release")
            .join(package_name)
            .with_extension("wasm");

        let (code_id, contract_addr) = deploy(wasm_bin_path.as_path(), self, config)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        Ok(ContractDeployResponse {
            code_id,
            contract_addr,
        }
        .into())
    }
}

async fn deploy(
    wasm_bin_path: &Path,
    args: ContractDeployRequest,
    config: &Config,
) -> Result<(u64, String), anyhow::Error> {
    let httpurl = Url::parse(&format!("http://{}", config.node_url))?;
    let tmrpc_client = HttpClient::new(httpurl.as_str())?;
    let cw_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    info!("🚀 Deploying {} Contract", args.label);
    let code_id = if config.contract_has_changed(wasm_bin_path).await? {
        let deploy_output: WasmdTxResponse = serde_json::from_str(&cw_client.deploy(
            &config.chain_id,
            &config.tx_sender,
            wasm_bin_path.display().to_string(),
        )?)?;
        let res = block_tx_commit(&tmrpc_client, deploy_output.txhash).await?;

        let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
        let code_id: u64 = log[0].events[1].attributes[1].value.parse()?;
        config.save_codeid_to_cache(wasm_bin_path, code_id).await?;

        code_id
    } else {
        config.get_cached_codeid(wasm_bin_path).await?
    };

    info!("🚀 Communicating with Relay to Instantiate...");
    let init_msg = RelayMessage::Instantiate {
        init_msg: args.init_msg,
    }
    .run_relay(config.enclave_rpc())
    .await?;

    info!("🚀 Instantiating {}", args.label);

    let init_output: WasmdTxResponse = serde_json::from_str(&cw_client.init(
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
