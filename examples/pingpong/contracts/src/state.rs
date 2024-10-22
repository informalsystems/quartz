use cosmwasm_std::HexBinary;
use cw_storage_plus::Map;

// Maps pubkeys (String representation of HexBinary) to messages (HexBinary representaton of encrypted data)
// The message that a pubkey maps to is encrypted either to that pubkey or the enclave's pubkey
pub const PINGS: Map<String, HexBinary> = Map::new("pings");
