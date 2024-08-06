pub mod dcap;
use mc_sgx_dcap_types::{Collateral, Quote3, QuoteSignType};
pub use dcap::{
    DcapVerifier,
    DcapVerifierOutput,
    Evidence,
    TrustedIdentity,
    TrustedMrEnclaveIdentity,
    TrustedMrSignerIdentity,
    VerificationOutput,
};