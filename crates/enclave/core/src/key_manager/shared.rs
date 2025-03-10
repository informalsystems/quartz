use std::sync::Arc;

use tokio::sync::{RwLock, RwLockReadGuard};

use crate::key_manager::KeyManager;

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
