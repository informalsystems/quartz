use anyhow::anyhow;
use cosmrs::{abci::GasInfo, crypto::secp256k1::SigningKey, AccountId};
use cw_client::{CwClient, GrpcClient};
use futures_util::StreamExt;
use log::{debug, error, info, trace};
use quartz_tm_prover::{
    config::{Config as TmProverConfig, ProofOutput},
    prover::prove,
};
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Value};
use tendermint::{block::Height, chain::Id as TmChainId, Hash};
use tendermint_rpc::{query::EventType, SubscriptionClient, WebSocketClient};

use crate::chain_client::ChainClient;

/// A default, thread-safe Tendermint chain client.
/// This implementation uses -
///     - gRPC for sending transactions and running queries
///     - websocket for waiting for blocks
///     - tendermint HTTP RPC for generating light client proofs
pub struct DefaultChainClient {
    chain_id: TmChainId,
    grpc_client: GrpcClient,
    node_url: Url,
    ws_url: Url,
    trusted_height: Height,
    trusted_hash: Hash,
}

impl DefaultChainClient {
    pub fn new(
        chain_id: TmChainId,
        signer: SigningKey,
        grpc_url: Url,
        node_url: Url,
        ws_url: Url,
        trusted_height: Height,
        trusted_hash: Hash,
    ) -> Self {
        info!("Creating new default chain client for chain ID: {chain_id}");
        DefaultChainClient {
            chain_id,
            grpc_client: GrpcClient::new(signer, grpc_url),
            node_url,
            ws_url,
            trusted_height,
            trusted_hash,
        }
    }
}

pub enum Query {
    Json(Value),
    String(String),
}

impl From<Value> for Query {
    fn from(value: Value) -> Self {
        Self::Json(value)
    }
}

impl From<String> for Query {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[async_trait::async_trait]
impl ChainClient for DefaultChainClient {
    type Contract = AccountId;
    type Error = anyhow::Error;
    type Proof = ProofOutput;
    type Query = Query;
    type TxConfig = DefaultTxConfig;
    type TxOutput = String;

    async fn query_contract<R: DeserializeOwned + Default + Send>(
        &self,
        contract: &Self::Contract,
        query: impl Into<Self::Query> + Send,
    ) -> Result<R, Self::Error> {
        debug!("Querying contract: {contract}");
        match query.into() {
            Query::Json(q) => {
                trace!("Executing JSON query");
                self.grpc_client.query_smart(contract, q).await
            }
            Query::String(q) => {
                trace!("Executing raw query");
                self.grpc_client.query_raw(contract, q).await
            }
        }
    }

    async fn existence_proof(
        &self,
        contract: &Self::Contract,
        storage_key: &str,
    ) -> Result<Self::Proof, Self::Error> {
        debug!("Generating existence proof for contract {contract} with storage key {storage_key}");

        let prover_config = TmProverConfig {
            primary: self.node_url.as_str().parse()?,
            witnesses: self.node_url.as_str().parse()?,
            trusted_height: self.trusted_height,
            trusted_hash: self.trusted_hash,
            verbose: "1".parse()?,
            contract_address: contract.clone(),
            storage_key: storage_key.to_string(),
            chain_id: self.chain_id.to_string(),
            ..Default::default()
        };

        let proof_output = tokio::task::spawn_blocking(move || {
            trace!("Spawning blocking task for proof generation");
            // Create a new runtime inside the blocking thread.
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                prove(prover_config).await.map_err(|report| {
                    error!("Tendermint prover failed: {}", report);
                    anyhow!("Tendermint prover failed. Report: {}", report)
                })
            })
        })
        .await??; // Handle both JoinError and your custom error
        Ok(proof_output)
    }

    async fn send_tx<M: Serialize>(
        &self,
        contract: &Self::Contract,
        msgs: impl Iterator<Item = M> + Send + Sync,
        config: Self::TxConfig,
    ) -> Result<Self::TxOutput, Self::Error> {
        debug!(
            "Sending transaction to contract {contract} with gas {}",
            config.gas
        );
        self.grpc_client
            .tx_execute(
                contract,
                &self.chain_id,
                config.gas,
                "",
                msgs.map(|m| json!(m)),
                &config.amount,
            )
            .await
    }

    async fn simulate_tx<M: Serialize>(
        &self,
        contract: &Self::Contract,
        msgs: impl Iterator<Item = M> + Send + Sync,
        config: DefaultTxConfig,
    ) -> Result<GasInfo, Self::Error> {
        debug!(
            "Simulating a transaction to contract {contract} with gas {}",
            config.gas
        );
        self.grpc_client
            .tx_simulate(
                contract,
                &self.chain_id,
                config.gas,
                "",
                msgs.map(|m| json!(m)),
                &config.amount,
            )
            .await
    }

    async fn wait_for_blocks(&self, blocks: u8) -> Result<(), Self::Error> {
        debug!("Waiting for {} blocks", blocks);
        let (client, driver) = WebSocketClient::new(self.ws_url.to_string().as_str()).await?;

        let driver_handle = tokio::spawn(async move { driver.run().await });

        // Subscription functionality
        let mut subs = client.subscribe(EventType::NewBlock.into()).await?;

        // Wait 2 NewBlock events
        let mut ev_count = 2_i32;

        while let Some(res) = subs.next().await {
            let _ev = res?;
            ev_count -= 1;
            trace!("Received new block event, {} remaining", ev_count);
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
}

pub struct DefaultTxConfig {
    pub gas: u64,
    pub amount: String,
}
