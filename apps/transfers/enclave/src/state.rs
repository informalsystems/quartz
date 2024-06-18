use std::HashMap;
use cosmwasm_std::{Addr, Uint128};

pub let state: HashMap<Addr, Uint128> = HashMap::default();

pub struct State {
    state: HashMap<Addr, Uint128>,
}

pub struct RawState {
    state: HashMap<Addr, Uint128>
}