
# Quartz CosmWasm (quartz-contract-core)

Quartz CosmWasm (quartz-contract-core) is a high-level framework for building attestation-aware smart contracts on CosmWasm. It provides a robust foundation for developing secure, Intel SGX-based contracts with built-in remote attestation support.

## Features

- `Attested<M, A>` wrapper for a message and its attestation
- MockAttestation type for development ease 
- Session management for secure communication between contract and enclave
- Verify DCAP attestations (by calling `dcap-verifier` and `tcbinfo` contracts) 
- Mock SGX support for testing environments

See also the [spec.md](./SPEC.md)

## Installation

Add `quartz-contract-core` to your `Cargo.toml`:

```toml
[dependencies]
quartz-contract-core = { path = "../packages/quartz-contract-core" }
```

## Usage

Here's a basic example of how to use `quartz-contract-core` in your CosmWasm contract:

```rust
use quartz_cw::prelude::*;
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: QuartzExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        QuartzExecuteMsg::Attested(attested_msg) => {
            // Handle attested message
            // Verification of the attestation is done automatically
            let result = attested_msg.handle(deps, env, info)?;
            Ok(result)
        },
        // Other message handlers...
    }
}
```

## Key Components

1. `Attested<M, A>`: A wrapper struct for holding a message and its attestation.
2. `Attestation`: A trait for attestation types (DCAP, Mock).
3. `HasUserData`: A trait for extracting user data from attestations.
4. `RawHandler`: A trait for handling raw messages.

## Configuration

You can enable mock SGX support for testing by adding the `mock-sgx` feature to your `Cargo.toml`:

```toml
[dependencies]
quartz-contract-core = { path = "../packages/quartz-contract-core", features = ["mock-sgx"] }
```

## Testing

To run the tests:

```sh
cargo test
```
