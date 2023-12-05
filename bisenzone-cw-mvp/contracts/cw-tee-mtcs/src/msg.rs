use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg;

#[cw_serde]
pub enum ExecuteMsg {
    JoinComputeNode(execute::JoinComputeNodeMsg),
}

pub mod execute {
    use super::*;

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
    #[returns(query::GetRequestsResponse)]
    GetRequests {},
}

pub mod query {
    use super::*;

    use crate::state::{RawNonce, Request};

    #[cw_serde]
    pub struct GetRequestsResponse {
        pub requests: Vec<(RawNonce, Request)>,
    }
}
