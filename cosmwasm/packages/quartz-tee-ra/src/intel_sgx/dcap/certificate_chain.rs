use der::{pem::LineEnding, DateTime, EncodePem};
use mc_attestation_verifier::{CertificateChainVerifier, CertificateChainVerifierError};
use x509_cert::{crl::CertificateList, Certificate};
use x509_parser::{parse_x509_certificate, pem::parse_x509_pem};

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
        let enc_certs = certificate_chain
            .into_iter()
            .map(|cert| cert.to_pem(LineEnding::LF))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;

        let pem_chain = enc_certs
            .iter()
            .map(|enc_cert| parse_x509_pem(enc_cert.as_ref()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
        let cert_chain = pem_chain
            .iter()
            .map(|pem| parse_x509_certificate(&pem.1.contents))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
        // Skip applying the Certificate Revocation List entirely
        /*
           let enc_crls = crls
               .into_iter()
               .map(|crl| der_encode(crl))
               .collect::<Result<Vec<_>, _>>()
               .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
           let _crls = enc_crls
               .iter()
               .map(|enc_crl| parse_x509_crl(enc_crl.as_ref()))
               .collect::<Result<Vec<_>, _>>();
           .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
        */

        let v: Vec<_> = cert_chain.to_vec();
        let mut issuers: Vec<usize> = (1..v.len()).collect();
        issuers.push(v.len() - 1);
        let subjects: Vec<usize> = (0..v.len()).collect();
        for (i, s) in core::iter::zip(issuers, subjects) {
            let r = v[s].1.verify_signature(Some(v[i].1.public_key()));
            r.map_err(|_| CertificateChainVerifierError::SignatureVerification)?
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
