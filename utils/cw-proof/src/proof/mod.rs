use alloc::vec::Vec;

use ibc_relayer_types::core::ics23_commitment::error::Error as ProofError;
use ics23::CommitmentProof;
use tendermint::merkle::proof::ProofOps;

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
            .map_err(ProofError::commitment_proof_decoding_failed)?;

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
