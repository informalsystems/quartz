use der::DateTime;
use mc_attestation_verifier::{CertificateChainVerifier, CertificateChainVerifierError};
use x509_cert::{crl::CertificateList, Certificate};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct TlsCertificateChainVerifier;

impl CertificateChainVerifier for TlsCertificateChainVerifier {
    fn verify_certificate_chain<'a, 'b>(
        &self,
        _certificate_chain: impl IntoIterator<Item = &'a Certificate>,
        _crls: impl IntoIterator<Item = &'b CertificateList>,
        _time: impl Into<Option<DateTime>>,
    ) -> Result<(), CertificateChainVerifierError> {
        todo!()
    }
}
