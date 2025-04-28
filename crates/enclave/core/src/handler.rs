use cosmrs::AccountId;
use k256::ecdsa::VerifyingKey;
use log::{debug, error};
use quartz_proto::quartz::{
    InstantiateRequest, InstantiateResponse, SessionCreateRequest, SessionCreateResponse,
    SessionSetPubKeyRequest, SessionSetPubKeyResponse,
};
use tonic::Status;

use crate::{attestor::Attestor, key_manager::KeyManager, store::Store, Enclave};

pub type A<E> = <<E as Enclave>::Attestor as Attestor>::Attestation;
pub type RA<E> = <<E as Enclave>::Attestor as Attestor>::RawAttestation;

pub mod instantiate;
pub mod session_create;
pub mod session_set_pubkey;

/// A trait representing an asynchronous handler that processes a given context
/// and produces a response.
///
/// The [`Handler`] trait is used to define components that can transform or process
/// data based on an external context. It is generic over a `Context` type, which represents
/// the environment or input data required by the handler to produce a response.
#[async_trait::async_trait]
pub trait Handler<Context>: Send + Sync + 'static {
    /// The error type returned by this handler.
    type Error;
    /// The response type produced by this handler.
    type Response;

    /// Consumes the handler and processes the provided context.
    ///
    /// # Parameters
    ///
    /// - `ctx`: A reference to the context required to process the request.
    ///
    /// # Returns
    ///
    /// A `Result` containing the response if successful, or an error otherwise.
    async fn handle(self, ctx: &Context) -> Result<Self::Response, Self::Error>;
}

/// Core enclave requests AKA handshake requests
#[derive(Clone, Debug)]
pub enum CoreEnclaveRequest {
    Instantiate(InstantiateRequest),
    SessionCreate(SessionCreateRequest),
    SessionSetPubKey(SessionSetPubKeyRequest),
}

/// Core enclave responses
#[derive(Clone, Debug)]
pub enum CoreEnclaveResponse {
    Instantiate(InstantiateResponse),
    SessionCreate(SessionCreateResponse),
    SessionSetPubKey(SessionSetPubKeyResponse),
}

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for CoreEnclaveRequest
where
    E: Enclave,
    E::KeyManager: KeyManager<PubKey = VerifyingKey>,
    E::Store: Store<Contract = AccountId>,
{
    type Error = Status;
    type Response = CoreEnclaveResponse;

    async fn handle(self, ctx: &E) -> Result<Self::Response, Self::Error> {
        match self {
            CoreEnclaveRequest::Instantiate(req) => {
                debug!("Handling instantiate request");
                req.handle(ctx).await.map(CoreEnclaveResponse::Instantiate)
            }
            CoreEnclaveRequest::SessionCreate(req) => {
                debug!("Handling session create request");
                req.handle(ctx)
                    .await
                    .map(CoreEnclaveResponse::SessionCreate)
            }
            CoreEnclaveRequest::SessionSetPubKey(req) => {
                debug!("Handling session set pubkey request");
                req.handle(ctx)
                    .await
                    .map(CoreEnclaveResponse::SessionSetPubKey)
            }
        }
    }
}

pub fn ensure_seq_num_consistency(
    seq_num_in_store: u64,
    seq_num_on_chain: u64,
    pending_sequenced_requests: usize,
) -> Result<(), Status> {
    debug!(
        "Checking sequence number consistency - store: {}, chain: {}, pending: {}",
        seq_num_in_store, seq_num_on_chain, pending_sequenced_requests
    );

    if seq_num_on_chain < seq_num_in_store {
        error!(
            "Replay attempt detected - chain seq num ({}) < store seq num ({})",
            seq_num_on_chain, seq_num_in_store
        );
        return Err(Status::failed_precondition("replay attempted"));
    }

    // make sure number of pending requests are equal to the diff b/w on-chain v/s in-mem seq num
    let seq_num_diff = seq_num_on_chain - seq_num_in_store;
    if seq_num_diff != pending_sequenced_requests as u64 {
        error!(
            "Sequence number mismatch - diff: {}, pending: {}",
            seq_num_diff, pending_sequenced_requests
        );
        return Err(Status::failed_precondition(format!(
            "seq_num_diff mismatch: num({seq_num_diff}) v/s diff({pending_sequenced_requests})"
        )));
    }

    debug!("Sequence number consistency check passed");
    Ok(())
}
