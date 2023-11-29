use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

#[cw_serde]
pub struct State {
    pub owner: String,
}

pub const STATE: Item<State> = Item::new("state");
