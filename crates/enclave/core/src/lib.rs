/*!
# Quartz Core Enclave

This crate provides a *framework* for writing quartz application enclaves and implements the *core*
enclave logic for the quartz handshake protocol. Quartz enforces all user <> enclave communication
to happen via a blockchain for replay protection.

At a high level, the code here implements:
- The quartz enclave framework which includes various components that enable app devs to write
  secure enclaves with replay protection. This includes trait definitions and default implementations.
- Core enclave logic for the quartz handshake. This includes -
    - **Event handlers** for handling core events.
    - **Request handlers** for handling core requests.
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

use std::path::PathBuf;

use anyhow::anyhow;
use cosmrs::AccountId;
use log::{debug, trace, warn};
use quartz_contract_core::state::Config;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::mpsc,
};

use crate::{
    attestor::{Attestor, DefaultAttestor},
    backup_restore::{Backup, Export, Import},
    key_manager::{default::DefaultKeyManager, shared::SharedKeyManager, KeyManager},
    store::{default::DefaultStore, Store},
};

pub mod attestor;
pub mod backup_restore;
pub mod chain_client;
pub mod event;
pub mod grpc;
pub mod handler;
pub mod host;
pub mod key_manager;
pub mod proof_of_publication;
pub mod store;
pub mod types;

/// A type alias for a default, thread-safe enclave.
///
/// `DefaultSharedEnclave` is a specialization of [`DefaultEnclave`] that uses the default
/// attestation and storage components, along with a shared key-manager and store to allow safe
/// concurrent access.
pub type DefaultSharedEnclave<C, K = DefaultKeyManager> =
    DefaultEnclave<C, DefaultAttestor, SharedKeyManager<K>, DefaultStore>;

/// Represents the core functionality running inside a TEE.
///
/// An `Enclave` is the trusted environment where sensitive logic and data
/// reside. It provides access to three essential components:
///
/// 1. [`Attestor`]: Generates attestations, proving that the code truly runs within the expected
///    enclave.
/// 2. [`KeyManager`]: Manages a cryptographic key for encrypted communication with the enclave.
///    The associated public key is published on-chain and private requests are expected to be
///    encrypted to this public key so they can only be decrypted inside the enclave.
/// 3. [`Store`]: A basic data store for core data used during the handshake.
///
/// For convenience, Quartz includes a default generic implementation (e.g. [`DefaultEnclave`]) that
/// may suffice for many applications.
#[async_trait::async_trait]
pub trait Enclave: Send + Sync + 'static {
    /// The type of attestor used by this enclave, handling generation of attestation quotes.
    type Attestor: Attestor;
    /// The type of key-manager used by this enclave, providing a cryptographic key for encryption.
    type KeyManager: KeyManager;
    /// The type of store used by this enclave, managing enclave state required for the handshake.
    type Store: Store;

    /// Returns a handle of this enclave's attestor. Since this async code, the expectation is that
    /// the instance is shared and thread-safe. (e.g. behind a mutex)
    async fn attestor(&self) -> Self::Attestor;

    /// Returns a handle of this enclave's key-manager. Since this async code, the expectation is that
    /// the instance is shared and thread-safe. (e.g. behind a mutex)
    async fn key_manager(&self) -> Self::KeyManager;

    /// Returns a handle of this enclave's store. Since this async code, the expectation is that
    /// the instance is shared and thread-safe. (e.g. behind a mutex)
    async fn store(&self) -> &Self::Store;
}

/// Notification the enclave may emit.
#[derive(Debug, Clone)]
pub enum Notification {
    /// Fired once the enclave finishes its remote-attestation handshake.
    HandshakeComplete,
}

/// The default generic implementation of the [`Enclave`] trait for convenience.
/// Includes a generic context for additional application-specific data or configuration.
#[derive(Clone, Debug)]
pub struct DefaultEnclave<C, A = DefaultAttestor, K = DefaultKeyManager, S = DefaultStore> {
    pub attestor: A,
    pub key_manager: K,
    pub store: S,
    pub ctx: C,
    pub notifier_tx: mpsc::Sender<Notification>,
}

impl<C: Send + Sync + 'static> DefaultSharedEnclave<C> {
    pub fn shared(
        attestor: DefaultAttestor,
        config: Config,
        ctx: C,
    ) -> (DefaultSharedEnclave<C>, mpsc::Receiver<Notification>) {
        let (notifier_tx, notifier_rx) = mpsc::channel(10); // ← NEW

        (
            DefaultSharedEnclave {
                attestor,
                key_manager: SharedKeyManager::wrapping(DefaultKeyManager::default()),
                store: DefaultStore::new(config),
                ctx,
                notifier_tx,
            },
            notifier_rx,
        )
    }

    /// Consumes a `DefaultEnclave` and returns another one with the specified key-manager.
    pub fn with_key_manager<K: KeyManager>(
        self,
        key_manager: K,
    ) -> DefaultEnclave<C, <Self as Enclave>::Attestor, K, <Self as Enclave>::Store> {
        debug!("Updating enclave with new key manager");
        DefaultEnclave {
            attestor: self.attestor,
            key_manager,
            store: self.store,
            ctx: self.ctx,
            notifier_tx: self.notifier_tx,
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
        trace!("Retrieving enclave attestor");
        self.attestor.clone()
    }

    async fn key_manager(&self) -> Self::KeyManager {
        trace!("Retrieving enclave key manager");
        self.key_manager.clone()
    }

    async fn store(&self) -> &Self::Store {
        trace!("Retrieving enclave store");
        &self.store
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DefaultBackup {
    store: Vec<u8>,
    key_manager: Vec<u8>,
    attestor: Vec<u8>,
    ctx: Vec<u8>,
}

#[async_trait::async_trait]
impl<C, A, K, S> Backup for DefaultEnclave<C, A, K, S>
where
    C: Send + Sync + Export + Import,
    A: Attestor + Clone + Export + Import,
    K: KeyManager + Clone + Export + Import,
    S: Store<Contract = AccountId> + Clone + Export + Import,
{
    type Config = PathBuf;
    type Error = anyhow::Error;

    async fn backup(&self, config: Self::Config) -> Result<(), Self::Error> {
        trace!("Backing up to {config:?}");

        let exported_store = self
            .store
            .export()
            .await
            .map_err(|e| anyhow!("store export failed: {e:?}"))?;
        let exported_key_manager = self
            .key_manager
            .export()
            .await
            .map_err(|e| anyhow!("key-manager export failed: {e:?}"))?;
        let exported_attestor = self
            .attestor
            .export()
            .await
            .map_err(|e| anyhow!("attestor export failed: {e:?}"))?;
        let exported_ctx = self
            .ctx
            .export()
            .await
            .map_err(|e| anyhow!("ctx export failed: {e:?}"))?;
        let backup = DefaultBackup {
            store: exported_store,
            key_manager: exported_key_manager,
            attestor: exported_attestor,
            ctx: exported_ctx,
        };
        let backup_ser = serde_json::to_vec(&backup).expect("infallible serializer");

        let mut sealed_file = File::create(config)
            .await
            .map_err(|e| anyhow!("backup file creation failed: {e:?}"))?;
        sealed_file
            .write_all(backup_ser.as_slice())
            .await
            .map_err(|e| anyhow!("backup writes failed: {e:?}"))?;

        Ok(())
    }

    async fn try_restore(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        trace!("Restoring from {config:?}");

        let mut sealed_file = File::open(config).await?;
        let mut backup_ser = vec![];
        sealed_file.read_to_end(&mut backup_ser).await?;
        let backup: DefaultBackup = serde_json::from_slice(&backup_ser)?;

        let imported_store = S::import(backup.store)
            .await
            .map_err(|e| anyhow!("store import failed: {e:?}"))?;
        let imported_key_manager = K::import(backup.key_manager)
            .await
            .map_err(|e| anyhow!("key-manager import failed: {e:?}"))?;
        let imported_attestor = A::import(backup.attestor)
            .await
            .map_err(|e| anyhow!("attestor import failed: {e:?}"))?;
        let imported_ctx = C::import(backup.ctx)
            .await
            .map_err(|e| anyhow!("ctx import failed: {e:?}"))?;

        self.store = imported_store;
        self.key_manager = imported_key_manager;
        self.attestor = imported_attestor;
        self.ctx = imported_ctx;

        // if restored from a previous backup - manually notify host of handshake completion
        self.notifier_tx
            .send(Notification::HandshakeComplete)
            .await
            .expect("Receiver half of the channel must NOT be closed");

        Ok(())
    }
}
