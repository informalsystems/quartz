use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError, Storage};
use cw_storage_plus::Item;
use quartz_cw::state::EPOCH_COUNTER;

pub type RawHash = HexBinary;
pub type RawCipherText = HexBinary;

pub type ObligationsItem<'a> = Item<'a, BTreeMap<RawHash, RawCipherText>>;
pub type SetoffsItem<'a> = Item<'a, BTreeMap<RawHash, RawCipherText>>;

#[cw_serde]
pub struct State {
    pub owner: String,
}

pub const STATE: Item<State> = Item::new("state");
pub const OBLIGATIONS_KEY: &str = "obligations";
pub const SETOFFS_KEY: &str = "setoffs";

pub fn current_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
    Ok(format!("{}/{key}", EPOCH_COUNTER.load(storage)?))
}
