use serde::Serialize;

pub mod default;
pub mod shared;

#[async_trait::async_trait]
pub trait KeyManager: Send + Sync + 'static {
    type PubKey: Serialize;

    async fn pub_key(&self) -> Self::PubKey;
}
