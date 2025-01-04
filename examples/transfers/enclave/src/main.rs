#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

pub mod cli;
pub mod proto;
pub mod state;
pub mod transfers_server;
pub mod wslistener;

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use anyhow::{anyhow, Error as AnyhowError};
use clap::Parser;
use cli::Cli;
use cosmrs::AccountId;
use cosmwasm_std::{Addr, HexBinary, Uint64};
use k256::ecdsa::VerifyingKey;
use quartz_common::{
    contract::state::{Config, LightClientOpts, SEQUENCE_NUM_KEY},
    enclave::{
        attestor::{self, Attestor, DefaultAttestor},
        chain_client::ChainClient,
        handler::Handler,
        host::Host,
        key_manager::KeyManager,
        kv_store::{ConfigKey, ContractKey, NonceKey, TypedStore},
        server::{QuartzServer, WsListenerConfig},
        DefaultEnclave, Enclave,
    },
};
use serde_json::json;
use tendermint_rpc::event::Event as TmEvent;
use tokio::sync::mpsc;
use tonic::{Request, Response, Status};
use tracing::info;
use transfers_contract::msg::{
    execute::Request as TransferRequest,
    QueryMsg::{GetRequests, GetState},
};
use transfers_server::{TransfersOp, TransfersService};

use crate::{
    proto::{
        settlement_server::Settlement, QueryRequest, QueryResponse, UpdateRequest, UpdateResponse,
    },
    transfers_server::{QueryRequestMessage, UpdateRequestMessage},
    wslistener::WsListener,
};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();

    let admin_sk = std::env::var("ADMIN_SK")
        .map_err(|_| anyhow::anyhow!("Admin secret key not found in env vars"))?;

    let light_client_opts = LightClientOpts::new(
        args.chain_id.clone(),
        args.trusted_height.into(),
        Vec::from(args.trusted_hash)
            .try_into()
            .expect("invalid trusted hash"),
        (
            args.trust_threshold.numerator(),
            args.trust_threshold.denominator(),
        ),
        args.trusting_period,
        args.max_clock_drift,
        args.max_block_lag,
    )?;

    #[cfg(not(feature = "mock-sgx"))]
    let attestor = attestor::DcapAttestor {
        fmspc: args.fmspc.expect("FMSPC is required for DCAP"),
    };

    #[cfg(feature = "mock-sgx")]
    let attestor = attestor::MockAttestor::default();

    let config = Config::new(
        attestor.mr_enclave()?,
        Duration::from_secs(30 * 24 * 60),
        light_client_opts,
        args.tcbinfo_contract.map(|c| c.to_string()),
        args.dcap_verifier_contract.map(|c| c.to_string()),
    );

    let ws_config = WsListenerConfig {
        node_url: args.node_url,
        ws_url: args.ws_url,
        grpc_url: args.grpc_url,
        tx_sender: args.tx_sender,
        trusted_hash: args.trusted_hash,
        trusted_height: args.trusted_height,
        chain_id: args.chain_id,
        admin_sk,
    };

    // Event queue
    let (tx, mut rx) = mpsc::channel::<TransfersOp<DefaultAttestor>>(1);
    // Consumer task: dequeue and process events
    tokio::spawn(async move {
        while let Some(op) = rx.recv().await {
            if let Err(e) = op.client.process(op.event, op.config).await {
                println!("Error processing queued event: {}", e);
            }
        }
    });

    let contract = Arc::new(Mutex::new(None));
    let sk = Arc::new(Mutex::new(None));

    QuartzServer::new(
        config.clone(),
        contract.clone(),
        sk.clone(),
        attestor.clone(),
        ws_config,
    )
    .add_service(TransfersService::new(config, sk, contract, attestor, tx))
    .serve(args.rpc_addr)
    .await?;

    Ok(())
}

#[tonic::async_trait]
impl<A, K, S> Settlement for DefaultEnclave<A, K, S>
where
    A: Attestor + Clone,
    K: KeyManager<PubKey = VerifyingKey> + Clone,
    S: TypedStore<ContractKey<AccountId>> + TypedStore<NonceKey> + TypedStore<ConfigKey> + Clone,
{
    async fn run(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        request.handle(self).await
    }

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        request.handle(self).await
    }
}

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for UpdateRequest {
    type Error = Status;
    type Response = UpdateResponse;

    async fn handle(self, _ctx: &E) -> Result<Self::Response, Self::Error> {
        todo!()
    }
}

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for QueryRequest {
    type Error = Status;
    type Response = QueryResponse;

    async fn handle(self, _ctx: &E) -> Result<Self::Response, Self::Error> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub enum EnclaveRequest {
    Update(UpdateRequest),
    Query(QueryRequest),
}

#[derive(Clone, Debug)]
pub enum EnclaveResponse {
    Update(UpdateResponse),
    Query(QueryResponse),
}

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for EnclaveRequest {
    type Error = Status;
    type Response = EnclaveResponse;

    async fn handle(self, ctx: &E) -> Result<Self::Response, Self::Error> {
        match self {
            EnclaveRequest::Update(request) => {
                request.handle(ctx).await.map(EnclaveResponse::Update)
            }
            EnclaveRequest::Query(request) => request.handle(ctx).await.map(EnclaveResponse::Query),
        }
    }
}

#[derive(Clone, Debug)]
pub struct TransferEvent {
    pub contract: AccountId,
}

impl TryFrom<TmEvent> for TransferEvent {
    type Error = AnyhowError;

    fn try_from(event: TmEvent) -> Result<Self, Self::Error> {
        let Some(events) = &event.events else {
            return Err(anyhow!("no events in tx"));
        };

        if !events.keys().any(|k| k.starts_with("wasm-transfer.action")) {
            return Err(anyhow!("irrelevant event"));
        };

        let contract = events
            .get("execute._contract_address")
            .ok_or_else(|| anyhow!("missing execute._contract_address in events"))?
            .first()
            .ok_or_else(|| anyhow!("execute._contract_address is empty"))?
            .parse::<AccountId>()
            .map_err(|e| anyhow!("failed to parse contract address: {}", e))?;

        Ok(TransferEvent { contract })
    }
}

#[async_trait::async_trait]
impl<C> Handler<C> for TransferEvent
where
    C: ChainClient<Contract = AccountId>,
{
    type Error = AnyhowError;
    type Response = UpdateRequest;

    async fn handle(self, ctx: &C) -> Result<Self::Response, Self::Error> {
        let contract = self.contract;

        // Query contract state
        let requests: Vec<TransferRequest> = ctx
            .query_contract(&contract, json!(GetRequests {}).to_string())
            .await
            .map_err(|e| anyhow!("Problem querying contract state: {}", e))?;

        let state: HexBinary = ctx
            .query_contract(&contract, json!(GetState {}).to_string())
            .await
            .map_err(|e| anyhow!("Problem querying contract state: {}", e))?;

        let seq_num: Uint64 = ctx
            .query_contract(&contract, SEQUENCE_NUM_KEY.to_string())
            .await
            .map_err(|e| anyhow!("Problem querying contract state: {}", e))?;

        // Request body contents
        let update_contents = UpdateRequestMessage {
            state,
            requests,
            seq_num: seq_num.into(),
        };

        // Wait 2 blocks
        info!("Waiting 2 blocks for light client proof");
        ctx.wait_for_blocks(2)
            .await
            .map_err(|e| anyhow!("Problem waiting for proof: {}", e))?;

        // Call tm prover with trusted hash and height
        let proof = ctx
            .existence_proof(&contract, "requests")
            .await
            .map_err(|e| anyhow!("Problem getting existence proof: {}", e))?;

        // Merge the UpdateRequestMessage with the proof
        let mut proof_json = serde_json::to_value(proof)?;
        proof_json["msg"] = serde_json::to_value(&update_contents)?;

        // Build final request object
        let request = UpdateRequest {
            message: json!(proof_json).to_string(),
        };

        Ok(request)
    }
}

#[derive(Clone, Debug)]
pub struct QueryEvent {
    pub contract: AccountId,
    pub sender: String,
    pub ephemeral_pubkey: String,
}

impl TryFrom<TmEvent> for QueryEvent {
    type Error = AnyhowError;

    fn try_from(event: TmEvent) -> Result<Self, Self::Error> {
        let Some(events) = &event.events else {
            return Err(anyhow!("no events in tx"));
        };

        if !events.keys().any(|k| k.starts_with("wasm-transfer.action")) {
            return Err(anyhow!("irrelevant event"));
        };

        let contract = events
            .get("execute._contract_address")
            .ok_or_else(|| anyhow!("missing execute._contract_address in events"))?
            .first()
            .ok_or_else(|| anyhow!("execute._contract_address is empty"))?
            .parse::<AccountId>()
            .map_err(|e| anyhow!("failed to parse contract address: {}", e))?;

        let sender = events
            .get("message.sender")
            .ok_or_else(|| anyhow!("Missing message.sender in events"))?
            .first()
            .ok_or_else(|| anyhow!("execute.sender is empty"))?
            .to_owned();

        let ephemeral_pubkey = events
            .get("wasm-query_balance.emphemeral_pubkey")
            .ok_or_else(|| anyhow!("Missing wasm-query_balance.emphemeral_pubkey in events"))?
            .first()
            .ok_or_else(|| anyhow!("execute.query_balance.emphemeral_pubkey is empty"))?
            .to_owned();

        Ok(QueryEvent {
            contract,
            sender,
            ephemeral_pubkey,
        })
    }
}

#[async_trait::async_trait]
impl<C> Handler<C> for QueryEvent
where
    C: ChainClient<Contract = AccountId>,
{
    type Error = AnyhowError;
    type Response = QueryRequest;

    async fn handle(self, ctx: &C) -> Result<Self::Response, Self::Error> {
        let QueryEvent {
            contract,
            sender,
            ephemeral_pubkey,
        } = self;

        // Query contract state
        let state: HexBinary = ctx
            .query_contract(&contract, json!(GetState {}).to_string())
            .await
            .map_err(|e| anyhow!("Problem querying contract state: {}", e))?;

        // Build request
        let update_contents = QueryRequestMessage {
            state,
            address: Addr::unchecked(sender), // sender comes from TX event, therefore is checked
            ephemeral_pubkey: HexBinary::from_hex(&ephemeral_pubkey)?,
        };

        // Send QueryRequestMessage to enclave over tonic gRPC client
        let request = QueryRequest {
            message: json!(update_contents).to_string(),
        };

        Ok(request)
    }
}

#[derive(Clone, Debug)]
pub enum EnclaveEvent {
    Transfer(TransferEvent),
    Query(QueryEvent),
}

impl TryFrom<TmEvent> for EnclaveEvent {
    type Error = ();

    fn try_from(value: TmEvent) -> Result<Self, Self::Error> {
        if let Ok(event) = TransferEvent::try_from(value.clone()) {
            Ok(Self::Transfer(event))
        } else if let Ok(event) = QueryEvent::try_from(value) {
            Ok(Self::Query(event))
        } else {
            Err(())
        }
    }
}

#[async_trait::async_trait]
impl<C> Handler<C> for EnclaveEvent
where
    C: ChainClient<Contract = AccountId>,
{
    type Error = AnyhowError;
    type Response = EnclaveRequest;

    async fn handle(self, ctx: &C) -> Result<Self::Response, Self::Error> {
        match self {
            EnclaveEvent::Transfer(event) => event.handle(ctx).await.map(EnclaveRequest::Update),
            EnclaveEvent::Query(event) => event.handle(ctx).await.map(EnclaveRequest::Query),
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmrs::crypto::secp256k1::SigningKey;
    use quartz_common::{
        enclave::{
            chain_client::default::DefaultChainClient,
            host::DefaultHost,
            key_manager::{default::DefaultKeyManager, shared::SharedKeyManager},
            kv_store::{default::DefaultKvStore, shared::SharedKvStore},
        },
        proto::core_server::CoreServer,
    };
    use tokio::time::sleep;
    use tonic::transport::Server;

    use super::*;
    use crate::proto::settlement_server::SettlementServer;

    #[tokio::test]
    async fn test_tonic_service() -> Result<(), Box<dyn std::error::Error>> {
        let addr = "127.0.0.1:9095".parse().expect("hardcoded correct ip");
        let enclave = DefaultEnclave {
            attestor: attestor::MockAttestor,
            key_manager: SharedKeyManager::wrapping(DefaultKeyManager::default()),
            store: SharedKvStore::wrapping(DefaultKvStore::default()),
        };
        Server::builder()
            .add_service(CoreServer::new(enclave.clone()))
            .add_service(SettlementServer::new(enclave.clone()))
            .serve_with_shutdown(addr, async {
                sleep(Duration::from_secs(10)).await;
                println!("Shutting down...");
            })
            .await?;

        let ws_url = "ws://127.0.0.1/websocket"
            .parse()
            .expect("hardcoded correct URL");
        let chain_grpc_url = "http://127.0.0.1:9090"
            .parse()
            .expect("hardcoded correct URL");
        let host = DefaultHost::<_, _, EnclaveRequest, EnclaveEvent>::new(
            enclave,
            DefaultChainClient::new(SigningKey::random(), chain_grpc_url),
        );
        host.serve(ws_url).await?;

        Ok(())
    }
}
