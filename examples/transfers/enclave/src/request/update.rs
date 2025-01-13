use quartz_common::enclave::{handler::Handler, Enclave};
use tonic::Status;
use transfers_contract::msg::execute;

use crate::proto::UpdateRequest;

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for UpdateRequest {
    type Error = Status;
    type Response = execute::UpdateMsg;

    async fn handle(self, _ctx: &E) -> Result<Self::Response, Self::Error> {
        todo!()
    }
}
