use alloc::vec::Vec;

use ics23::CommitmentProof;
use tendermint::merkle::proof::ProofOps;

use crate::error::ProofError;

pub mod cw;
pub mod key;
pub mod prefix;

// Copied from hermes
pub fn convert_tm_to_ics_merkle_proof(
    tm_proof: &ProofOps,
) -> Result<Vec<CommitmentProof>, ProofError> {
    let mut proofs = Vec::new();

    for op in &tm_proof.ops {
        let mut parsed = CommitmentProof { proof: None };

        prost::Message::merge(&mut parsed, op.data.as_slice())
            .map_err(|_| ProofError::CommitmentProofDecodingFailed)?;

        proofs.push(parsed);
    }

    Ok(proofs)
}

pub trait Proof {
    type Key;
    type Value;
    type ProofOps;

    fn verify(self, root: Vec<u8>) -> Result<(), ProofError>;
}
