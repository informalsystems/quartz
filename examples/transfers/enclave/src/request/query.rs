use quartz_common::enclave::{handler::Handler, Enclave};
use tonic::Status;

use crate::proto::{QueryRequest, QueryResponse};

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for QueryRequest {
    type Error = Status;
    type Response = QueryResponse;

    async fn handle(self, _ctx: &E) -> Result<Self::Response, Self::Error> {
        todo!()
    }
}
