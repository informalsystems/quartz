use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized {},
    // Add any other custom errors you like here.
    // Look at https://docs.rs/thiserror/1.0.21/thiserror/ for details.
    #[error("Certificate verification failed")]
    CertificateVerificationError,
    #[error("failed to verify tcbinfo")]
    TcbInfoVerificationError,
    #[error("invalid public key")]
    PublicKeyReadError,
    #[error("invalid date and time")]
    DateTimeReadError,
    #[error("invalid tcbinfo")]
    TcbInfoReadError,
}
