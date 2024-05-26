use std::{error::Error, process::Command};

use cosmrs::{tendermint::chain::Id, AccountId};
use reqwest::Url;
use serde::{Deserialize, Serialize};

pub trait WasmdClient {
    type Address: AsRef<str>;
    type Query: ToString;
    type ChainId: AsRef<str>;
    type Error;

    fn query_smart<R: FromVec>(
        &self,
        contract: &Self::Address,
        query: Self::Query,
        chain_id: &Id,
    ) -> Result<R, Self::Error>;

    fn tx_execute<M: ToString>(
        &self,
        contract: &Self::Address,
        chain_id: &Id,
        gas: u64,
        sender: String,
        msg: M,
    ) -> Result<(), Self::Error>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct QueryResult<T> {
    pub data: T,
}

pub trait FromVec: Sized {
    fn from_vec(value: Vec<u8>) -> Self;
}

impl<T: for<'any> Deserialize<'any>> FromVec for T {
    fn from_vec(value: Vec<u8>) -> Self {
        serde_json::from_slice(&value).unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct CliWasmdClient {
    url: Url,
}

impl CliWasmdClient {
    pub fn new(url: Url) -> Self {
        Self { url }
    }
}

impl WasmdClient for CliWasmdClient {
    type Address = AccountId;
    type Query = serde_json::Value;
    type ChainId = Id;
    type Error = Box<dyn Error>;

    fn query_smart<R: FromVec>(
        &self,
        contract: &Self::Address,
        query: Self::Query,
        chain_id: &Id,
    ) -> Result<R, Self::Error> {
        let mut wasmd = Command::new("wasmd");
        let command = wasmd
            .args(["--node", self.url.as_str()])
            .args(["query", "wasm"])
            .args(["contract-state", "smart", contract.as_ref()])
            .arg(query.to_string())
            .args(["--chain-id", chain_id.as_ref()])
            .args(["--output", "json"]);

        let output = command.output()?;
        println!("{:?} => {:?}", command, output);

        let query_result = R::from_vec(output.stdout);
        Ok(query_result)
    }

    fn tx_execute<M: ToString>(
        &self,
        contract: &Self::Address,
        chain_id: &Id,
        gas: u64,
        sender: String,
        msg: M,
    ) -> Result<(), Self::Error> {
        let mut wasmd = Command::new("wasmd");
        let command = wasmd
            .args(["--node", self.url.as_str()])
            .args(["tx", "wasm"])
            .args(["execute", contract.as_ref(), &msg.to_string()])
            .args(["--chain-id", chain_id.as_ref()])
            .args(["--gas", &gas.to_string()])
            .args(["--from", sender.as_ref()])
            .arg("-y");

        let output = command.output()?;
        println!("{:?} => {:?}", command, output);

        Ok(())
    }
}
