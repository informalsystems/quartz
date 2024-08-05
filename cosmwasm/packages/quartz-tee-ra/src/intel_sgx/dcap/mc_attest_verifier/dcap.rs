// Copyright (c) 2023 The MobileCoin Foundation

//! Verify the contents of a Quote3.

use der::DateTime;
use mc_attestation_verifier::{
    Evidence, EvidenceValue, EvidenceVerifier, TrustedIdentity, VerificationOutput, Verifier,
};

use super::super::certificate_chain::TlsCertificateChainVerifier;

#[derive(Debug)]
pub struct DcapVerifier {
    verifier: EvidenceVerifier<TlsCertificateChainVerifier>,
}

pub type DcapVerifierOutput = EvidenceValue;

impl DcapVerifier {
    /// Create a new instance of the DcapVerifier.
    ///
    /// # Arguments
    /// * `trusted_identities` - The allowed identities that can be used in an
    ///   enclave. Verification will succeed if any of these match.
    /// * `time` - The time to use to verify the validity of the certificates
    ///   and collateral. If time is provided, verification will fail if this
    ///   time is before or after any of the validity periods. Otherwise, time
    ///   validation of certificates will be skipped.
    pub fn new<I, ID>(trusted_identities: I, time: impl Into<Option<DateTime>>) -> Self
    where
        I: IntoIterator<Item = ID>,
        ID: Into<TrustedIdentity>,
    {
        let certificate_verifier = TlsCertificateChainVerifier;
        let verifier = EvidenceVerifier::new(certificate_verifier, trusted_identities, time);
        Self { verifier }
    }

    /// Verify the `evidence`
    pub fn verify(&self, evidence: &Evidence<Vec<u8>>) -> VerificationOutput<DcapVerifierOutput> {
        self.verifier.verify(evidence)
    }
}
