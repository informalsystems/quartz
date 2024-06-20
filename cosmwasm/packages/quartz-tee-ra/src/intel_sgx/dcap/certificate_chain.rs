use der::DateTime;
use mc_attestation_verifier::{CertificateChainVerifier, CertificateChainVerifierError};
use x509_cert::{crl::CertificateList, Certificate,der::Encode};
use x509_parser::prelude::X509Certificate;
use x509_parser::pem::{parse_x509_pem, Pem};
use x509_parser::{parse_x509_certificate, x509::X509Version};
use der::EncodeValue;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct TlsCertificateChainVerifier;

impl TlsCertificateChainVerifier {
    pub fn new(_root_ca: &str) -> Self {
        // FIXME(hu55a1n1)
        Self
    }
}

fn certificate_to_x509_certificate(cert: &Certificate) -> Result<X509Certificate<'_>, Box<dyn std::error::Error>> {
    let mut encoded_cert = Vec::new();
    cert.encode_value(&mut encoded_cert)?;
    let (_,x509_cert) = parse_x509_certificate(&encoded_cert.clone())?;
    Ok(x509_cert.clone())
}


impl CertificateChainVerifier for TlsCertificateChainVerifier {
    fn verify_certificate_chain<'a, 'b>(
        &self,
        _certificate_chain: impl IntoIterator<Item = &'a Certificate>,
        _crls: impl IntoIterator<Item = &'b CertificateList>,
        _time: impl Into<Option<DateTime>>,
    ) -> Result<(), CertificateChainVerifierError> {
	let _chain = _certificate_chain.into_iter().map(certificate_to_x509_certificate);
	//let unverified = UnverifiedCertChain::try_from_certificates(certificate_chain)?;
	/*
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
        let crls = CertificateRevocationList::try_from_crls(crls)?;*/
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
        let _chain = [LEAF_CERT, PROCESSOR_CA, ROOT_CA]
            .iter()
            .map(|cert| Certificate::from_pem(cert).expect("failed to parse cert"))
            .collect::<Vec<_>>();
        let _crls = [ROOT_CRL, PROCESSOR_CRL]
            .iter()
            .map(|crl| CertificateList::from_der(crl).expect("failed to parse CRL"))
            .collect::<Vec<_>>();
	/*
        let verifier = TlsCertificateChainVerifier::new(ROOT_CA);
        assert!(verifier
            .verify_certificate_chain(chain.iter(), crls.iter(), None)
        .is_ok());*/
    }
/*
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
*/
    // TODO(hu55a1n1) - add [PKITS tests](https://csrc.nist.gov/projects/pki-testing)
}
