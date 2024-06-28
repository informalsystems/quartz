use cosmwasm_std::{Addr, Int128};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct IncreaseBalance {
    pub receiver: Addr,
    pub amount: Int128
}

#[cw_serde]
pub struct DecreaseBalance {
    pub receiver: Addr,
    pub amount: Int128
}