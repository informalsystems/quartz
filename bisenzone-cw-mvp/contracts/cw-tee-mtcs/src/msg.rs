use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg;

#[cw_serde]
pub enum ExecuteMsg {
    BootstrapKeyManager(execute::BootstrapKeyManagerMsg),
    RegisterEpochKey(execute::RegisterEpochKeyMsg),
    JoinComputeNode(execute::JoinComputeNodeMsg),
}

pub mod execute {
    use super::*;

    #[cw_serde]
    pub struct BootstrapKeyManagerMsg {
        pub compute_mrenclave: String,
        pub key_manager_mrenclave: String,
        pub tcb_info: String,
    }

    #[cw_serde]
    pub struct RegisterEpochKeyMsg {
        pub epoch_key: String,
    }

    #[cw_serde]
    pub struct JoinComputeNodeMsg {
        pub io_exchange_key: String,
        pub address: String,
        pub nonce: String,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(query::GetSgxStateResponse)]
    GetSgxState {},
    #[returns(query::GetRequestsResponse)]
    GetRequests {},
}

pub mod query {
    use super::*;

    use crate::state::{RawMrenclave, RawNonce, Request};

    #[cw_serde]
    pub struct GetSgxStateResponse {
        pub compute_mrenclave: RawMrenclave,
        pub key_manager_mrenclave: RawMrenclave,
    }

    #[cw_serde]
    pub struct GetRequestsResponse {
        pub requests: Vec<(RawNonce, Request)>,
    }
}
