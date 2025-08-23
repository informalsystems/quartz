use quartz_contract_core::state::{Config, Nonce};

pub mod default;

/// A trait representing a key-value store for managing core enclave state.
///
/// The [`Store`] trait defines an asynchronous interface for reading and writing
/// various pieces of state that are essential to the handshake protocol.
#[async_trait::async_trait]
pub trait Store: Send + Sync + 'static {
    /// The type representing the contract that the store manages.
    type Contract: Send + Sync;
    /// The error type returned by store operations.
    type Error: ToString + Send + Sync;

    /// Retrieves the current enclave configuration.
    async fn get_config(&self) -> Result<Option<Config>, Self::Error>;

    /// Sets a new configuration for the enclave.
    async fn set_config(&self, config: Config) -> Result<Option<Config>, Self::Error>;

    /// Retrieves the contract associated with the enclave.
    async fn get_contract(&self) -> Result<Option<Self::Contract>, Self::Error>;

    /// Sets the contract associated with the enclave.
    async fn set_contract(
        &self,
        contract: Self::Contract,
    ) -> Result<Option<Self::Contract>, Self::Error>;

    /// Retrieves the current nonce.
    async fn get_nonce(&self) -> Result<Option<Nonce>, Self::Error>;

    /// Sets a new nonce for replay protection.
    async fn set_nonce(&self, nonce: Nonce) -> Result<Option<Nonce>, Self::Error>;

    /// Retrieves the current sequence number.
    async fn get_seq_num(&self) -> Result<u64, Self::Error>;

    /// Increments the sequence number by the given count.
    async fn inc_seq_num(&self, count: usize) -> Result<u64, Self::Error>;
}
