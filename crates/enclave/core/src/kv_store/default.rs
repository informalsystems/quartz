use cosmrs::AccountId;
use displaydoc::Display;
use quartz_contract_core::state::{Config, Nonce};

use crate::kv_store::{ConfigKey, ContractKey, KvStore, NonceKey};

#[derive(Clone, Debug, Default)]
pub struct DefaultKvStore {
    config: Option<Config>,
    contract: Option<AccountId>,
    nonce: Option<Nonce>,
}

impl DefaultKvStore {
    pub fn new(config: Config) -> Self {
        DefaultKvStore {
            config: Some(config),
            contract: None,
            nonce: None,
        }
    }
}

#[derive(Debug, Display)]
pub enum StoreError {}

#[async_trait::async_trait]
impl KvStore<ContractKey<AccountId>, AccountId> for DefaultKvStore {
    type Error = StoreError;

    async fn set(
        &mut self,
        _key: ContractKey<AccountId>,
        value: AccountId,
    ) -> Result<Option<AccountId>, Self::Error> {
        Ok(self.contract.replace(value))
    }

    async fn get(&self, _key: ContractKey<AccountId>) -> Result<Option<AccountId>, Self::Error> {
        Ok(self.contract.clone().take())
    }

    async fn delete(&mut self, _key: ContractKey<AccountId>) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[async_trait::async_trait]
impl KvStore<NonceKey, Nonce> for DefaultKvStore {
    type Error = StoreError;

    async fn set(&mut self, _key: NonceKey, value: Nonce) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.replace(value))
    }

    async fn get(&self, _key: NonceKey) -> Result<Option<Nonce>, Self::Error> {
        Ok(self.nonce.clone().take())
    }

    async fn delete(&mut self, _key: NonceKey) -> Result<(), Self::Error> {
        unimplemented!()
    }
}

#[async_trait::async_trait]
impl KvStore<ConfigKey, Config> for DefaultKvStore {
    type Error = StoreError;

    async fn set(&mut self, _key: ConfigKey, value: Config) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.replace(value))
    }

    async fn get(&self, _key: ConfigKey) -> Result<Option<Config>, Self::Error> {
        Ok(self.config.clone().take())
    }

    async fn delete(&mut self, _key: ConfigKey) -> Result<(), Self::Error> {
        unimplemented!()
    }
}
