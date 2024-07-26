use std::{env, env::current_dir, fs::File, io::Read, path::Path, str::FromStr};

use anyhow::anyhow;
use clap::Parser;
use cosmrs::tendermint::chain::Id as ChainId; // TODO see if this redundancy in dependencies can be decreased
use cosmrs::AccountId;
use cw_tee_mtcs::msg::ExecuteMsg as MtcsExecuteMsg;
use cycles_sync::wasmd_client::{CliWasmdClient, WasmdClient};
use futures_util::stream::StreamExt;
use quartz_common::contract::prelude::QuartzExecuteMsg;
use reqwest::Url;
use scripts::{
    types::WasmdTxResponse,
    utils::{block_tx_commit, run_relay, wasmaddr_to_id},
};
use serde::Serialize;
use serde_json::json;
use tendermint::{block::Height, Hash};
use tendermint_rpc::{query::EventType, HttpClient, SubscriptionClient, WebSocketClient};
use tm_prover::{config::Config as TmProverConfig, prover::prove};

#[derive(Serialize)]
struct Message<'a> {
    message: &'a str,
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
}

fn default_rpc_addr() -> String {
    env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    // Convert contract address string parameter to an AccountId
    // TODO: is this all the address validation that's needed?
    let httpurl = Url::parse(&format!("http://{}", cli.node_url))?;
    let wsurl = format!("ws://{}/websocket", cli.node_url);

    let tmrpc_client = HttpClient::new(httpurl.as_str()).unwrap();

    let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    // test(&client, &wasmd_client).await?;
    // panic!();
    // read trusted hash and height from files
    let base_path = current_dir()?.join("../../../");
    let trusted_files_path = base_path.join("quartz-app/");
    let (trusted_height, trusted_hash) = read_hash_height(trusted_files_path.as_path()).await?;

    // run sessioncreate in relayer script
    // export EXECUTE_CREATE=$(./scripts/relay.sh SessionCreate)
    // TODO: this is not the right return type
    let res: MtcsExecuteMsg = run_relay(base_path.as_path(), "SessionCreate", None)?; // need to define the return type

    // submit SessionCreate to contract

    //      RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_CREATE" --from admin --chain-id testing -y --output json)
    //      TX_HASH=$(echo $RES | jq -r '.["txhash"]')
    //      make sure this is json
    let output: WasmdTxResponse = serde_json::from_str(
        wasmd_client
            .tx_execute(
                &cli.contract.clone(),
                &ChainId::from_str("testing")?,
                2000000,
                cli.sender.clone(),
                json!(res),
            )?
            .as_str(),
    )?;
    println!("\n\n SessionCreate tx output: {:?}", output);

    // wait for tx to commit (in a loop?)
    let tx_hash = Hash::from_str(&output.txhash).expect("Invalid hex string for transaction hash");
    block_tx_commit(&tmrpc_client, tx_hash).await?;

    // tendermint client subscription loop
    // wait 2 blocks
    two_block_waitoor(&wsurl).await?;

    //cd $ROOT/cycles-protocol/packages/tm-prover
    //export PROOF_FILE="light-client-proof.json"
    // TODO: move all the proof related files into a directory in scripts dir
    let proof_path = current_dir()?.join("../../../packages/tm-prover/light-client-proof.json");
    println!("Proof path: {:?}", proof_path.to_str());

    // run tm prover cargo binary with trusted hash and height
    // TODO: decouple logic in tm_prover
    let mut config = TmProverConfig::default();
    config.chain_id = "testing".parse()?;
    config.primary = httpurl.as_str().parse()?;
    config.witnesses = httpurl.as_str().parse()?;
    config.trusted_height = trusted_height;
    config.trusted_hash = trusted_hash;
    config.trace_file = Some(proof_path.clone());
    config.verbose = "1".parse()?;
    config.contract_address = cli.contract.clone();
    config.storage_key = "quartz_session".to_owned();

    if let Err(report) = prove(config).await {
        return Err(anyhow!("Tendermint prover failed. Report: {}", report));
    }

    // read proof file
    let proof = read_file(proof_path.as_path()).await?;
    let json_msg = serde_json::to_string(&Message { message: &proof })?;

    // execute SessionSetPubKey on enclave
    // cd $ROOT/cycles-protocol/packages/relayer
    // export EXECUTE_SETPUB=$(./scripts/relay.sh SessionSetPubKey "$POP_MSG")

    let res: MtcsExecuteMsg = run_relay(
        base_path.as_path(),
        "SessionSetPubKey",
        Some(json_msg.as_str()),
    )?;
    // submit SessionSetPubKey to contract

    //      RES=$($CMD tx wasm execute "$CONTRACT" "$EXECUTE_SETPUB" --from admin --chain-id testing -y --output json)
    //      TX_HASH=$(echo $RES | jq -r '.["txhash"]')
    //      wait for tx to commit
    let output: WasmdTxResponse = serde_json::from_str(
        wasmd_client
            .tx_execute(
                &cli.contract.clone(),
                &ChainId::from_str("testing")?,
                2000000,
                cli.sender.clone(),
                json!(res),
            )?
            .as_str(),
    )?;

    println!("\n\n SessionSetPubKey tx output: {:?}", output);

    // wait for tx to commit (in a loop?)
    let tx_hash = Hash::from_str(&output.txhash).expect("Invalid hex string for transaction hash");

    block_tx_commit(&tmrpc_client, tx_hash).await?;

    if let MtcsExecuteMsg::Quartz(QuartzExecuteMsg::RawSessionSetPubKey(quartz)) = res {
        println!("\n\n\n{}", quartz.msg.pub_key); // TODO: return this instead later
    } else {
        return Err(anyhow!("Invalid relay response from SessionSetPubKey"));
    }

    // query results
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
