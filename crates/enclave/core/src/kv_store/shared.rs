use std::sync::Arc;

use tokio::sync::RwLock;

use crate::kv_store::KvStore;

#[derive(Clone, Debug)]
pub struct SharedKvStore<S> {
    inner: Arc<RwLock<S>>,
}

impl<S> SharedKvStore<S> {
    pub fn wrapping(store: S) -> Self {
        Self {
            inner: Arc::new(RwLock::new(store)),
        }
    }
}

#[async_trait::async_trait]
impl<S, K, V> KvStore<K, V> for SharedKvStore<S>
where
    S: KvStore<K, V>,
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    type Error = S::Error;

    async fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error> {
        self.inner.write().await.set(key, value).await
    }

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error> {
        self.inner.read().await.get(key).await
    }

    async fn delete(&mut self, key: K) -> Result<(), Self::Error> {
        self.inner.write().await.delete(key).await
    }
}
