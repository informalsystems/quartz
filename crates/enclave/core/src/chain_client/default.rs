use cosmrs::{crypto::secp256k1::SigningKey, AccountId};
use cw_client::{CwClient, GrpcClient};
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::chain_client::ChainClient;

pub struct DefaultChainClient {
    grpc_client: GrpcClient,
}

impl DefaultChainClient {
    pub fn new(signer: SigningKey, url: Url) -> Self {
        DefaultChainClient {
            grpc_client: GrpcClient::new(signer, url),
        }
    }
}

#[async_trait::async_trait]
impl ChainClient for DefaultChainClient {
    type Contract = AccountId;
    type Error = anyhow::Error;
    type Proof = ();

    async fn query_contract<R: DeserializeOwned + Default>(
        &self,
        contract: &Self::Contract,
        query: String,
    ) -> Result<R, Self::Error> {
        self.grpc_client
            .query_raw(&contract, query.to_string())
            .await
    }

    async fn existence_proof(
        &self,
        _contract: &Self::Contract,
        _storage_key: &str,
    ) -> Result<Self::Proof, Self::Error> {
        todo!()
    }

    async fn wait_for_blocks(&self, _blocks: u8) -> Result<Self::Proof, Self::Error> {
        todo!()
    }
}
