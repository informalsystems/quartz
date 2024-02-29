use std::sync::{Arc, Mutex};

use k256::ecdsa::SigningKey;
use quartz_cw::{
    msg::{
        execute::{session_create::SessionCreate, session_set_pub_key::SessionSetPubKey},
        instantiate::CoreInstantiate,
    },
    state::{Config, Nonce},
};
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest as RawInstantiateRequest,
    InstantiateResponse as RawInstantiateResponse, SessionCreateRequest as RawSessionCreateRequest,
    SessionCreateResponse as RawSessionCreateResponse,
    SessionSetPubKeyRequest as RawSessionSetPubKeyRequest,
    SessionSetPubKeyResponse as RawSessionSetPubKeyResponse,
};
use quartz_relayer::types::{InstantiateResponse, SessionCreateResponse, SessionSetPubKeyResponse};
use rand::Rng;
use tonic::{Request, Response, Result as TonicResult, Status};

use crate::attestor::Attestor;

#[derive(Clone, Debug)]
pub struct CoreService<A> {
    config: Config,
    nonce: Arc<Mutex<Nonce>>,
    sk: Arc<Mutex<Option<SigningKey>>>,
    attestor: A,
}

impl<A> CoreService<A>
where
    A: Attestor,
{
    pub fn new(config: Config, sk: Arc<Mutex<Option<SigningKey>>>, attestor: A) -> Self {
        Self {
            config,
            nonce: Arc::new(Mutex::new([0u8; 32])),
            sk,
            attestor,
        }
    }
}

#[tonic::async_trait]
impl<A> Core for CoreService<A>
where
    A: Attestor + Send + Sync + 'static,
{
    async fn instantiate(
        &self,
        _request: Request<RawInstantiateRequest>,
    ) -> TonicResult<Response<RawInstantiateResponse>> {
        let core_instantiate_msg = CoreInstantiate::new(self.config.clone());

        let quote = self
            .attestor
            .quote(core_instantiate_msg)
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = InstantiateResponse::new(self.config.clone(), quote);
        Ok(Response::new(response.into()))
    }
    async fn session_create(
        &self,
        _request: Request<RawSessionCreateRequest>,
    ) -> TonicResult<Response<RawSessionCreateResponse>> {
        // FIXME(hu55a1n1) - disallow calling more than once
        let mut nonce = self.nonce.lock().unwrap();
        *nonce = rand::thread_rng().gen::<Nonce>();

        let session_create_msg = SessionCreate::new(*nonce);

        let quote = self
            .attestor
            .quote(session_create_msg)
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = SessionCreateResponse::new(*nonce, quote);
        Ok(Response::new(response.into()))
    }

    async fn session_set_pub_key(
        &self,
        _request: Request<RawSessionSetPubKeyRequest>,
    ) -> TonicResult<Response<RawSessionSetPubKeyResponse>> {
        // FIXME(hu55a1n1) - disallow calling more than once
        let nonce = self.nonce.lock().unwrap();

        let sk = SigningKey::random(&mut rand::thread_rng());
        *self.sk.lock().unwrap() = Some(sk.clone());
        let pk = sk.verifying_key();

        let session_set_pub_key_msg = SessionSetPubKey::new(*nonce, *pk);

        let quote = self
            .attestor
            .quote(session_set_pub_key_msg)
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = SessionSetPubKeyResponse::new(*nonce, *pk, quote);
        Ok(Response::new(response.into()))
    }
}
