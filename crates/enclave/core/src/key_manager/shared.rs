use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard};

use crate::{
    backup_restore::{Export, Import},
    key_manager::KeyManager,
};

/// A thread-safe wrapper for a key-manager.
#[derive(Clone, Debug)]
pub struct SharedKeyManager<K> {
    pub inner: Arc<RwLock<K>>,
}

impl<K> SharedKeyManager<K> {
    pub fn wrapping(key_manager: K) -> Self {
        Self {
            inner: Arc::new(RwLock::new(key_manager)),
        }
    }

    pub async fn read_lock(&self) -> RwLockReadGuard<'_, K> {
        self.inner.read().await
    }
}

#[async_trait::async_trait]
impl<K: KeyManager> KeyManager for SharedKeyManager<K> {
    type PubKey = K::PubKey;

    async fn pub_key(&self) -> Self::PubKey {
        self.inner.read().await.pub_key().await
    }
}

#[async_trait::async_trait]
impl<K: KeyManager + Import + Default> Import for SharedKeyManager<K> {
    type Error = K::Error;

    async fn import(data: Vec<u8>) -> Result<Self, Self::Error> {
        let km = K::import(data).await?;
        Ok(Self::wrapping(km))
    }
}

#[async_trait::async_trait]
impl<K: KeyManager + Export> Export for SharedKeyManager<K> {
    type Error = K::Error;

    async fn export(&self) -> Result<Vec<u8>, Self::Error> {
        let guard = self.inner.read().await;
        guard.export().await
    }
}
