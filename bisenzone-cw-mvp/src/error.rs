use cosmwasm_std::StdError;
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
}
