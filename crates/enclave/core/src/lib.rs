#![doc = include_str!("../README.md")]
// #![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

use cosmrs::AccountId;
use k256::ecdsa::VerifyingKey;
use quartz_proto::quartz::{
    core_server::Core, InstantiateRequest, InstantiateResponse, SessionCreateRequest,
    SessionCreateResponse, SessionSetPubKeyRequest, SessionSetPubKeyResponse,
};
use serde::de::DeserializeOwned;
use tonic::{Request, Response, Status};

use crate::{
    attestor::{Attestor, DefaultAttestor},
    chain_client::{default::DefaultChainClient, ChainClient},
    handler::Handler,
    key_manager::{default::DefaultKeyManager, KeyManager},
    kv_store::{default::DefaultKvStore, ConfigKey, ContractKey, NonceKey, TypedStore},
};

pub mod attestor;
pub mod chain_client;
pub mod error;
pub mod handler;
pub mod key_manager;
pub mod kv_store;
pub mod server;
pub mod types;

#[async_trait::async_trait]
pub trait Enclave: Send + Sync + 'static {
    type Attestor: Attestor;
    type ChainClient: ChainClient<Contract = Self::Contract>;
    type Contract: DeserializeOwned + Clone + ToString + Send + Sync;
    type KeyManager: KeyManager;
    type Store: TypedStore<ContractKey<Self::Contract>>
        + TypedStore<NonceKey>
        + TypedStore<ConfigKey>;

    async fn attestor(&self) -> Self::Attestor;
    async fn chain_client(&self) -> Self::ChainClient;
    async fn key_manager(&self) -> Self::KeyManager;
    async fn store(&self) -> Self::Store;
}

#[derive(Clone, Debug)]
pub struct DefaultEnclave<
    A = DefaultAttestor,
    C = DefaultChainClient,
    K = DefaultKeyManager,
    S = DefaultKvStore,
> {
    pub attestor: A,
    pub chain_client: C,
    pub key_manager: K,
    pub store: S,
}

#[async_trait::async_trait]
impl<A, C, K, S> Enclave for DefaultEnclave<A, C, K, S>
where
    A: Attestor + Clone,
    C: ChainClient<Contract = AccountId> + Clone,
    K: KeyManager + Clone,
    S: TypedStore<ContractKey<AccountId>> + TypedStore<NonceKey> + TypedStore<ConfigKey> + Clone,
{
    type Attestor = A;
    type ChainClient = C;
    type Contract = AccountId;
    type KeyManager = K;
    type Store = S;

    async fn attestor(&self) -> Self::Attestor {
        self.attestor.clone()
    }

    async fn chain_client(&self) -> Self::ChainClient {
        self.chain_client.clone()
    }

    async fn key_manager(&self) -> Self::KeyManager {
        self.key_manager.clone()
    }

    async fn store(&self) -> Self::Store {
        self.store.clone()
    }
}

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
