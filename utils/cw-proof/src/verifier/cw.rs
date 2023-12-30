use alloc::vec::Vec;

use ics23::CommitmentProof;

use crate::error::ProofError;
use crate::verifier::{ics23::Ics23MembershipVerifier, multi::MultiVerifier, Verifier};

#[derive(Clone, Debug)]
pub struct CwVerifier(MultiVerifier<Ics23MembershipVerifier<Vec<u8>, Vec<u8>>, 2>);

impl CwVerifier {
    pub fn new() -> Self {
        let mv = MultiVerifier::new([
            Ics23MembershipVerifier::new(ics23::iavl_spec()),
            Ics23MembershipVerifier::new(ics23::tendermint_spec()),
        ]);
        Self(mv)
    }

    pub fn verify(
        &self,
        proofs: &[CommitmentProof; 2],
        root: &Vec<u8>,
        keys: &[Vec<u8>; 2],
        value: &Vec<u8>,
    ) -> Result<(), ProofError> {
        if root.is_empty() {
            return Err(ProofError::EmptyMerkleRoot);
        }

        let verified = self.0.verify_against_root(proofs, keys, value, root)?;
        if !verified {
            return Err(ProofError::VerificationFailure);
        }

        Ok(())
    }
}
