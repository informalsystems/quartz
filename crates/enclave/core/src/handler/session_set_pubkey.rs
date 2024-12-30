use cosmrs::AccountId;
use k256::ecdsa::VerifyingKey;
use quartz_contract_core::{
    msg::execute::{attested::Attested, session_set_pub_key::SessionSetPubKey},
    state::{Session, SESSION_KEY},
};
use quartz_proto::quartz::{
    SessionSetPubKeyRequest as RawSessionSetPubKeyRequest,
    SessionSetPubKeyResponse as RawSessionSetPubKeyResponse,
};
use tonic::Status;

use crate::{
    attestor::Attestor,
    handler::{Handler, A, RA},
    key_manager::KeyManager,
    kv_store::{
        ConfigKey, ConfigKeyName, ContractKey, ContractKeyName, KvStore, NonceKey, NonceKeyName,
    },
    server::ProofOfPublication,
    types::SessionSetPubKeyResponse,
    Enclave,
};

#[async_trait::async_trait]
impl<E> Handler<E> for RawSessionSetPubKeyRequest
where
    E: Enclave<Contract = AccountId>,
    E::KeyManager: KeyManager<PubKey = VerifyingKey>,
{
    type Error = Status;
    type Response = RawSessionSetPubKeyResponse;

    async fn handle(&mut self, ctx: &mut E) -> Result<Self::Response, Self::Error> {
        // verify proof of publication
        let proof: ProofOfPublication<Option<()>> = serde_json::from_str(&self.message)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let contract = ctx
            .store()
            .get(ContractKey::new(ContractKeyName))
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("contract not found"))?;
        let config = ctx
            .store()
            .get(ConfigKey::new(ConfigKeyName))
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("config not found"))?;
        let (value, _msg) = proof
            .verify(
                config.light_client_opts(),
                contract,
                SESSION_KEY.to_string(),
                None,
            )
            .map_err(Status::failed_precondition)?;

        // make sure session nonce matches what we have locally
        let session: Session = serde_json::from_slice(&value).unwrap();
        let nonce = ctx
            .store()
            .get(NonceKey::new(NonceKeyName))
            .map_err(|e| Status::internal(e.to_string()))?
            .ok_or_else(|| Status::not_found("nonce not found"))?;
        if session.nonce() != nonce {
            return Err(Status::unauthenticated("nonce mismatch"));
        }

        // generate enclave key
        ctx.key_manager().keygen();
        let pk = ctx
            .key_manager()
            .pub_key()
            .ok_or_else(|| Status::internal("failed to get public key"))?;

        // create `SessionSetPubKey` msg and attest to it
        let msg = SessionSetPubKey::new(nonce, pk);
        let attestation = ctx
            .attestor()
            .attestation(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;
        let attested_msg = Attested::new(msg, attestation);

        // return response with attested `SessionCreate` msg
        let response: SessionSetPubKeyResponse<A<E>, RA<E>> =
            SessionSetPubKeyResponse::new(attested_msg);
        Ok(response.into())
    }
}
