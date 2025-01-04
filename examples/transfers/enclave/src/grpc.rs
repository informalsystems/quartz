use cosmrs::AccountId;
use k256::ecdsa::VerifyingKey;
use quartz_common::enclave::{
    attestor::Attestor,
    handler::Handler,
    key_manager::KeyManager,
    kv_store::{ConfigKey, ContractKey, NonceKey, TypedStore},
    DefaultEnclave,
};
use tonic::{Request, Response, Status};

use crate::proto::{
    settlement_server::Settlement, QueryRequest, QueryResponse, UpdateRequest, UpdateResponse,
};

#[tonic::async_trait]
impl<A, K, S> Settlement for DefaultEnclave<A, K, S>
where
    A: Attestor + Clone,
    K: KeyManager<PubKey = VerifyingKey> + Clone,
    S: TypedStore<ContractKey<AccountId>> + TypedStore<NonceKey> + TypedStore<ConfigKey> + Clone,
{
    async fn run(
        &self,
        request: Request<UpdateRequest>,
    ) -> Result<Response<UpdateResponse>, Status> {
        request.handle(self).await
    }

    async fn query(
        &self,
        request: Request<QueryRequest>,
    ) -> Result<Response<QueryResponse>, Status> {
        request.handle(self).await
    }
}
