use std::sync::{Arc, Mutex};

use k256::ecdsa::SigningKey;
use tonic::{Request, Response, Result as TonicResult, Status};

use crate::{
    attestor::Attestor,
    proto::{clearing_server::Clearing, RunClearingRequest, RunClearingResponse},
};

#[derive(Clone, Debug)]
pub struct MtcsService<A> {
    sk: Arc<Mutex<Option<SigningKey>>>,
    attestor: A,
}

impl<A> MtcsService<A>
where
    A: Attestor,
{
    pub fn new(sk: Arc<Mutex<Option<SigningKey>>>, attestor: A) -> Self {
        Self { sk, attestor }
    }
}

#[tonic::async_trait]
impl<A> Clearing for MtcsService<A>
where
    A: Attestor + Send + Sync + 'static,
{
    async fn run(
        &self,
        _request: Request<RunClearingRequest>,
    ) -> TonicResult<Response<RunClearingResponse>> {
        todo!()
    }
}
