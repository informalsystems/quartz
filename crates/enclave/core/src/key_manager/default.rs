use k256::ecdsa::{Error, SigningKey, VerifyingKey};
use log::{debug, info};

use crate::{
    backup_restore::{Export, Import},
    key_manager::KeyManager,
};

/// A default secp256k1 key-manager.
#[derive(Clone)]
pub struct DefaultKeyManager {
    pub sk: SigningKey,
}

impl Default for DefaultKeyManager {
    fn default() -> Self {
        info!("Creating new default key manager with random signing key");
        Self {
            sk: SigningKey::random(&mut rand::thread_rng()),
        }
    }
}

#[async_trait::async_trait]
impl KeyManager for DefaultKeyManager {
    type PubKey = PubKey;

    async fn pub_key(&self) -> Self::PubKey {
        debug!("Retrieving public key from key manager");
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

#[async_trait::async_trait]
impl Import for DefaultKeyManager {
    type Error = Error;

    async fn import(&mut self, data: Vec<u8>) -> Result<(), Self::Error> {
        self.sk = SigningKey::from_slice(&data)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Export for DefaultKeyManager {
    type Error = Error;

    async fn export(&self) -> Result<Vec<u8>, Self::Error> {
        Ok(self.sk.to_bytes().to_vec())
    }
}
