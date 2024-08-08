#![deny(
    warnings,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications
)]
#![forbid(unsafe_code)]

extern crate cosmwasm_std;

pub mod contract;
pub mod error;
pub mod msg;
pub mod state;
