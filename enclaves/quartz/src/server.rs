use tonic::{Request, Response, Status};

use crate::proto::quartz::{core_server::Core, SessionCreateRequest, SessionCreateResponse};

#[derive(Debug, Default)]
pub struct CoreService {}

#[tonic::async_trait]
impl Core for CoreService {
    async fn session_create(
        &self,
        request: Request<SessionCreateRequest>,
    ) -> Result<Response<SessionCreateResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = SessionCreateResponse {
            message: "Hello!".to_string(),
        };

        Ok(Response::new(reply))
    }
}
