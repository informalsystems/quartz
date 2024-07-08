use std::collections::{BTreeMap, BTreeSet};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{HexBinary, StdError, Storage, Uint64};
use cw_storage_plus::{Item, Map};
use quartz_cw::state::EPOCH_COUNTER;

pub type RawHash = HexBinary;
pub type RawCipherText = HexBinary;

pub type ObligationsItem = Item<BTreeMap<RawHash, RawCipherText>>;
pub type SetoffsItem = Item<BTreeMap<RawHash, SettleOff>>;
pub type LiquiditySourcesItem = Item<BTreeSet<HexBinary>>;

pub const OBLIGATIONS: Map<&str, BTreeMap<RawHash, RawCipherText>> = Map::new("obligations");
pub const SETOFFS: Map<&str, BTreeMap<RawHash, SettleOff>> = Map::new("setoffs");
pub const LIQUIDITY_SOURCES: Map<&str, BTreeSet<HexBinary>> = Map::new("liquidity_sources");

#[cw_serde]
pub struct State {
    pub owner: String,
}

#[cw_serde]
pub struct Transfer {
    pub payer: String,
    pub payee: String,
    pub amount: u64,
}

#[cw_serde]
#[serde(untagged)]
pub enum SettleOff {
    SetOff(Vec<RawCipherText>),
    Transfer(Transfer),
}

pub const STATE: Item<State> = Item::new("state");
pub const OBLIGATIONS_KEY: &str = "obligations";
pub const SETOFFS_KEY: &str = "setoffs";
pub const LIQUIDITY_SOURCES_KEY: &str = "liquidity_sources";

// pub fn current_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
//     epoch_key(key, EPOCH_COUNTER.load(storage)?)
// }

// pub fn previous_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
//     epoch_key(key, EPOCH_COUNTER.load(storage)? - 1)
// }

// pub fn epoch_key(key: &str, epoch: usize) -> Result<String, StdError> {
//     Ok(format!("{}/{key}", epoch))
// }

pub fn current_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
    let epoch = EPOCH_COUNTER.load(storage)?;
    epoch_key(key, epoch.into())
}

pub fn previous_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
    let epoch = EPOCH_COUNTER.load(storage)?;
    if epoch == Uint64::zero() {
        return Err(StdError::generic_err(
            "Cannot get previous epoch for epoch 0",
        ));
    }
    epoch_key(key, epoch - Uint64::new(1))
}

pub fn epoch_key(key: &str, epoch: Uint64) -> Result<String, StdError> {
    Ok(format!("{}/{}", epoch, key))
}
