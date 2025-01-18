use cosmwasm_std::HexBinary;
use cw_storage_plus::Map;

pub const PINGS_KEY: &str = "pings";

// Maps pubkeys (String representation of HexBinary) to messages (HexBinary representaton of encrypted data)
// The message that a pubkey maps to is encrypted either to that pubkey or the enclave's pubkey
pub const PINGS: Map<String, HexBinary> = Map::new(PINGS_KEY);
