use cosmwasm_std::StdError;
use cw20_base::ContractError as Cw20ContractError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Specified path does not form a cycle")]
    PathNotCycle,

    #[error("Amount is greater than utilization")]
    ClearingTooMuch,

    #[error("Cw20 error: {0}")]
    Cw20(Cw20ContractError),
}

impl From<Cw20ContractError> for ContractError {
    fn from(e: Cw20ContractError) -> Self {
        Self::Cw20(e)
    }
}
