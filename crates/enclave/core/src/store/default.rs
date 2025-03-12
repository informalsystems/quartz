use std::sync::Arc;

use cosmrs::AccountId;
use displaydoc::Display;
use quartz_contract_core::state::{Config, Nonce};
use tokio::sync::RwLock;

use crate::store::Store;

#[derive(Clone, Debug, Default)]
pub struct DefaultStore {
    config: Arc<RwLock<Option<Config>>>,
    contract: Arc<RwLock<Option<AccountId>>>,
    nonce: Arc<RwLock<Option<Nonce>>>,
    seq_num: Arc<RwLock<u64>>,
}

impl DefaultStore {
    pub fn new(config: Config) -> Self {
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
        Ok(self.config.read().await.clone())
    }

    async fn set_config(&self, config: Config) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.write().await.replace(config))
    }

    async fn get_contract(&self) -> Result<Option<Self::Contract>, Self::Error> {
        Ok(self.contract.read().await.clone())
    }

    async fn set_contract(
        &self,
        contract: Self::Contract,
    ) -> Result<Option<Self::Contract>, Self::Error> {
        Ok(self.contract.write().await.replace(contract))
    }

    async fn get_nonce(&self) -> Result<Option<Nonce>, Self::Error> {
        Ok(*self.nonce.read().await)
    }

    async fn set_nonce(&self, nonce: Nonce) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.write().await.replace(nonce))
    }

    async fn get_seq_num(&self) -> Result<u64, Self::Error> {
        Ok(*self.seq_num.read().await)
    }

    async fn inc_seq_num(&self, count: usize) -> Result<u64, Self::Error> {
        let mut seq_num = self.seq_num.write().await;
        let prev_seq_num = *seq_num;
        *seq_num += count as u64;
        Ok(prev_seq_num)
    }
}
