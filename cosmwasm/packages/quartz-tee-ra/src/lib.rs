#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_used,
    rust_2018_idioms,
    unused_lifetimes
)]
#![deny(
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_qualifications,
    warnings
)]
// FIXME(hu55a1n1) - uncomment once we have better wrappers for FFI structs and ctors
// #![forbid(unsafe_code)]

pub mod intel_sgx;

pub use intel_sgx::{
    dcap::verify as verify_dcap_attestation,
    epid::{types::IASReport, verifier::verify as verify_epid_attestation},
    Error,
};
