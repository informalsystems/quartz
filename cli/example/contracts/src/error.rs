use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use cw_utils::PaymentError;
use quartz_common::contract::error::Error as QuartzError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    Quartz(#[from] QuartzError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Duplicate entry found")]
    DuplicateEntry,

    #[error("Invalid length")]
    BadLength,

    #[error("Cw20 error: {0}")]
    Cw20(Cw20ContractError),

    #[error("Payment error: {0}")]
    CwUtil(PaymentError),
}

impl From<Cw20ContractError> for ContractError {
    fn from(e: Cw20ContractError) -> Self {
        Self::Cw20(e)
    }
}

impl From<PaymentError> for ContractError {
    fn from(e: PaymentError) -> Self {
        Self::CwUtil(e)
    }
}
