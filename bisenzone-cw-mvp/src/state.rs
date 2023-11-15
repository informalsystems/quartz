use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_storage_plus::{Item, Map};

#[cw_serde]
pub struct State {
    pub owner: String,
}

pub const STATE: Item<State> = Item::new("state");
pub const UTILIZATION: Map<(&Addr, &Addr), Uint128> = Map::new("utilization");
