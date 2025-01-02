use std::collections::HashMap;

use displaydoc::Display;

use crate::kv_store::KvStore;

#[derive(Clone, Debug, Default)]
pub struct BincodeKvStore {
    map: HashMap<String, Vec<u8>>,
}

#[derive(Debug, Display)]
pub enum BincodeError {
    /// encode error: {0}
    Encode(bincode::error::EncodeError),
    /// decode error: {0}
    Decode(bincode::error::DecodeError),
}

#[async_trait::async_trait]
impl<K, V> KvStore<K, V> for BincodeKvStore
where
    K: ToString + Send + Sync + 'static,
    V: bincode::Encode + bincode::Decode + Send + Sync + 'static,
{
    type Error = BincodeError;

    async fn set(&mut self, key: K, value: V) -> Result<Option<V>, Self::Error> {
        let key = key.to_string();
        let value = bincode::encode_to_vec(&value, bincode::config::standard())
            .map_err(BincodeError::Encode)?;
        let prev_value = self
            .map
            .insert(key, value)
            .map(|v| bincode::decode_from_slice(&v, bincode::config::standard()))
            .transpose()
            .map_err(BincodeError::Decode)?
            .map(|(v, _)| v);
        Ok(prev_value)
    }

    async fn get(&self, key: K) -> Result<Option<V>, Self::Error> {
        let key = key.to_string();
        Ok(self
            .map
            .get(&key)
            .map(|v| bincode::decode_from_slice(v, bincode::config::standard()))
            .transpose()
            .map_err(BincodeError::Decode)?
            .map(|(v, _)| v))
    }

    async fn delete(&mut self, key: K) -> Result<(), Self::Error> {
        let key = key.to_string();
        let _ = self.map.remove(&key);
        Ok(())
    }
}
