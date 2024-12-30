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

use serde::de::DeserializeOwned;

use crate::{
    attestor::Attestor,
    chain_client::ChainClient,
    key_manager::KeyManager,
    kv_store::{ConfigKey, ContractKey, NonceKey, TypedStore},
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
