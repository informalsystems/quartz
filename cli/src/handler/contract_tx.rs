use async_trait::async_trait;
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use reqwest::Url;
use tracing::info;
use tokio::time::{sleep, Duration};
use serde_json::Value;

use super::utils::types::WasmdTxResponse;

use crate::{
    error::Error,
    handler::Handler,
    request::contract_tx::ContractTxRequest,
    response::{contract_tx::ContractTxResponse, Response},
    Config,
};

#[async_trait]
impl Handler for ContractTxRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, _: Config) -> Result<Self::Response, Self::Error> {
        let tx_hash = tx(self)
            .await
            .map_err(|e| Error::GenericErr(e.to_string()))?;

        Ok(ContractTxResponse { tx_hash }.into())
    }
}

async fn tx(args: ContractTxRequest) -> Result<String, anyhow::Error> {
     let httpurl = Url::parse(&format!("http://{}", args.node_url))?;
     let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);
 
     info!("🚀 Submitting Tx with msg: {}", args.msg);

     let tx_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.tx_execute(
        &args.contract,
        &args.chain_id,
        args.gas,
        &args.sender,
        args.msg,
        args.amount,
     )?)?;

    let hash = tx_output.txhash.to_string();
    info!("🚀 Successfully sent tx with hash {}", hash);
    info!("Waiting 5 seconds for tx to be included in block.....");

    // TODO - a more robust timeout mechanism with polling blocks
    sleep(Duration::from_secs(5)).await;

    // Query the transaction
    let tx_result: Value = wasmd_client.query_tx(&hash)?;

    // Check if the transaction was successful, otherwise return raw log or error
    if let Some(code) = tx_result["code"].as_u64() {
        if code == 0 {
            info!("Transaction was successful!");
        } else {
            return Err(anyhow::anyhow!("Transaction failed. Inspect raw log: {}", tx_result["raw_log"]));
        }
    } else {
        return Err(anyhow::anyhow!("Unable to determine transaction status"));
    }

    Ok(tx_output.txhash.to_string())
}
