use crate::proof::prefix::ConstPrefix;
use std::marker::PhantomData;

pub trait IntoKeys {
    fn into_keys(self) -> Vec<Vec<u8>>;
}

impl<K> IntoKeys for K
where
    K: Into<Vec<u8>>,
{
    fn into_keys(self) -> Vec<Vec<u8>> {
        vec![self.into()]
    }
}

pub struct PrefixedKey<P, K> {
    key: K,
    prefix: PhantomData<P>,
}

impl<P, K> PrefixedKey<P, K> {
    pub fn new(key: K) -> Self {
        Self {
            key,
            prefix: PhantomData,
        }
    }
}

impl<P, K> IntoKeys for PrefixedKey<P, K>
where
    K: Into<Vec<u8>>,
    P: ConstPrefix,
{
    fn into_keys(self) -> Vec<Vec<u8>> {
        vec![P::PREFIX.to_string().into_bytes(), self.key.into()]
    }
}
