use std::collections::BTreeMap;

use anyhow;
use cosmwasm_std::{Addr, HexBinary, Uint128};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct State {
    pub state: BTreeMap<Addr, Uint128>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct RawState {
    pub state: BTreeMap<Addr, Uint128>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RawEncryptedState {
    pub ciphertext: HexBinary,
}

impl From<State> for RawState {
    fn from(o: State) -> Self {
        Self { state: o.state }
    }
}

impl TryFrom<RawState> for State {
    type Error = anyhow::Error;

    fn try_from(o: RawState) -> Result<Self, anyhow::Error> {
        Ok(Self { state: o.state })
    }
}
