# Quartz TEE Remote Attestation (quartz-tee-ra)

Quartz TEE Remote Attestation is a Rust library that provides functionality for verifying Intel SGX remote attestations. It supports both EPID and DCAP attestation protocols, making it a versatile tool for secure enclave-based applications.

## Features

- EPID attestation verification
- DCAP attestation verification
- Support for Intel SGX quote parsing and validation
- Integration with MobileCoin's attestation verifier

## Installation

Add `quartz-tee-ra` to your `Cargo.toml`:

```toml
[dependencies]
quartz-tee-ra = { path = "../packages/quartz-tee-ra" }
```

## Usage

Here's a basic example of how to use `quartz-tee-ra` for DCAP attestation verification:

```rust
use quartz_tee_ra::{verify_dcap_attestation, intel_sgx::dcap::{Quote3, Collateral, TrustedIdentity}};

fn verify_attestation(quote: Quote3<Vec<u8>>, collateral: Collateral, identities: &[TrustedIdentity]) {
    let verification_output = verify_dcap_attestation(quote, collateral, identities);
    
    if verification_output.is_success().into() {
        println!("Attestation verified successfully!");
    } else {
        println!("Attestation verification failed: {:?}", verification_output);
    }
}
```

For EPID attestation:

```rust
use quartz_tee_ra::{verify_epid_attestation, IASReport};

fn verify_epid_attestation(report: IASReport, mrenclave: Vec<u8>, user_data: Vec<u8>) {
    match verify_epid_attestation(report, mrenclave, user_data) {
        Ok(_) => println!("EPID attestation verified successfully!"),
        Err(e) => println!("EPID attestation verification failed: {:?}", e),
    }
}
```

## API Reference

The main functions exported by this library are:


```21:25:cosmwasm/packages/quartz-tee-ra/src/lib.rs
pub use intel_sgx::{
    dcap::verify as verify_dcap_attestation,
    epid::{types::IASReport, verifier::verify as verify_epid_attestation},
    Error,
};
```


## Dependencies

This package relies on several external crates and MobileCoin libraries. Key dependencies include:


```11:36:cosmwasm/packages/quartz-tee-ra/Cargo.toml

[dependencies]
# external
der.workspace = true
hex-literal.workspace = true
num-bigint.workspace = true
serde.workspace = true
serde_json.workspace = true
sha2.workspace = true
thiserror.workspace = true
x509-cert.workspace = true
x509-parser.workspace = true

# mobilecoin
mc-attestation-verifier.workspace = true
mc-sgx-dcap-types.workspace = true

# cosmos
cosmwasm-schema.workspace = true
cosmwasm-std.workspace = true

[dev-dependencies]
hex = "0.4.3"
mc-sgx-dcap-types.workspace = true
mc-sgx-core-types.workspace = true
mc-sgx-dcap-sys-types.workspace = true
```


## Development

To run tests:

```sh
cargo test
```

## License

[LICENSE_NAME] - see the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Safety

This crate uses `#![deny(unsafe_code)]` to ensure everything is implemented in 100% Safe Rust.


```1:17:cosmwasm/packages/quartz-tee-ra/src/lib.rs
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
```


Note: There's a TODO to uncomment the `#![forbid(unsafe_code)]` once better wrappers for FFI structs and constructors are implemented.

## Disclaimer

This software is provided "as is" and without any express or implied warranties, including, without limitation, the implied warranties of merchantability and fitness for a particular purpose.