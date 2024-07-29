use std::{env::current_dir, str::FromStr};

use clap::Parser;
use cosmrs::tendermint::chain::Id as ChainId;
use cw_tee_mtcs::msg::InstantiateMsg as MtcsInstantiateMsg;
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use quartz_common::contract::msg::RawInstantiateMsg;
use reqwest::Url;
use scripts::{
    types::{Log, WasmdTxResponse},
    utils::{block_tx_commit, run_relay},
};
use serde_json::json;
use tendermint::Hash;
use tendermint_rpc::HttpClient;

#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[clap(long, default_value = "143.244.186.205:26657")]
    node_url: String,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    let base_path = current_dir()?.join("../../../");

    println!("\n🚀 Communicating with Relay to Instantiate...\n");
    let init_msg: RawInstantiateMsg = run_relay(base_path.as_path(), "Instantiate", None)?; // need to define the return type
    let init_msg: MtcsInstantiateMsg = MtcsInstantiateMsg(init_msg);

    let httpurl = Url::parse(&format!("http://{}", cli.node_url))?;
    let tmrpc_client = HttpClient::new(httpurl.as_str()).unwrap();
    let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    println!("\n🚀 Deploying MTCS Contract\n");
    let contract_path = base_path.join(
        "quartz-app/contracts/cw-tee-mtcs/target/wasm32-unknown-unknown/release/cw_tee_mtcs.wasm",
    );
    // TODO: uncertain about the path -> string conversion
    let deploy_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.deploy(
        &ChainId::from_str("testing")?,
        String::from("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
        contract_path.display().to_string(),
    )?)?;

    let tx_hash =
        Hash::from_str(&deploy_output.txhash).expect("Invalid hex string for transaction hash");
    let res = block_tx_commit(&tmrpc_client, tx_hash).await?;

    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
    let code_id: usize = log[0].events[1].attributes[1].value.parse()?;

    println!("\n🚀 Instantiating MTCS Contract\n");

    let deploy_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.init(
        &ChainId::from_str("testing")?,
        String::from("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
        code_id,
        json!(init_msg),
        format!("MTCS Contract V{}", code_id),
    )?)?;

    let tx_hash =
        Hash::from_str(&deploy_output.txhash).expect("Invalid hex string for transaction hash");
    let res = block_tx_commit(&tmrpc_client, tx_hash).await?;
    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
    let contract_addr: &String = &log[0].events[1].attributes[0].value;

    println!("\n🚀 Successfully deployed and instantiated contract!");
    println!("🆔 Code ID: {}", code_id);
    println!("📌 Contract Address: {}", contract_addr);

    println!("{contract_addr}");
    Ok(())
}

//RES=$($CMD tx wasm instantiate "$CODE_ID" "$INSTANTIATE_MSG" --from "$USER_ADDR" --label $LABEL $TXFLAG -y --no-admin --output json)
