use std::sync::Arc;

use cosmrs::AccountId;
use displaydoc::Display;
use log::{debug, info, trace};
use quartz_contract_core::state::{Config, Nonce};
use serde::{Deserialize, Serialize};
use serde_json::Error;
use tokio::sync::RwLock;

use crate::{
    backup_restore::{Export, Import},
    store::Store,
};

/// A default, thread-safe in-memory store.
#[derive(Clone, Debug, Default)]
pub struct DefaultStore {
    config: Arc<RwLock<Option<Config>>>,
    contract: Arc<RwLock<Option<AccountId>>>,
    nonce: Arc<RwLock<Option<Nonce>>>,
    seq_num: Arc<RwLock<u64>>,
}

impl DefaultStore {
    pub fn new(config: Config) -> Self {
        info!("Creating new default store with config: {config:?}");
        DefaultStore {
            config: Arc::new(RwLock::new(Some(config))),
            contract: Default::default(),
            nonce: Default::default(),
            seq_num: Default::default(),
        }
    }
}

#[derive(Debug, Display)]
pub enum StoreError {}

#[async_trait::async_trait]
impl Store for DefaultStore {
    type Contract = AccountId;
    type Error = StoreError;

    async fn get_config(&self) -> Result<Option<Config>, Self::Error> {
        debug!("Retrieving enclave configuration");
        Ok(self.config.read().await.clone())
    }

    async fn set_config(&self, config: Config) -> Result<Option<Config>, Self::Error> {
        debug!("Setting new enclave configuration");
        Ok(self.config.write().await.replace(config))
    }

    async fn get_contract(&self) -> Result<Option<Self::Contract>, Self::Error> {
        debug!("Retrieving enclave contract");
        Ok(self.contract.read().await.clone())
    }

    async fn set_contract(
        &self,
        contract: Self::Contract,
    ) -> Result<Option<Self::Contract>, Self::Error> {
        debug!("Setting new enclave contract: {contract}");
        Ok(self.contract.write().await.replace(contract))
    }

    async fn get_nonce(&self) -> Result<Option<Nonce>, Self::Error> {
        debug!("Retrieving enclave nonce");
        Ok(*self.nonce.read().await)
    }

    async fn set_nonce(&self, nonce: Nonce) -> Result<Option<Nonce>, Self::Error> {
        debug!("Setting new enclave nonce: {nonce:?}");
        Ok(self.nonce.write().await.replace(nonce))
    }

    async fn get_seq_num(&self) -> Result<u64, Self::Error> {
        debug!("Retrieving sequence number");
        Ok(*self.seq_num.read().await)
    }

    async fn inc_seq_num(&self, count: usize) -> Result<u64, Self::Error> {
        debug!("Incrementing sequence number by {count}");
        let mut seq_num = self.seq_num.write().await;
        let prev_seq_num = *seq_num;
        *seq_num += count as u64;
        trace!(
            "Sequence number incremented from {} to {}",
            prev_seq_num,
            *seq_num
        );
        Ok(prev_seq_num)
    }
}

#[derive(Serialize, Deserialize)]
struct StoreDTO {
    config: Option<Config>,
    contract: Option<AccountId>,
    nonce: Option<Nonce>,
    seq_num: u64,
}
#[async_trait::async_trait]
impl Import for DefaultStore {
    type Error = Error;

    async fn import(data: Vec<u8>) -> Result<Self, Self::Error> {
        let dto: StoreDTO = serde_json::from_slice(&data)?;

        Ok(Self {
            config: Arc::new(RwLock::new(dto.config)),
            contract: Arc::new(RwLock::new(dto.contract)),
            nonce: Arc::new(RwLock::new(dto.nonce)),
            seq_num: Arc::new(RwLock::new(dto.seq_num)),
        })
    }
}

#[async_trait::async_trait]
impl Export for DefaultStore {
    type Error = Error;

    async fn export(&self) -> Result<Vec<u8>, Self::Error> {
        let dto = StoreDTO {
            config: self.config.read().await.clone(),
            contract: self.contract.read().await.clone(),
            nonce: *self.nonce.read().await,
            seq_num: *self.seq_num.read().await,
        };

        Ok(serde_json::to_vec(&dto)?)
    }
}
