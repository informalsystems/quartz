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
#![forbid(unsafe_code)]

pub mod intel_sgx;

pub use intel_sgx::epid::types::IASReport;
pub use intel_sgx::epid::verifier::verify as verify_epid_attestation;
pub use intel_sgx::Error;
