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
use quartz_contract_core::state::Config;
use serde::de::DeserializeOwned;

use crate::{
    attestor::{Attestor, DefaultAttestor},
    key_manager::{default::DefaultKeyManager, shared::SharedKeyManager, KeyManager},
    kv_store::{
        default::DefaultKvStore, shared::SharedKvStore, ConfigKey, ContractKey, NonceKey,
        TypedStore,
    },
};

pub mod attestor;
pub mod chain_client;
pub mod error;
pub mod event;
pub mod grpc;
pub mod handler;
pub mod host;
pub mod key_manager;
pub mod kv_store;
pub mod server;
pub mod types;

pub type DefaultSharedEnclave<C> = DefaultEnclave<
    C,
    DefaultAttestor,
    SharedKeyManager<DefaultKeyManager>,
    SharedKvStore<DefaultKvStore>,
>;

#[async_trait::async_trait]
pub trait Enclave: Send + Sync + 'static {
    type Attestor: Attestor;
    type Contract: DeserializeOwned + Clone + ToString + Send + Sync;
    type KeyManager: KeyManager;
    type Store: TypedStore<ContractKey<Self::Contract>>
        + TypedStore<NonceKey>
        + TypedStore<ConfigKey>;

    async fn attestor(&self) -> Self::Attestor;
    async fn key_manager(&self) -> Self::KeyManager;
    async fn store(&self) -> Self::Store;
}

#[derive(Clone, Debug)]
pub struct DefaultEnclave<C, A = DefaultAttestor, K = DefaultKeyManager, S = DefaultKvStore> {
    pub attestor: A,
    pub key_manager: K,
    pub store: S,
    pub ctx: C,
}

impl<C> DefaultSharedEnclave<C> {
    pub fn shared(attestor: DefaultAttestor, config: Config, ctx: C) -> DefaultSharedEnclave<C> {
        DefaultSharedEnclave {
            attestor,
            key_manager: SharedKeyManager::wrapping(DefaultKeyManager::default()),
            store: SharedKvStore::wrapping(DefaultKvStore::new(config)),
            ctx,
        }
    }
}

#[async_trait::async_trait]
impl<C, A, K, S> Enclave for DefaultEnclave<C, A, K, S>
where
    C: Send + Sync + 'static,
    A: Attestor + Clone,
    K: KeyManager + Clone,
    S: TypedStore<ContractKey<AccountId>> + TypedStore<NonceKey> + TypedStore<ConfigKey> + Clone,
{
    type Attestor = A;
    type Contract = AccountId;
    type KeyManager = K;
    type Store = S;

    async fn attestor(&self) -> Self::Attestor {
        self.attestor.clone()
    }

    async fn key_manager(&self) -> Self::KeyManager {
        self.key_manager.clone()
    }

    async fn store(&self) -> Self::Store {
        self.store.clone()
    }
}
