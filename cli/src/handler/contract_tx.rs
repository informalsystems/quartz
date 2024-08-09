use async_trait::async_trait;
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use reqwest::Url;
use tracing::{debug, info, trace};

use super::utils::{
    types::WasmdTxResponse,
};
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
 
     info!("\n🚀 Submitting Tx {}\n", args.msg);

     let tx_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.tx_execute(
        &args.contract,
        &args.chain_id,
        args.gas,
        &args.sender,
        args.msg,
     )?)?;
    //  let res = block_tx_commit(&tmrpc_client, tx_output.txhash).await?; 
    //  let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;

     info!("\n🚀 Successfully sent tx TODO_TX_HASH and instantiated contract!");
     info!("{}", tx_output.txhash.to_string());
 
     Ok(tx_output.txhash.to_string())
}
