use std::time::Duration;

use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest, InstantiateResponse, SessionCreateRequest,
    SessionCreateResponse,
};
use serde::{Deserialize, Serialize};
use tendermint::Hash;
use tendermint_light_client::types::{Height, TrustThreshold};
use tonic::{Request, Response, Status};

#[derive(Clone, Debug)]
pub struct CoreService(pub Config);

#[tonic::async_trait]
impl Core for CoreService {
    async fn instantiate(
        &self,
        request: Request<InstantiateRequest>,
    ) -> Result<Response<InstantiateResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = InstantiateResponse {
            message: "Hello!".to_string(),
        };

        Ok(Response::new(reply))
    }
    async fn session_create(
        &self,
        request: Request<SessionCreateRequest>,
    ) -> Result<Response<SessionCreateResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = SessionCreateResponse {
            message: "Hello!".to_string(),
        };

        Ok(Response::new(reply))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    epoch_duration: Duration,
    light_client_opts: LightClientOpts,
}

impl Config {
    pub fn new(epoch_duration: Duration, light_client_opts: LightClientOpts) -> Self {
        Self {
            epoch_duration,
            light_client_opts,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LightClientOpts {
    chain_id: String,
    target_height: Height,
    trusted_height: Height,
    trusted_hash: Hash,
    trust_threshold: TrustThreshold,
    trusting_period: u64,
    max_clock_drift: u64,
    max_block_lag: u64,
}

impl LightClientOpts {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        chain_id: String,
        target_height: Height,
        trusted_height: Height,
        trusted_hash: Hash,
        trust_threshold: TrustThreshold,
        trusting_period: u64,
        max_clock_drift: u64,
        max_block_lag: u64,
    ) -> Self {
        Self {
            chain_id,
            target_height,
            trusted_height,
            trusted_hash,
            trust_threshold,
            trusting_period,
            max_clock_drift,
            max_block_lag,
        }
    }
}
