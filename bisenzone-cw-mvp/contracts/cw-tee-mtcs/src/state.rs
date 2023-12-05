use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

pub type RawNonce = String;
pub type RawPublicKey = String;
pub type RawAddress = String;

#[cw_serde]
pub struct State {
    pub owner: String,
}

#[cw_serde]
pub enum Request {
    JoinComputeNode((RawPublicKey, RawAddress)),
}

pub const STATE: Item<State> = Item::new("state");
pub const REQUESTS: Map<&RawNonce, Request> = Map::new("requests");
