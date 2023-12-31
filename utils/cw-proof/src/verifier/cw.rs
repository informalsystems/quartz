use alloc::borrow::Cow;
use alloc::vec::Vec;

use ics23::CommitmentProof;

use crate::error::ProofError;
use crate::verifier::{ics23::Ics23MembershipVerifier, multi::MultiVerifier, Verifier};

type Key = Vec<u8>;
type Value<'a> = Cow<'a, [u8]>;

#[derive(Clone, Debug)]
pub struct CwVerifier<'a>(MultiVerifier<Ics23MembershipVerifier<Key, Value<'a>>, 2>);

impl CwVerifier<'_> {
    pub fn verify(
        &self,
        proofs: &[CommitmentProof; 2],
        root: &Vec<u8>,
        keys: &[Vec<u8>; 2],
        value: &[u8],
    ) -> Result<(), ProofError> {
        if root.is_empty() {
            return Err(ProofError::EmptyMerkleRoot);
        }

        self.0
            .verify_against_root(proofs, keys, &Cow::Borrowed(value), root)?
            .then_some(())
            .ok_or(ProofError::VerificationFailure)
    }
}

impl Default for CwVerifier<'_> {
    fn default() -> Self {
        let mv = MultiVerifier::new([
            Ics23MembershipVerifier::new(ics23::iavl_spec()),
            Ics23MembershipVerifier::new(ics23::tendermint_spec()),
        ]);
        Self(mv)
    }
}
