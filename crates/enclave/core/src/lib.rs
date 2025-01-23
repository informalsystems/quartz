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

use core::fmt::Display;
use std::sync::Arc;

use cosmrs::AccountId;
use quartz_contract_core::state::{Config, Nonce};
use serde::de::DeserializeOwned;
use tokio::sync::Mutex;

use crate::{
    attestor::{Attestor, DefaultAttestor},
    key_manager::{default::DefaultKeyManager, shared::SharedKeyManager, KeyManager},
};

pub mod attestor;
pub mod chain_client;
pub mod event;
pub mod grpc;
pub mod handler;
pub mod host;
pub mod key_manager;
pub mod proof_of_publication;
pub mod types;

pub type DefaultSharedEnclave<C> =
    DefaultEnclave<C, DefaultAttestor, SharedKeyManager<DefaultKeyManager>>;

#[async_trait::async_trait]
pub trait Enclave: Send + Sync + 'static {
    type Attestor: Attestor;
    type Contract: DeserializeOwned + Clone + ToString + Send + Sync;
    type KeyManager: KeyManager;
    type Error: Display;

    async fn attestor(&self) -> Self::Attestor;
    async fn key_manager(&self) -> Self::KeyManager;
    async fn get_config(&self) -> Result<Option<Config>, Self::Error>;
    async fn get_contract(&self) -> Result<Option<Self::Contract>, Self::Error>;
    async fn get_nonce(&self) -> Result<Option<Nonce>, Self::Error>;
    async fn set_config(&self, config: Config) -> Result<Option<Config>, Self::Error>;
    async fn set_contract(
        &self,
        contract: Self::Contract,
    ) -> Result<Option<Self::Contract>, Self::Error>;
    async fn set_nonce(&self, nonce: Nonce) -> Result<Option<Nonce>, Self::Error>;
}

#[derive(Clone, Debug)]
pub struct DefaultEnclave<C, A = DefaultAttestor, K = DefaultKeyManager> {
    pub attestor: A,
    pub key_manager: K,
    pub ctx: C,
    pub config: Arc<Mutex<Option<Config>>>,
    pub contract: Arc<Mutex<Option<AccountId>>>,
    pub nonce: Arc<Mutex<Option<Nonce>>>,
}

impl<C> DefaultSharedEnclave<C> {
    pub fn shared(attestor: DefaultAttestor, config: Config, ctx: C) -> DefaultSharedEnclave<C> {
        DefaultSharedEnclave {
            attestor,
            key_manager: SharedKeyManager::wrapping(DefaultKeyManager::default()),
            ctx,
            config: Arc::new(Mutex::new(Some(config))),
            contract: Default::default(),
            nonce: Default::default(),
        }
    }
}

#[async_trait::async_trait]
impl<C, A, K> Enclave for DefaultEnclave<C, A, K>
where
    C: Send + Sync + 'static,
    A: Attestor + Clone,
    K: KeyManager + Clone,
{
    type Attestor = A;
    type Contract = AccountId;
    type KeyManager = K;
    type Error = String;

    async fn attestor(&self) -> Self::Attestor {
        self.attestor.clone()
    }

    async fn key_manager(&self) -> Self::KeyManager {
        self.key_manager.clone()
    }

    async fn get_config(&self) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.lock().await.clone())
    }

    async fn get_contract(&self) -> Result<Option<Self::Contract>, Self::Error> {
        Ok(self.contract.lock().await.clone())
    }

    async fn get_nonce(&self) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.lock().await.clone())
    }

    async fn set_config(&self, config: Config) -> Result<Option<Config>, Self::Error> {
        let mut stored = self.config.lock().await;
        let old = stored.clone();
        *stored = Some(config);
        Ok(old)
    }

    async fn set_contract(
        &self,
        contract: Self::Contract,
    ) -> Result<Option<Self::Contract>, Self::Error> {
        let mut stored = self.contract.lock().await;
        let old = stored.clone();
        *stored = Some(contract);
        Ok(old)
    }

    async fn set_nonce(&self, nonce: Nonce) -> Result<Option<Nonce>, Self::Error> {
        let mut stored = self.nonce.lock().await;
        let old = stored.clone();
        *stored = Some(nonce);
        Ok(old)
    }
}
