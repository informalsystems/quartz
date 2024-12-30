use std::{
    fmt::{Display, Formatter},
    marker::PhantomData,
};

use quartz_contract_core::state::Nonce;

pub trait KvStore<K, V> {
    type Error: ToString;

    fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error>;

    fn get(&self, key: K) -> Result<Option<V>, Self::Error>;

    fn delete(&mut self, key: K) -> Result<(), Self::Error>;
}

pub trait TypedStore<K: ValueForKey>: KvStore<K, <K as ValueForKey>::Value> {}

pub trait ValueForKey {
    type Value;
}

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

pub struct ContractKeyName;
pub struct NonceKeyName;
