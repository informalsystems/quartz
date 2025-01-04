use quartz_common::enclave::{handler::Handler, Enclave};
use tonic::Status;

use crate::proto::{UpdateRequest, UpdateResponse};

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for UpdateRequest {
    type Error = Status;
    type Response = UpdateResponse;

    async fn handle(self, _ctx: &E) -> Result<Self::Response, Self::Error> {
        todo!()
    }
}
