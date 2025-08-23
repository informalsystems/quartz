use anyhow::{anyhow, Error as AnyhowError};
use cosmrs::AccountId;
use ping_pong_contract::{msg::execute::Ping, state::PINGS_KEY};
use quartz_common::enclave::{
    chain_client::{default::Query, ChainClient},
    handler::Handler,
};
use serde_json::json;
use tendermint_rpc::event::Event as TmEvent;
use tracing::info;

use crate::{proto::PingRequest, request::EnclaveRequest};

#[derive(Clone, Debug)]
pub enum EnclaveEvent {
    Ping(PingEvent),
}

impl TryFrom<TmEvent> for EnclaveEvent {
    type Error = AnyhowError;

    fn try_from(value: TmEvent) -> Result<Self, Self::Error> {
        if let Ok(event) = PingEvent::try_from(value.clone()) {
            Ok(Self::Ping(event))
        } else {
            Err(anyhow::anyhow!("unsupported event"))
        }
    }
}

#[async_trait::async_trait]
impl<C> Handler<C> for EnclaveEvent
where
    C: ChainClient<Contract = AccountId, Query = Query>,
{
    type Error = AnyhowError;
    type Response = EnclaveRequest;

    async fn handle(self, ctx: &C) -> Result<Self::Response, Self::Error> {
        match self {
            EnclaveEvent::Ping(event) => event.handle(ctx).await.map(EnclaveRequest::Ping),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PingEvent {
    pub contract: AccountId,
    pub ping: Ping,
}

impl TryFrom<TmEvent> for PingEvent {
    type Error = AnyhowError;

    fn try_from(event: TmEvent) -> Result<Self, Self::Error> {
        let Some(events) = &event.events else {
            return Err(anyhow!("no events in tx"));
        };

        if !events.keys().any(|k| k.starts_with("wasm.action")) {
            return Err(anyhow!("irrelevant event"));
        };

        let contract = events
            .get("execute._contract_address")
            .ok_or_else(|| anyhow!("missing execute._contract_address in events"))?
            .first()
            .ok_or_else(|| anyhow!("execute._contract_address is empty"))?
            .parse::<AccountId>()
            .map_err(|e| anyhow!("failed to parse contract address: {}", e))?;

        let ping: Ping = {
            let ping_str = events
                .get("wasm.ping_data")
                .and_then(|v| v.first())
                .ok_or_else(|| anyhow!("Missing ping data in event"))?;
            serde_json::from_str(ping_str)?
        };

        Ok(Self { contract, ping })
    }
}

#[async_trait::async_trait]
impl<C> Handler<C> for PingEvent
where
    C: ChainClient<Contract = AccountId, Query = Query>,
{
    type Error = AnyhowError;
    type Response = PingRequest;

    async fn handle(self, ctx: &C) -> Result<Self::Response, Self::Error> {
        let contract = self.contract;

        // Wait 2 blocks
        info!("Waiting 2 blocks for light client proof");
        ctx.wait_for_blocks(2)
            .await
            .map_err(|e| anyhow!("Problem waiting for proof: {}", e))?;

        // Call tm prover with trusted hash and height
        let proof = ctx
            .existence_proof(&contract, PINGS_KEY)
            .await
            .map_err(|e| anyhow!("Problem getting existence proof: {}", e))?;

        // Merge the UpdateRequestMessage with the proof
        let mut proof_json = serde_json::to_value(proof)?;
        proof_json["msg"] = serde_json::to_value(&self.ping)?;

        // Build final request object
        let request = PingRequest {
            message: json!(proof_json).to_string(),
        };

        Ok(request)
    }
}
