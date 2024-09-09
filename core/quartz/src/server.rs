use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cw_proof::proof::{
    cw::{CwProof, RawCwProof},
    Proof,
};
use k256::ecdsa::SigningKey;
use quartz_cw::{
    msg::{
        execute::{
            attested::Attested, session_create::SessionCreate,
            session_set_pub_key::SessionSetPubKey,
        },
        instantiate::CoreInstantiate,
    },
    state::{Config, LightClientOpts, Nonce, Session},
};
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest as RawInstantiateRequest,
    InstantiateResponse as RawInstantiateResponse, SessionCreateRequest as RawSessionCreateRequest,
    SessionCreateResponse as RawSessionCreateResponse,
    SessionSetPubKeyRequest as RawSessionSetPubKeyRequest,
    SessionSetPubKeyResponse as RawSessionSetPubKeyResponse,
};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tendermint_light_client::{
    light_client::Options,
    types::{LightBlock, TrustThreshold},
};
use tm_stateless_verifier::make_provider;
use tonic::{Request, Response, Result as TonicResult, Status};

use crate::{
    attestor::Attestor,
    types::{InstantiateResponse, SessionCreateResponse, SessionSetPubKeyResponse},
};

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
        let msg = CoreInstantiate::new(self.config.clone());

        let attestation = self
            .attestor
            .attestation(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;
        let attested_msg = Attested::new(msg, attestation);

        let response: InstantiateResponse<A::Attestation, A::RawAttestation> =
            InstantiateResponse::new(attested_msg);
        Ok(Response::new(response.into()))
    }

    async fn session_create(
        &self,
        _request: Request<RawSessionCreateRequest>,
    ) -> TonicResult<Response<RawSessionCreateResponse>> {
        // FIXME(hu55a1n1) - disallow calling more than once
        let mut nonce = self.nonce.lock().unwrap();
        *nonce = rand::thread_rng().gen::<Nonce>();
        let msg = SessionCreate::new(*nonce);

        let attestation = self
            .attestor
            .attestation(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;
        let attested_msg = Attested::new(msg, attestation);

        let response: SessionCreateResponse<A::Attestation, A::RawAttestation> =
            SessionCreateResponse::new(attested_msg);
        Ok(Response::new(response.into()))
    }

    async fn session_set_pub_key(
        &self,
        request: Request<RawSessionSetPubKeyRequest>,
    ) -> TonicResult<Response<RawSessionSetPubKeyResponse>> {
        // FIXME(hu55a1n1) - disallow calling more than once
        let proof: ProofOfPublication<Option<()>> =
            serde_json::from_str(&request.into_inner().message)
                .map_err(|e| Status::invalid_argument(e.to_string()))?;

        let (value, _msg) = proof
            .verify(self.config.light_client_opts())
            .map_err(Status::failed_precondition)?;

        let session: Session = serde_json::from_slice(&value).unwrap();
        let nonce = self.nonce.lock().unwrap();

        if session.nonce() != *nonce {
            return Err(Status::unauthenticated("nonce mismatch"));
        }

        let sk = SigningKey::random(&mut rand::thread_rng());
        *self.sk.lock().unwrap() = Some(sk.clone());
        let pk = sk.verifying_key();

        let msg = SessionSetPubKey::new(*nonce, *pk);

        let attestation = self
            .attestor
            .attestation(msg.clone())
            .map_err(|e| Status::internal(e.to_string()))?;
        let attested_msg = Attested::new(msg, attestation);

        let response: SessionSetPubKeyResponse<A::Attestation, A::RawAttestation> =
            SessionSetPubKeyResponse::new(attested_msg);
        Ok(Response::new(response.into()))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProofOfPublication<M> {
    light_client_proof: Vec<LightBlock>,
    merkle_proof: RawCwProof,
    msg: M,
}

impl<M> ProofOfPublication<M> {
    pub fn verify(self, light_client_opts: &LightClientOpts) -> Result<(Vec<u8>, M), String> {
        let config_trust_threshold = light_client_opts.trust_threshold();
        let trust_threshold =
            TrustThreshold::new(config_trust_threshold.0, config_trust_threshold.1).unwrap();

        let config_trusting_period = light_client_opts.trusting_period();
        let trusting_period = Duration::from_secs(config_trusting_period);

        let config_clock_drift = light_client_opts.max_clock_drift();
        let clock_drift = Duration::from_secs(config_clock_drift);
        let options = Options {
            trust_threshold,
            trusting_period,
            clock_drift,
        };

        let target_height = self.light_client_proof.last().unwrap().height();

        let primary_block = make_provider(
            light_client_opts.chain_id(),
            light_client_opts.trusted_height().try_into().unwrap(),
            light_client_opts
                .trusted_hash()
                .to_vec()
                .try_into()
                .unwrap(),
            self.light_client_proof,
            options,
        )
        .and_then(|mut primary| primary.verify_to_height(target_height))
        .map_err(|e| e.to_string())?;

        let proof = CwProof::from(self.merkle_proof);
        proof
            .verify(
                primary_block
                    .signed_header
                    .header
                    .app_hash
                    .as_bytes()
                    .to_vec(),
            )
            .map_err(|e| e.to_string())?;

        Ok((proof.value, self.msg))
    }
}
