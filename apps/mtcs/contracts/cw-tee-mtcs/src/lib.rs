// #![deny(
//     warnings,
//     trivial_casts,
//     trivial_numeric_casts,
//     unused_import_braces,
//     unused_qualifications
// )]
// #![forbid(unsafe_code)]

pub mod contract;
mod error;
pub mod msg;
pub mod state;

pub use crate::error::ContractError;
