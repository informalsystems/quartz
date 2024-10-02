# Quartz CosmWasm Packages

CosmWasm packages for building Quartz apps and verifying remote
attestations from SGX.

The main interface for CosmWasm developers is package `core`.

## Packages

1. `quartz-contract-core`. High-level framework for building attestation-aware smart contracts by wrapping CosmWasm messages in TEE attestations (e.g. DCAP).
1. `quartz-dcap-verifier`: Standalone smart contract for verifying DCAP attestations that can be called by other contracts.
1. `quartz-tee-ra`: Implements core types for SGX quotes and verification for
   DCAP attestations
1. `quartz-tcbinfo`: Standalone smart contract for verifying attestations come
   from valid/secure Trusted Compute Base (TCB)
