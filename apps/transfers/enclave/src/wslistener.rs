//TODO: get rid of this
use std::{collections::BTreeMap, str::FromStr};
use tracing::{debug, error, info};

use std::env;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{anyhow, Error, Result};
use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use cosmwasm_std::{Addr, HexBinary};
use futures_util::StreamExt;
use quartz_common::{
    contract::msg::execute::attested::{
        MockAttestation, RawAttested, RawAttestedMsgSansHandler, RawMockAttestation,
    },
    enclave::{
        attestor::Attestor,
        server::{WebSocketHandler, WsListenerConfig},
    },
};
use reqwest::Url;
use serde_json::json;
use tendermint_rpc::{event::Event, query::EventType, SubscriptionClient, WebSocketClient};
use tm_prover::{config::Config as TmProverConfig, prover::prove};
use tonic::Request;
use transfers_contract::msg::{
    execute::{QueryResponseMsg, Request as TransferRequest, UpdateMsg},
    AttestedMsg, ExecuteMsg,
    QueryMsg::{GetRequests, GetState},
};
use wasmd_client::{CliWasmdClient, QueryResult, WasmdClient};

use crate::{
    proto::{settlement_server::Settlement, QueryRequest, UpdateRequest},
    transfers_server::{
        QueryRequestMessage, TransfersOp, TransfersOpEvent,
        TransfersService, UpdateRequestMessage,
    },
};

#[derive(Clone, Debug)]
enum TransfersOpEventTypes {
    Query,
    Transfer,
}

impl TryFrom<Event> for TransfersOpEvent {
    type Error = Error;

    fn try_from(event: Event) -> Result<Self, Error> {
        if let Some(events) = &event.events {
            for (key, _) in events {
                match key.as_str() {
                    k if k.starts_with("wasm-query_balance") => {
                        let (contract_address, ephemeral_pubkey, sender) =
                            extract_event_info(TransfersOpEventTypes::Query, &events)
                                .map_err(|_| anyhow!("Failed to extract event info from query event"))?;

                        return Ok(TransfersOpEvent::Query {
                            contract_address,
                            ephemeral_pubkey: ephemeral_pubkey.ok_or(anyhow!("Missing ephemeral_pubkey"))?,
                            sender: sender.ok_or(anyhow!("Missing sender"))?,
                        });
                    }
                    k if k.starts_with("wasm-transfer.action") => {
                        let (contract_address, _, _) =
                            extract_event_info(TransfersOpEventTypes::Transfer, &events)
                                .map_err(|_| anyhow!("Failed to extract event info from transfer event"))?;

                        return Ok(TransfersOpEvent::Transfer { contract_address });
                    }
                    _ => {}
                }
            }
        }

        Err(anyhow!("Unsupported event."))
    }
}

// TODO: Need to prevent listener from taking actions until handshake is completed
#[async_trait::async_trait]
impl<A: Attestor + Clone> WebSocketHandler for TransfersService<A> {
    async fn handle(&self, event: Event, config: WsListenerConfig) -> Result<()> {
        let op_event = TransfersOpEvent::try_from(event)?;

        self.queue_producer
            .send(TransfersOp {
                client: self.clone(),
                event: op_event,
                config,
            })
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
pub trait WsListener: Send + Sync + 'static {
    async fn process(&self, event: TransfersOpEvent, config: WsListenerConfig) -> Result<()>;
}

#[async_trait::async_trait]
impl<A: Attestor> WsListener for TransfersService<A> {
    async fn process(&self, event: TransfersOpEvent, config: WsListenerConfig) -> Result<()> {
        match event {
            TransfersOpEvent::Transfer { contract_address } => {
                println!("Processing transfer event");
                transfer_handler(self, &contract_address, &config).await?;
            }
            TransfersOpEvent::Query {
                contract_address,
                ephemeral_pubkey,
                sender,
            } => {
                println!("Processing query event");
                query_handler(self, &contract_address, &sender, &ephemeral_pubkey, &config).await?;
            }
        }

        let wsurl = config.websocket_url;
        // Wait some blocks to make sure transaction was confirmed
        two_block_waitoor(&wsurl).await?;

        Ok(())
    }
}

fn extract_event_info(
    op_event: TransfersOpEventTypes,
    events: &BTreeMap<String, Vec<String>>,
) -> Result<(AccountId, Option<String>, Option<String>)> {
    let mut sender = None;
    let mut ephemeral_pubkey = None;

    // Set common info data for all events
    let contract_address = events
        .get("execute._contract_address")
        .ok_or_else(|| anyhow!("Missing execute._contract_address in events"))?
        .first()
        .ok_or_else(|| anyhow!("execute._contract_address is empty"))?
        .parse::<AccountId>()
        .map_err(|e| anyhow!("Failed to parse contract address: {}", e))?;

    // Set info for specific events
    match op_event {
        TransfersOpEventTypes::Query => {
            sender = events
                .get("message.sender")
                .ok_or_else(|| anyhow!("Missing message.sender in events"))?
                .first()
                .cloned();

            ephemeral_pubkey = events
                .get("wasm-query_balance.emphemeral_pubkey")
                .ok_or_else(|| anyhow!("Missing wasm-query_balance.emphemeral_pubkey in events"))?
                .first()
                .cloned();
        }
        _ => {}
    }

    Ok((contract_address, ephemeral_pubkey, sender))
}

async fn transfer_handler<A: Attestor>(
    client: &TransfersService<A>,
    contract: &AccountId,
    ws_config: &WsListenerConfig,
) -> Result<()> {
    let chain_id = &ChainId::from_str(&ws_config.chain_id)?;
    let httpurl = Url::parse(&ws_config.node_url.clone())?;
    let wasmd_client = CliWasmdClient::new(httpurl.clone());
    // Query chain
    // Get epoch, obligations, liquidity sources
    let resp: QueryResult<Vec<TransferRequest>> = wasmd_client
        .query_smart(contract, json!(GetRequests {}))
        .map_err(|e| anyhow!("Problem querying contract state 1: {}", e))?;
    let requests = resp.data;

    let resp: QueryResult<HexBinary> = wasmd_client
    .query_smart(contract, json!(GetState {}))
    .map_err(|e| anyhow!("Problem querying contract state 2: {}", e))?;
    let state = resp.data;
    
    // Request body contents
    let update_contents = UpdateRequestMessage { state, requests };

    // Wait 2 blocks
    info!("Waiting 2 blocks for light client proof");
    let wsurl = ws_config.node_url.clone();
    two_block_waitoor(&wsurl).await?;

    // Call tm prover with trusted hash and height
    let prover_config = TmProverConfig {
        primary: httpurl.as_str().parse()?,
        witnesses: httpurl.as_str().parse()?,
        trusted_height: ws_config.trusted_height,
        trusted_hash: ws_config.trusted_hash,
        verbose: "1".parse()?, // TODO: both tm-prover and cli define the same Verbosity struct. Need to define this once and import
        contract_address: contract.clone(),
        storage_key: "requests".to_string(),
        chain_id: ws_config.chain_id.to_string(),
        ..Default::default()
    };

    let proof_output = tokio::task::spawn_blocking(move || {
        // Create a new runtime inside the blocking thread.
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async {
            prove(prover_config)
                .await
                .map_err(|report| anyhow!("Tendermint prover failed. Report: {}", report))
        })
    })
    .await??; // Handle both JoinError and your custom error

    // Merge the UpdateRequestMessage with the proof
    let mut proof_json = serde_json::to_value(proof_output)?;
    proof_json["msg"] = serde_json::to_value(&update_contents)?;

    // Build final request object
    let request = Request::new(UpdateRequest {
        message: json!(proof_json).to_string(),
    });

    // Send UpdateRequestMessage request to enclave over tonic gRPC client
    let update_response = client
        .run(request)
        .await
        .map_err(|e| anyhow!("Failed to communicate to relayer. {e}"))?
        .into_inner();

    // Extract json from enclave response
    let attested: RawAttested<UpdateMsg, HexBinary> =
        serde_json::from_str(&update_response.message)
            .map_err(|e| anyhow!("Error deserializing UpdateMsg from enclave: {}", e))?;

    // Build on-chain response
    // TODO add non-mock support
    let transfer_msg = ExecuteMsg::Update::<RawMockAttestation>(AttestedMsg {
        msg: RawAttestedMsgSansHandler(attested.msg),
        attestation: MockAttestation(
            attested
                .attestation
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("slice with incorrect length"))?,
        )
        .into(),
    });

    // Post response to chain
    let output = wasmd_client.tx_execute(
        contract,
        chain_id,
        300000,
        &ws_config.tx_sender,
        json!(transfer_msg),
        "40000untrn",
    )?;

    println!("Output TX: {}", output);

    Ok(())
}

async fn query_handler<A: Attestor>(
    client: &TransfersService<A>,
    contract: &AccountId,
    msg_sender: &String,
    pubkey: &String,
    ws_config: &WsListenerConfig,
) -> Result<()> {
    let chain_id = &ChainId::from_str(&ws_config.chain_id)?;
    let httpurl = Url::parse(&ws_config.node_url)?;
    let wasmd_client = CliWasmdClient::new(httpurl);
    // Query Chain
    // Get state
    let resp: QueryResult<HexBinary> = wasmd_client
        .query_smart(contract, json!(GetState {}))
        .map_err(|e| anyhow!("Problem querying contract state: {}", e))?;
    let state = resp.data;

    // Build request
    let update_contents = QueryRequestMessage {
        state,
        address: Addr::unchecked(msg_sender), // sender comes from TX event, therefore is checked
        ephemeral_pubkey: HexBinary::from_hex(pubkey)?,
    };

    // Send QueryRequestMessage to enclave over tonic gRPC client
    let request = Request::new(QueryRequest {
        message: json!(update_contents).to_string(),
    });

    let query_response = client
        .query(request)
        .await
        .map_err(|e| anyhow!("Failed to communicate to relayer. {e}"))?
        .into_inner();

    // Extract json from the enclave response
    let attested: RawAttested<QueryResponseMsg, HexBinary> =
        serde_json::from_str(&query_response.message)
            .map_err(|e| anyhow!("Error deserializing QueryResponseMsg from enclave: {}", e))?;

    // Build on-chain response
    // TODO add non-mock support
    let query_msg = ExecuteMsg::QueryResponse::<RawMockAttestation>(AttestedMsg {
        msg: RawAttestedMsgSansHandler(attested.msg),
        attestation: MockAttestation(
            attested
                .attestation
                .as_slice()
                .try_into()
                .map_err(|_| anyhow!("slice with incorrect length"))?,
        )
        .into(),
    });

    // Post response to chain
    let output = wasmd_client.tx_execute(
        contract,
        chain_id,
        300000,
        &ws_config.tx_sender,
        json!(query_msg),
        "40000untrn",
    )?;

    println!("Output TX: {}", output);
    Ok(())
}

async fn two_block_waitoor(wsurl: &str) -> Result<(), Error> {
    info!("WSURL at 2 block waitor in wslistener {}", wsurl);

    let (client, driver) = WebSocketClient::new(wsurl).await?;

    let driver_handle = tokio::spawn(async move { driver.run().await });

    // Subscription functionality
    let mut subs = client.subscribe(EventType::NewBlock.into()).await?;

    // Wait 2 NewBlock events
    let mut ev_count = 2_i32;

    while let Some(res) = subs.next().await {
        let _ev = res?;
        ev_count -= 1;
        if ev_count == 0 {
            break;
        }
    }

    // Signal to the driver to terminate.
    client.close()?;
    // Await the driver's termination to ensure proper connection closure.
    let _ = driver_handle.await?;

    Ok(())
}



// use std::{collections::BTreeMap, str::FromStr};
// use tracing::{debug, error, info};
// use std::env;
// use std::fs;
// use std::time::{SystemTime, UNIX_EPOCH};

// use anyhow::{anyhow, Error, Result};
// use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
// use cosmwasm_std::{Addr, HexBinary};
// use futures_util::StreamExt;
// use quartz_common::{
//     contract::msg::execute::attested::{
//         MockAttestation, RawAttested, RawAttestedMsgSansHandler, RawMockAttestation,
//     },
//     enclave::{
//         attestor::Attestor,
//         server::{WebSocketHandler, WsListenerConfig},
//     },
// };
// use reqwest::Url;
// use serde_json::json;
// use tendermint_rpc::{event::Event, query::EventType, SubscriptionClient, WebSocketClient};
// use tm_prover::{config::Config as TmProverConfig, prover::prove};
// use tonic::Request;
// use transfers_contract::msg::{
//     execute::{QueryResponseMsg, Request as TransferRequest, UpdateMsg},
//     AttestedMsg, ExecuteMsg,
//     QueryMsg::{GetRequests, GetState},
// };
// use wasmd_client::{CliWasmdClient, QueryResult, WasmdClient};

// use crate::{
//     proto::{settlement_server::Settlement, QueryRequest, UpdateRequest},
//     transfers_server::{
//         QueryRequestMessage, TransfersOp, TransfersOpEvent,
//         TransfersService, UpdateRequestMessage,
//     },
// };

// use tokio::sync::{Mutex, Semaphore};
// use std::sync::Arc;
// use lazy_static::lazy_static;
// use tokio::time::{interval, Duration};

// lazy_static! {
//     static ref HANDLER_SEMAPHORE: Semaphore = Semaphore::new(1);
//     // static ref WS_POOL: Arc<Mutex<WebSocketPool>> = Arc::new(Mutex::new(WebSocketPool { connections: Vec::new() }));
//     static ref WS_POOL: Arc<Mutex<WebSocketPool>> = Arc::new(Mutex::new(WebSocketPool::new()));

// }

// struct WebSocketPool {
//     connections: Vec<WebSocketClient>,
// }

// impl WebSocketPool {
//     async fn get_connection(&mut self, url: &str) -> Result<WebSocketClient> {
//         if let Some(conn) = self.connections.pop() {
//             Ok(conn)
//         } else {
//             let (client, driver) = WebSocketClient::new(url).await?;
//             tokio::spawn(async move { driver.run().await });
//             Ok(client)
//         }
//     }

//     fn return_connection(&mut self, conn: WebSocketClient) {
//         self.connections.push(conn);
//     }
// }

// #[derive(Clone, Debug)]
// enum TransfersOpEventTypes {
//     Query,
//     Transfer,
// }

// impl TryFrom<Event> for TransfersOpEvent {
//     type Error = Error;

//     fn try_from(event: Event) -> Result<Self, Error> {
//         if let Some(events) = &event.events {
//             for (key, _) in events {
//                 match key.as_str() {
//                     k if k.starts_with("wasm-query_balance") => {
//                         let (contract_address, ephemeral_pubkey, sender) =
//                             extract_event_info(TransfersOpEventTypes::Query, &events)
//                                 .map_err(|_| anyhow!("Failed to extract event info from query event"))?;

//                         return Ok(TransfersOpEvent::Query {
//                             contract_address,
//                             ephemeral_pubkey: ephemeral_pubkey.ok_or(anyhow!("Missing ephemeral_pubkey"))?,
//                             sender: sender.ok_or(anyhow!("Missing sender"))?,
//                         });
//                     }
//                     k if k.starts_with("wasm-transfer.action") => {
//                         let (contract_address, _, _) =
//                             extract_event_info(TransfersOpEventTypes::Transfer, &events)
//                                 .map_err(|_| anyhow!("Failed to extract event info from transfer event"))?;

//                         return Ok(TransfersOpEvent::Transfer { contract_address });
//                     }
//                     _ => {}
//                 }
//             }
//         }

//         Err(anyhow!("Unsupported event."))
//     }
// }

// #[async_trait::async_trait]
// impl<A: Attestor + Clone> WebSocketHandler for TransfersService<A> {
//     async fn handle(&self, event: Event, config: WsListenerConfig) -> Result<()> {
//         let op_event = TransfersOpEvent::try_from(event)?;

//         self.queue_producer
//             .send(TransfersOp {
//                 client: self.clone(),
//                 event: op_event,
//                 config,
//             })
//             .await?;

//         Ok(())
//     }
// }

// #[tonic::async_trait]
// pub trait WsListener: Send + Sync + 'static {
//     async fn process(&self, event: TransfersOpEvent, config: WsListenerConfig) -> Result<()>;
// }

// #[async_trait::async_trait]
// impl<A: Attestor> WsListener for TransfersService<A> {
//     async fn process(&self, event: TransfersOpEvent, config: WsListenerConfig) -> Result<()> {
//         match event {
//             TransfersOpEvent::Transfer { contract_address } => {
//                 info!("Processing transfer event");
//                 transfer_handler(self, &contract_address, &config).await?;
//             }
//             TransfersOpEvent::Query {
//                 contract_address,
//                 ephemeral_pubkey,
//                 sender,
//             } => {
//                 info!("Processing query event");
//                 query_handler(self, &contract_address, &sender, &ephemeral_pubkey, &config).await?;
//             }
//         }

//         let wsurl = config.websocket_url;
//         two_block_waitoor(&wsurl).await?;

//         Ok(())
//     }
// }

// fn extract_event_info(
//     op_event: TransfersOpEventTypes,
//     events: &BTreeMap<String, Vec<String>>,
// ) -> Result<(AccountId, Option<String>, Option<String>)> {
//     let mut sender = None;
//     let mut ephemeral_pubkey = None;

//     let contract_address = events
//         .get("execute._contract_address")
//         .ok_or_else(|| anyhow!("Missing execute._contract_address in events"))?
//         .first()
//         .ok_or_else(|| anyhow!("execute._contract_address is empty"))?
//         .parse::<AccountId>()
//         .map_err(|e| anyhow!("Failed to parse contract address: {}", e))?;

//     match op_event {
//         TransfersOpEventTypes::Query => {
//             sender = events
//                 .get("message.sender")
//                 .ok_or_else(|| anyhow!("Missing message.sender in events"))?
//                 .first()
//                 .cloned();

//             ephemeral_pubkey = events
//                 .get("wasm-query_balance.emphemeral_pubkey")
//                 .ok_or_else(|| anyhow!("Missing wasm-query_balance.emphemeral_pubkey in events"))?
//                 .first()
//                 .cloned();
//         }
//         _ => {}
//     }

//     Ok((contract_address, ephemeral_pubkey, sender))
// }

// async fn transfer_handler<A: Attestor>(
//     client: &TransfersService<A>,
//     contract: &AccountId,
//     ws_config: &WsListenerConfig,
// ) -> Result<()> {
//     let _permit = HANDLER_SEMAPHORE.acquire().await?;
//     info!("Starting transfer handler");

//     let timestamp = SystemTime::now()
//         .duration_since(UNIX_EPOCH)
//         .expect("Time went backwards")
//         .as_nanos();

//     let wasm_dir = format!("/tmp/neutrond_wasm_{}", timestamp);
//     env::set_var("NEUTROND_WASM_DIR", &wasm_dir);

//     fs::create_dir_all(&wasm_dir).expect("Failed to create Wasm directory");

//     let _cleanup = defer::defer(|| {
//         debug!("Attempting to clean up directory: {}", wasm_dir);
//         if let Err(e) = fs::remove_dir_all(&wasm_dir) {
//             error!("Failed to remove temporary Wasm directory: {}", e);
//         } else {
//             info!("Successfully removed temporary Wasm directory: {}", wasm_dir);
//         }
//     });

//     let chain_id = &ChainId::from_str(&ws_config.chain_id)?;
//     let httpurl = Url::parse(&ws_config.node_url.clone())?;
//     let wasmd_client = CliWasmdClient::new(httpurl.clone());

//     let resp: QueryResult<Vec<TransferRequest>> = wasmd_client
//         .query_smart(contract, json!(GetRequests {}))
//         .map_err(|e| anyhow!("Problem querying contract state 1: {}", e))?;
//     let requests = resp.data;

//     let resp: QueryResult<HexBinary> = wasmd_client
//         .query_smart(contract, json!(GetState {}))
//         .map_err(|e| anyhow!("Problem querying contract state 2: {}", e))?;
//     let state = resp.data;
    
//     let update_contents = UpdateRequestMessage { state, requests };

//     info!("Waiting 2 blocks for light client proof");
//     let wsurl = ws_config.node_url.clone();
//     two_block_waitoor(&wsurl).await?;

//     let prover_config = TmProverConfig {
//         primary: httpurl.as_str().parse()?,
//         witnesses: httpurl.as_str().parse()?,
//         trusted_height: ws_config.trusted_height,
//         trusted_hash: ws_config.trusted_hash,
//         verbose: "1".parse()?,
//         contract_address: contract.clone(),
//         storage_key: "requests".to_string(),
//         chain_id: ws_config.chain_id.to_string(),
//         ..Default::default()
//     };

//     let proof_output = tokio::task::spawn_blocking(move || {
//         let rt = tokio::runtime::Runtime::new()?;
//         rt.block_on(async {
//             prove(prover_config)
//                 .await
//                 .map_err(|report| anyhow!("Tendermint prover failed. Report: {}", report))
//         })
//     })
//     .await??;

//     let mut proof_json = serde_json::to_value(proof_output)?;
//     proof_json["msg"] = serde_json::to_value(&update_contents)?;

//     let request = Request::new(UpdateRequest {
//         message: json!(proof_json).to_string(),
//     });

//     let update_response = client
//         .run(request)
//         .await
//         .map_err(|e| anyhow!("Failed to communicate to relayer. {e}"))?
//         .into_inner();

//     let attested: RawAttested<UpdateMsg, HexBinary> =
//         serde_json::from_str(&update_response.message)
//             .map_err(|e| anyhow!("Error deserializing UpdateMsg from enclave: {}", e))?;

//     let transfer_msg = ExecuteMsg::Update::<RawMockAttestation>(AttestedMsg {
//         msg: RawAttestedMsgSansHandler(attested.msg),
//         attestation: MockAttestation(
//             attested
//                 .attestation
//                 .as_slice()
//                 .try_into()
//                 .map_err(|_| anyhow!("slice with incorrect length"))?,
//         )
//         .into(),
//     });

//     let output = wasmd_client.tx_execute(
//         contract,
//         chain_id,
//         300000,
//         &ws_config.tx_sender,
//         json!(transfer_msg),
//         "40000untrn",
//     )?;

//     info!("Output TX: {}", output);
//     info!("Transfer handler completed successfully");
//     Ok(())
// }

// async fn query_handler<A: Attestor>(
//     client: &TransfersService<A>,
//     contract: &AccountId,
//     msg_sender: &String,
//     pubkey: &String,
//     ws_config: &WsListenerConfig,
// ) -> Result<()> {
//     let _permit = HANDLER_SEMAPHORE.acquire().await?;
//     info!("Starting query handler");

//     let chain_id = &ChainId::from_str(&ws_config.chain_id)?;
//     let httpurl = Url::parse(&ws_config.node_url)?;
//     let wasmd_client = CliWasmdClient::new(httpurl);

//     let resp: QueryResult<HexBinary> = wasmd_client
//         .query_smart(contract, json!(GetState {}))
//         .map_err(|e| anyhow!("Problem querying contract state: {}", e))?;
//     let state = resp.data;

//     let update_contents = QueryRequestMessage {
//         state,
//         address: Addr::unchecked(msg_sender),
//         ephemeral_pubkey: HexBinary::from_hex(pubkey)?,
//     };

//     let request = Request::new(QueryRequest {
//         message: json!(update_contents).to_string(),
//     });

//     let query_response = client
//         .query(request)
//         .await
//         .map_err(|e| anyhow!("Failed to communicate to relayer. {e}"))?
//         .into_inner();

//     let attested: RawAttested<QueryResponseMsg, HexBinary> =
//         serde_json::from_str(&query_response.message)
//             .map_err(|e| anyhow!("Error deserializing QueryResponseMsg from enclave: {}", e))?;

//     let query_msg = ExecuteMsg::QueryResponse::<RawMockAttestation>(AttestedMsg {
//         msg: RawAttestedMsgSansHandler(attested.msg),
//         attestation: MockAttestation(
//             attested
//                 .attestation
//                 .as_slice()
//                 .try_into()
//                 .map_err(|_| anyhow!("slice with incorrect length"))?,
//         )
//         .into(),
//     });

//     let output = wasmd_client.tx_execute(
//         contract,
//         chain_id,
//         300000,
//         &ws_config.tx_sender,
//         json!(query_msg),
//         "40000untrn",
//     )?;

//     info!("Output TX: {}", output);
//     info!("Query handler completed successfully");
//     Ok(())
// }

// async fn two_block_waitoor(wsurl: &str) -> Result<(), Error> {
//     info!("WSURL at 2 block waitor in wslistener {}", wsurl);

//     let mut pool = WS_POOL.lock().await;
//     let client = pool.get_connection(wsurl).await?;

//     let mut subs = client.subscribe(EventType::NewBlock.into()).await?;

//     let mut ev_count = 2_i32;

//     while let Some(res) = subs.next().await {
//         let _ev = res?;
//         ev_count -= 1;
//         if ev_count == 0 {
//             break;
//         }
//     }

//     pool.return_connection(client);

//     Ok(())
// }

// async fn cleanup_old_temp_dirs() {
//     let temp_dir = "/tmp";
//     let current_time = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();

//     if let Ok(entries) = fs::read_dir(temp_dir) {
//         for entry in entries.filter_map(Result::ok) {
//             let path = entry.path();
//             if let Some(file_name) = path.file_name() {
//                 if file_name.to_string_lossy().starts_with("neutrond_wasm_") {
//                     if let Ok(metadata) = fs::metadata(&path) {
//                         if let Ok(created) = metadata.created() {
//                             let age = current_time - created.duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
//                             if age > 3600 { // Remove directories older than 1 hour
//                                 if let Err(e) = fs::remove_dir_all(&path) {
//                                     error!("Failed to remove old directory {:?}: {}", path, e);
//                                 } else {
//                                     info!("Removed old directory: {:?}", path);
//                                 }
//                             }
//                         }
//                     }
//                 }
//             }
//         }
//     }
// }

// async fn run_periodic_cleanup() {
//     let mut interval = interval(Duration::from_secs(3600)); // Run every hour
//     loop {
//         interval.tick().await;
//         cleanup_old_temp_dirs().await;
//     }
// }

// mod defer {
//     use std::ops::Drop;

//     pub struct Defer<F: FnOnce()>(Option<F>);

//     impl<F: FnOnce()> Drop for Defer<F> {
//         fn drop(&mut self) {
//             if let Some(f) = self.0.take() {
//                 f();
//             }
//         }
//     }

//     pub fn defer<F: FnOnce()>(f: F) -> Defer<F> {
//         Defer(Some(f))
//     }
// }

// // Add this function to initialize the WebSocket listener
// pub async fn initialize_ws_listener<A: Attestor + Clone>(
//     service: TransfersService<A>,
//     ws_config: WsListenerConfig,
// ) -> Result<()> {
//     info!("Initializing WebSocket listener");

//     // Start the periodic cleanup task
//     tokio::spawn(run_periodic_cleanup());

//     // Create a WebSocket client
//     let (ws_client, driver) = WebSocketClient::new(ws_config.websocket_url.clone()).await?;

//     // Spawn a task to run the WebSocket driver
//     tokio::spawn(async move {
//         if let Err(e) = driver.run().await {
//             error!("WebSocket driver error: {}", e);
//         }
//     });

//     // Subscribe to the required events
//     let mut event_sub = ws_client.subscribe(EventType::Tx.into()).await?;

//     info!("WebSocket listener initialized and subscribed to events");

//     // Process incoming events
//     while let Some(event) = event_sub.next().await {
//         match event {
//             Ok(event) => {
//                 if let Err(e) = service.handle(event, ws_config.clone()).await {
//                     error!("Error handling event: {}", e);
//                 }
//             }
//             Err(e) => {
//                 error!("Error receiving event: {}", e);
//             }
//         }
//     }

//     Ok(())
// }

// // This function should be called from your main application to start the WebSocket listener
// pub async fn start_ws_listener<A: Attestor + Clone>(
//     service: TransfersService<A>,
//     ws_config: WsListenerConfig,
// ) -> Result<()> {
//     info!("Starting WebSocket listener");

//     loop {
//         match initialize_ws_listener(service.clone(), ws_config.clone()).await {
//             Ok(_) => {
//                 error!("WebSocket listener unexpectedly terminated. Restarting...");
//             }
//             Err(e) => {
//                 error!("WebSocket listener error: {}. Restarting...", e);
//             }
//         }

//         // Wait before attempting to reconnect
//         tokio::time::sleep(Duration::from_secs(5)).await;
//     }
// }