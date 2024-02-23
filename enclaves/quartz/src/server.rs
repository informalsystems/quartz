use std::{
    fs::{read, File},
    io::{Result as IoResult, Write},
};

use quartz_cw::{
    msg::{execute::attested::HasUserData, instantiate::CoreInstantiate},
    state::{Config, UserData},
};
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest as RawInstantiateRequest,
    InstantiateResponse as RawInstantiateResponse, SessionCreateRequest, SessionCreateResponse,
};
use tonic::{Request, Response, Status};

use crate::types::InstantiateResponse;

type TonicResult<T> = Result<T, Status>;

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
        _request: Request<RawInstantiateRequest>,
    ) -> TonicResult<Response<RawInstantiateResponse>> {
        let core_instantiate_msg = CoreInstantiate::new(self.config.clone());

        let user_data = core_instantiate_msg.user_data();
        let quote = attestion_quote(user_data).map_err(|e| Status::internal(e.to_string()))?;

        let response = InstantiateResponse::new(self.config.clone(), quote);
        Ok(Response::new(response.into()))
    }
    async fn session_create(
        &self,
        request: Request<SessionCreateRequest>,
    ) -> TonicResult<Response<SessionCreateResponse>> {
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
