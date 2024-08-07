use std::process::Command;

use async_trait::async_trait;
use tracing::{debug, info};

use crate::{
    error::Error,
    handler::Handler,
    request::enclave_start::EnclaveStartRequest,
    response::{enclave_start::EnclaveStartResponse, Response},
    Config,
};

#[async_trait]
impl Handler for EnclaveStartRequest {
    type Error = Error;
    type Response = Response;

    async fn handle(self, config: Config) -> Result<Self::Response, Self::Error> {

        // gramine private key

        // gramine manifest

        // gramine sign 

        // run quartz enclave

        Ok(EnclaveStartResponse.into())
    }
}
