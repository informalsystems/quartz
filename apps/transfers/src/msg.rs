use std::collections::BTreeMap;

use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, HexBinary};
use quartz_cw::prelude::*;

use crate::state::{RawHash, SettleOff};

#[cw_serde]
pub struct InstantiateMsg(pub QuartzInstantiateMsg);

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    Quartz(QuartzExecuteMsg),

    // clear text deposit/withdraw
    Deposit {},
    Withdraw {},

    // ciphertext transfer and result
    TransferRequest(TransferRequestMsg),
    Update(UpdateMsg),
}
pub mod execute {
    use super::*;

    #[cw_serde]
    pub struct TransferRequestMsg {
        pub ciphertext: HexBinary,
        pub digest: HexBinary,
        // pub proof: π
    }

    #[cw_serde]
    pub struct UpdateMsg {
        pub ciphertext: HexBinary,
        pub quantity: u128,
        pub withdrawals: Map<Addr, u128>,
        // pub proof: π
    }

}
