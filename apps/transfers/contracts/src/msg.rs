use std::collections::BTreeMap;

use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, HexBinary, Uint128};
use quartz_cw::{
    msg::execute::attested::{RawAttested, RawAttestedMsgSansHandler, RawEpidAttestation},
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
    // quartz initialization
    Quartz(QuartzExecuteMsg),

    // ----- user txs
    // clear text
    Deposit,
    Withdraw,

    // ciphertext
    TransferRequest(execute::TransferRequestMsg),
    // ---- end user txs
    ClearTextTransferRequest(execute::ClearTextTransferRequestMsg),

    // enclave msg
    Update(RawAttested<RawAttestedMsgSansHandler<execute::UpdateMsg>, RawEpidAttestation>),
}

pub mod execute {
    use quartz_cw::{msg::execute::attested::HasUserData, state::UserData};
    use sha2::{Digest, Sha256};

    use super::*;

    #[cw_serde]
    pub struct TransferRequestMsg {
        pub ciphertext: HexBinary,
        pub digest: HexBinary,
        // pub proof: π
    }

    #[cw_serde]
    pub struct ClearTextTransferRequestMsg {
        pub sender: Addr,
        pub receiver: Addr,
        pub amount: Uint128,
        // pub proof: π
    }

    #[cw_serde]
    pub struct UpdateMsg {
        pub ciphertext: HexBinary,
        pub quantity: u32,
        pub withdrawals: BTreeMap<Addr, Uint128>,
        // pub proof: π
    }

    impl HasUserData for UpdateMsg {
        fn user_data(&self) -> UserData {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(&self).expect("infallible serializer"));
            let digest: [u8; 32] = hasher.finalize().into();

            let mut user_data = [0u8; 64];
            user_data[0..32].copy_from_slice(&digest);
            user_data
        }
    }
}
