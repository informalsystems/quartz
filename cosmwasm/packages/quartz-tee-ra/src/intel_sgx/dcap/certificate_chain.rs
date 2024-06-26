use der::DateTime;
use mc_attestation_verifier::{CertificateChainVerifier, CertificateChainVerifierError};
use x509_cert::{crl::CertificateList, Certificate};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct TlsCertificateChainVerifier;

impl TlsCertificateChainVerifier {
    pub fn new(_root_ca: &str) -> Self {
        // FIXME(hu55a1n1)
        Self
    }
}

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

#[cfg(test)]
mod test {
    use der::{Decode, DecodePem};

    use super::*;

    const LEAF_CERT: &str = include_str!("../../../data/leaf_cert.pem");
    const PROCESSOR_CA: &str = include_str!("../../../data/processor_ca.pem");
    const ROOT_CA: &str = include_str!("../../../data/root_ca.pem");
    const PROCESSOR_CRL: &[u8] = include_bytes!("../../../data/processor_crl.der");
    const ROOT_CRL: &[u8] = include_bytes!("../../../data/root_crl.der");

    #[test]
    #[ignore]
    fn verify_valid_cert_chain() {
        let chain = [LEAF_CERT, PROCESSOR_CA, ROOT_CA]
            .iter()
            .map(|cert| Certificate::from_pem(cert).expect("failed to parse cert"))
            .collect::<Vec<_>>();
        let crls = [ROOT_CRL, PROCESSOR_CRL]
            .iter()
            .map(|crl| CertificateList::from_der(crl).expect("failed to parse CRL"))
            .collect::<Vec<_>>();
        let verifier = TlsCertificateChainVerifier::new(ROOT_CA);
        assert!(verifier
            .verify_certificate_chain(chain.iter(), crls.iter(), None)
            .is_ok());
    }

    #[test]
    #[ignore]
    fn invalid_cert_chain() {
        let chain = [LEAF_CERT, ROOT_CA]
            .iter()
            .map(|cert| Certificate::from_pem(cert).expect("failed to parse cert"))
            .collect::<Vec<_>>();
        let crls = [ROOT_CRL, PROCESSOR_CRL]
            .iter()
            .map(|crl| CertificateList::from_der(crl).expect("failed to parse CRL"))
            .collect::<Vec<_>>();
        let verifier = TlsCertificateChainVerifier::new(ROOT_CA);
        assert_eq!(
            verifier.verify_certificate_chain(chain.iter(), crls.iter(), None),
            Err(CertificateChainVerifierError::SignatureVerification)
        );
    }

    #[test]
    #[ignore]
    fn unordered_cert_chain_succeeds() {
        let chain = [PROCESSOR_CA, ROOT_CA, LEAF_CERT]
            .iter()
            .map(|cert| Certificate::from_pem(cert).expect("failed to parse cert"))
            .collect::<Vec<_>>();
        let crls = [ROOT_CRL, PROCESSOR_CRL]
            .iter()
            .map(|crl| CertificateList::from_der(crl).expect("failed to parse CRL"))
            .collect::<Vec<_>>();
        let verifier = TlsCertificateChainVerifier::new(ROOT_CA);
        assert!(verifier
            .verify_certificate_chain(chain.iter(), crls.iter(), None)
            .is_ok());
    }

    // TODO(hu55a1n1) - add [PKITS tests](https://csrc.nist.gov/projects/pki-testing)
}
