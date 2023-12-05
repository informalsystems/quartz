use cosmwasm_std::StdError;
use hex::FromHexError;
use k256::ecdsa::Error as K256Error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Invalid pubkey")]
    InvalidPubKey(PublicKeyError),
}

#[derive(Error, Debug)]
pub enum PublicKeyError {
    #[error("Not Secp256K1")]
    K256(K256Error),
    #[error("Invalid hex")]
    Hex(FromHexError),
}

impl<T: Into<PublicKeyError>> From<T> for ContractError {
    fn from(e: T) -> Self {
        let e = e.into();
        Self::InvalidPubKey(e)
    }
}

impl From<K256Error> for PublicKeyError {
    fn from(e: K256Error) -> Self {
        PublicKeyError::K256(e)
    }
}

impl From<FromHexError> for PublicKeyError {
    fn from(e: FromHexError) -> Self {
        PublicKeyError::Hex(e)
    }
}
