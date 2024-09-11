use der::{DateTime, Encode};
use ecdsa::{signature::Verifier, Signature, VerifyingKey};
use mc_attestation_verifier::{CertificateChainVerifier, CertificateChainVerifierError};
use p256::NistP256;
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
        certificate_chain: impl IntoIterator<Item = &'a Certificate>,
        _crls: impl IntoIterator<Item = &'b CertificateList>,
        _time: impl Into<Option<DateTime>>,
    ) -> Result<(), CertificateChainVerifierError> {
        let cert_chain: Vec<_> = certificate_chain.into_iter().cloned().collect();

        let mut issuers: Vec<usize> = (1..cert_chain.len()).collect();
        issuers.push(cert_chain.len() - 1);
        let subjects: Vec<usize> = (0..cert_chain.len()).collect();

        for (i, s) in core::iter::zip(issuers, subjects) {
            let issuer_public_key = cert_chain[i]
                .tbs_certificate
                .subject_public_key_info
                .subject_public_key
                .as_bytes()
                .ok_or(CertificateChainVerifierError::GeneralCertificateError)?;

            let verifying_key = VerifyingKey::<NistP256>::from_sec1_bytes(issuer_public_key)
                .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;

            let tbs_certificate = cert_chain[s]
                .tbs_certificate
                .to_der()
                .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;

            let signature = Signature::<NistP256>::from_der(
                cert_chain[s]
                    .signature
                    .as_bytes()
                    .expect("Signature bytes should be present"),
            )
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;

            verifying_key
                .verify(&tbs_certificate, &signature)
                .map_err(|_| CertificateChainVerifierError::SignatureVerification)?;
        }

        Ok(())
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
