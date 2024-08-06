use serde::{Deserialize, Serialize};
use tendermint::{abci::Event as TmEvent, Hash};

// Rust libraries don't seem to implement this type from the wasmd go implementation
// TODO: Replace String with types from Rust libraries
// TODO: Move this into WasmdClient
#[derive(Deserialize, Debug, Default)]
pub struct WasmdTxResponse {
    pub height: String,
    pub txhash: Hash,
    pub codespace: String,
    pub code: u32,
    pub data: String,
    pub raw_log: String,
    pub logs: Vec<serde_json::Value>,
    pub info: String,
    pub gas_wanted: String,
    pub gas_used: String,
    pub tx: Option<serde_json::Value>,
    pub timestamp: String,
    pub events: Vec<TmEvent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attribute {
    pub key: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Event {
    pub attributes: Vec<Attribute>,
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Log {
    pub events: Vec<Event>,
    pub msg_index: u32,
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum RelayMessage {
    Instantiate,
    SessionCreate,
    SessionSetPubKey(String),
}

impl std::fmt::Display for RelayMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelayMessage::Instantiate => write!(f, "Instantiate"),
            RelayMessage::SessionCreate => write!(f, "SessionCreate"),
            RelayMessage::SessionSetPubKey(_) => write!(f, "SessionSetPubKey"),
        }
    }
}
