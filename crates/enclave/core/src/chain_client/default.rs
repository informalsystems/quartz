use cosmrs::AccountId;

use crate::chain_client::ChainClient;

#[derive(Clone, Debug, Default)]
pub struct DefaultChainClient;

impl ChainClient for DefaultChainClient {
    const CHAIN_ID: &'static str = "pion-1";
    type Contract = AccountId;
}
