/*!
# Quartz Core Enclave

This crate provides a *framework* for writing quartz application enclaves and implements the *core*
enclave logic for the quartz handshake protocol. Quartz enforces all user <> enclave communication
to happen via a blockchain for replay protection.

At a high level, the code here implements:
- The quartz enclave framework which includes various components that enable app devs to write
  secure enclaves with replay protection. This includes trait definitions and default implementations.
- Core enclave logic for the quartz handshake. This includes -
    - Event handlers for handling core events.
    - Request handlers for handling core requests.
    - gRPC service implementation for the request handlers.

---

## Framework Design

The framework separates trusted and untrusted code by defining two abstractions - the host and the
enclave, each represented by a separate trait.

### Host vs. Enclave Separation

The **host** (untrusted) code is responsible for:
- Identifying which chain events the application wants to handle.
- Collecting all necessary on-chain data for each event to form a provable request
  to the enclave (e.g., by fetching on-chain state, light client proofs, merkle proofs, etc.).
- Calling the enclave with the created request.
- Sending transactions to the blockchain on behalf of the enclave (AKA the response).

The **enclave** (trusted) code is responsible for:
- Determining which *requests* these events correspond to.
- Verifying request data integrity (via light client proofs and merkle proofs).
- Handling each request securely inside the TEE.
- (Optionally) generating responses to be posted to the chain.
- Attesting to the responses using remote-attestation (to be verified on-chain).

Through this layered approach:
- **Host** code (generally) runs outside the TEE, bridging the blockchain and the enclave.
- **Enclave** code runs inside a Gramine-based TEE, protecting private data and cryptographic
  operations.

---

## Lifecycle of a request

Below is a simplified lifecycle for a typical user <> enclave interaction involving a quartz app
enclave:

1. **User** *sends a request to the contract* (on-chain).
2. **Contract** *triggers an event* reflecting that new request.
3. **Host** (untrusted) *listens for relevant events* from the chain.
4. On seeing an event, the **Host** *constructs an enclave request* that encapsulates all the relevant
   data for handling the event.
5. The **Host** then *calls the enclave* with that request.
6. **Enclave** (trusted) *handles the request*, verifies the data, performs the necessary
  computations, and (optionally) returns an attested response.
7. The **Host** *sends the response* back to the chain, e.g. via a transaction.

---

## Usage
See the app enclaves in the `examples` directory for usage examples.

*/

#![doc = include_str!("../README.md")]
// #![forbid(unsafe_code)]
#![warn(
    clippy::checked_conversions,
    clippy::panic,
    clippy::panic_in_result_fn,
    trivial_casts,
    trivial_numeric_casts,
    rust_2018_idioms,
    unused_lifetimes,
    unused_import_braces,
    unused_qualifications
)]

use cosmrs::AccountId;
use quartz_contract_core::state::Config;

use crate::{
    attestor::{Attestor, DefaultAttestor},
    key_manager::{default::DefaultKeyManager, shared::SharedKeyManager, KeyManager},
    store::{default::DefaultStore, Store},
};

pub mod attestor;
pub mod chain_client;
pub mod event;
pub mod grpc;
pub mod handler;
pub mod host;
pub mod key_manager;
pub mod proof_of_publication;
pub mod store;
pub mod types;

pub type DefaultSharedEnclave<C, K = DefaultKeyManager> =
    DefaultEnclave<C, DefaultAttestor, SharedKeyManager<K>, DefaultStore>;

#[async_trait::async_trait]
pub trait Enclave: Send + Sync + 'static {
    type Attestor: Attestor;
    type KeyManager: KeyManager;
    type Store: Store;

    async fn attestor(&self) -> Self::Attestor;
    async fn key_manager(&self) -> Self::KeyManager;
    async fn store(&self) -> &Self::Store;
}

#[derive(Clone, Debug)]
pub struct DefaultEnclave<C, A = DefaultAttestor, K = DefaultKeyManager, S = DefaultStore> {
    pub attestor: A,
    pub key_manager: K,
    pub store: S,
    pub ctx: C,
}

impl<C: Send + Sync + 'static> DefaultSharedEnclave<C> {
    pub fn shared(attestor: DefaultAttestor, config: Config, ctx: C) -> DefaultSharedEnclave<C> {
        DefaultSharedEnclave {
            attestor,
            key_manager: SharedKeyManager::wrapping(DefaultKeyManager::default()),
            store: DefaultStore::new(config),
            ctx,
        }
    }

    pub fn with_key_manager<K: KeyManager>(
        self,
        key_manager: K,
    ) -> DefaultEnclave<C, <Self as Enclave>::Attestor, K, <Self as Enclave>::Store> {
        DefaultEnclave {
            attestor: self.attestor,
            key_manager,
            store: self.store,
            ctx: self.ctx,
        }
    }
}

#[async_trait::async_trait]
impl<C, A, K, S> Enclave for DefaultEnclave<C, A, K, S>
where
    C: Send + Sync + 'static,
    A: Attestor + Clone,
    K: KeyManager + Clone,
    S: Store<Contract = AccountId> + Clone,
{
    type Attestor = A;
    type KeyManager = K;
    type Store = S;

    async fn attestor(&self) -> Self::Attestor {
        self.attestor.clone()
    }

    async fn key_manager(&self) -> Self::KeyManager {
        self.key_manager.clone()
    }

    async fn store(&self) -> &Self::Store {
        &self.store
    }
}
