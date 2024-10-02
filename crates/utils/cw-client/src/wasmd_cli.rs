use std::process::Command;

use anyhow::anyhow;
use cosmrs::{tendermint::chain::Id, AccountId};
use reqwest::Url;
use serde::de::DeserializeOwned;

use crate::CwClient;

#[derive(Clone, Debug)]
pub struct CliClient {
    bin: String,
    url: Url,
    gas_price: String,
}

impl CliClient {
    pub fn new(bin: String, url: Url, gas_price: String) -> Self {
        Self {
            bin,
            url,
            gas_price,
        }
    }

    pub fn wasmd(url: Url) -> Self {
        Self {
            bin: "wasmd".to_string(),
            url,
            gas_price: "0.0025ucosm".to_string(),
        }
    }

    pub fn neutrond(url: Url) -> Self {
        Self {
            bin: "neutrond".to_string(),
            url,
            gas_price: "0.0053untrn".to_string(),
        }
    }

    fn new_command(&self) -> Command {
        Command::new(self.bin.as_str())
    }
}

#[async_trait::async_trait]
impl CwClient for CliClient {
    type Address = AccountId;
    type Query = serde_json::Value;
    type RawQuery = String;
    type ChainId = Id;
    type Error = anyhow::Error;

    async fn query_smart<R: DeserializeOwned + Send>(
        &self,
        contract: &Self::Address,
        query: Self::Query,
    ) -> Result<R, Self::Error> {
        let mut command = self.new_command();
        let command = command
            .args(["--node", self.url.as_str()])
            .args(["query", "wasm"])
            .args(["contract-state", "smart", contract.as_ref()])
            .arg(query.to_string())
            .args(["--output", "json"]);

        let output = command.output()?;
        if !output.status.success() {
            return Err(anyhow!("{:?}", output));
        }

        let query_result: R = serde_json::from_slice(&output.stdout)
            .map_err(|e| anyhow!("Error deserializing: {}", e))?;
        Ok(query_result)
    }

    fn query_raw<R: DeserializeOwned + Default>(
        &self,
        contract: &Self::Address,
        query: Self::RawQuery,
    ) -> Result<R, Self::Error> {
        let mut command = self.new_command();
        let command = command
            .args(["--node", self.url.as_str()])
            .args(["query", "wasm"])
            .args(["contract-state", "raw", contract.as_ref()])
            .arg(&query)
            .args(["--output", "json"]);

        let output = command.output()?;
        if !output.status.success() {
            return Err(anyhow!("{:?}", output));
        }

        let query_result: R = serde_json::from_slice(&output.stdout).unwrap_or_default();
        Ok(query_result)
    }

    fn query_tx<R: DeserializeOwned + Default>(&self, txhash: &str) -> Result<R, Self::Error> {
        let mut command = self.new_command();
        let command = command
            .args(["--node", self.url.as_str()])
            .args(["query", "tx"])
            .arg(txhash)
            .args(["--output", "json"]);

        let output = command.output()?;
        if !output.status.success() {
            return Err(anyhow!("{:?}", output));
        }

        let query_result: R = serde_json::from_slice(&output.stdout).unwrap_or_default();
        Ok(query_result)
    }

    async fn tx_execute<M: ToString + Send>(
        &self,
        contract: &Self::Address,
        chain_id: &Id,
        gas: u64,
        sender: &str,
        msg: M,
    ) -> Result<String, Self::Error> {
        let mut command = self.new_command();
        let command = command
            .args(["--node", self.url.as_str()])
            .args(["--chain-id", chain_id.as_ref()])
            .args(["tx", "wasm"])
            .args(["execute", contract.as_ref(), &msg.to_string()])
            .args(["--gas", &gas.to_string()])
            .args(["--from", sender])
            .args(["--output", "json"])
            .arg("-y");

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow!("{:?}", output));
        }

        // TODO: find the rust type for the tx output and return that
        Ok((String::from_utf8(output.stdout)?).to_string())
    }

    fn deploy<M: ToString>(
        &self,
        chain_id: &Id,
        sender: &str,
        wasm_path: M,
    ) -> Result<String, Self::Error> {
        let mut command = self.new_command();
        let command = command
            .args(["--node", self.url.as_str()])
            .args(["tx", "wasm", "store", &wasm_path.to_string()])
            .args(["--from", sender])
            .args(["--chain-id", chain_id.as_ref()])
            .args(["--gas-prices", &self.gas_price])
            .args(["--gas", "auto"])
            .args(["--gas-adjustment", "1.3"])
            .args(["-o", "json"])
            .arg("-y");

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow!("{:?}", output));
        }

        // TODO: find the rust type for the tx output and return that
        Ok((String::from_utf8(output.stdout)?).to_string())
    }

    fn init<M: ToString>(
        &self,
        chain_id: &Id,
        sender: &str,
        code_id: u64,
        init_msg: M,
        label: &str,
    ) -> Result<String, Self::Error> {
        let mut command = self.new_command();
        let command = command
            .args(["--node", self.url.as_str()])
            .args(["tx", "wasm", "instantiate"])
            .args([&code_id.to_string(), &init_msg.to_string()])
            .args(["--label", label])
            .args(["--from", sender])
            .arg("--no-admin")
            .args(["--chain-id", chain_id.as_ref()])
            .args(["--gas-prices", &self.gas_price])
            .args(["--gas", "auto"])
            .args(["--gas-adjustment", "1.3"])
            .args(["-o", "json"])
            .arg("-y");

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow!("{:?}", output));
        }

        // TODO: find the rust type for the tx output and return that
        Ok((String::from_utf8(output.stdout)?).to_string())
    }

    fn trusted_height_hash(&self) -> Result<(u64, String), Self::Error> {
        let mut command = self.new_command();
        let command = command.args(["--node", self.url.as_str()]).arg("status");

        let output = command.output()?;

        if !output.status.success() {
            return Err(anyhow!("{:?}", output));
        }

        let query_result: serde_json::Value =
            serde_json::from_slice(&output.stdout).unwrap_or_default();

        let trusted_height = query_result["SyncInfo"]["latest_block_height"]
            .as_str()
            .ok_or(anyhow!("Could not query height"))?;

        let trusted_height = trusted_height.parse::<u64>()?;

        let trusted_hash = query_result["SyncInfo"]["latest_block_hash"]
            .as_str()
            .ok_or(anyhow!("Could not query height"))?
            .to_string();

        Ok((trusted_height, trusted_hash))
    }
}
