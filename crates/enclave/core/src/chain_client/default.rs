use cosmrs::{crypto::secp256k1::SigningKey, AccountId};
use cw_client::{CwClient, GrpcClient};
use reqwest::Url;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use tendermint::chain::Id as TmChainId;

use crate::chain_client::ChainClient;

pub struct DefaultChainClient {
    chain_id: TmChainId,
    grpc_client: GrpcClient,
}

impl DefaultChainClient {
    pub fn new(chain_id: TmChainId, signer: SigningKey, url: Url) -> Self {
        DefaultChainClient {
            chain_id,
            grpc_client: GrpcClient::new(signer, url),
        }
    }
}

#[async_trait::async_trait]
impl ChainClient for DefaultChainClient {
    type Contract = AccountId;
    type Error = anyhow::Error;
    type Proof = ();
    type Query = String;
    type TxOutput = String;

    async fn query_contract<R: DeserializeOwned + Default>(
        &self,
        contract: &Self::Contract,
        query: String,
    ) -> Result<R, Self::Error> {
        self.grpc_client
            .query_raw(contract, query.to_string())
            .await
    }

    async fn existence_proof(
        &self,
        _contract: &Self::Contract,
        _storage_key: &str,
    ) -> Result<Self::Proof, Self::Error> {
        todo!()
    }

    async fn send_tx<T: Serialize + Send + Sync>(
        &self,
        contract: &Self::Contract,
        tx: T,
        gas: u64,
        _fees: u128,
    ) -> Result<Self::TxOutput, Self::Error> {
        self.grpc_client
            .tx_execute(contract, &self.chain_id, gas, "", json!(tx), "")
            .await
    }

    async fn wait_for_blocks(&self, _blocks: u8) -> Result<Self::Proof, Self::Error> {
        todo!()
    }
}
