use cosmwasm_schema::cw_serde;
use quartz_cw::{
    msg::execute::attested::{RawAttested, RawAttestedMsgSansHandler, RawDefaultAttestation},
    prelude::*,
};
use serde::{Deserialize, Serialize};

#[cw_serde]
pub struct InstantiateMsg<RA = RawDefaultAttestation> {
    pub quartz: QuartzInstantiateMsg<RA>,
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetBalance { address: String },
}

type AttestedMsg<M, RA = RawDefaultAttestation> = RawAttested<RawAttestedMsgSansHandler<M>, RA>;

#[cw_serde]
#[allow(clippy::large_enum_variant)]
pub enum ExecuteMsg<RA = RawDefaultAttestation> {
    // quartz initialization
    Quartz(QuartzExecuteMsg),

    // User msgs
    // clear text
    Deposit,
    Withdraw,
    ClearTextTransferRequest(execute::ClearTextTransferRequestMsg),
    // ciphertext
    TransferRequest(execute::TransferRequestMsg),
    QueryRequest(execute::QueryRequestMsg),

    // Enclave msgs
    Update(AttestedMsg<execute::UpdateMsg, RA>),
    QueryResponse(AttestedMsg<execute::QueryResponseMsg, RA>),
}

pub mod execute {
    use cosmwasm_schema::cw_serde;
    use cosmwasm_std::{Addr, HexBinary, Uint128};
    use quartz_cw::{msg::execute::attested::HasUserData, state::UserData};
    use sha2::{Digest, Sha256};

    #[cw_serde]
    pub struct ClearTextTransferRequestMsg {
        pub sender: Addr,
        pub receiver: Addr,
        pub amount: Uint128,
        // pub proof: π
    }

    #[cw_serde]
    pub struct QueryRequestMsg {
        pub emphemeral_pubkey: HexBinary,
    }

    #[cw_serde]
    pub struct TransferRequestMsg {
        pub ciphertext: HexBinary,
        pub digest: HexBinary,
        // pub proof: π
    }

    #[cw_serde]
    pub enum Request {
        Transfer(HexBinary),
        Withdraw(Addr),
        Deposit(Addr, Uint128),
    }

    #[cw_serde]
    pub struct UpdateMsg {
        pub ciphertext: HexBinary,
        pub quantity: u32,
        pub withdrawals: Vec<(Addr, Uint128)>,
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

    // {
    //     "query_response": {
    //         "msg": {
    //             "address": "wasm1vx5jkp8gd4j6krpttgeddplslw9aamuzmxaesw",
    //             "encrypted_bal": "0497ac92d27e7faa7264045e925ed9aab59e495d626e549bb66383647d5b5479f42904ef4b50d806efeb8c1be53d079ac489ac8c948e83e6ae41674203d0753b8fddc600bd44c77f706b8159b21624f45272ca094c1a76e709aab7a3a6b956c9ee189d64a2078886b34940e1442e64d9cde535"
    //         },
    //         "attestation": null
    //     }
    // }

    #[cw_serde]
    pub struct QueryResponseMsg {
        pub address: Addr,
        pub encrypted_bal: HexBinary,
        // pub proof: π
    }

    impl HasUserData for QueryResponseMsg {
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
