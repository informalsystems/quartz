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
    TransferRequest(RawAttested<execute::RawTransferRequestMsg, RawEpidAttestation>),
    Update(execute::UpdateMsg),
}

pub mod execute {
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
    use quartz_cw::error::Error;
    use quartz_cw::handler::Handler;
    use quartz_cw::msg::execute::attested::HasUserData;
    use quartz_cw::msg::HasDomainType;
    use quartz_cw::state::UserData;
    use sha2::{Digest, Sha256};

    use super::*;

    #[cw_serde]
    pub struct RawTransferRequestMsg {
        pub ciphertext: HexBinary,
        pub digest: HexBinary,
        // pub proof: π
    }

    #[derive(Clone, Debug, PartialEq)]
    pub struct TransferRequestMsg(pub RawTransferRequestMsg);

    impl HasUserData for TransferRequestMsg {
        fn user_data(&self) -> UserData {
            let mut hasher = Sha256::new();
            hasher.update(
                serde_json::to_string(&self.0)
                    .expect("infallible serializer"),
            );
            let digest: [u8; 32] = hasher.finalize().into();

            let mut user_data = [0u8; 64];
            user_data[0..32].copy_from_slice(&digest);
            user_data
        }
    }

    impl HasDomainType for RawTransferRequestMsg {
        type DomainType = TransferRequestMsg;
    }

    impl TryFrom<RawTransferRequestMsg> for TransferRequestMsg {
        type Error = StdError;

        fn try_from(value: RawTransferRequestMsg) -> Result<Self, Self::Error> {
            Ok(Self(value))
        }
    }

    impl From<TransferRequestMsg> for RawTransferRequestMsg {
        fn from(value: TransferRequestMsg) -> Self {
            value.0
        }
    }

    impl Handler for TransferRequestMsg {
        fn handle(self, _deps: DepsMut<'_>, _env: &Env, _info: &MessageInfo) -> Result<Response, Error> {
            // basically handle `transfer_request` here
            Ok(Response::default())
        }
    }

    #[cw_serde]
    pub struct UpdateMsg {
        pub ciphertext: HexBinary,
        pub quantity: u32,
        pub withdrawals: BTreeMap<Addr, Uint128>,
        // pub proof: π
    }
}

