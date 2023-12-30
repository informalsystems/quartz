use alloc::borrow::ToOwned;
use core::borrow::Borrow;

use crate::verifier::Verifier;

#[derive(Clone, Debug)]
pub struct MultiVerifier<V, const N: usize> {
    verifiers: [V; N],
}

impl<V, const N: usize> MultiVerifier<V, N> {
    pub fn new(verifiers: [V; N]) -> Self {
        assert!(N > 0, "cannot create empty multi-verifiers");
        Self { verifiers }
    }
}

impl<V, const N: usize> Verifier for MultiVerifier<V, N>
where
    V: Verifier,
    V::Root: Into<V::Value> + Clone,
{
    type Proof = [V::Proof; N];
    type Root = V::Root;
    type Key = [V::Key; N];
    type Value = V::Value;
    type ValueRef = V::ValueRef;
    type Error = V::Error;

    fn verify(
        &self,
        proofs: &Self::Proof,
        keys: &Self::Key,
        value: &Self::ValueRef,
    ) -> Result<Self::Root, Self::Error> {
        let mut root = None;
        let mut value: V::Value = value.to_owned();

        for (idx, verifier) in self.verifiers.iter().enumerate() {
            let proof = &proofs[idx];
            let key = &keys[N - idx - 1];
            let sub_root = verifier.verify(proof, key, value.borrow())?;

            value = sub_root.clone().into();
            root = Some(sub_root);
        }

        Ok(root.expect("MultiVerifier cannot be instantiated with 0 verifiers"))
    }
}
