use ibc_proto::ibc::core::commitment::v1::{MerkleProof as RawMerkleProof, MerkleRoot};
use ibc_relayer_types::{
    core::ics23_commitment::error::Error as ProofError, core::ics23_commitment::specs::ProofSpecs,
};
use ics23::{
    calculate_existence_root, commitment_proof::Proof, verify_membership, CommitmentProof,
    ProofSpec,
};
use tendermint::merkle::proof::ProofOps as TendermintProof;

// Copied from hermes and patched to allow non-string keys
#[derive(Clone, Debug, PartialEq)]
pub struct MerkleProof {
    proofs: Vec<CommitmentProof>,
}

/// Convert to ics23::CommitmentProof
impl From<RawMerkleProof> for MerkleProof {
    fn from(proof: RawMerkleProof) -> Self {
        Self {
            proofs: proof.proofs,
        }
    }
}

impl From<MerkleProof> for RawMerkleProof {
    fn from(proof: MerkleProof) -> Self {
        Self {
            proofs: proof.proofs,
        }
    }
}

impl MerkleProof {
    pub fn verify_membership(
        &self,
        specs: &ProofSpecs,
        root: MerkleRoot,
        keys: Vec<Vec<u8>>,
        value: Vec<u8>,
        start_index: usize,
    ) -> Result<(), ProofError> {
        // validate arguments
        if self.proofs.is_empty() {
            return Err(ProofError::empty_merkle_proof());
        }
        if root.hash.is_empty() {
            return Err(ProofError::empty_merkle_root());
        }
        let num = self.proofs.len();
        let ics23_specs = Vec::<ProofSpec>::from(specs.clone());
        if ics23_specs.len() != num {
            return Err(ProofError::number_of_specs_mismatch());
        }
        if keys.len() != num {
            return Err(ProofError::number_of_keys_mismatch());
        }
        if value.is_empty() {
            return Err(ProofError::empty_verified_value());
        }

        let mut subroot = value.clone();
        let mut value = value;

        // keys are represented from root-to-leaf
        for ((proof, spec), key) in self
            .proofs
            .iter()
            .zip(ics23_specs.iter())
            .zip(keys.iter().rev())
            .skip(start_index)
        {
            match &proof.proof {
                Some(Proof::Exist(existence_proof)) => {
                    subroot =
                        calculate_existence_root::<ics23::HostFunctionsManager>(existence_proof)
                            .map_err(|_| ProofError::invalid_merkle_proof())?;

                    if !verify_membership::<ics23::HostFunctionsManager>(
                        proof, spec, &subroot, key, &value,
                    ) {
                        return Err(ProofError::verification_failure());
                    }
                    value = subroot.clone();
                }
                _ => return Err(ProofError::invalid_merkle_proof()),
            }
        }

        if root.hash != subroot {
            return Err(ProofError::verification_failure());
        }

        Ok(())
    }
}

pub fn convert_tm_to_ics_merkle_proof(
    tm_proof: &TendermintProof,
) -> Result<MerkleProof, ProofError> {
    let mut proofs = Vec::new();

    for op in &tm_proof.ops {
        let mut parsed = CommitmentProof { proof: None };

        prost::Message::merge(&mut parsed, op.data.as_slice())
            .map_err(ProofError::commitment_proof_decoding_failed)?;

        proofs.push(parsed);
    }

    Ok(MerkleProof::from(RawMerkleProof { proofs }))
}
