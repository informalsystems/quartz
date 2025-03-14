use k256::ecdsa::{SigningKey, VerifyingKey};

use crate::key_manager::KeyManager;

#[derive(Clone)]
pub struct DefaultKeyManager {
    pub sk: SigningKey,
}

impl Default for DefaultKeyManager {
    fn default() -> Self {
        Self {
            sk: SigningKey::random(&mut rand::thread_rng()),
        }
    }
}

#[async_trait::async_trait]
impl KeyManager for DefaultKeyManager {
    type PubKey = PubKey;

    async fn pub_key(&self) -> Self::PubKey {
        PubKey(self.sk.clone().into())
    }
}

#[derive(Clone, Debug)]
pub struct PubKey(VerifyingKey);

impl From<PubKey> for Vec<u8> {
    fn from(value: PubKey) -> Self {
        value.0.to_sec1_bytes().into()
    }
}

impl From<PubKey> for VerifyingKey {
    fn from(value: PubKey) -> Self {
        value.0
    }
}
