pub trait KeyManager {
    type PubKey;

    fn keygen(&mut self);
    fn pub_key(&self) -> Option<Self::PubKey>;
}
