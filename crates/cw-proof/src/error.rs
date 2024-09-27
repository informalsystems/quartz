use displaydoc::Display;

#[derive(Clone, Debug, Display)]
pub enum ProofError {
    /// failed to decode commitment proof
    CommitmentProofDecodingFailed,
    /// empty merkle root
    EmptyMerkleRoot,
    /// empty verified value
    EmptyVerifiedValue,
    /// invalid merkle proof
    InvalidMerkleProof,
    /// proof verification failed
    VerificationFailure,
}
