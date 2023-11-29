use crate::msg::execute::{JoinComputeNodeMsg, Nonce, ShareEpochKeyMsg};
use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct State {
    pub owner: String,
}

#[cw_serde]
pub enum Request {
    JoinComputeNode(JoinComputeNodeMsg),
    ShareEpochKey(ShareEpochKeyMsg),
}

pub const STATE: Item<State> = Item::new("state");
pub const REQUESTS: Map<&Nonce, &Request> = Map::new("requests");
