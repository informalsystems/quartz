use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary, Uint128};
use quartz_cw::{
    msg::execute::attested::{RawAttested, RawEpidAttestation},
    prelude::*,
};

#[cw_serde]
pub struct InstantiateMsg {
    pub quartz: QuartzInstantiateMsg,
    pub denom: String,
}

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg {
    Quartz(QuartzExecuteMsg),

    // clear text deposit/withdraw
    Deposit,
    Withdraw,

    // ciphertext transfer and result
    TransferRequest(RawAttested<execute::TransferRequestMsg, RawEpidAttestation>),
    Update(execute::UpdateMsg),
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
        pub quantity: u32,
        pub withdrawals: BTreeMap<Addr, Uint128>,
        // pub proof: π
    }
}
