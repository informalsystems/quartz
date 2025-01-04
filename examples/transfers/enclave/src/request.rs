use quartz_common::enclave::{handler::Handler, Enclave};
use tonic::Status;

use crate::proto::{QueryRequest, QueryResponse, UpdateRequest, UpdateResponse};

pub mod query;
pub mod update;

#[derive(Clone, Debug)]
pub enum EnclaveRequest {
    Update(UpdateRequest),
    Query(QueryRequest),
}

#[derive(Clone, Debug)]
pub enum EnclaveResponse {
    Update(UpdateResponse),
    Query(QueryResponse),
}

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for EnclaveRequest {
    type Error = Status;
    type Response = EnclaveResponse;

    async fn handle(self, ctx: &E) -> Result<Self::Response, Self::Error> {
        match self {
            EnclaveRequest::Update(request) => {
                request.handle(ctx).await.map(EnclaveResponse::Update)
            }
            EnclaveRequest::Query(request) => request.handle(ctx).await.map(EnclaveResponse::Query),
        }
    }
}
