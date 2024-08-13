use std::{
    env,
    path::{Path, PathBuf},
};

use cosmrs::tendermint::chain::Id as ChainId;
use serde::{Deserialize, Serialize};
use tokio::fs;

use crate::error::Error;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Enable mock SGX mode for testing purposes.
    /// This flag disables the use of an Intel SGX processor and allows the system to run without remote attestations.
    #[serde(default)]
    pub mock_sgx: bool,

    /// Name or address of private key with which to sign
    #[serde(default = "default_admin")]
    pub tx_sender: String,

    /// The network chain ID
    #[serde(default = "default_chain_id")]
    pub chain_id: ChainId,

    /// <host>:<port> to tendermint rpc interface for this chain
    #[serde(default = "default_node_url")]
    pub node_url: String,

    /// RPC interface for the Quartz enclave
    #[serde(default = "default_rpc_addr")]
    pub enclave_rpc_addr: String,

    /// Port enclave is listening on
    #[serde(default = "default_port")]
    pub enclave_rpc_port: u16,    

    /// Path to Quartz app directory
    /// Defaults to current working dir
    #[serde(default = "default_app_dir")]
    pub app_dir: PathBuf,
}

fn default_rpc_addr() -> String {
    env::var("RPC_URL").unwrap_or_else(|_| "http://127.0.0.1".to_string())
}

fn default_node_url() -> String {
    env::var("NODE_URL").unwrap_or_else(|_| "http://127.0.0.1:26657".to_string())
}

fn default_admin() -> String {
    String::from("admin")
}

fn default_chain_id() -> ChainId {
    "testing".parse().expect("default chain_id failed")
}

fn default_port() -> u16 {
    11090
}

fn default_app_dir() -> PathBuf {
    ".".parse().expect("default app_dir pathbuf failed")
}

impl Default for Config {
    fn default() -> Self {
        Config {
            mock_sgx: false,
            tx_sender: default_admin(),
            chain_id: default_chain_id(),
            node_url: default_node_url(),
            enclave_rpc_addr: default_rpc_addr(),
            enclave_rpc_port: default_port(),
            app_dir: default_app_dir(),
        }
    }
}

pub async fn load_config(app_dir: &Path, write: bool) -> Result<Config, Error> {
    let config_path = app_dir.join("quartz.toml");
    if config_path.exists() {
        let config_str = fs::read_to_string(config_path)
            .await
            .expect("Failed to read TOML file");

        return Ok(toml::from_str(&config_str).expect("Failed to deserialize TOML"));
    }

    let config = Config::default();

    if write {
        fs::write(
            config_path,
            &toml::to_string_pretty(&config)
                .map_err(|e| Error::GenericErr(e.to_string()))?
                .as_bytes(),
        )
        .await
        .map_err(|e| Error::GenericErr(e.to_string()))?;
    }

    Ok(config)
}
