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

        #[error("Certificate verification error")]
        CertificateVerificationError,
    
        #[error("TCB Info verification error")]
        TcbInfoVerificationError,
    
        #[error("Certificate parsing error")]
        CertificateParsingError,
    
        #[error("TCB Info parsing error")]
        TcbInfoParsingError,
    
        #[error("DateTime parsing error")]
        DateTimeParsingError,
    
        #[error("Public key parsing error")]
        PublicKeyParsingError,
}