// Copyright (c) 2023 The MobileCoin Foundation

//! Verify the contents of a Quote3.

use der::DateTime;
use mc_attestation_verifier::{
    Accessor, And, AndOutput, Evidence, EvidenceValue, EvidenceVerifier, ReportDataVerifier,
    TrustedIdentity, VerificationOutput, Verifier,
};
use mc_sgx_core_types::ReportData;

// use super::DCAP_ROOT_ANCHOR;
use super::super::certificate_chain::TlsCertificateChainVerifier;
use super::super::mc_attest_verifier_types::verification::EnclaveReportDataContents;

#[derive(Debug)]
pub struct DcapVerifier {
    verifier: And<EvidenceVerifier<TlsCertificateChainVerifier>, ReportDataHashVerifier>,
}

pub type DcapVerifierOutput = AndOutput<EvidenceValue, ReportData>;

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
    pub fn new<I, ID>(
        trusted_identities: I,
        time: impl Into<Option<DateTime>>,
        report_data: EnclaveReportDataContents,
    ) -> Self
    where
        I: IntoIterator<Item = ID>,
        ID: Into<TrustedIdentity>,
    {
        let certificate_verifier = TlsCertificateChainVerifier;
        let verifier = And::new(
            EvidenceVerifier::new(certificate_verifier, trusted_identities, time),
            ReportDataHashVerifier::new(report_data),
        );
        Self { verifier }
    }

    /// Verify the `evidence`
    pub fn verify(&self, evidence: &Evidence<Vec<u8>>) -> VerificationOutput<DcapVerifierOutput> {
        self.verifier.verify(evidence)
    }
}

#[derive(Debug, Clone)]
pub struct ReportDataHashVerifier {
    report_data_verifier: ReportDataVerifier,
}

impl ReportDataHashVerifier {
    pub fn new(report_data: EnclaveReportDataContents) -> Self {
        let mut expected_report_data_bytes = [0u8; 64];
        expected_report_data_bytes[..32].copy_from_slice(report_data.sha256().as_ref());
        let mut mask = [0u8; 64];
        mask[..32].copy_from_slice([0xffu8; 32].as_ref());
        let report_data_verifier =
            ReportDataVerifier::new(expected_report_data_bytes.into(), mask.into());

        Self {
            report_data_verifier,
        }
    }
}

impl<E: Accessor<ReportData>> Verifier<E> for ReportDataHashVerifier {
    type Value = ReportData;

    fn verify(&self, evidence: &E) -> VerificationOutput<Self::Value> {
        self.report_data_verifier.verify(evidence)
    }
}
