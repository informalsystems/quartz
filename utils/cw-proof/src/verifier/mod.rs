use alloc::borrow::ToOwned;
use core::borrow::Borrow;

pub mod cw;
pub mod ics23;
pub mod multi;

trait Verifier {
    type Proof;
    type Root: Eq;
    type Key;
    type Value: Borrow<Self::ValueRef>;
    type ValueRef: ?Sized + ToOwned<Owned = Self::Value>;
    type Error;

    fn verify(
        &self,
        proof: &Self::Proof,
        key: &Self::Key,
        value: &Self::ValueRef,
    ) -> Result<Self::Root, Self::Error>;

    fn verify_against_root(
        &self,
        proof: &Self::Proof,
        key: &Self::Key,
        value: &Self::ValueRef,
        root: &Self::Root,
    ) -> Result<bool, Self::Error> {
        let found_root = self.verify(proof, key, value)?;
        Ok(root == &found_root)
    }
}
