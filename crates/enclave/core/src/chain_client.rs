use std::fmt::Display;

use serde::{de::DeserializeOwned, Serialize};

pub mod default;

#[async_trait::async_trait]
pub trait ChainClient: Send + Sync + 'static {
    type Contract: Send + Sync + 'static;
    type Error: Display + Send + Sync + 'static;
    type Proof: Serialize + Send + Sync + 'static;

    async fn query_contract<R: DeserializeOwned + Default>(
        &self,
        contract: &Self::Contract,
        query: String,
    ) -> Result<R, Self::Error>;

    async fn existence_proof(
        &self,
        contract: &Self::Contract,
        storage_key: &str,
    ) -> Result<Self::Proof, Self::Error>;

    async fn wait_for_blocks(&self, blocks: u8) -> Result<Self::Proof, Self::Error>;
}
