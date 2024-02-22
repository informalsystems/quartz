use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest, InstantiateResponse, SessionCreateRequest,
    SessionCreateResponse,
};
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct CoreService;

#[tonic::async_trait]
impl Core for CoreService {
    async fn instantiate(
        &self,
        request: Request<InstantiateRequest>,
    ) -> Result<Response<InstantiateResponse>, Status> {
        println!("Got a request: {:?}", request);

        let reply = InstantiateResponse {
            message: "Hello!".to_string(),
        };

        Ok(Response::new(reply))
    }
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
