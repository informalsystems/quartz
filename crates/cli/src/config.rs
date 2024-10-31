use std::path::PathBuf;

use cosmrs::{tendermint::chain::Id as ChainId, AccountId};
use quartz_common::enclave::types::Fmspc;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// Enable mock SGX mode for testing purposes.
    /// This flag disables the use of an Intel SGX processor and allows the system to run without remote attestations.
    #[serde(default)]
    pub mock_sgx: bool,

    /// SGX-specific configuration
    #[serde(default)]
    pub sgx_config: SgxConfiguration,

    /// Name or address of private key with which to sign
    #[serde(default = "default_tx_sender")]
    pub tx_sender: String,

    /// The network chain ID
    #[serde(default = "default_chain_id")]
    pub chain_id: ChainId,

    /// <host>:<port> to tendermint rpc interface for this chain
    #[serde(default = "default_node_url")]
    #[serde_as(as = "DisplayFromStr")]
    pub node_url: Url,

    /// websocket URL
    #[serde(default = "default_ws_url")]
    #[serde_as(as = "DisplayFromStr")]
    pub ws_url: Url,

    /// gRPC URL
    #[serde(default = "default_grpc_url")]
    #[serde_as(as = "DisplayFromStr")]
    pub grpc_url: Url,

    /// RPC interface for the Quartz enclave
    #[serde(default = "default_rpc_addr")]
    pub enclave_rpc_addr: String,

    /// Port enclave is listening on
    #[serde(default = "default_port")]
    pub enclave_rpc_port: u16,

    /// Path to Quartz app directory.
    /// Defaults to current working dir
    #[serde(default = "default_app_dir", skip_serializing)]
    pub app_dir: PathBuf,

    /// Trusted height for light client proofs
    #[serde(default)]
    pub trusted_height: u64,

    /// Trusted hash for block at trusted_height for light client proofs
    #[serde(default)]
    pub trusted_hash: String,

    /// Whether to build for release or debug
    #[serde(default)]
    pub release: bool,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SgxConfiguration {
    /// FMSPC (Family-Model-Stepping-Platform-Custom SKU) as hex string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fmspc: Option<String>,

    /// Address of the TcbInfo contract
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tcbinfo_contract: Option<AccountId>,

    /// Address of the DCAP verifier contract
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dcap_verifier_contract: Option<AccountId>,

    /// Internal reference to parent mock_sgx setting
    #[serde(skip)]
    mock_sgx: bool,
}

impl Default for SgxConfiguration {
    fn default() -> Self {
        Self::new(false)
    }
}

impl SgxConfiguration {
    pub fn new(mock_sgx: bool) -> Self {
        Self {
            fmspc: None,
            tcbinfo_contract: None,
            dcap_verifier_contract: None,
            mock_sgx,
        }
    }

       pub fn validate(&self) -> Result<(), String> {
        if !self.mock_sgx {
            self.check_required_field(&self.fmspc, "FMSPC")?;
            self.check_required_field(&self.tcbinfo_contract, "tcbinfo_contract")?;
            self.check_required_field(&self.dcap_verifier_contract, "dcap_verifier_contract")?;
        }
        Ok(())
    }

    pub fn get_fmspc(&self) -> Result<quartz_common::enclave::types::Fmspc, String> {
        let fmspc_hex = self.fmspc.as_ref()
            .ok_or_else(|| "FMSPC is required".to_string())?;
        
        let fmspc_bytes = hex::decode(fmspc_hex)
            .map_err(|e| format!("Invalid FMSPC hex string: {}", e))?;
        
        if fmspc_bytes.len() != 6 {
            return Err("FMSPC must be 6 bytes long".to_string());
        }

        let fmspc: [u8; 6] = fmspc_bytes.try_into()
            .map_err(|_| "Failed to convert FMSPC bytes to array".to_string())?;

        Ok(quartz_common::enclave::types::Fmspc(fmspc))
    }


    

    fn check_required_field<T>(&self, field: &Option<T>, field_name: &str) -> Result<(), String> {
        if field.is_none() {
            return Err(format!(
                "{} is required when not in mock SGX mode",
                field_name
            ));
        }
        Ok(())
    }
}

fn default_rpc_addr() -> String {
    "http://127.0.0.1".to_string()
}

fn default_node_url() -> Url {
    "http://127.0.0.1:26657"
        .parse()
        .expect("valid hardcoded URL")
}

fn default_ws_url() -> Url {
    "ws://127.0.0.1/websocket"
        .parse()
        .expect("valid hardcoded URL")
}

fn default_grpc_url() -> Url {
    "http://127.0.0.1:9090"
        .parse()
        .expect("valid hardcoded URL")
}

fn default_tx_sender() -> String {
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
        let mock_sgx = false;
        Config {
            mock_sgx,
            sgx_config: SgxConfiguration::new(mock_sgx),
            tx_sender: default_tx_sender(),
            chain_id: default_chain_id(),
            node_url: default_node_url(),
            ws_url: default_ws_url(),
            grpc_url: default_grpc_url(),
            enclave_rpc_addr: default_rpc_addr(),
            enclave_rpc_port: default_port(),
            app_dir: default_app_dir(),
            trusted_height: u64::default(),
            trusted_hash: String::default(),
            release: false,
        }
    }
}

impl AsRef<Config> for Config {
    fn as_ref(&self) -> &Config {
        self
    }
}

impl Config {
    pub fn enclave_rpc(&self) -> String {
        format!("{}:{}", self.enclave_rpc_addr, self.enclave_rpc_port)
    }

    pub fn set_mock_sgx(&mut self, mock_sgx: bool) {
        self.mock_sgx = mock_sgx;
        self.sgx_config = SgxConfiguration::new(mock_sgx);
    }
}
