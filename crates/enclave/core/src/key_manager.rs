#[async_trait::async_trait]
pub trait KeyManager: Send + Sync + 'static {
    type PubKey;

    async fn keygen(&mut self);
    async fn pub_key(&self) -> Option<Self::PubKey>;
}
