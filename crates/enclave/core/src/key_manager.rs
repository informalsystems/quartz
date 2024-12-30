pub trait KeyManager: Send + Sync {
    type PubKey;

    fn keygen(&mut self);
    fn pub_key(&self) -> Option<Self::PubKey>;
}
