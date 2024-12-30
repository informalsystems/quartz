use quartz_contract_core::{
    msg::execute::{attested::Attested, session_create::SessionCreate},
    state::Nonce,
};
use quartz_proto::quartz::{
    SessionCreateRequest as RawSessionCreateRequest,
    SessionCreateResponse as RawSessionCreateResponse,
};
use rand::Rng;
use tonic::Status;

use crate::{
    attestor::Attestor,
    handler::{Handler, A, RA},
    kv_store::{ContractKey, ContractKeyName, KvStore, NonceKey, NonceKeyName},
    types::SessionCreateResponse,
    Enclave,
};

impl<E: Enclave> Handler<E> for RawSessionCreateRequest {
    type Error = Status;
    type Response = RawSessionCreateResponse;

    fn handle(&mut self, ctx: &mut E) -> Result<Self::Response, Self::Error> {
        // store contract
        let deployed_contract: E::Contract = serde_json::from_str(&self.message)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let prev_contract = ctx
            .store()
            .set(ContractKey::new(ContractKeyName), deployed_contract.clone())
            .map_err(|e| Status::internal(e.to_string()))?;
        if prev_contract.is_some() {
            return Err(Status::already_exists(
                "contract already exists".to_string(),
            ));
        }

        // generate nonce and store it
        let nonce = rand::thread_rng().gen::<Nonce>();
        let prev_nonce = ctx
            .store()
            .set(NonceKey::new(NonceKeyName), nonce)
            .map_err(|e| Status::internal(e.to_string()))?;
        if prev_nonce.is_some() {
            return Err(Status::already_exists("nonce already exists".to_string()));
        }

        // create `SessionCreate` msg and attest to it
        let msg = SessionCreate::new(nonce, deployed_contract.to_string());
        let attestation = ctx
            .attestor()
            .attestation(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;
        let attested_msg = Attested::new(msg, attestation);

        // return response with attested `SessionCreate` msg
        let response: SessionCreateResponse<A<E>, RA<E>> = SessionCreateResponse::new(attested_msg);
        Ok(response.into())
    }
}
