# common

A simple crate to re-export quartz core and protobufs:

```rust
#[cfg(feature = "contract")]
pub use quartz_contract_core as contract;
#[cfg(feature = "enclave")]
pub use quartz_enclave_core as enclave;
#[cfg(feature = "proto")]
pub use quartz_proto::quartz as proto;
```
