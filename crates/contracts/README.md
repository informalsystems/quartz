# Quartz CosmWasm Packages

This repository contains a collection of packages designed to facilitate the development of CosmWasm smart contracts for Quartz apps.

## Packages

### 1. `quartz-contract-core` 

The `quartz-contract-core` package offers a high-level framework for building attestation-aware smart contracts by wrapping CosmWasm messages in TEE attestations (e.g. DCAP). 

- Defines `Attested<M, A>` wrapper for a message and its attestation
- Supports a MockAttestation type for development ease 
- Implements session management for secure communication between contract and enclave
The `quartz-contract-core` package handles actual DCAP verification within smart contracts by calling the standalone `quartz-dcap-verifier` and `quartz-tcbinfo` contracts.

### 2. `quartz-dcap-verifier` 

Your personal DCAP detective! This package is a standalone smart contract for verifying DCAP attestations that can be called by other contracts.

- Thin wrapper for standalone smart contract around the functionality provided in the `quartz-tee-ra` package
- Provides query and execute entry points for attestation checks

### 3. `quartz-tee-ra` 

This `quartz-tee-ra` handles Intel SGX remote attestation for DCAP.

- Verifies DCAP attestations
- Provides core types and structures for SGX quotes
- Implements cryptographic verification of attestation reports

### 4. `quartz-tcbinfo` 

The `quartz-tcbinfo` package manages and verifies TCB information crucial for maintaining enclave security.

- Stores and retrieves TCB information
- Verifies TCB signatures and certificates
- Provides a CosmWasm contract for TCB management

### 5. `cw-client` 

The `cw-client` package offers a Rust client interface for interacting with Wasmd nodes.

- Defines traits for querying and executing CosmWasm contracts
- Provides utilities for deploying and interacting with contracts


## Examples

Check out the `integration_tests` modules in each package for usage examples and best practices!

## Contributing

We welcome contributions from the community! If you find any issues or have suggestions for improvement, please open an issue or submit a pull request. Make sure to follow the contribution guidelines outlined in the repository.

<!-- ## License //TODO check which license is needed

The Quartz CosmWasm Packages are released under the [MIT License](LICENSE). -->
## Contact 

If you have any questions or need further assistance, please reach out to us at [quartz.support@informal.systems](mailto:quartz.support@informal.systems).

