use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg;

#[cw_serde]
pub enum ExecuteMsg {
    JoinComputeNode(execute::JoinComputeNodeMsg),
}

pub mod execute {
    use super::*;

    use crate::state::Nonce;

    #[cw_serde]
    pub struct JoinComputeNodeMsg {
        pub compute_node_pub_key: String,
        pub nonce: Nonce,
    }
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
