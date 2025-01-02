use std::marker::PhantomData;

use tokio::sync::{mpsc, oneshot};

use crate::kv_store::KvStore;

#[derive(Clone, Debug)]
pub enum KvStoreAction<K, V> {
    Set(K, V),
    Get(K),
    Delete(K),
}

#[derive(Clone, Debug)]
pub enum KvStoreActionResult<V> {
    Set(Option<V>),
    Get(Option<V>),
    Delete,
    Err,
}

#[derive(Debug)]
#[allow(unused)]
pub struct KvStoreRequest<K, V> {
    action: KvStoreAction<K, V>,
    resp_tx: oneshot::Sender<KvStoreActionResult<V>>,
}

#[derive(Clone, Debug)]
#[allow(unused)]
pub struct MpscKvStore<S, K, V> {
    req_tx: mpsc::Sender<KvStoreRequest<K, V>>,
    _phantom: PhantomData<S>,
}

#[async_trait::async_trait]
impl<S, K, V> KvStore<K, V> for MpscKvStore<S, K, V>
where
    S: KvStore<K, V>,
    K: Send + Sync + 'static,
    V: Send + Sync + 'static,
{
    type Error = S::Error;

    async fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.req_tx
            .send(KvStoreRequest {
                action: KvStoreAction::Set(key, value),
                resp_tx,
            })
            .await
            .expect("KvStoreRequest channel closed");

        if let KvStoreActionResult::Set(prev_v) = resp_rx.await.expect("resp_rx") {
            Ok(prev_v)
        } else {
            unreachable!()
        }
    }

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.req_tx
            .send(KvStoreRequest {
                action: KvStoreAction::Get(key),
                resp_tx,
            })
            .await
            .expect("KvStoreRequest channel closed");

        if let KvStoreActionResult::Get(prev_v) = resp_rx.await.expect("resp_rx") {
            Ok(prev_v)
        } else {
            unreachable!()
        }
    }

    async fn delete(&mut self, key: K) -> Result<(), Self::Error> {
        let (resp_tx, resp_rx) = oneshot::channel();
        self.req_tx
            .send(KvStoreRequest {
                action: KvStoreAction::Delete(key),
                resp_tx,
            })
            .await
            .expect("KvStoreRequest channel closed");

        if let KvStoreActionResult::Delete = resp_rx.await.expect("resp_rx") {
            Ok(())
        } else {
            unreachable!()
        }
    }
}
