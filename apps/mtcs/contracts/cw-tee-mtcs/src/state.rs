use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary, StdError, Storage, Uint128};
use cw_storage_plus::{Item, Map};
use quartz_cw::state::EPOCH_COUNTER;

pub type RawHash = HexBinary;
pub type RawCipherText = HexBinary;

pub type ObligationsItem<'a> = Item<'a, BTreeMap<RawHash, RawCipherText>>;
pub type SetoffsItem<'a> = Item<'a, BTreeMap<RawHash, SettleOff>>;

#[cw_serde]
pub struct State {
    pub owner: String,
}

#[cw_serde]
pub struct Transfer {
    pub payer: String,
    pub payee: String,
    pub amount: (String, Uint128),
}

#[cw_serde]
#[serde(untagged)]
pub enum SettleOff {
    SetOff(Vec<RawCipherText>),
    Transfer(Transfer),
}

#[cw_serde]
pub enum LiquiditySourceType {
    Escrow,
    Overdraft,
    External,
}

#[cw_serde]
pub struct LiquiditySource {
    pub address: Addr,
    pub source_type: LiquiditySourceType,
}


pub const STATE: Item<State> = Item::new("state");
pub const OBLIGATIONS_KEY: &str = "obligations";
pub const SETOFFS_KEY: &str = "setoffs";
pub const LIQUIDITY_SOURCES_KEY: &str = "epoch_liquidity_sources";
pub const LIQUIDITY_SOURCES: Map<&str, LiquiditySource> = Map::new("liquidity_sources");

pub fn current_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
    epoch_key(key, EPOCH_COUNTER.load(storage)?)
}

pub fn previous_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
    epoch_key(key, EPOCH_COUNTER.load(storage)? - 1)
}

pub fn epoch_key(key: &str, epoch: usize) -> Result<String, StdError> {
    Ok(format!("{}/{key}", epoch))
}