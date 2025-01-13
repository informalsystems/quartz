use quartz_common::enclave::{handler::Handler, Enclave};
use tonic::Status;
use transfers_contract::msg::execute;

use crate::proto::QueryRequest;

#[async_trait::async_trait]
impl<E: Enclave> Handler<E> for QueryRequest {
    type Error = Status;
    type Response = execute::QueryResponseMsg;

    async fn handle(self, _ctx: &E) -> Result<Self::Response, Self::Error> {
        todo!()
    }
}
