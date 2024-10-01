use thiserror::Error;

pub mod dcap;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Specified user data does not match the report")]
    UserDataMismatch,
    #[error("Specified MRENCLAVE does not match the report")]
    MrEnclaveMismatch,
    #[error("DCAP specific error: {0:?}")]
    Dcap(dcap::VerificationOutput<dcap::DcapVerifierOutput>),
}
