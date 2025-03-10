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
    type PubKey = VerifyingKey;

    async fn pub_key(&self) -> Self::PubKey {
        self.sk.clone().into()
    }
}
