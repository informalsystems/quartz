use std::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};

use quartz_contract_core::state::{Config, Nonce};

#[async_trait::async_trait]
pub trait KvStore<K, V>: Send + Sync + 'static {
    type Error: ToString;

    async fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error>;

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error>;

    async fn delete(&mut self, key: K) -> Result<(), Self::Error>;
}

pub trait TypedStore<K: ValueForKey>: KvStore<K, <K as ValueForKey>::Value> {}

impl<S, K, V> TypedStore<K> for S
where
    S: KvStore<K, V>,
    K: ValueForKey<Value = V>,
{
}

pub trait ValueForKey {
    type Value;
}

#[derive(Clone, Debug)]
pub struct TypedKey<K, V> {
    key: K,
    _phantom: PhantomData<V>,
}

impl<K, V> Display for TypedKey<K, V>
where
    K: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.key.fmt(f)
    }
}

impl<K, V> TypedKey<K, V> {
    pub fn new(key: K) -> Self {
        Self {
            key,
            _phantom: PhantomData,
        }
    }
}

impl<K, V> ValueForKey for TypedKey<K, V> {
    type Value = V;
}

pub type ContractKey<C> = TypedKey<ContractKeyName, C>;
pub type NonceKey = TypedKey<NonceKeyName, Nonce>;
pub type ConfigKey = TypedKey<ConfigKeyName, Config>;

#[derive(Clone, Debug)]
pub struct ContractKeyName;

#[derive(Clone, Debug)]
pub struct NonceKeyName;

#[derive(Clone, Debug)]
pub struct ConfigKeyName;
