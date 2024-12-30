pub trait Signer {
    type PubKey;

    fn keygen(&mut self);
    fn pub_key(&self) -> Self::PubKey;
}
