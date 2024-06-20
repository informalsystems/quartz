use der::{pem::LineEnding,DateTime, EncodePem, EncodeValue, Error as DerError};
use mc_attestation_verifier::{CertificateChainVerifier, CertificateChainVerifierError};
use x509_cert::{crl::CertificateList, Certificate};
use x509_parser::{parse_x509_certificate, parse_x509_crl,pem::parse_x509_pem};



#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Eq, PartialEq)]
pub struct TlsCertificateChainVerifier;

impl TlsCertificateChainVerifier {
    pub fn new(_root_ca: &str) -> Self {
        // FIXME(hu55a1n1)
        Self
    }
}

fn der_encode(cert: &impl EncodeValue) -> Result<Vec<u8>, DerError> {
    let mut encoded_cert = Vec::new();
    cert.encode_value(&mut encoded_cert)?;
    Ok(encoded_cert)
}

impl CertificateChainVerifier for TlsCertificateChainVerifier {
    fn verify_certificate_chain<'a, 'b>(
        &self,
        certificate_chain: impl IntoIterator<Item = &'a Certificate>,
        crls: impl IntoIterator<Item = &'b CertificateList>,
        _time: impl Into<Option<DateTime>>,
    ) -> Result<(), CertificateChainVerifierError> {
        let enc_certs = certificate_chain
            .into_iter()
            .map(|cert| cert.to_pem(LineEnding::LF))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
        let _pem_chain = enc_certs
            .iter()
            .map(|enc_cert| parse_x509_pem(enc_cert.as_ref()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
	let _cert_chain = _pem_chain
	    .iter()
	    .map(|pem| parse_x509_certificate(&pem.1.contents))
	    .collect::<Result<Vec< _>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
	
        let enc_crls = crls
            .into_iter()
            .map(|crl| der_encode(crl))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
        let _crls = enc_crls
            .iter()
            .map(|enc_crl| parse_x509_crl(enc_crl.as_ref()))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| CertificateChainVerifierError::GeneralCertificateError)?;
	
	let res = _cert_chain.last().unwrap().1.verify_signature(None);
	eprintln!("Verification: {:?}", res);
	assert!(res.is_ok());
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
