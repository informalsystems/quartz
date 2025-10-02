use std::fmt::Display;

use cosmrs::abci::GasInfo;
use serde::{de::DeserializeOwned, Serialize};

use crate::chain_client::default::DefaultTxConfig;

pub mod default;

/// Abstraction over a blockchain client.
///
/// This trait defines the common operations needed to interact with a blockchain, including
/// querying contract state, obtaining existence proofs, sending transactions, and waiting for a
/// specified number of blocks.
#[async_trait::async_trait]
pub trait ChainClient: Send + Sync + 'static {
    /// The contract type that this client interacts with.
    type Contract: Send + Sync + 'static;
    /// The error type returned by blockchain operations.
    type Error: Display + Send + Sync + 'static;
    /// The type representing cryptographic proofs for on-chain data.
    type Proof: Serialize + Send + Sync + 'static;
    /// The type used to represent query messages.
    type Query: Send + Sync + 'static;
    /// The output type returned after sending a transaction.
    type TxOutput: Send + Sync + 'static;

    /// Sends a query to the specified contract and returns a deserialized result.
    ///
    /// # Parameters
    ///
    /// - `contract`: A reference to the contract identifier.
    /// - `query`: A query message convertible into the client's query type.
    ///
    /// # Returns
    ///
    /// A `Result` containing the deserialized query result on success,
    /// or an error of type `Self::Error` if the query fails.
    async fn query_contract<R: DeserializeOwned + Default + Send>(
        &self,
        contract: &Self::Contract,
        query: impl Into<Self::Query> + Send,
    ) -> Result<R, Self::Error>;

    /// Retrieves an existence proof for a given storage key in the contract.
    ///
    /// # Parameters
    ///
    /// - `contract`: A reference to the contract identifier.
    /// - `storage_key`: The storage key for which to obtain the existence proof.
    ///
    /// # Returns
    ///
    /// A `Result` containing the existence proof of type `Self::Proof` on success,
    /// or an error of type `Self::Error` if the operation fails.
    async fn existence_proof(
        &self,
        contract: &Self::Contract,
        storage_key: &str,
    ) -> Result<Self::Proof, Self::Error>;

    /// Sends a transaction to the specified contract.
    ///
    /// # Parameters
    ///
    /// - `contract`: A reference to the contract identifier.
    /// - `msgs`: The transaction messages, which must be serializable.
    /// - `config`: The transaction configuration (e.g., gas, fees).
    ///
    /// # Returns
    ///
    /// A `Result` containing the transaction output of type `Self::TxOutput` on success,
    /// or an error of type `Self::Error` if the transaction fails.
    async fn send_tx<M: Serialize>(
        &self,
        contract: &Self::Contract,
        msgs: impl Iterator<Item = M> + Send + Sync,
        config: DefaultTxConfig,
    ) -> Result<Self::TxOutput, Self::Error>;

    /// Simulates a transaction returning the gas_info.
    ///
    /// # Parameters
    ///
    /// - `contract`: A reference to the contract identifier.
    /// - `msgs`: The transaction messages, which must be serializable.
    /// - `config`: The transaction configuration (e.g., gas, fees).
    ///
    /// # Returns
    ///
    /// A `Result` containing the `GasInfo` on success,
    /// or an error of type `Self::Error` if the transaction fails.
    async fn simulate_tx<M: Serialize>(
        &self,
        contract: &Self::Contract,
        msgs: impl Iterator<Item = M> + Send + Sync,
        config: DefaultTxConfig,
    ) -> Result<GasInfo, Self::Error>;

    /// Waits for a specified number of blocks to be produced on the blockchain.
    ///
    /// # Parameters
    ///
    /// - `blocks`: The number of blocks to wait for.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success (`Ok(())`) or an error of type `Self::Error`
    /// if the operation fails.
    async fn wait_for_blocks(&self, blocks: u8) -> Result<(), Self::Error>;
}
