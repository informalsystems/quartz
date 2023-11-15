use std::collections::HashMap;

use cosmwasm_std::Addr;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub utilization: HashMap<Addr, HashMap<Addr, u64>>,
    pub owner: Addr,
}

pub const STATE: Item<State> = Item::new("state");
