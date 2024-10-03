
# Quartz CosmWasm (quartz-contract-core) Specification

## Abstract

This document specifies the `quartz-contract-core` package, which provides a high-level framework for building attestation-aware smart contracts on CosmWasm. The package implements secure message handling, state management, and attestation verification for Intel SGX-based contracts.

## Table of Contents

- [Introduction](#introduction)
- [Functionality](#functionality)
- [Implementation](#implementation)
- [Properties](#properties)
- [Assumptions](#assumptions)

## Introduction

The `quartz-contract-core` package is designed to facilitate the development of secure, attestation-aware smart contracts on the CosmWasm platform. It provides a set of tools and abstractions that allow developers to easily integrate Intel SGX remote attestation into their smart contracts, ensuring that only authorized enclaves can interact with the contract's sensitive functions.

### Scope

This specification covers the core components of the `quartz-contract-core` package, including:

1. The `Attested<M, A>` wrapper for secure message handling
2. Attestation types and verification processes
3. State management utilities
4. Message handling and execution flow

### Design Goals

- Provide a secure and easy-to-use framework for attestation-aware smart contracts
- Abstract away the complexities of SGX attestation verification
- Ensure compatibility with DCAP attestation protocols (EPID is deprecated now by Intel)
- Allow for easy testing and mocking of SGX environments

## Functionality

The `quartz-contract-core` package provides the following key functionalities:

1. Secure message wrapping with `Attested<M, A>`
2. Attestation verification for DCAP protocols
3. State management utilities for CosmWasm contracts
4. Execution handlers for attested messages

### Definitions

- `Attested<M, A>`: A wrapper struct that holds a message `M` and its attestation `A`.
- `Attestation`: A trait that defines the common interface for different attestation types.
- `DcapAttestation`: A struct representing a DCAP attestation.
- `MockAttestation`: A struct for mocking attestations in test environments.

## Implementation

The `quartz-contract-core` package is implemented in Rust and relies on the CosmWasm framework. The main components are:

### Attested Message Wrapper

The `Attested<M, A>` struct is a wrapper for holding a message and its attestation:

```rust
/// A wrapper struct for holding a message and its attestation.
#[derive(Clone, Debug, PartialEq)]
pub struct Attested<M, A> {
    pub msg: M,
    pub attestation: A,
}

impl<M, A> Attested<M, A> {
    pub fn new(msg: M, attestation: A) -> Self {
        Self { msg, attestation }
    }
}
```


### Attestation Trait

The `Attestation` trait defines the common interface for different attestation types:

```rust
pub trait Attestation {
    fn verify(&self, deps: Deps<'_>, env: &Env) -> Result<(), Error>;
}
```


### Attestation Verification

The attestation verification process is handled by the `quartz-tee-ra` package, which is a dependency of `quartz-contract-core`. The verification functions are called within the execution handlers of `quartz-contract-core`.

```rust
pub use intel_sgx::{
    dcap::verify as verify_dcap_attestation,
    Error,
};
```


### Execution Handler

The execution handler is responsible for processing attested messages and verifying their attestations before executing the contained message.

```rust
pub fn handle_attested<M, A, H>(
    deps: DepsMut<'_>,
    env: Env,
    info: MessageInfo,
    attested: Attested<M, A>,
) -> Result<Response, Error>
where
    M: DeserializeOwned + Serialize,
    A: Attestation,
    H: Handler<M>,
{
    // Verify the attestation
    attested.attestation.verify(deps.as_ref(), &env)?;

    // Extract the message from the attested wrapper
    let msg = attested.msg;

    // Dispatch the message to the appropriate handler
    H::handle(deps, env, info, msg)
}
```


## Properties

The `quartz-contract-core` package ensures the following properties:

1. **Attestation Integrity**: All messages wrapped with `Attested<M, A>` must have a valid attestation that can be verified.

2. **Message Confidentiality**: The contents of attested messages are only accessible after successful attestation verification.

3. **Enclave Identity Verification**: The package verifies that messages come from authorized enclaves by checking the MRENCLAVE value.

4. **DCAP Support**: The package supports DCAP attestation protocols.

5. **Testability**: The mock-sgx feature allows for easy testing of contracts without a real SGX environment.

## Assumptions

The `quartz-contract-core` package operates under the following assumptions:

1. The underlying SGX hardware and software stack is secure and not compromised.

2. The attestation verification process provided by the Intel Attestation Service (IAS) or DCAP infrastructure is reliable and secure.

3. The CosmWasm runtime environment correctly executes the contract code and maintains the integrity of the contract's state.

4. The communication channel between the enclave and the smart contract is secure and resistant to man-in-the-middle attacks.

5. The contract developers correctly implement the `quartz-contract-core` abstractions and follow secure coding practices.

6. The MRENCLAVE values of authorized enclaves are known and correctly configured in the contract's state.

By adhering to this specification, developers can create secure, attestation-aware smart contracts that leverage the power of Intel SGX technology within the CosmWasm ecosystem.# Quartz CosmWasm (`quartz-contract-core`) Specification
