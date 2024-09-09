use std::{
    error::Error,
    fs::{read, File},
    io::{Error as IoError, ErrorKind, Write},
};

use mc_sgx_dcap_sys_types::sgx_ql_qve_collateral_t;
use quartz_cw::{
    msg::execute::attested::{HasUserData, RawDcapAttestation},
    state::{MrEnclave, UserData},
};
use quartz_tee_ra::intel_sgx::dcap::Collateral;
use reqwest::{
    blocking::Client,
    header::{ACCEPT, CONTENT_TYPE},
};
use serde::Serialize;
use serde_json::Value as JsonValue;

use crate::types::Fmspc;

/// The trait defines the interface for generating attestations from within an enclave.
pub trait Attestor {
    type Error: ToString;
    type Attestation: Serialize;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error>;

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error>;

    fn attestation(&self, user_data: impl HasUserData) -> Result<Self::Attestation, Self::Error>;
}

/// An `Attestor` for generating EPID attestations for Gramine based enclaves.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct EpidAttestor;

impl Attestor for EpidAttestor {
    type Error = IoError;
    type Attestation = Vec<u8>;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error> {
        let user_data = user_data.user_data();
        let mut user_report_data = File::create("/dev/attestation/user_report_data")?;
        user_report_data.write_all(user_data.as_slice())?;
        user_report_data.flush()?;
        read("/dev/attestation/quote")
    }

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error> {
        let quote = self.quote(NullUserData)?;
        Ok(quote[112..(112 + 32)]
            .try_into()
            .expect("hardcoded array size"))
    }

    fn attestation(&self, user_data: impl HasUserData) -> Result<Self::Attestation, Self::Error> {
        self.quote(user_data)
    }
}

/// An `Attestor` for generating DCAP attestations for Gramine based enclaves.
#[derive(Clone, PartialEq, Debug)]
pub struct DcapAttestor {
    pub fmspc: Fmspc,
}

impl Attestor for DcapAttestor {
    type Error = IoError;
    type Attestation = RawDcapAttestation;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error> {
        let user_data = user_data.user_data();
        let mut user_report_data = File::create("/dev/attestation/user_report_data")?;
        user_report_data.write_all(user_data.as_slice())?;
        user_report_data.flush()?;
        read("/dev/attestation/quote")
    }

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error> {
        let quote = self.quote(NullUserData)?;
        Ok(quote[112..(112 + 32)]
            .try_into()
            .expect("hardcoded array size"))
    }

    fn attestation(&self, user_data: impl HasUserData) -> Result<Self::Attestation, Self::Error> {
        fn pccs_query_pck() -> Result<(JsonValue, JsonValue), Box<dyn Error>> {
            // FIXME(hu55a1n1): get the URL from CLI
            let url = "https://127.0.0.1:11089/sgx/certification/v4/pckcrl?ca=processor";

            let client = Client::new();
            let response = client
                .get(&url)
                .header(ACCEPT, "application/json")
                .header(CONTENT_TYPE, "application/json")
                .send()?;

            if response.status().is_success() {
                let json_response: JsonValue = response.json()?;
                // response has pck-crl and header has issuer chain!
                if let (Some(pck_crl), Some(pck_crl_issuer_chain)) = (
                    json_response.get("pck_crl"),
                    json_response.get("pck_crl_issuer_chain"),
                ) {
                    Ok((pck_crl.clone(), pck_crl_issuer_chain.clone()))
                } else {
                    Err(Box::new(IoError::new(
                        ErrorKind::Other,
                        "PCCS query: bad response",
                    )))
                }
            } else {
                Err(Box::new(IoError::new(
                    ErrorKind::Other,
                    "PCCS query: failed",
                )))
            }
        }

        fn collateral(tcb_info: &str) -> Collateral {
            let mut sgx_collateral = sgx_ql_qve_collateral_t::default();

            // SAFETY: Version is a union which is inherently unsafe
            #[allow(unsafe_code)]
            let version = unsafe { sgx_collateral.__bindgen_anon_1.__bindgen_anon_1.as_mut() };
            version.major_version = 3;
            version.minor_version = 1;

            let pck_issuer_cert =
                include_str!("../../../cosmwasm/packages/quartz-tee-ra/data/processor_ca.pem");
            let root_cert =
                include_str!("../../../cosmwasm/packages/quartz-tee-ra/data/root_ca.pem");

            let mut pck_crl_chain = [pck_issuer_cert, root_cert].join("\n").as_bytes().to_vec();
            pck_crl_chain.push(0);
            sgx_collateral.pck_crl_issuer_chain = pck_crl_chain.as_ptr() as _;
            sgx_collateral.pck_crl_issuer_chain_size = pck_crl_chain.len() as u32;

            let mut root_crl =
                include_bytes!("../../../cosmwasm/packages/quartz-tee-ra/data/root_crl.der")
                    .to_vec();
            root_crl.push(0);
            sgx_collateral.root_ca_crl = root_crl.as_ptr() as _;
            sgx_collateral.root_ca_crl_size = root_crl.len() as u32;

            let mut pck_crl =
                include_bytes!("../../../cosmwasm/packages/quartz-tee-ra/data/processor_crl.der")
                    .to_vec();
            pck_crl.push(0);
            sgx_collateral.pck_crl = pck_crl.as_ptr() as _;
            sgx_collateral.pck_crl_size = pck_crl.len() as u32;

            let tcb_cert =
                include_str!("../../../cosmwasm/packages/quartz-tee-ra/data/tcb_signer.pem");
            let mut tcb_chain = [tcb_cert, root_cert].join("\n").as_bytes().to_vec();
            tcb_chain.push(0);
            sgx_collateral.tcb_info_issuer_chain = tcb_chain.as_ptr() as _;
            sgx_collateral.tcb_info_issuer_chain_size = tcb_chain.len() as u32;

            sgx_collateral.tcb_info = tcb_info.as_ptr() as _;
            sgx_collateral.tcb_info_size = tcb_info.len() as u32;

            // For live data the QE identity uses the same chain as the TCB info
            sgx_collateral.qe_identity_issuer_chain = tcb_chain.as_ptr() as _;
            sgx_collateral.qe_identity_issuer_chain_size = tcb_chain.len() as u32;

            const QE_IDENTITY_JSON: &str =
                include_str!("../../../cosmwasm/packages/quartz-tee-ra/data/qe_identity.json");
            sgx_collateral.qe_identity = QE_IDENTITY_JSON.as_ptr() as _;
            sgx_collateral.qe_identity_size = QE_IDENTITY_JSON.len() as u32;

            Collateral::try_from(&sgx_collateral).expect("Failed to parse collateral")
        }

        let (_pck_crl, _pck_crl_issuer_chain) =
            pccs_query_pck().map_err(|e| IoError::new(ErrorKind::Other, e.to_string()))?;

        let quote = self.quote(user_data)?;

        // FIXME(hu55a1n1): replace `pck_crl_chain` and `pck_crl` in the collateral (below) with data queried from PCCS (above)
        let collateral = serde_json::to_value(collateral(&self.fmspc.to_string()))?;

        Ok(RawDcapAttestation {
            quote: quote.into(),
            collateral,
        })
    }
}

/// A mock `Attestor` that creates a quote consisting of just the user report data. (only meant for
/// testing purposes)
#[derive(Clone, PartialEq, Debug, Default)]
pub struct MockAttestor;

impl Attestor for MockAttestor {
    type Error = String;
    type Attestation = Vec<u8>;

    fn quote(&self, user_data: impl HasUserData) -> Result<Vec<u8>, Self::Error> {
        let user_data = user_data.user_data();
        Ok(user_data.to_vec())
    }

    fn mr_enclave(&self) -> Result<MrEnclave, Self::Error> {
        Ok(Default::default())
    }

    fn attestation(&self, user_data: impl HasUserData) -> Result<Self::Attestation, Self::Error> {
        self.quote(user_data)
    }
}

struct NullUserData;

impl HasUserData for NullUserData {
    fn user_data(&self) -> UserData {
        [0u8; 64]
    }
}
