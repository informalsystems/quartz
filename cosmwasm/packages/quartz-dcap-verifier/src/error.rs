use core::{any::type_name, fmt::Display};

use cosmwasm_std::StdError;

pub fn into_std_err<E: Display + 'static>(err: E) -> StdError {
    StdError::generic_err(format!("{:?}: {err}", type_name::<E>()))
}
