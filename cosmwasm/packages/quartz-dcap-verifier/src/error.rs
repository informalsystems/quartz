use core::{any::type_name, fmt::Display};

use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
}

pub fn into_std_err<E: Display + 'static>(err: E) -> StdError {
    StdError::generic_err(format!("{:?}: {err}", type_name::<E>()))
}
