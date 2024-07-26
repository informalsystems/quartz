use std::{cmp::Ordering, collections::BTreeMap};

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary, StdError, Storage, Uint128, Uint64};
use cw_storage_plus::{Item, Map};
use quartz_common::contract::state::EPOCH_COUNTER;

pub type RawHash = HexBinary;
pub type RawCipherText = HexBinary;

pub type ObligationsItem = Item<BTreeMap<RawHash, RawCipherText>>;
pub type SetoffsItem = Item<BTreeMap<RawHash, SettleOff>>;

#[cw_serde]
pub struct State {
    pub owner: String,
}

#[cw_serde]
pub struct Transfer {
    pub payer: Addr,
    pub payee: Addr,
    pub amount: (String, Uint128),
}

#[cw_serde]
#[serde(untagged)]
pub enum SettleOff {
    SetOff(Vec<RawCipherText>),
    Transfer(Transfer),
}

#[cw_serde]
#[derive(Copy)]
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

impl std::cmp::Ord for LiquiditySource {
    fn cmp(&self, other: &Self) -> Ordering {
        self.address.cmp(&other.address)
    }
}

impl PartialOrd for LiquiditySource {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.address.cmp(&other.address))
    }
}

// PartialEq implemented in #[cw_serde]
impl Eq for LiquiditySource {}

pub const STATE: Item<State> = Item::new("state");
pub const OBLIGATIONS_KEY: &str = "obligations";
pub const SETOFFS_KEY: &str = "setoffs";
pub const LIQUIDITY_SOURCES_KEY: &str = "epoch_liquidity_sources";
pub const LIQUIDITY_SOURCES: Map<&str, Vec<LiquiditySource>> = Map::new("liquidity_sources");

pub fn current_epoch_key(key: &str, storage: &dyn Storage) -> Result<String, StdError> {
    epoch_key(key, EPOCH_COUNTER.load(storage)?)
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