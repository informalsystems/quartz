use quartz_cw::{
    msg::{execute::session_create::SessionCreate, instantiate::CoreInstantiate},
    state::{Config, Nonce},
};
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest as RawInstantiateRequest,
    InstantiateResponse as RawInstantiateResponse, SessionCreateRequest as RawSessionCreateRequest,
    SessionCreateResponse as RawSessionCreateResponse,
};
use quartz_relayer::types::{InstantiateResponse, SessionCreateResponse};
use rand::Rng;
use tonic::{Request, Response, Status};

use crate::attestor::Attestor;

type TonicResult<T> = Result<T, Status>;

#[derive(Clone, PartialEq, Debug)]
pub struct CoreService<A> {
    config: Config,
    attestor: A,
}

impl<A> CoreService<A>
where
    A: Attestor,
{
    pub fn new(config: Config, attestor: A) -> Self {
        Self { config, attestor }
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
        let nonce = rand::thread_rng().gen::<Nonce>();
        let session_create_msg = SessionCreate::new(nonce);

        let quote = self
            .attestor
            .quote(session_create_msg)
            .map_err(|e| Status::internal(e.to_string()))?;

        let response = SessionCreateResponse::new(nonce, quote);
        Ok(Response::new(response.into()))
    }
}
