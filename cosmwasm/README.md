# Quartz CosmWasm Packages

Welcome to the Quartz CosmWasm Packages repository! This repository contains a collection of packages designed to facilitate the development of CosmWasm smart contracts for the Bisenzone ecosystem.

## Packages

### 1. `quartz-cw` 🧠

The `quartz-cw` package offers a high-level framework for building attestation-aware smart contracts.

- Defines `Attested<T>` wrapper for secure message handling
- Provides traits and structures for easy contract development
- Implements state management and message handling utilities

### 2. `quartz-dcap-verifier` 🔍

Your personal DCAP detective! This package specializes in verifying DCAP attestations within CosmWasm contracts.

- Offers a CosmWasm contract for DCAP attestation verification
- Provides query and execute entry points for attestation checks

### 3. `quartz-tee-ra` 🔐

This `quartz-tee-ra` handles Intel SGX remote attestation for both EPID and DCAP protocols.

- Verifies EPID and DCAP attestations
- Provides core types and structures for SGX quotes
- Implements cryptographic verification of attestation reports

### 4. `tcbinfo` 📊

The `tcbinfo` package manages and verifies TCB information crucial for maintaining enclave security.

- Stores and retrieves TCB information
- Verifies TCB signatures and certificates
- Provides a CosmWasm contract for TCB management

### 5. `wasmd-client` 🌉

The `wasmd-client` package offers a Rust client interface for interacting with Wasmd nodes.

- Defines traits for querying and executing CosmWasm contracts
- Provides utilities for deploying and interacting with contracts


## 📚 Examples

Check out the `integration_tests` modules in each package for usage examples and best practices!

## 🤝 Contributing

We welcome contributions from the community! If you find any issues or have suggestions for improvement, please open an issue or submit a pull request. Make sure to follow the contribution guidelines outlined in the repository.

<!-- ## 📄 License //TODO check which license is needed

The Quartz CosmWasm Packages are released under the [MIT License](LICENSE). -->

## Contact 📄

If you have any questions or need further assistance, please reach out to us at [quartz.support@informal.systems](mailto:quartz.support@informal.systems).

