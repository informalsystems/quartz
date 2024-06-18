use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary};
use cw_storage_plus::Item;

pub const STATE: Item<HexBinary> = Item::new("massive");
pub const REQUESTS: Item<Vec<Request>> = Item::new("requests");

pub const DENOM: Item<String> = Item::new("donation_denom");

#[cw_serde]
pub enum Request {
    Ciphertext(HexBinary),
    Deposit(Addr, u128),
    Withdraw(Addr),
}
