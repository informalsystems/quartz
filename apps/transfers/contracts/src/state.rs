use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary};
use cw_storage_plus::Item;

pub const STATE: Item<HexBinary> = Item::new("state");
pub const REQUESTS: Item<Vec<Request>> = Item::new("requests");

pub const DENOM: Item<String> = Item::new("denom");

#[cw_serde]
pub enum Request {
    Transfer(HexBinary),
    Deposit(Addr, u128),
    Withdraw(Addr),
}
