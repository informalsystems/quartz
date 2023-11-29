use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

pub type Nonce = [u8; 32];
pub type RawPublicKey = String;

#[cw_serde]
pub struct State {
    pub owner: String,
}

#[cw_serde]
pub enum Request {
    JoinComputeNode(RawPublicKey),
}

pub const STATE: Item<State> = Item::new("state");
pub const REQUESTS: Map<&Nonce, Request> = Map::new("requests");
