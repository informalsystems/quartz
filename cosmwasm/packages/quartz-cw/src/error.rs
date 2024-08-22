use cosmwasm_std::StdError;
use k256::ecdsa::Error as K256Error;
use quartz_tee_ra::Error as RaVerificationError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("{0}")]
    RaVerification(#[from] RaVerificationError),
    #[error("Not Secp256K1")]
    K256(K256Error),
    #[error("invalid session nonce or attempt to reset pub_key")]
    BadSessionTransition,
    #[error("tcbinfo query error")]
    TcbInfoQueryError,
}

impl From<K256Error> for Error {
    fn from(e: K256Error) -> Self {
        Self::K256(e)
    }
}
