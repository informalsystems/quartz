use quartz_common::enclave::{handler::Handler, DefaultSharedEnclave};
use tonic::{Request, Response, Status};

use crate::proto::{ping_pong_server::PingPong, PingRequest, PongResponse};

#[tonic::async_trait]
impl PingPong for DefaultSharedEnclave<()> {
    async fn run(&self, request: Request<PingRequest>) -> Result<Response<PongResponse>, Status> {
        let response = request.handle(self).await?;
        Ok(response.map(|r| PongResponse {
            message: serde_json::to_string(&r).unwrap(),
        }))
    }
}
