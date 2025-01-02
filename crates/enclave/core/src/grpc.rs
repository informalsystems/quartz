use cosmrs::AccountId;
use k256::ecdsa::VerifyingKey;
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest, InstantiateResponse, SessionCreateRequest,
    SessionCreateResponse, SessionSetPubKeyRequest, SessionSetPubKeyResponse,
};
use tonic::{Request, Response, Status};

use crate::{
    attestor::Attestor,
    chain_client::ChainClient,
    handler::Handler,
    key_manager::KeyManager,
    kv_store::{ConfigKey, ContractKey, NonceKey, TypedStore},
    DefaultEnclave,
};

#[async_trait::async_trait]
impl<A, C, K, S> Core for DefaultEnclave<A, C, K, S>
where
    A: Attestor + Clone,
    C: ChainClient<Contract = AccountId> + Clone,
    K: KeyManager<PubKey = VerifyingKey> + Clone,
    S: TypedStore<ContractKey<AccountId>> + TypedStore<NonceKey> + TypedStore<ConfigKey> + Clone,
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
        request.handle(self).await
    }
}
