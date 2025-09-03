//! gRPC service implementation for core enclave requests (handshake)
use cosmrs::AccountId;
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest, InstantiateResponse, SessionCreateRequest,
    SessionCreateResponse, SessionSetPubKeyRequest, SessionSetPubKeyResponse,
};
use tendermint::{block::Height, Hash};
use tonic::{Request, Response, Status};

use crate::{
    attestor::Attestor, handler::Handler, key_manager::KeyManager, store::Store, DefaultEnclave,
    Notification,
};

#[async_trait::async_trait]
impl<T, C> Handler<C> for Request<T>
where
    T: Handler<C>,
    C: Send + Sync,
{
    type Error = T::Error;
    type Response = Response<T::Response>;

    async fn handle(self, ctx: &C) -> Result<Self::Response, Self::Error> {
        let request = self.into_inner();
        let response = request.handle(ctx).await?;
        Ok(Response::new(response))
    }
}

#[async_trait::async_trait]
impl<C, A, K, S> Core for DefaultEnclave<C, A, K, S>
where
    C: Send + Sync + 'static,
    A: Attestor + Clone,
    K: KeyManager + Clone,
    S: Store<Contract = AccountId, Height = Height, Hash = Hash> + Clone,
{
    async fn instantiate(
        &self,
        request: Request<InstantiateRequest>,
    ) -> Result<Response<InstantiateResponse>, Status> {
        request.handle(self).await
    }

    async fn session_create(
        &self,
        request: Request<SessionCreateRequest>,
    ) -> Result<Response<SessionCreateResponse>, Status> {
        request.handle(self).await
    }

    async fn session_set_pub_key(
        &self,
        request: Request<SessionSetPubKeyRequest>,
    ) -> Result<Response<SessionSetPubKeyResponse>, Status> {
        let response = request.handle(self).await?;

        self.notifier_tx
            .send(Notification::HandshakeComplete)
            .await
            .expect("Receiver half of the channel must NOT be closed");

        Ok(response)
    }
}
