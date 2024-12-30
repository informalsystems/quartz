use quartz_contract_core::msg::{execute::attested::Attested, instantiate::CoreInstantiate};
use quartz_proto::quartz::{
    InstantiateRequest as RawInstantiateRequest, InstantiateResponse as RawInstantiateResponse,
};
use tonic::Status;

use crate::{
    attestor::Attestor,
    handler::{Handler, A, RA},
    types::InstantiateResponse,
    Enclave,
};

impl<E: Enclave> Handler<E> for RawInstantiateRequest {
    type Error = Status;
    type Response = RawInstantiateResponse;

    fn handle(&mut self, ctx: &mut E) -> Result<Self::Response, Self::Error> {
        // create `CoreInstantiate` msg and attest to it
        let msg = CoreInstantiate::new(ctx.config());
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
