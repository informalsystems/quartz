use quartz_cw::{msg::instantiate::CoreInstantiate, state::Config};
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest as RawInstantiateRequest,
    InstantiateResponse as RawInstantiateResponse, SessionCreateRequest, SessionCreateResponse,
};
use quartz_relayer::types::InstantiateResponse;
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
        request: Request<SessionCreateRequest>,
    ) -> TonicResult<Response<SessionCreateResponse>> {
        println!("Got a request: {:?}", request);

        let reply = SessionCreateResponse {
            message: "Hello!".to_string(),
        };

        Ok(Response::new(reply))
    }
}
