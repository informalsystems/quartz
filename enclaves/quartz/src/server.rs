use std::time::Duration;

use quartz_cw::{
    msg::{execute::attested::HasUserData, instantiate::CoreInstantiate},
    state::{Config, UserData},
};
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest, InstantiateResponse, SessionCreateRequest,
    SessionCreateResponse,
};
use serde::{Deserialize, Serialize};
use tendermint::Hash;
use tendermint_light_client::types::{Height, TrustThreshold};
use tonic::{Request, Response, Status};

#[derive(Clone, Debug)]
pub struct CoreService {
    config: Config,
}

impl CoreService {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

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

pub fn attestion_quote(user_data: UserData) -> IoResult<Vec<u8>> {
    let mut user_report_data = File::create("/dev/attestation/user_report_data")?;
    user_report_data.write_all(user_data.as_slice())?;
    user_report_data.flush()?;

    let quote = read("/dev/attestation/quote")?;
    Ok(quote)
}
