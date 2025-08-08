use std::sync::Arc;

use anyhow::anyhow;
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
    type Error = anyhow::Error;

    async fn import(mut self, data: Vec<u8>) -> Result<Self, Self::Error> {
        {
            // Get a write-guard, move the old value out, run its `import`,
            // then store the new value back and keep using the same wrapper.
            let mut guard = self.inner.write().await;

            let old_k: K = std::mem::take(&mut *guard);
            let new_k = old_k.import(data).await.map_err(|e| anyhow!("{:?}", e))?;

            *guard = new_k;
        }
        Ok(self)
    }
}

#[async_trait::async_trait]
impl<K: KeyManager + Export> Export for SharedKeyManager<K> {
    async fn export(&self) -> Vec<u8> {
        let guard = self.inner.read().await;
        guard.export().await
    }
}
