use cosmwasm_schema::cw_serde;
use cw_storage_plus::Item;

pub type RawNonce = String;
pub type RawPublicKey = String;
pub type RawAddress = String;
pub type RawMrenclave = String;
pub type RawTcbInfo = String;

pub type Mrenclave = [u8; 32];

#[cw_serde]
pub struct State {
    pub owner: String,
}

#[cw_serde]
pub enum Request {
    JoinComputeNode((RawPublicKey, RawAddress)),
}

#[cw_serde]
pub struct SgxState {
    pub compute_mrenclave: RawMrenclave,
    pub key_manager_mrenclave: RawMrenclave,
    pub tcb_info: RawTcbInfo,
}

#[cw_serde]
pub struct EpochState {
    pub epoch_key: RawPublicKey,
}

pub const STATE: Item<State> = Item::new("state");
pub const REQUESTS: Item<Vec<(RawNonce, Request)>> = Item::new("requests");
pub const SGX_STATE: Item<SgxState> = Item::new("sgx_state");
pub const EPOCH_STATE: Item<EpochState> = Item::new("epoch_state");
