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
    // quartz initialization
    Quartz(QuartzExecuteMsg),

    // ----- user txs
    // clear text
    Deposit,
    Withdraw,
    ClearTextTransferRequest(execute::ClearTextTransferRequestMsg),

    // ciphertext
    TransferRequest(execute::TransferRequestMsg),
    QueryRequest(execute::QueryRequestMsg),
    // ---- end user txs


    // msgs sent by the enclave
    Update(RawAttested<execute::RawUpdateMsg, RawEpidAttestation>),
    QueryResponse(execute::QueryResponseMsg),
}

pub mod execute {
    use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};
    use quartz_cw::{
        error::Error,
        handler::Handler,
        msg::{execute::attested::HasUserData, HasDomainType},
        state::UserData,
    };
    use sha2::{Digest, Sha256};
    use super::*;

    #[cw_serde]
    pub struct ClearTextTransferRequestMsg {
        pub sender: Addr,
        pub receiver: Addr,
        pub amount: Uint128,
        // pub proof: π
    }

    #[cw_serde]
    pub struct QueryRequestMsg {}

    #[cw_serde]
    pub struct TransferRequestMsg {
        pub ciphertext: HexBinary,
        pub digest: HexBinary,
        // pub proof: π
    }

    // Ciphertext of a transfer request
    #[cw_serde]
    pub enum Request {
        Transfer(HexBinary),
        Withdraw(Addr),
        Deposit(Addr, Uint128),
    }

    #[cw_serde]
    pub struct RawUpdateMsg {
        pub ciphertext: HexBinary,
        pub quantity: u32,
        pub withdrawals: Vec<(Addr, Uint128)>,
        // pub proof: π
    }


    #[derive(Clone, Debug, PartialEq)]
    pub struct UpdateMsg(pub RawUpdateMsg);

    impl HasUserData for UpdateMsg {
        fn user_data(&self) -> UserData {
            let mut hasher = Sha256::new();
            hasher.update(serde_json::to_string(&self.0).expect("infallible serializer"));
            let digest: [u8; 32] = hasher.finalize().into();

            let mut user_data = [0u8; 64];
            user_data[0..32].copy_from_slice(&digest);
            user_data
        }
    }

    impl HasDomainType for RawUpdateMsg {
        type DomainType = UpdateMsg;
    }

    impl TryFrom<RawUpdateMsg> for UpdateMsg {
        type Error = StdError;

        fn try_from(value: RawUpdateMsg) -> Result<Self, Self::Error> {
            Ok(Self(value))
        }
    }

    impl From<UpdateMsg> for RawUpdateMsg {
        fn from(value: UpdateMsg) -> Self {
            value.0
        }
    }

    impl Handler for UpdateMsg {
        fn handle(
            self,
            _deps: DepsMut<'_>,
            _env: &Env,
            _info: &MessageInfo,
        ) -> Result<Response, Error> {
            // basically handle `transfer_request` here
            Ok(Response::default())
        }
    }

    #[cw_serde]
    pub struct RawQueryResponseMsg {
        pub address: Addr,
        pub encrypted_bal: HexBinary,
    }

    #[cw_serde]
    pub struct QueryResponseMsg {
        pub address: Addr,
        pub encrypted_bal: HexBinary,
        // pub proof: π
    }

    // #[derive(Clone, Debug, PartialEq)]
    // pub struct QueryResponseMsg(pub RawQueryResponseMsg);


    // impl HasUserData for QueryResponseMsg {
    //     fn user_data(&self) -> UserData {
    //         let mut hasher = Sha256::new();
    //         hasher.update(serde_json::to_string(&self.0).expect("infallible serializer"));
    //         let digest: [u8; 32] = hasher.finalize().into();

    //         let mut user_data = [0u8; 64];
    //         user_data[0..32].copy_from_slice(&digest);
    //         user_data
    //     }
    // }

    // impl HasDomainType for RawQueryResponseMsg {
    //     type DomainType = QueryResponseMsg;
    // }

    // impl TryFrom<RawQueryResponseMsg> for QueryResponseMsg {
    //     type Error = StdError;

    //     fn try_from(value: RawQueryResponseMsg) -> Result<Self, Self::Error> {
    //         Ok(Self(value))
    //     }
    // }

    // impl From<QueryResponseMsg> for RawQueryResponseMsg {
    //     fn from(value: QueryResponseMsg) -> Self {
    //         value.0
    //     }
    // }

    // impl Handler for QueryResponseMsg {
    //     fn handle(
    //         self,
    //         _deps: DepsMut<'_>,
    //         _env: &Env,
    //         _info: &MessageInfo,
    //     ) -> Result<Response, Error> {
    //         // basically handle `transfer_request` here
    //         Ok(Response::default())
    //     }
    // }


}
