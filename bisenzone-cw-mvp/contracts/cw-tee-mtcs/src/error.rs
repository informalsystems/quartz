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

    #[error("Not Secp256K1")]
    K256(K256Error),

    #[error("Invalid hex")]
    Hex(FromHexError),

    #[error("Invalid length")]
    BadLength,
}

impl From<K256Error> for ContractError {
    fn from(e: K256Error) -> Self {
        ContractError::K256(e)
    }
}

impl From<FromHexError> for ContractError {
    fn from(e: FromHexError) -> Self {
        ContractError::Hex(e)
    }
}
