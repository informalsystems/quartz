use quartz_contract_core::msg::{execute::attested::Attested, instantiate::CoreInstantiate};
use quartz_proto::quartz::{
    InstantiateRequest as RawInstantiateRequest, InstantiateResponse as RawInstantiateResponse,
};
use tonic::Status;

use crate::{
    attestor::Attestor,
    handler::{Handler, A, RA},
    kv_store::{ConfigKey, ConfigKeyName, KvStore},
    types::InstantiateResponse,
    Enclave,
};

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for RawInstantiateRequest {
    type Error = Status;
    type Response = RawInstantiateResponse;

    async fn handle(&mut self, ctx: &mut E) -> Result<Self::Response, Self::Error> {
        // create `CoreInstantiate` msg and attest to it
        let config = ctx
            .store()
            .get(ConfigKey::new(ConfigKeyName))
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("config not found"))?;
        let msg = CoreInstantiate::new(config);
        let attestation = ctx
            .attestor()
            .attestation(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;
        let attested_msg = Attested::new(msg, attestation);

        // return response with attested `CoreInstantiate` msg
        let response: InstantiateResponse<A<E>, RA<E>> = InstantiateResponse::new(attested_msg);
        Ok(response.into())
    }
}
