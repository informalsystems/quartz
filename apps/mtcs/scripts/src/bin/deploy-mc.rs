use std::{env::current_dir, str::FromStr};

use anyhow::anyhow;
use clap::Parser;
use cosmrs::tendermint::chain::Id as ChainId;
use cosmwasm_std::Addr;
use cw_tee_mtcs::msg::InstantiateMsg as MtcsInstantiateMsg;
use cycles_sync::wasmd_client::{CliWasmdClient, QueryResult, WasmdClient};
use mtcs_overdraft::{
    msg::{Cw4ExecuteMsg, InstantiateMsg, Member, QueryMsg, QueryResp},
    AddHook,
};
use quartz_common::contract::msg::RawInstantiateMsg;
use reqwest::Url;
use scripts::{
    types::{Log, WasmdTxResponse},
    utils::{block_tx_commit, wasmaddr_to_id},
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

    let httpurl = Url::parse(&format!("http://{}", cli.node_url))?;
    let tmrpc_client = HttpClient::new(httpurl.as_str()).unwrap();
    let wasmd_client = CliWasmdClient::new(Url::parse(httpurl.as_str())?);

    let contracts_path = current_dir()?.join("../../../contracts");

    println!("\n🚀 Deploying Memberships Contract\n");
    let group_wasm = contracts_path.join("cw4_group.wasm");
    // TODO: uncertain about the path -> string conversion
    let group_deploy_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.deploy(
        &ChainId::from_str("testing")?,
        String::from("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
        group_wasm.as_path().to_string_lossy(),
    )?)?;

    let tx_hash = Hash::from_str(&group_deploy_output.txhash)
        .expect("Invalid hex string for transaction hash");
    let res = block_tx_commit(&tmrpc_client, tx_hash).await?;

    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
    let memberships_code_id: u64 = log[0].events[1].attributes[1].value.parse()?;

    println!("\n🚀 Deploying Mutual Credit Contract\n");

    let mc_wasm =
        contracts_path.join("overdraft/target/wasm32-unknown-unknown/release/mtcs_overdraft.wasm");
    // TODO: uncertain about the path -> string conversion
    let mc_deploy_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.deploy(
        &ChainId::from_str("testing")?,
        String::from("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
        mc_wasm.as_path().to_string_lossy(),
    )?)?;

    let tx_hash =
        Hash::from_str(&mc_deploy_output.txhash).expect("Invalid hex string for transaction hash");
    let res = block_tx_commit(&tmrpc_client, tx_hash).await?;

    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;
    let mc_code_id: u64 = log[0].events[1].attributes[1].value.parse()?;

    println!(
        "\n🚀 Initializing Mutual Credit Contract with Memberships Contract Code Id: {}\n",
        memberships_code_id
    );

    let init_msg = InstantiateMsg {
        owner: Addr::unchecked("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
        memberships_code_id,
    };

    let deploy_output: WasmdTxResponse = serde_json::from_str(&wasmd_client.init(
        &ChainId::from_str("testing")?,
        String::from("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
        mc_code_id as usize,
        json!(init_msg),
        format!("MTCS Contract V{}", mc_code_id),
    )?)?;

    let tx_hash =
        Hash::from_str(&deploy_output.txhash).expect("Invalid hex string for transaction hash");
    let res = block_tx_commit(&tmrpc_client, tx_hash).await?;
    let log: Vec<Log> = serde_json::from_str(&res.tx_result.log)?;

    let mc_contract_addr = &log[0].events[1].attributes[0].value;

    let resp: QueryResult<QueryResp> = wasmd_client
        .query_smart(
            &wasmaddr_to_id(mc_contract_addr)?,
            json!(QueryMsg::GroupAddr {}),
        )
        .map_err(|e| anyhow!("Problem querying liquidity sources: {}", e))?;

    let group_contract_addr = &resp.data.message;

    // *** ADD HOOK ***

    // TODO: Make sure it succeeded (or return error + contract reuse)
    let _add_hook_output: WasmdTxResponse = serde_json::from_str(
        wasmd_client
            .tx_execute(
                &wasmaddr_to_id(group_contract_addr)?,
                &ChainId::from_str("testing")?,
                2000000,
                String::from("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
                json!(AddHook {
                    addr: mc_contract_addr.to_owned()
                }),
            )?
            .as_str(),
    )?;

    // ***  Regsiter test members ***
    let update_members_output: WasmdTxResponse = serde_json::from_str(
        wasmd_client
            .tx_execute(
                &wasmaddr_to_id(group_contract_addr)?,
                &ChainId::from_str("testing")?,
                2000000,
                String::from("wasm14qdftsfk6fwn40l0xmruga08xlczl4g05npy70"),
                json!(Cw4ExecuteMsg::UpdateMembers {
                    remove: vec![],
                    add: vec![
                        Member {
                            addr: String::from("wasm124tuy67a9dcvfgcr4gjmz60syd8ddaugl33v0n"),
                            weight: 0u64
                        },
                        Member {
                            addr: String::from("wasm1ctkqmg45u85jnf5ur9796h7ze4hj6ep5y7m7l6"),
                            weight: 0u64
                        }
                    ]
                }),
            )?
            .as_str(),
    )?;

    println!("{:?}", update_members_output);

    println!("\n🚀 Successfully deployed and instantiated mutual credit contract!");
    println!("\nMutual Credit");
    println!("🆔 Code ID: {}", mc_code_id);
    println!("📌 Contract Address: {}", mc_contract_addr);
    println!("\nMemberships sub-contract");
    println!("🆔 Code ID: {}", memberships_code_id);
    println!("📌 Contract Address: {}", group_contract_addr);

    Ok(())
}

//RES=$($CMD tx wasm instantiate "$CODE_ID" "$INSTANTIATE_MSG" --from "$USER_ADDR" --label $LABEL $TXFLAG -y --no-admin --output json)
